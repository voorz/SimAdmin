//! API 处理器模块 (ModemManager 版)
//!
//! 包含所有 HTTP API 的处理函数

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::fs;
use std::process::{Command, Output};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::{error, info, warn};
use zbus::Connection;

use crate::{
    config::ApnConfig,
    esim::EsimApiError,
    models::*,
    modem_manager::{
        answer_call, apply_roaming_policy, current_sim_identity, get_airplane_mode,
        get_band_lock_status, get_baseband_restart_progress, get_call_by_path, get_call_settings,
        get_cell_location, get_cells_data, get_data_connection_status, get_device_info_data,
        get_is_roaming_mm, get_network_info_data, get_operators_list, get_radio_mode,
        get_signal_strength, get_sim_info_data_with_cache, hangup_all_calls, hangup_call,
        list_apn_contexts, list_current_calls, make_call, power_cycle_sim_for_profile_switch,
        register_operator_auto, register_operator_manual, restart_baseband, scan_operators,
        send_sms, set_airplane_mode, set_apn_on_bearer, set_band_lock, set_call_waiting,
        set_data_connection_with_apn, set_radio_mode, start_cell_monitoring,
        stop_cell_monitoring, background_fetch_smsc,
        find_nm_modem_connection_pub, nm_set_autoconnect_pub,
    },
    state::AppState,
    utils::{
        format_uptime, get_active_interfaces, read_cpu_info, read_cpu_load_sync, read_disk_info,
        read_interface_stats, read_memory_info, read_network_interfaces, read_system_info,
        read_uptime, sample_cpu_usage,
    },
};

const ESIM_SIM_IDENTITY_TIMEOUT_SECS: u64 = 3;
const ESIM_SIM_ENRICH_TIMEOUT_SECS: u64 = 12;
const SMS_DB_MAINTENANCE_DELETE_THRESHOLD: usize = 100;
const SMS_DB_MAINTENANCE_DELAY_SECS: u64 = 60;

// ============ 基础接口 ============

/// 处理 OPTIONS 请求（CORS 预检）
pub async fn options_handler() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// GET /api/health
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "message": "Service is running",
            "platform": "linux-modem",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

fn esim_error_response<T: Default>(error: EsimApiError) -> (StatusCode, Json<ApiResponse<T>>) {
    let status = match error {
        EsimApiError::Disabled => StatusCode::FORBIDDEN,
        EsimApiError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        EsimApiError::Command(_) => StatusCode::OK,
    };
    (status, Json(ApiResponse::<T>::error(error.message())))
}

fn esim_command_succeeded(response: &EsimCommandResponse) -> bool {
    response.code == 0
        && (response.status.is_empty()
            || response.status.eq_ignore_ascii_case("success")
            || response.status.eq_ignore_ascii_case("ok"))
}

fn esim_profile_is_active(profile: &EsimProfile) -> bool {
    matches!(
        profile.state.trim().to_ascii_lowercase().as_str(),
        "enabled" | "active" | "1" | "true"
    )
}

fn enrich_profiles_with_current_sim(profiles: &mut [EsimProfile], sim: &SimInfoResponse) {
    if !sim.present {
        return;
    }
    let current_index = profiles
        .iter()
        .position(|profile| !sim.iccid.is_empty() && profile.iccid == sim.iccid)
        .or_else(|| profiles.iter().position(esim_profile_is_active));

    let Some(profile) = current_index.and_then(|index| profiles.get_mut(index)) else {
        return;
    };

    if profile.state == "unknown" || !sim.iccid.is_empty() && profile.iccid == sim.iccid {
        profile.state = "enabled".to_string();
    }
    if profile.imsi.is_none() && !sim.imsi.is_empty() {
        profile.imsi = Some(sim.imsi.clone());
    }
    if profile.msisdn.is_none() {
        if let Some(number) = sim
            .phone_numbers
            .iter()
            .find(|number| !number.trim().is_empty())
        {
            profile.msisdn = Some(number.clone());
        }
    }
    if profile.smsc.is_none() && !sim.sms_center.is_empty() {
        profile.smsc = Some(sim.sms_center.clone());
    }
    if profile.mcc.is_none() && !sim.mcc.is_empty() {
        profile.mcc = Some(sim.mcc.clone());
    }
    if profile.mnc.is_none() && !sim.mnc.is_empty() {
        profile.mnc = Some(sim.mnc.clone());
    }
}

fn split_profile_operator_code(code: &str) -> (String, String) {
    let digits: String = code.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.len() >= 6 {
        (digits[..3].to_string(), digits[3..6].to_string())
    } else if digits.len() >= 5 {
        (digits[..3].to_string(), digits[3..].to_string())
    } else {
        (String::new(), String::new())
    }
}

fn enrich_profiles_with_current_identity(
    profiles: &mut [EsimProfile],
    identity: &crate::modem_manager::SimIdentity,
) {
    let current_index = profiles
        .iter()
        .position(|profile| !identity.iccid.is_empty() && profile.iccid == identity.iccid)
        .or_else(|| profiles.iter().position(esim_profile_is_active));

    let Some(profile) = current_index.and_then(|index| profiles.get_mut(index)) else {
        return;
    };

    if profile.state == "unknown" || !identity.iccid.is_empty() && profile.iccid == identity.iccid {
        profile.state = "enabled".to_string();
    }
    if profile.imsi.is_none() && !identity.imsi.is_empty() {
        profile.imsi = Some(identity.imsi.clone());
    }
    let (mcc, mnc) = split_profile_operator_code(&identity.operator_id);
    if profile.mcc.is_none() && !mcc.is_empty() {
        profile.mcc = Some(mcc);
    }
    if profile.mnc.is_none() && !mnc.is_empty() {
        profile.mnc = Some(mnc);
    }
}

fn profile_cache_value(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn optional_profile_cache_value(value: &Option<String>) -> Option<String> {
    value.as_deref().and_then(profile_cache_value)
}

fn profile_cache_entry(profile: &EsimProfile) -> EsimProfileCacheEntry {
    EsimProfileCacheEntry {
        iccid: profile.iccid.trim().to_string(),
        name: profile_cache_value(&profile.name),
        provider: profile_cache_value(&profile.provider),
        profile_class: profile_cache_value(&profile.profile_class),
        imsi: optional_profile_cache_value(&profile.imsi),
        msisdn: optional_profile_cache_value(&profile.msisdn),
        smsc: optional_profile_cache_value(&profile.smsc),
        smdp: optional_profile_cache_value(&profile.smdp),
        isdp_aid: optional_profile_cache_value(&profile.isdp_aid),
        mcc: optional_profile_cache_value(&profile.mcc),
        mnc: optional_profile_cache_value(&profile.mnc),
        updated_at: String::new(),
    }
}

fn fill_cached_string(target: &mut String, cached: Option<String>) {
    if target.trim().is_empty() {
        if let Some(value) = cached.and_then(|item| profile_cache_value(&item)) {
            *target = value;
        }
    }
}

fn fill_cached_option(target: &mut Option<String>, cached: Option<String>) {
    if target.as_deref().unwrap_or("").trim().is_empty() {
        if let Some(value) = cached.and_then(|item| profile_cache_value(&item)) {
            *target = Some(value);
        }
    }
}

fn hydrate_profile_from_cache(db: &Database, profile: &mut EsimProfile) {
    let cache = match db.get_esim_profile_cache(&profile.iccid) {
        Ok(Some(cache)) => cache,
        Ok(None) => return,
        Err(err) => {
            warn!(iccid = %profile.iccid, error = %err, "Failed to read eSIM profile cache");
            return;
        }
    };

    fill_cached_string(&mut profile.name, cache.name);
    fill_cached_string(&mut profile.provider, cache.provider);
    fill_cached_string(&mut profile.profile_class, cache.profile_class);
    fill_cached_option(&mut profile.imsi, cache.imsi);
    fill_cached_option(&mut profile.msisdn, cache.msisdn);
    fill_cached_option(&mut profile.smsc, cache.smsc);
    fill_cached_option(&mut profile.smdp, cache.smdp);
    fill_cached_option(&mut profile.isdp_aid, cache.isdp_aid);
    fill_cached_option(&mut profile.mcc, cache.mcc);
    fill_cached_option(&mut profile.mnc, cache.mnc);
}

fn hydrate_profiles_from_cache(db: &Database, profiles: &mut [EsimProfile]) {
    for profile in profiles {
        hydrate_profile_from_cache(db, profile);
    }
}

fn cache_esim_profiles(db: &Database, profiles: &[EsimProfile]) {
    for profile in profiles {
        if let Err(err) = db.upsert_esim_profile_cache(&profile_cache_entry(profile)) {
            warn!(iccid = %profile.iccid, error = %err, "Failed to write eSIM profile cache");
        }
    }
}

fn profile_from_cache_entry(entry: EsimProfileCacheEntry) -> EsimProfile {
    EsimProfile {
        iccid: entry.iccid,
        name: entry.name.unwrap_or_default(),
        provider: entry.provider.unwrap_or_default(),
        state: "unknown".to_string(),
        profile_class: entry.profile_class.unwrap_or_default(),
        imsi: entry.imsi,
        msisdn: entry.msisdn,
        smsc: entry.smsc,
        smdp: entry.smdp,
        isdp_aid: entry.isdp_aid,
        mcc: entry.mcc,
        mnc: entry.mnc,
        disable_allowed: Some(true),
        delete_allowed: Some(true),
        raw: json!({
            "source": "cache",
            "updated_at": entry.updated_at,
        }),
    }
}

fn cached_profiles_requested(query: &std::collections::HashMap<String, String>) -> bool {
    query
        .get("cached")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false)
}

// ============ 工作模式 / eSIM ============

/// GET /api/work-mode
pub async fn get_work_mode_handler(State(app): State<AppState>) -> impl IntoResponse {
    let mode = app.config_manager.get_work_mode();
    let worker_running = app.esim_supervisor.worker_running().await;
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Success",
            WorkModeResponse {
                mode,
                worker_running,
            },
        )),
    )
}

/// POST /api/work-mode
pub async fn set_work_mode_handler(
    State(app): State<AppState>,
    Json(payload): Json<WorkModeRequest>,
) -> impl IntoResponse {
    if !payload.confirm {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<WorkModeResponse>::error(
                "Changing work mode requires confirm=true",
            )),
        );
    }

    match app.esim_supervisor.switch_mode(payload.mode).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Work mode updated", data)),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WorkModeResponse>::error(err)),
        ),
    }
}

/// GET /api/esim/lpac/status
pub async fn get_esim_lpac_status_handler(State(app): State<AppState>) -> impl IntoResponse {
    match app.esim_supervisor.get_lpac_status().await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(err) => esim_error_response::<EsimLpacStatusResponse>(err),
    }
}

/// POST /api/esim/lpac/repair
pub async fn repair_esim_lpac_handler(
    State(app): State<AppState>,
    Json(payload): Json<EsimLpacRepairRequest>,
) -> impl IntoResponse {
    match app.esim_supervisor.repair_lpac(payload).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("lpac repaired", data)),
        ),
        Err(err) => esim_error_response::<EsimLpacRepairResponse>(err),
    }
}

/// GET /api/esim/euicc
pub async fn get_esim_euicc_handler(State(app): State<AppState>) -> impl IntoResponse {
    match app.esim_supervisor.get_euicc_info().await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(err) => esim_error_response::<EsimEuiccInfo>(err),
    }
}

/// GET /api/esim/profiles
pub async fn get_esim_profiles_handler(
    State(app): State<AppState>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    if cached_profiles_requested(&query) {
        return match app.database.list_esim_profile_cache() {
            Ok(entries) => (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Cached profiles",
                    EsimProfilesResponse {
                        profiles: entries.into_iter().map(profile_from_cache_entry).collect(),
                    },
                )),
            ),
            Err(err) => (
                StatusCode::OK,
                Json(ApiResponse::<EsimProfilesResponse>::error(format!(
                    "Failed to read cached profiles: {err}"
                ))),
            ),
        };
    }

    match app.esim_supervisor.get_profiles().await {
        Ok(mut data) => {
            hydrate_profiles_from_cache(&app.database, &mut data.profiles);
            match tokio::time::timeout(
                std::time::Duration::from_secs(ESIM_SIM_IDENTITY_TIMEOUT_SECS),
                current_sim_identity(&app.dbus_conn),
            )
            .await
            {
                Ok(Some(identity)) => {
                    enrich_profiles_with_current_identity(&mut data.profiles, &identity)
                }
                Ok(None) => {}
                Err(_) => warn!(
                    timeout_secs = ESIM_SIM_IDENTITY_TIMEOUT_SECS,
                    "Timed out enriching eSIM profiles with current SIM identity"
                ),
            }
            match tokio::time::timeout(
                std::time::Duration::from_secs(ESIM_SIM_ENRICH_TIMEOUT_SECS),
                get_sim_info_data_with_cache(&app.dbus_conn, Some(&app.database)),
            )
            .await
            {
                Ok(Ok(sim_info)) => enrich_profiles_with_current_sim(&mut data.profiles, &sim_info),
                Ok(Err(err)) => {
                    warn!(error = %err, "Failed to enrich eSIM profiles with current SIM")
                }
                Err(_) => warn!(
                    timeout_secs = ESIM_SIM_ENRICH_TIMEOUT_SECS,
                    "Timed out enriching eSIM profiles with current SIM"
                ),
            }
            cache_esim_profiles(&app.database, &data.profiles);
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Success", data)),
            )
        }
        Err(err) => esim_error_response::<EsimProfilesResponse>(err),
    }
}

/// POST /api/esim/profiles/{iccid}/enable
pub async fn enable_esim_profile_handler(
    State(app): State<AppState>,
    Path(iccid): Path<String>,
) -> impl IntoResponse {
    match app.esim_supervisor.enable_profile(iccid).await {
        Ok(mut data) => {
            if esim_command_succeeded(&data) {
                let auto_connect_data = !app.data_user_disabled.load(Ordering::SeqCst);
                let allow_roaming = app.config_manager.get_roaming_allowed();
                let apn_config = app.config_manager.get_apn_config();
                match power_cycle_sim_for_profile_switch(
                    &app.dbus_conn,
                    auto_connect_data,
                    allow_roaming,
                    Some(apn_config),
                )
                .await
                {
                    Ok(recovery) => {
                        let lpac_data = data.data.take();
                        data.msg =
                            "Profile enabled and modem SIM power-cycle completed".to_string();
                        data.data = Some(json!({
                            "lpac_data": lpac_data,
                            "baseband_restart": recovery,
                        }));
                        if app.sms_resync.request_scan("profile-switch") {
                            info!("Requested SMS resync after eSIM profile switch");
                        } else {
                            warn!("Failed to request SMS resync after eSIM profile switch");
                        }
                    }
                    Err(err) => {
                        data.code = 1;
                        data.status = "error".to_string();
                        data.msg = format!(
                            "Profile enable command succeeded, but modem SIM power-cycle failed: {err}"
                        );
                        if app
                            .sms_resync
                            .request_scan("profile-switch-recovery-failed")
                        {
                            info!("Requested SMS resync after failed eSIM profile recovery");
                        } else {
                            warn!(
                                "Failed to request SMS resync after failed eSIM profile recovery"
                            );
                        }
                    }
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Profile enable requested",
                    data,
                )),
            )
        }
        Err(err) => esim_error_response::<EsimCommandResponse>(err),
    }
}

/// POST /api/esim/profiles/{iccid}/rename
pub async fn rename_esim_profile_handler(
    State(app): State<AppState>,
    Path(iccid): Path<String>,
    Json(payload): Json<EsimRenameRequest>,
) -> impl IntoResponse {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<EsimCommandResponse>::error(
                "Profile name cannot be empty",
            )),
        );
    }
    match app.esim_supervisor.rename_profile(iccid, name).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Profile renamed", data)),
        ),
        Err(err) => esim_error_response::<EsimCommandResponse>(err),
    }
}

/// DELETE /api/esim/profiles/{iccid}
pub async fn delete_esim_profile_handler(
    State(app): State<AppState>,
    Path(iccid): Path<String>,
) -> impl IntoResponse {
    match app.esim_supervisor.delete_profile(iccid.clone()).await {
        Ok(data) => {
            if esim_command_succeeded(&data) {
                if let Err(err) = app.database.delete_esim_profile_cache(&iccid) {
                    warn!(iccid = %iccid, error = %err, "Failed to delete eSIM profile cache");
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Profile deleted", data)),
            )
        }
        Err(err) => esim_error_response::<EsimCommandResponse>(err),
    }
}

// ============ 设备信息 ============

/// GET /api/device
pub async fn get_device_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_device_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DeviceInfoResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ SIM 卡 ============

/// GET /api/sim
pub async fn get_sim_info(
    State((conn, db)): State<(Arc<Connection>, Arc<Database>)>,
) -> impl IntoResponse {
    match get_sim_info_data_with_cache(&conn, Some(&db)).await {
        Ok(data) => {
            // 如果 SMSC 为空，后台异步通过 AT+CRSM 读取 EF_SMSP 并缓存
            if data.sms_center.is_empty() {
                let conn_bg = Arc::clone(&conn);
                let db_bg = Arc::clone(&db);
                tokio::spawn(async move {
                    background_fetch_smsc(&conn_bg, &db_bg).await;
                });
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Success", data)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SimInfoResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ 网络信息 ============

/// GET /api/network
pub async fn get_network_info(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_network_info_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<NetworkInfoResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/cells
pub async fn get_cells(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_cells_data(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CellsResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/cell-monitor/start
pub async fn start_cell_monitor_handler(State(app): State<AppState>) -> impl IntoResponse {
    if app.cell_monitoring_active.swap(true, Ordering::SeqCst) {
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor already active",
                json!({}),
            )),
        );
    }

    match start_cell_monitoring().await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor activated",
                json!({}),
            )),
        ),
        Err(e) => {
            app.cell_monitoring_active.store(false, Ordering::SeqCst);
            (
                StatusCode::OK,
                Json(ApiResponse::<serde_json::Value>::error(format!(
                    "Failed: {}",
                    e
                ))),
            )
        }
    }
}

/// POST /api/cell-monitor/stop
pub async fn stop_cell_monitor_handler(State(app): State<AppState>) -> impl IntoResponse {
    if !app.cell_monitoring_active.swap(false, Ordering::SeqCst) {
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor already inactive",
                json!({}),
            )),
        );
    }

    match stop_cell_monitoring().await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Cell monitor deactivated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/radio-mode
pub async fn get_radio_mode_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_radio_mode(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RadioModeResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/radio-mode
pub async fn set_radio_mode_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<RadioModeRequest>,
) -> impl IntoResponse {
    match set_radio_mode(&conn, payload.mode).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Radio mode updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/band-lock
pub async fn get_band_lock_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_band_lock_status(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<BandLockStatus>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/band-lock
pub async fn set_band_lock_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<BandLockRequest>,
) -> impl IntoResponse {
    match set_band_lock(&conn, &payload).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Band selection updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/location/cell-info
pub async fn get_cell_location_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_cell_location(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CellLocationResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/network/operators
pub async fn get_network_operators(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_operators_list(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<OperatorListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/network/operators/scan
pub async fn scan_network_operators(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match scan_operators(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<OperatorListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/network/register-manual
pub async fn register_network_manual(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<ManualRegisterRequest>,
) -> impl IntoResponse {
    match register_operator_manual(&conn, &payload.mccmnc).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Registration started",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/network/register-auto
pub async fn register_network_auto(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match register_operator_auto(&conn).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Auto registration started",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/apn
pub async fn get_apn_list_handler(State(app): State<AppState>) -> impl IntoResponse {
    let apn_config = app.config_manager.get_apn_config();
    match list_apn_contexts(&app.dbus_conn, Some(&apn_config)).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<ApnListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/apn
pub async fn set_apn_handler(
    State(app): State<AppState>,
    Json(payload): Json<SetApnRequest>,
) -> impl IntoResponse {
    let mut apn_config = app.config_manager.get_apn_config();
    if let Some(apn) = &payload.apn {
        apn_config.apn = apn.trim().to_string();
    }
    if let Some(protocol) = &payload.protocol {
        apn_config.protocol = protocol.trim().to_string();
    }
    if let Some(username) = &payload.username {
        apn_config.username = username.trim().to_string();
    }
    if let Some(password) = &payload.password {
        apn_config.password = password.clone();
    }
    if let Some(auth_method) = &payload.auth_method {
        apn_config.auth_method = auth_method.trim().to_string();
    }
    if apn_config.protocol.trim().is_empty() {
        apn_config.protocol = ApnConfig::default().protocol;
    }
    if apn_config.auth_method.trim().is_empty() {
        apn_config.auth_method = ApnConfig::default().auth_method;
    }

    if let Err(err) = app.config_manager.set_apn_config(apn_config) {
        return (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to save APN config: {}",
                err
            ))),
        );
    }

    let context_path = payload.context_path.trim();
    if context_path.is_empty() || context_path.ends_with("/bearer/default") {
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "APN config saved",
                json!({}),
            )),
        );
    }

    match set_apn_on_bearer(&app.dbus_conn, &payload).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("APN updated", json!({}))),
        ),
        Err(e) => {
            warn!(error = %e, "APN config saved but bearer update failed");
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "APN config saved",
                    json!({ "bearer_update_error": e.to_string() }),
                )),
            )
        }
    }
}

/// GET /api/cell-lock
pub async fn get_cell_lock_status_handler(State(app): State<AppState>) -> impl IntoResponse {
    let store = app.cell_lock.lock().await;
    let data = store.status();
    drop(store);
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", data)),
    )
}

/// POST /api/cell-lock
pub async fn set_cell_lock_handler(
    State(app): State<AppState>,
    Json(payload): Json<CellLockRequest>,
) -> impl IntoResponse {
    let mut store = app.cell_lock.lock().await;
    match store.apply(&payload) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "OK",
                CellLockResult { success: true },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CellLockResult>::error(e)),
        ),
    }
}

/// POST /api/cell-lock/unlock-all
pub async fn unlock_all_cells_handler(State(app): State<AppState>) -> impl IntoResponse {
    let mut store = app.cell_lock.lock().await;
    store.unlock_all();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Unlocked",
            CellLockResult { success: true },
        )),
    )
}

/// GET /api/network/interfaces
pub async fn get_network_interfaces_info(State(dbus_conn): State<Arc<Connection>>) -> impl IntoResponse {
    match read_network_interfaces(Some(&dbus_conn)).await {
        Ok(interfaces) => {
            let total_count = interfaces.len();
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Success",
                    NetworkInterfacesResponse {
                        interfaces,
                        total_count,
                    },
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<NetworkInterfacesResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/device-network/ddns/config
pub async fn get_device_ddns_config_handler(State(app): State<AppState>) -> impl IntoResponse {
    let config = app.config_manager.get_ddns_config();
    let access_secret_set = !config.access_secret.trim().is_empty();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Success",
            ddns_config_response(config, access_secret_set),
        )),
    )
}

/// POST /api/device-network/ddns/config
pub async fn set_device_ddns_config_handler(
    State(app): State<AppState>,
    Json(mut payload): Json<crate::config::DdnsConfig>,
) -> impl IntoResponse {
    let current = app.config_manager.get_ddns_config();
    if is_masked_secret(&payload.access_id) {
        payload.access_id = current.access_id;
    }
    if payload.access_secret.trim().is_empty() {
        payload.access_secret = current.access_secret;
    } else if is_masked_secret(&payload.access_secret) {
        payload.access_secret = current.access_secret;
    }
    if payload.interval_seconds == 0 {
        payload.interval_seconds = 300;
    }
    if payload.ttl == 0 {
        payload.ttl = 600;
    }

    match app.config_manager.set_ddns_config(payload.clone()) {
        Ok(()) => {
            let access_secret_set = !payload.access_secret.trim().is_empty();
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "DDNS config updated",
                    ddns_config_response(payload, access_secret_set),
                )),
            )
        }
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

fn ddns_config_response(
    mut config: crate::config::DdnsConfig,
    access_secret_set: bool,
) -> serde_json::Value {
    config.access_id = mask_secret(&config.access_id);
    config.access_secret = mask_secret(&config.access_secret);
    let mut value = serde_json::to_value(config).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.insert("access_secret_set".to_string(), json!(access_secret_set));
    }
    value
}

fn mask_secret(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let prefix: String = trimmed.chars().take(3).collect();
    format!("{prefix}******")
}

fn is_masked_secret(value: &str) -> bool {
    value.contains('*')
}

/// GET /api/device-network/ddns/status
pub async fn get_device_ddns_status_handler(State(app): State<AppState>) -> impl IntoResponse {
    let config = app.config_manager.get_ddns_config();
    let status = app.ddns_manager.status(&config).await;
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", status)),
    )
}

/// POST /api/device-network/ddns/sync
pub async fn sync_device_ddns_handler(State(app): State<AppState>) -> impl IntoResponse {
    match app
        .ddns_manager
        .sync_now(app.config_manager.clone(), app.notification_sender.clone())
        .await
    {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "DDNS sync completed",
                data,
            )),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<DdnsSyncResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// GET /api/device-network/ddns/logs
pub async fn get_device_ddns_logs_handler(State(app): State<AppState>) -> impl IntoResponse {
    let logs = app.ddns_manager.logs().await;
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", logs)),
    )
}

/// POST /api/device-network/ddns/logs/clear
pub async fn clear_device_ddns_logs_handler(State(app): State<AppState>) -> impl IntoResponse {
    app.ddns_manager.clear_logs().await;
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "DDNS logs cleared",
            json!({}),
        )),
    )
}

/// GET /api/device-network/wlan/status
pub async fn get_device_wlan_status_handler() -> impl IntoResponse {
    match crate::device_network::wlan_status().await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanStatusResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// POST /api/device-network/wlan/enabled
pub async fn set_device_wlan_enabled_handler(
    Json(payload): Json<WlanEnabledRequest>,
) -> impl IntoResponse {
    match crate::device_network::wlan_set_enabled(payload).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "WLAN state updated",
                data,
            )),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanStatusResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// POST /api/device-network/wlan/scan
pub async fn scan_device_wlan_handler() -> impl IntoResponse {
    match crate::device_network::wlan_scan().await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanScanResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// GET /api/device-network/wlan/profiles
pub async fn get_device_wlan_profiles_handler() -> impl IntoResponse {
    match crate::device_network::wlan_profiles().await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanProfilesResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// POST /api/device-network/wlan/forget
pub async fn forget_device_wlan_handler(
    Json(payload): Json<WlanForgetRequest>,
) -> impl IntoResponse {
    match crate::device_network::wlan_forget(payload).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "WLAN profile forgotten",
                data,
            )),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanProfilesResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// POST /api/device-network/wlan/connect
pub async fn connect_device_wlan_handler(
    Json(payload): Json<WlanConnectRequest>,
) -> impl IntoResponse {
    match crate::device_network::wlan_connect(payload).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("WLAN connected", data)),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanStatusResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// POST /api/device-network/wlan/disconnect
pub async fn disconnect_device_wlan_handler() -> impl IntoResponse {
    match crate::device_network::wlan_disconnect().await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("WLAN disconnected", data)),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanStatusResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// POST /api/device-network/wlan/profile
pub async fn save_device_wlan_profile_handler(
    Json(payload): Json<WlanProfileRequest>,
) -> impl IntoResponse {
    match crate::device_network::wlan_save_profile(payload).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "WLAN profile updated",
                data,
            )),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(ApiResponse::<WlanStatusResponse>::error(format!(
                "Failed: {}",
                err
            ))),
        ),
    }
}

/// GET /api/network/signal-strength
pub async fn get_signal_strength_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_signal_strength(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SignalStrengthResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ 数据连接 ============

/// GET /api/data
pub async fn get_data_status(State(app): State<AppState>) -> impl IntoResponse {
    if app.data_user_disabled.load(Ordering::SeqCst) {
        return (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                DataConnectionResponse { active: false },
            )),
        );
    }

    match get_data_connection_status(&app.dbus_conn).await {
        Ok(active) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                DataConnectionResponse { active },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DataConnectionResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/data
pub async fn set_data_status(
    State(app): State<AppState>,
    Json(payload): Json<DataConnectionRequest>,
) -> impl IntoResponse {
    let allow_roaming = app.config_manager.get_roaming_allowed();
    let apn_config = app.config_manager.get_apn_config();
    match set_data_connection_with_apn(
        &app.dbus_conn,
        payload.active,
        allow_roaming,
        Some(&apn_config),
    )
    .await
    {
        Ok(_) => {
            if let Err(err) = app.config_manager.set_data_enabled(payload.active) {
                return (
                    StatusCode::OK,
                    Json(ApiResponse::<DataConnectionResponse>::error(format!(
                        "Failed to save data switch state: {}",
                        err
                    ))),
                );
            }
            app.data_user_disabled
                .store(!payload.active, Ordering::SeqCst);
            // 同步 NM autoconnect 状态，防止用户关闭数据后 NM 自动重连
            tokio::spawn(async move {
                if let Ok(profile) = find_nm_modem_connection_pub().await {
                    let _ = nm_set_autoconnect_pub(&profile, payload.active).await;
                }
            });
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Data connection updated",
                    DataConnectionResponse {
                        active: payload.active,
                    },
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<DataConnectionResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn restart_baseband_handler(State(app): State<AppState>) -> impl IntoResponse {
    let auto_connect_data = !app.data_user_disabled.load(Ordering::SeqCst);
    let allow_roaming = app.config_manager.get_roaming_allowed();
    let apn_config = app.config_manager.get_apn_config();
    match restart_baseband(
        &app.dbus_conn,
        auto_connect_data,
        allow_roaming,
        Some(apn_config),
    )
    .await
    {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Baseband restarted",
                data,
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<BasebandRestartResponse>::error(format!(
                "重启基带失败：{}",
                e
            ))),
        ),
    }
}

pub async fn get_baseband_restart_status_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Success",
            get_baseband_restart_progress(),
        )),
    )
}

/// GET /api/roaming
pub async fn get_roaming_status_handler(State(app): State<AppState>) -> impl IntoResponse {
    let roaming_allowed = app.config_manager.get_roaming_allowed();
    match get_is_roaming_mm(&app.dbus_conn).await {
        Ok(is_roaming) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                RoamingResponse {
                    roaming_allowed,
                    is_roaming,
                },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RoamingResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/roaming
pub async fn set_roaming_status_handler(
    State(app): State<AppState>,
    Json(payload): Json<RoamingRequest>,
) -> impl IntoResponse {
    match apply_roaming_policy(&app.dbus_conn, &app.config_manager, payload.allowed).await {
        Ok(_) => {
            let roaming_allowed = app.config_manager.get_roaming_allowed();
            match get_is_roaming_mm(&app.dbus_conn).await {
                Ok(is_roaming) => (
                    StatusCode::OK,
                    Json(ApiResponse::success_with_message(
                        "Success",
                        RoamingResponse {
                            roaming_allowed,
                            is_roaming,
                        },
                    )),
                ),
                Err(e) => (
                    StatusCode::OK,
                    Json(ApiResponse::<RoamingResponse>::error(format!(
                        "Failed: {}",
                        e
                    ))),
                ),
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<RoamingResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/airplane-mode
pub async fn set_airplane_mode_handler(
    State(app): State<AppState>,
    Json(payload): Json<AirplaneModeRequest>,
) -> impl IntoResponse {
    if payload.enabled {
        app.airplane_mode_requested.store(true, Ordering::SeqCst);
    }

    match set_airplane_mode(&app.dbus_conn, payload.enabled).await {
        Ok(_) => {
            app.airplane_mode_requested
                .store(payload.enabled, Ordering::SeqCst);
            match get_airplane_mode(&app.dbus_conn).await {
                Ok(status) => (
                    StatusCode::OK,
                    Json(ApiResponse::success_with_message(
                        if payload.enabled {
                            "Airplane mode enabled"
                        } else {
                            "Airplane mode disabled"
                        },
                        status,
                    )),
                ),
                Err(e) => (
                    StatusCode::OK,
                    Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                        "Failed: {}",
                        e
                    ))),
                ),
            }
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/airplane-mode
pub async fn get_airplane_mode_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_airplane_mode(&conn).await {
        Ok(status) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", status)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<AirplaneModeResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

// ============ 短信功能 ============

use crate::db::{Database, EsimProfileCacheEntry};

fn schedule_sms_db_maintenance(app: &AppState, deleted: usize) {
    if deleted < SMS_DB_MAINTENANCE_DELETE_THRESHOLD {
        return;
    }

    if app.sms_db_maintenance_pending.swap(true, Ordering::SeqCst) {
        info!(
            deleted,
            threshold = SMS_DB_MAINTENANCE_DELETE_THRESHOLD,
            "SMS database maintenance already scheduled"
        );
        return;
    }

    let db = Arc::clone(&app.database);
    let pending = Arc::clone(&app.sms_db_maintenance_pending);
    tokio::spawn(async move {
        info!(
            deleted,
            delay_secs = SMS_DB_MAINTENANCE_DELAY_SECS,
            "SMS database maintenance scheduled"
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(
            SMS_DB_MAINTENANCE_DELAY_SECS,
        ))
        .await;

        let result = tokio::task::spawn_blocking(move || db.vacuum()).await;
        match result {
            Ok(Ok(())) => info!("SMS database maintenance completed"),
            Ok(Err(err)) => warn!(error = %err, "SMS database maintenance failed"),
            Err(err) => warn!(error = %err, "SMS database maintenance task failed"),
        }
        pending.store(false, Ordering::SeqCst);
    });
}

/// POST /api/sms/send
pub async fn send_sms_handler(
    State(conn): State<Arc<Connection>>,
    State(db): State<Arc<Database>>,
    Json(payload): Json<SendSmsRequest>,
) -> impl IntoResponse {
    match send_sms(&conn, &payload.phone_number, &payload.content).await {
        Ok(path) => {
            // 存入数据库
            let _ = db.insert_sms(
                "outgoing",
                &payload.phone_number,
                &payload.content,
                "sent",
                None,
            );
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "SMS sent",
                    json!({ "path": path }),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to send SMS: {}",
                e
            ))),
        ),
    }
}

/// GET /api/sms/list
pub async fn get_sms_list_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<SmsListRequest>,
) -> (StatusCode, Json<ApiResponse<SmsListResponse>>) {
    let limit = if params.limit > 0 { params.limit } else { 50 };
    let offset = if params.offset >= 0 { params.offset } else { 0 };
    let direction = params
        .direction
        .as_deref()
        .filter(|value| matches!(*value, "incoming" | "outgoing"));

    match db.get_sms_messages(limit, offset, direction) {
        Ok(messages) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                SmsListResponse { messages },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SmsListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/sms/conversation
pub async fn get_sms_conversation_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<SmsConversationRequest>,
) -> (StatusCode, Json<ApiResponse<SmsListResponse>>) {
    let limit = if params.limit > 0 { params.limit } else { 50 };
    match db.get_sms_conversation(&params.phone_number, limit) {
        Ok(messages) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                SmsListResponse { messages },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SmsListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// GET /api/sms/stats
pub async fn get_sms_stats_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<SmsStatsResponse>>) {
    match db.get_sms_stats() {
        Ok(stats) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", stats)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<SmsStatsResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/sms/clear
pub async fn clear_sms_handler(
    State(app): State<AppState>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let deleted = app
        .database
        .get_sms_stats()
        .map(|stats| stats.total.max(0) as usize)
        .unwrap_or(SMS_DB_MAINTENANCE_DELETE_THRESHOLD);

    match app.database.clear_all_sms() {
        Ok(_) => {
            schedule_sms_db_maintenance(&app, deleted);
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "All SMS cleared",
                    json!({ "deleted": deleted }),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// DELETE /api/sms/message/{id}
pub async fn delete_sms_message_handler(
    State(db): State<Arc<Database>>,
    Path(id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.delete_sms(id) {
        Ok(deleted) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "SMS deleted",
                json!({ "deleted": deleted }),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// DELETE /api/sms/conversation/{phone_number}
pub async fn delete_sms_conversation_handler(
    State(app): State<AppState>,
    Path(phone_number): Path<String>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match app.database.delete_sms_conversation(&phone_number) {
        Ok(deleted) => {
            schedule_sms_db_maintenance(&app, deleted);
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "SMS conversation deleted",
                    json!({ "deleted": deleted }),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// POST /api/sms/batch-delete
pub async fn delete_sms_batch_handler(
    State(app): State<AppState>,
    Json(payload): Json<SmsBatchDeleteRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    if payload.ids.is_empty() && payload.phone_numbers.is_empty() {
        return (StatusCode::OK, Json(ApiResponse::error("No SMS selected")));
    }

    match app
        .database
        .delete_sms_batch(&payload.ids, &payload.phone_numbers)
    {
        Ok(deleted) => {
            schedule_sms_db_maintenance(&app, deleted);
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "SMS batch deleted",
                    json!({ "deleted": deleted }),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

// ============ 系统信息 ============

/// 读取温度传感器数据
// ============ 电话功能 ============

async fn track_call_start(
    app: &AppState,
    path: &str,
    direction: &str,
    phone_number: &str,
    answered: bool,
) {
    if let Ok(id) = app.database.insert_call(direction, phone_number, answered) {
        let mut active = app.active_calls.lock().await;
        active.insert(
            path.to_string(),
            crate::state::ActiveCallRecord {
                id,
                answered_at: answered.then(std::time::Instant::now),
                answered,
            },
        );
    }
}

async fn mark_tracked_call_answered(app: &AppState, path: &str) {
    let mut active = app.active_calls.lock().await;
    if let Some(record) = active.get_mut(path) {
        record.answered = true;
        if record.answered_at.is_none() {
            record.answered_at = Some(std::time::Instant::now());
        }
    }
}

async fn finish_tracked_call(app: &AppState, path: &str, answered_now: bool) {
    let mut record = {
        let mut active = app.active_calls.lock().await;
        active.remove(path)
    };
    if let Some(ref mut record) = record {
        if answered_now && record.answered_at.is_none() {
            record.answered_at = Some(std::time::Instant::now());
        }
        let duration = record
            .answered_at
            .map(|at| at.elapsed().as_secs() as i64)
            .unwrap_or(0);
        let _ = app
            .database
            .update_call_end(record.id, duration, record.answered || answered_now);
    }
}

pub async fn get_calls_handler(State(app): State<AppState>) -> impl IntoResponse {
    match list_current_calls(&app.dbus_conn).await {
        Ok(data) => {
            for call in &data.calls {
                if matches!(call.state.as_str(), "active" | "held") {
                    mark_tracked_call_answered(&app, &call.path).await;
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Success", data)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CallListResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn dial_call_handler(
    State(app): State<AppState>,
    Json(payload): Json<MakeCallRequest>,
) -> impl IntoResponse {
    let phone_number = payload.phone_number.trim().to_string();
    if phone_number.is_empty() {
        return (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(
                "Phone number is required",
            )),
        );
    }
    match make_call(&app.dbus_conn, &phone_number).await {
        Ok(path) => {
            track_call_start(&app, &path, "outgoing", &phone_number, false).await;
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Call started",
                    json!({ "path": path }),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to dial: {}",
                e
            ))),
        ),
    }
}

pub async fn hangup_call_handler(
    State(app): State<AppState>,
    Json(payload): Json<HangupCallRequest>,
) -> impl IntoResponse {
    let before = get_call_by_path(&app.dbus_conn, &payload.path).await.ok();
    match hangup_call(&app.dbus_conn, &payload.path).await {
        Ok(()) => {
            let answered = before
                .as_ref()
                .map(|call| call.state == "active" || call.state == "held")
                .unwrap_or(false);
            finish_tracked_call(&app, &payload.path, answered).await;
            if let Some(call) = before {
                if call.direction == "incoming"
                    && matches!(call.state.as_str(), "incoming" | "waiting")
                {
                    let _ = app
                        .database
                        .insert_call("missed", &call.phone_number, false);
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message("Call hung up", json!({}))),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to hang up: {}",
                e
            ))),
        ),
    }
}

pub async fn hangup_all_calls_handler(State(app): State<AppState>) -> impl IntoResponse {
    let before = list_current_calls(&app.dbus_conn).await.ok();
    match hangup_all_calls(&app.dbus_conn).await {
        Ok(()) => {
            if let Some(list) = before {
                for call in list.calls {
                    let answered = call.state == "active" || call.state == "held";
                    finish_tracked_call(&app, &call.path, answered).await;
                    if call.direction == "incoming"
                        && matches!(call.state.as_str(), "incoming" | "waiting")
                    {
                        let _ = app
                            .database
                            .insert_call("missed", &call.phone_number, false);
                    }
                }
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "All calls hung up",
                    json!({}),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to hang up calls: {}",
                e
            ))),
        ),
    }
}

pub async fn answer_call_handler(
    State(app): State<AppState>,
    Json(payload): Json<HangupCallRequest>,
) -> impl IntoResponse {
    let before = get_call_by_path(&app.dbus_conn, &payload.path).await.ok();
    match answer_call(&app.dbus_conn, &payload.path).await {
        Ok(()) => {
            if let Some(call) = before {
                track_call_start(&app, &payload.path, "incoming", &call.phone_number, true).await;
                mark_tracked_call_answered(&app, &payload.path).await;
            }
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(
                    "Call answered",
                    json!({}),
                )),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to answer call: {}",
                e
            ))),
        ),
    }
}

pub async fn get_call_history_handler(
    State(db): State<Arc<Database>>,
    Query(params): Query<CallHistoryRequest>,
) -> (StatusCode, Json<ApiResponse<CallHistoryResponse>>) {
    let limit = if params.limit > 0 { params.limit } else { 50 };
    let offset = if params.offset >= 0 { params.offset } else { 0 };
    match (db.get_call_history(limit, offset), db.get_call_stats()) {
        (Ok(records), Ok(stats)) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Success",
                CallHistoryResponse { records, stats },
            )),
        ),
        (Err(e), _) | (_, Err(e)) => (
            StatusCode::OK,
            Json(ApiResponse::<CallHistoryResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn delete_call_history_handler(
    State(db): State<Arc<Database>>,
    Path(id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.delete_call(id) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Call record deleted",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

pub async fn clear_call_history_handler(
    State(db): State<Arc<Database>>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match db.clear_all_calls() {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Call history cleared",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

pub async fn get_call_settings_handler(State(conn): State<Arc<Connection>>) -> impl IntoResponse {
    match get_call_settings(&conn).await {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CallSettingsResponse>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

pub async fn set_call_settings_handler(
    State(conn): State<Arc<Connection>>,
    Json(payload): Json<SetCallSettingRequest>,
) -> impl IntoResponse {
    if payload.property != "VoiceCallWaiting" {
        return (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(
                "Only VoiceCallWaiting is supported by ModemManager",
            )),
        );
    }
    let enabled = matches!(payload.value.as_str(), "enabled" | "on" | "true" | "1");
    match set_call_waiting(&conn, enabled).await {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Call setting updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed to update call setting: {}",
                e
            ))),
        ),
    }
}

pub async fn get_call_volume_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<CallVolumeResponse>::error(
            "Call volume control is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn set_call_volume_handler(
    Json(payload): Json<SetCallVolumeRequest>,
) -> impl IntoResponse {
    let _ = (
        payload.speaker_volume,
        payload.microphone_volume,
        payload.muted,
    );
    (
        StatusCode::OK,
        Json(ApiResponse::<CallVolumeResponse>::error(
            "Call volume control is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn get_call_forwarding_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<CallForwardingResponse>::error(
            "Call forwarding is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn set_call_forwarding_handler(
    Json(payload): Json<SetCallForwardingRequest>,
) -> impl IntoResponse {
    let _ = (payload.forward_type, payload.number, payload.timeout);
    (
        StatusCode::OK,
        Json(ApiResponse::<CallForwardingResponse>::error(
            "Call forwarding is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn get_ims_status_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<ImsStatusResponse>::error(
            "IMS status is not exposed by ModemManager on this backend",
        )),
    )
}

pub async fn get_voicemail_status_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::<VoicemailStatusResponse>::error(
            "Voicemail status is not exposed by ModemManager on this backend",
        )),
    )
}

fn read_temperature_sensors() -> Vec<ThermalZone> {
    use std::fs;
    use std::path::Path;

    let thermal_path = Path::new("/sys/class/thermal");
    let mut sensors = Vec::new();

    if let Ok(entries) = fs::read_dir(thermal_path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if name.starts_with("thermal_zone") {
                let zone_path = entry.path();
                let sensor_type = fs::read_to_string(zone_path.join("type"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let temperature = fs::read_to_string(zone_path.join("temp"))
                    .ok()
                    .and_then(|s| s.trim().parse::<i32>().ok())
                    .map(|t| t as f64 / 1000.0)
                    .unwrap_or(0.0);

                sensors.push(ThermalZone {
                    zone: name.to_string(),
                    sensor_type,
                    temperature,
                });
            }
        }
    }
    sensors.sort_by(|a, b| a.zone.cmp(&b.zone));
    sensors
}

/// GET /api/stats
pub async fn get_system_stats(State(dbus_conn): State<Arc<Connection>>) -> impl IntoResponse {
    let result: Result<SystemStatsResponse, String> = async {
        let interfaces =
            get_active_interfaces().map_err(|e| format!("Failed to get interfaces: {}", e))?;

        let mut initial: Vec<(String, u64, u64)> = Vec::new();
        for iface in &interfaces {
            if let Ok((rx, tx)) = read_interface_stats(iface, Some(&dbus_conn)).await {
                initial.push((iface.clone(), rx, tx));
            }
        }

        // 并行执行 CPU 采样 (200ms) 和网速采样间隔 (1000ms)，节省 200ms
        let (cpu_usage, _) = tokio::join!(
            async { sample_cpu_usage().await.unwrap_or(0.0) },
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)),
        );

        let mut speed_data = Vec::new();
        let elapsed = 1.0_f64;
        for (interface, rx1, tx1) in &initial {
            if let Ok((rx2, tx2)) = read_interface_stats(interface, Some(&dbus_conn)).await {
                let rx_speed = rx2.saturating_sub(*rx1);
                let tx_speed = tx2.saturating_sub(*tx1);
                speed_data.push(NetworkSpeed {
                    interface: interface.clone(),
                    rx_bytes_per_sec: rx_speed,
                    tx_bytes_per_sec: tx_speed,
                    total_rx_bytes: rx2,
                    total_tx_bytes: tx2,
                });
            }
        }

        let (total, available, cached, buffers) = read_memory_info()?;
        let used = total.saturating_sub(available);
        let used_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let disk = read_disk_info();
        let mut cpu_load = read_cpu_load_sync().unwrap_or_default();
        cpu_load.load_percent = cpu_usage;
        let (uptime, idle) = read_uptime()?;
        let formatted = format_uptime(uptime);
        let system_info = read_system_info()?;
        let temperature = read_temperature_sensors();

        Ok(SystemStatsResponse {
            network_speed: NetworkSpeedResponse {
                interfaces: speed_data,
                interval_seconds: elapsed,
            },
            memory: MemoryInfo {
                total_bytes: total,
                available_bytes: available,
                used_bytes: used,
                used_percent,
                cached_bytes: cached,
                buffers_bytes: buffers,
            },
            disk,
            cpu_load,
            uptime: UptimeInfo {
                uptime_seconds: uptime,
                idle_seconds: idle,
                uptime_formatted: formatted,
            },
            system_info,
            temperature,
        })
    }
    .await;

    match result {
        Ok(data) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", data)),
        ),
        Err(msg) => (
            StatusCode::OK,
            Json(ApiResponse::<SystemStatsResponse>::error(msg)),
        ),
    }
}

/// GET /api/stats/cpu
pub async fn get_cpu_info() -> impl IntoResponse {
    match read_cpu_info() {
        Ok(info) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", info)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<CpuInfo>::error(format!("Failed: {}", e))),
        ),
    }
}

/// GET /api/connectivity
pub async fn get_connectivity_check() -> (StatusCode, Json<ApiResponse<ConnectivityCheckResponse>>)
{
    // 两个 ping 并行执行，超时从 2s 缩短到 1s
    let (ipv4_result, ipv6_result) = tokio::join!(
        async_ping_host("223.5.5.5", false),
        async_ping_host("2400:3200::1", true),
    );
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "Connectivity check completed",
            ConnectivityCheckResponse {
                ipv4: ipv4_result,
                ipv6: ipv6_result,
            },
        )),
    )
}

async fn async_ping_host(target: &str, is_ipv6: bool) -> PingResult {
    let cmd = if is_ipv6 { "ping6" } else { "ping" };
    let output = tokio::process::Command::new(cmd)
        .args(["-c", "1", "-W", "1", target])
        .output()
        .await;
    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let latency = parse_ping_latency(&stdout);
                PingResult {
                    success: true,
                    latency_ms: latency,
                    target: target.to_string(),
                    error: None,
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                PingResult {
                    success: false,
                    latency_ms: None,
                    target: target.to_string(),
                    error: Some(if stderr.is_empty() {
                        "Unreachable".to_string()
                    } else {
                        stderr.trim().to_string()
                    }),
                }
            }
        }
        Err(e) => PingResult {
            success: false,
            latency_ms: None,
            target: target.to_string(),
            error: Some(format!("Failed: {}", e)),
        },
    }
}

fn parse_ping_latency(output: &str) -> Option<f64> {
    for line in output.lines() {
        if let Some(time_pos) = line.find("time=") {
            let after_time = &line[time_pos + 5..];
            let num_str: String = after_time
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(latency) = num_str.parse::<f64>() {
                return Some(latency);
            }
        }
    }
    None
}

/// POST /api/system/reboot
pub async fn system_reboot(Json(payload): Json<SystemRebootRequest>) -> impl IntoResponse {
    let delay = payload.delay_seconds;
    tokio::spawn(async move {
        run_safe_os_reboot_sequence(delay).await;
    });
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            format!("System will perform safe OS reboot in {} seconds", delay),
            json!({ "delay_seconds": delay }),
        )),
    )
}

async fn run_safe_os_reboot_sequence(delay_seconds: u32) {
    if delay_seconds > 0 {
        tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds as u64)).await;
    }

    info!("Starting safe OS reboot sequence");

    run_reboot_prep_command("disable modem radio", "mmcli", &["-m", "0", "-d"], false);
    run_reboot_prep_command(
        "stop ModemManager IPC service",
        "systemctl",
        &["stop", "ModemManager"],
        false,
    );
    run_reboot_prep_command("stop qmi-proxy", "killall", &["qmi-proxy"], true);
    cleanup_modemmanager_runtime_cache();
    run_reboot_prep_command("flush filesystem cache", "sync", &[], false);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    info!("Safe OS reboot preparation complete, executing reboot");
    if let Err(err) = Command::new("reboot").output() {
        error!(error = %err, "Failed to execute reboot command");
    }
}

fn run_reboot_prep_command(label: &str, program: &str, args: &[&str], allow_failure: bool) {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => {
            info!(step = label, "Safe OS reboot step completed");
        }
        Ok(output) => {
            let severity = if allow_failure {
                "optional"
            } else {
                "required"
            };
            warn_reboot_prep_failure(label, program, severity, &output);
        }
        Err(err) if allow_failure => {
            warn!(step = label, command = program, error = %err, "Optional safe OS reboot step failed");
        }
        Err(err) => {
            warn!(step = label, command = program, error = %err, "Safe OS reboot step failed");
        }
    }
}

fn cleanup_modemmanager_runtime_cache() {
    const CACHE_DIR: &str = "/var/lib/ModemManager";

    match fs::read_dir(CACHE_DIR) {
        Ok(entries) => {
            let mut removed = 0usize;
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let result = if path.is_dir() {
                            fs::remove_dir_all(&path)
                        } else {
                            fs::remove_file(&path)
                        };

                        match result {
                            Ok(()) => removed += 1,
                            Err(err) => warn!(
                                path = %path.display(),
                                error = %err,
                                "Failed to remove ModemManager runtime cache entry"
                            ),
                        }
                    }
                    Err(err) => warn!(
                        directory = CACHE_DIR,
                        error = %err,
                        "Failed to read ModemManager runtime cache entry"
                    ),
                }
            }
            info!(
                directory = CACHE_DIR,
                removed, "ModemManager runtime cache cleanup completed"
            );
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            info!(
                directory = CACHE_DIR,
                "ModemManager runtime cache directory does not exist"
            );
        }
        Err(err) => {
            warn!(
                directory = CACHE_DIR,
                error = %err,
                "Failed to open ModemManager runtime cache directory"
            );
        }
    }
}

fn warn_reboot_prep_failure(label: &str, program: &str, severity: &str, output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    warn!(
        step = label,
        command = program,
        severity = severity,
        status = %output.status,
        stderr = %stderr,
        stdout = %stdout,
        "Safe OS reboot step returned non-zero status"
    );
}

// ============ 通知配置 ============

pub async fn restart_service_handler() -> impl IntoResponse {
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let _ = Command::new("systemctl")
            .args(["restart", "simadmin"])
            .output();
    });
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message(
            "SimAdmin service will restart",
            json!({}),
        )),
    )
}

use crate::config::ConfigManager;
use crate::notification::NotificationSender;

/// GET /api/notifications/config
pub async fn get_notification_config_handler(
    State(config_manager): State<Arc<ConfigManager>>,
) -> (
    StatusCode,
    Json<ApiResponse<crate::config::NotificationConfig>>,
) {
    let config = config_manager.get_notifications();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", config)),
    )
}

/// POST /api/notifications/config
pub async fn set_notification_config_handler(
    State(config_manager): State<Arc<ConfigManager>>,
    Json(notification_config): Json<crate::config::NotificationConfig>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match config_manager.set_notifications(notification_config) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Notification config updated",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::error(format!("Failed: {}", e))),
        ),
    }
}

/// POST /api/notifications/test/{channel}
pub async fn test_notification_channel_handler(
    Path(channel): Path<crate::config::NotificationChannel>,
    State(notification_sender): State<Arc<NotificationSender>>,
) -> (
    StatusCode,
    Json<ApiResponse<crate::models::WebhookTestResponse>>,
) {
    match notification_sender.test_channel(channel).await {
        Ok(message) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Notification test successful",
                WebhookTestResponse {
                    success: true,
                    message,
                },
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Notification test failed",
                WebhookTestResponse {
                    success: false,
                    message: e,
                },
            )),
        ),
    }
}

// ============ OTA 更新 ============

/// GET /api/ota/status
pub async fn get_ota_status_handler() -> impl IntoResponse {
    let status = crate::ota::get_ota_status();
    (
        StatusCode::OK,
        Json(ApiResponse::success_with_message("Success", status)),
    )
}

/// POST /api/ota/upload
pub async fn upload_ota_handler(body: axum::body::Bytes) -> impl IntoResponse {
    match crate::ota::handle_ota_upload(&body) {
        Ok(response) => {
            let message = if response.validation.valid {
                "OTA uploaded and validated"
            } else {
                "OTA uploaded but validation failed"
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(message, response)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<crate::models::OtaUploadResponse>::error(
                format!("Failed: {}", e),
            )),
        ),
    }
}

/// POST /api/ota/latest-release
pub async fn get_latest_ota_release_handler(
    Json(req): Json<crate::models::OtaOnlinePrepareRequest>,
) -> impl IntoResponse {
    let result: Result<crate::models::OtaLatestReleaseResponse, String> = async {
        let proxy_prefix = crate::ota::normalize_proxy_prefix(req.proxy_prefix);
        let client = reqwest::Client::builder()
            .user_agent("SimAdmin OTA updater")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        crate::ota::fetch_latest_github_release(&client, &proxy_prefix).await
    }
    .await;

    match result {
        Ok(release) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message("Success", release)),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<crate::models::OtaLatestReleaseResponse>::error(format!(
                "Failed: {}. GitHub may have rate-limited this request; try again later or enable a proxy.",
                e
            ))),
        ),
    }
}

/// POST /api/ota/online-prepare
pub async fn prepare_online_ota_handler(
    Json(req): Json<crate::models::OtaOnlinePrepareRequest>,
) -> impl IntoResponse {
    let result: Result<crate::models::OtaUploadResponse, String> = async {
        let proxy_prefix = crate::ota::normalize_proxy_prefix(req.proxy_prefix);
        let client = reqwest::Client::builder()
            .user_agent("SimAdmin OTA updater")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let release = crate::ota::fetch_latest_github_release(&client, &proxy_prefix).await?;

        let asset = crate::ota::supported_release_asset(&release)
            .ok_or_else(|| "No supported OTA asset found in latest release".to_string())?;

        if asset.size > crate::ota::MAX_OTA_BYTES {
            return Err(format!(
                "OTA asset is too large: {} bytes exceeds {} bytes",
                asset.size,
                crate::ota::MAX_OTA_BYTES
            ));
        }

        let download_url = format!("{}{}", proxy_prefix, asset.browser_download_url);
        let bytes = client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| format!("Failed to download OTA asset: {}", e))?
            .error_for_status()
            .map_err(|e| format!("OTA asset download failed: {}", e))?
            .bytes()
            .await
            .map_err(|e| format!("Failed to read OTA asset: {}", e))?;

        if bytes.len() as u64 > crate::ota::MAX_OTA_BYTES {
            return Err(format!(
                "OTA asset is too large: {} bytes exceeds {} bytes",
                bytes.len(),
                crate::ota::MAX_OTA_BYTES
            ));
        }

        crate::ota::handle_ota_upload(&bytes)
    }
    .await;

    match result {
        Ok(response) => {
            let message = if response.validation.valid {
                "Online OTA downloaded and validated"
            } else {
                "Online OTA downloaded but validation failed"
            };
            (
                StatusCode::OK,
                Json(ApiResponse::success_with_message(message, response)),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<crate::models::OtaUploadResponse>::error(
                format!("Failed: {}", e),
            )),
        ),
    }
}

/// POST /api/ota/apply
pub async fn apply_ota_handler(
    Json(req): Json<crate::models::OtaApplyRequest>,
) -> impl IntoResponse {
    match crate::ota::apply_ota_update(req.restart_now) {
        Ok(message) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                &message,
                json!({ "applied": true }),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

/// POST /api/ota/cancel
pub async fn cancel_ota_handler() -> impl IntoResponse {
    match crate::ota::cancel_pending_update() {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::success_with_message(
                "Update cancelled",
                json!({}),
            )),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(ApiResponse::<serde_json::Value>::error(format!(
                "Failed: {}",
                e
            ))),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modem_manager::SimIdentity;

    #[test]
    fn enriches_enabled_esim_profile_from_current_sim_identity() {
        let mut profiles = vec![
            EsimProfile {
                iccid: "profile-a".to_string(),
                state: "disabled".to_string(),
                ..Default::default()
            },
            EsimProfile {
                iccid: "profile-b".to_string(),
                state: "disabled".to_string(),
                ..Default::default()
            },
        ];
        let identity = SimIdentity {
            iccid: "profile-b".to_string(),
            imsi: "234336".to_string(),
            operator_id: "234336".to_string(),
        };

        enrich_profiles_with_current_identity(&mut profiles, &identity);

        assert_eq!(profiles[1].state, "enabled");
        assert_eq!(profiles[1].imsi.as_deref(), Some("234336"));
        assert_eq!(profiles[1].mcc.as_deref(), Some("234"));
        assert_eq!(profiles[1].mnc.as_deref(), Some("336"));
        assert!(profiles[0].mcc.is_none());
    }

    #[test]
    fn splits_five_digit_operator_codes_for_profile_enrichment() {
        assert_eq!(
            split_profile_operator_code("46002"),
            ("460".to_string(), "02".to_string())
        );
    }
}
