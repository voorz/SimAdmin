//! 配置管理模块
//!
//! 使用 JSON 文件存储用户配置，支持热更新

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

use crate::models::WorkMode;

/// Webhook 配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebhookConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_true")]
    pub forward_sms: bool,
    #[serde(default = "default_true")]
    pub forward_calls: bool,
    #[serde(default = "default_true")]
    pub forward_ddns: bool,
    #[serde(default = "default_true")]
    pub forward_updates: bool,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub secret: String, // 可选的签名密钥
    #[serde(default = "default_sms_template")]
    pub sms_template: String, // 短信 payload 模板
    #[serde(default = "default_call_template")]
    pub call_template: String, // 通话 payload 模板
    #[serde(default = "default_ddns_template")]
    pub ddns_template: String, // DDNS payload 模板
    #[serde(default = "default_update_template")]
    pub update_template: String, // 版本更新 payload 模板
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageChannelConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub forward_sms: bool,
    #[serde(default = "default_true")]
    pub forward_calls: bool,
    #[serde(default = "default_true")]
    pub forward_ddns: bool,
    #[serde(default = "default_true")]
    pub forward_updates: bool,
    #[serde(default = "default_plain_sms_template")]
    pub sms_template: String,
    #[serde(default = "default_plain_call_template")]
    pub call_template: String,
    #[serde(default = "default_plain_ddns_template")]
    pub ddns_template: String,
    #[serde(default = "default_plain_update_template")]
    pub update_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarkConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default = "default_bark_server_url")]
    pub server_url: String,
    #[serde(default)]
    pub device_key: String,
    #[serde(default = "default_sms_title_template")]
    pub title_template: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub sound: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub click_url: String,
    #[serde(default)]
    pub copy: String,
    #[serde(default)]
    pub auto_copy: bool,
    #[serde(default = "default_true")]
    pub save_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPlusConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default)]
    pub token: String,
    #[serde(default = "default_sms_title_template")]
    pub title_template: String,
    #[serde(default)]
    pub topic: String,
    #[serde(default = "default_pushplus_template")]
    pub template: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub option: String,
    #[serde(default)]
    pub callback_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WecomAppConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default)]
    pub corp_id: String,
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub secret: String,
    #[serde(default = "default_wecom_to_user")]
    pub to_user: String,
    #[serde(default)]
    pub to_party: String,
    #[serde(default)]
    pub to_tag: String,
    #[serde(default)]
    pub safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WecomRobotConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingtalkRobotConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub secret: String,
    #[serde(default)]
    pub at_mobiles: String,
    #[serde(default)]
    pub at_all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingtalkAppConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default)]
    pub app_key: String,
    #[serde(default)]
    pub app_secret: String,
    #[serde(default)]
    pub robot_code: String,
    #[serde(default)]
    pub open_conversation_id: String,
    #[serde(default = "default_dingtalk_msg_key")]
    pub msg_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuRobotConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    #[serde(flatten)]
    pub common: MessageChannelConfig,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub chat_id: String,
    #[serde(default)]
    pub parse_mode: String,
    #[serde(default)]
    pub disable_web_page_preview: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default)]
    pub webhook: WebhookConfig,
    #[serde(default)]
    pub bark: BarkConfig,
    #[serde(default)]
    pub pushplus: PushPlusConfig,
    #[serde(default)]
    pub wecom_app: WecomAppConfig,
    #[serde(default)]
    pub wecom_robot: WecomRobotConfig,
    #[serde(default)]
    pub dingtalk_robot: DingtalkRobotConfig,
    #[serde(default)]
    pub dingtalk_app: DingtalkAppConfig,
    #[serde(default)]
    pub feishu_robot: FeishuRobotConfig,
    #[serde(default)]
    pub telegram: TelegramConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Webhook,
    Bark,
    #[serde(rename = "pushplus", alias = "push_plus")]
    PushPlus,
    WecomApp,
    WecomRobot,
    DingtalkRobot,
    DingtalkApp,
    FeishuRobot,
    Telegram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceNetworkConfig {
    #[serde(default)]
    pub ddns: DdnsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VersionUpdateNotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub proxy_prefix: String,
    #[serde(default)]
    pub last_notified_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecurityConfig {
    #[serde(default = "default_true")]
    pub password_protection_enabled: bool,
    #[serde(default = "default_password_min_length")]
    pub password_min_length: u8,
    #[serde(default = "default_true")]
    pub password_require_letters: bool,
    #[serde(default = "default_true")]
    pub password_require_digits: bool,
    #[serde(default = "default_true")]
    pub password_require_symbols: bool,
    #[serde(default = "default_session_ttl_seconds")]
    pub session_ttl_seconds: i64,
    #[serde(default = "default_idle_timeout_seconds")]
    pub idle_timeout_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DdnsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ddns_provider")]
    pub provider: String,
    #[serde(default)]
    pub access_id: String,
    #[serde(default)]
    pub access_secret: String,
    #[serde(default = "default_ddns_interval_seconds")]
    pub interval_seconds: u64,
    #[serde(default = "default_ddns_ttl")]
    pub ttl: u32,
    #[serde(default)]
    pub ipv4: DdnsIpConfig,
    #[serde(default = "default_ddns_ipv6_config")]
    pub ipv6: DdnsIpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DdnsIpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ddns_get_type")]
    pub get_type: String,
    #[serde(default)]
    pub interface_name: String,
    #[serde(default)]
    pub urls: Vec<String>,
    #[serde(default)]
    pub domains: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// 默认短信模板
fn default_sms_template() -> String {
    r#"{
  "msg_type": "text",
  "content": {
    "text": "📱 短信通知\n发送方: {{phone_number}}\n内容: {{content}}\n时间: {{timestamp}}\n本机号码: {{own_number}}"
  }
}"#
    .to_string()
}

/// 默认通话模板
fn default_call_template() -> String {
    r#"{
  "msg_type": "text",
  "content": {
    "text": "📞 来电通知\n号码: {{phone_number}}\n类型: {{direction}}\n时间: {{start_time}}\n时长: {{duration}}秒\n已接听: {{answered}}"
  }
}"#.to_string()
}

fn default_ddns_template() -> String {
    r#"{
  "msg_type": "text",
  "content": {
    "text": "SimAdmin DDNS 通知\n域名: {{domains}}\nIP类型: {{ip_type}}\n新IP: {{new_ip}}\n旧IP: {{old_ip}}\n服务商: {{provider}}\n记录类型: {{record_type}}\n状态: {{status}}\n消息: {{message}}\n更新时间: {{timestamp}}"
  }
}"#
    .to_string()
}

fn default_update_template() -> String {
    r#"{
  "msg_type": "text",
  "content": {
    "text": "SimAdmin 发现新版本\n固件包: {{asset_name}}\n版本号: {{version}}\nCommit: {{commit}}\n构建时间: {{build_time}}\nOTA包 MD5: {{md5}}\n\n请前往 OTA 在线更新模块检测版本，一键下载并升级。"
  }
}"#
    .to_string()
}

fn default_plain_sms_template() -> String {
    "📱 短信通知\n发送方: {{phone_number}}\n内容: {{content}}\n时间: {{timestamp}}\n本机号码: {{own_number}}".to_string()
}

fn default_plain_call_template() -> String {
    "📞 来电通知\n号码: {{phone_number}}\n类型: {{direction}}\n时间: {{start_time}}\n时长: {{duration}}秒\n已接听: {{answered}}".to_string()
}

fn default_plain_ddns_template() -> String {
    "SimAdmin DDNS 通知\n域名: {{domains}}\nIP类型: {{ip_type}}\n新IP: {{new_ip}}\n旧IP: {{old_ip}}\n服务商: {{provider}}\n记录类型: {{record_type}}\n状态: {{status}}\n消息: {{message}}\n更新时间: {{timestamp}}".to_string()
}

fn default_plain_update_template() -> String {
    "SimAdmin 发现新版本\n固件包: {{asset_name}}\n版本号: {{version}}\nCommit: {{commit}}\n构建时间: {{build_time}}\nOTA包 MD5: {{md5}}\n\n请前往 OTA 在线更新模块检测版本，一键下载并升级。".to_string()
}

fn default_sms_title_template() -> String {
    "SimAdmin 短信通知".to_string()
}

fn default_bark_server_url() -> String {
    "https://api.day.app".to_string()
}

fn default_pushplus_template() -> String {
    "txt".to_string()
}

fn default_wecom_to_user() -> String {
    "@all".to_string()
}

fn default_dingtalk_msg_key() -> String {
    "sampleText".to_string()
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            forward_sms: true,
            forward_calls: true,
            forward_ddns: true,
            forward_updates: true,
            headers: HashMap::new(),
            secret: String::new(),
            sms_template: default_sms_template(),
            call_template: default_call_template(),
            ddns_template: default_ddns_template(),
            update_template: default_update_template(),
        }
    }
}

impl Default for MessageChannelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            forward_sms: true,
            forward_calls: true,
            forward_ddns: true,
            forward_updates: true,
            sms_template: default_plain_sms_template(),
            call_template: default_plain_call_template(),
            ddns_template: default_plain_ddns_template(),
            update_template: default_plain_update_template(),
        }
    }
}

impl Default for BarkConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            server_url: default_bark_server_url(),
            device_key: String::new(),
            title_template: default_sms_title_template(),
            group: String::new(),
            sound: String::new(),
            level: String::new(),
            icon: String::new(),
            click_url: String::new(),
            copy: String::new(),
            auto_copy: false,
            save_history: true,
        }
    }
}

impl Default for PushPlusConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            token: String::new(),
            title_template: default_sms_title_template(),
            topic: String::new(),
            template: default_pushplus_template(),
            channel: String::new(),
            option: String::new(),
            callback_url: String::new(),
        }
    }
}

impl Default for WecomAppConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            corp_id: String::new(),
            agent_id: String::new(),
            secret: String::new(),
            to_user: default_wecom_to_user(),
            to_party: String::new(),
            to_tag: String::new(),
            safe: false,
        }
    }
}

impl Default for WecomRobotConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            webhook_url: String::new(),
            key: String::new(),
        }
    }
}

impl Default for DingtalkRobotConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            webhook_url: String::new(),
            access_token: String::new(),
            secret: String::new(),
            at_mobiles: String::new(),
            at_all: false,
        }
    }
}

impl Default for DingtalkAppConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            app_key: String::new(),
            app_secret: String::new(),
            robot_code: String::new(),
            open_conversation_id: String::new(),
            msg_key: default_dingtalk_msg_key(),
        }
    }
}

impl Default for FeishuRobotConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            webhook_url: String::new(),
            token: String::new(),
            secret: String::new(),
        }
    }
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            common: MessageChannelConfig::default(),
            bot_token: String::new(),
            chat_id: String::new(),
            parse_mode: String::new(),
            disable_web_page_preview: true,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            webhook: WebhookConfig::default(),
            bark: BarkConfig::default(),
            pushplus: PushPlusConfig::default(),
            wecom_app: WecomAppConfig::default(),
            wecom_robot: WecomRobotConfig::default(),
            dingtalk_robot: DingtalkRobotConfig::default(),
            dingtalk_app: DingtalkAppConfig::default(),
            feishu_robot: FeishuRobotConfig::default(),
            telegram: TelegramConfig::default(),
        }
    }
}

impl Default for DeviceNetworkConfig {
    fn default() -> Self {
        Self {
            ddns: DdnsConfig::default(),
        }
    }
}

impl Default for VersionUpdateNotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            proxy_prefix: String::new(),
            last_notified_version: None,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            password_protection_enabled: true,
            password_min_length: default_password_min_length(),
            password_require_letters: true,
            password_require_digits: true,
            password_require_symbols: true,
            session_ttl_seconds: default_session_ttl_seconds(),
            idle_timeout_seconds: default_idle_timeout_seconds(),
        }
    }
}

impl Default for DdnsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_ddns_provider(),
            access_id: String::new(),
            access_secret: String::new(),
            interval_seconds: default_ddns_interval_seconds(),
            ttl: default_ddns_ttl(),
            ipv4: DdnsIpConfig {
                enabled: true,
                get_type: default_ddns_get_type(),
                interface_name: String::new(),
                urls: default_ddns_ipv4_urls(),
                domains: Vec::new(),
            },
            ipv6: default_ddns_ipv6_config(),
        }
    }
}

impl Default for DdnsIpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            get_type: default_ddns_get_type(),
            interface_name: String::new(),
            urls: Vec::new(),
            domains: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_channel_accepts_frontend_pushplus_key() {
        assert!(matches!(
            serde_json::from_str::<NotificationChannel>(r#""pushplus""#).unwrap(),
            NotificationChannel::PushPlus
        ));
        assert!(matches!(
            serde_json::from_str::<NotificationChannel>(r#""push_plus""#).unwrap(),
            NotificationChannel::PushPlus
        ));
        assert_eq!(
            serde_json::to_string(&NotificationChannel::PushPlus).unwrap(),
            r#""pushplus""#
        );
    }
}

fn default_ddns_provider() -> String {
    "tencentcloud".to_string()
}

fn default_ddns_interval_seconds() -> u64 {
    300
}

fn default_ddns_ttl() -> u32 {
    600
}

fn default_ddns_get_type() -> String {
    "interface".to_string()
}

fn default_ddns_ipv4_urls() -> Vec<String> {
    vec![
        "https://api.ipify.org".to_string(),
        "https://ip.3322.net".to_string(),
        "https://4.ident.me".to_string(),
        "https://ddns.oray.com/checkip".to_string(),
        "https://4.ipw.cn".to_string(),
    ]
}

fn default_ddns_ipv6_urls() -> Vec<String> {
    vec![
        "https://api6.ipify.org".to_string(),
        "https://speed.neu6.edu.cn/getIP.php".to_string(),
        "https://v6.ident.me".to_string(),
        "https://myip6.ipip.net".to_string(),
        "https://6.ipw.cn".to_string(),
    ]
}

fn default_ddns_ipv6_config() -> DdnsIpConfig {
    DdnsIpConfig {
        enabled: false,
        get_type: default_ddns_get_type(),
        interface_name: String::new(),
        urls: default_ddns_ipv6_urls(),
        domains: Vec::new(),
    }
}

fn default_roaming_allowed() -> bool {
    true
}

fn default_data_enabled() -> bool {
    false
}

fn default_password_min_length() -> u8 {
    8
}

fn default_session_ttl_seconds() -> i64 {
    7 * 24 * 60 * 60
}

fn default_idle_timeout_seconds() -> i64 {
    60 * 60
}

fn default_apn_protocol() -> String {
    "dual".to_string()
}

fn default_apn_auth_method() -> String {
    "chap".to_string()
}

fn default_lpac_path() -> String {
    "/opt/simadmin/lpac/lpac".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApnConfig {
    #[serde(default)]
    pub apn: String,
    #[serde(default = "default_apn_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_apn_auth_method")]
    pub auth_method: String,
}

impl Default for ApnConfig {
    fn default() -> Self {
        Self {
            apn: String::new(),
            protocol: default_apn_protocol(),
            username: String::new(),
            password: String::new(),
            auth_method: default_apn_auth_method(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsimConfig {
    #[serde(default = "default_lpac_path")]
    pub lpac_path: String,
}

impl Default for EsimConfig {
    fn default() -> Self {
        Self {
            lpac_path: default_lpac_path(),
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub webhook: WebhookConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub device_network: DeviceNetworkConfig,
    #[serde(default)]
    pub version_update_notifications: VersionUpdateNotificationConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    /// 是否允许蜂窝数据漫游（写入 ModemManager Simple.Connect 的 allow-roaming）
    #[serde(default = "default_roaming_allowed")]
    pub roaming_allowed: bool,
    #[serde(default = "default_data_enabled")]
    pub data_enabled: bool,
    #[serde(default)]
    pub apn: ApnConfig,
    #[serde(default)]
    pub work_mode: WorkMode,
    #[serde(default)]
    pub esim: EsimConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            webhook: WebhookConfig::default(),
            notifications: NotificationConfig::default(),
            device_network: DeviceNetworkConfig::default(),
            version_update_notifications: VersionUpdateNotificationConfig::default(),
            security: SecurityConfig::default(),
            roaming_allowed: default_roaming_allowed(),
            data_enabled: default_data_enabled(),
            apn: ApnConfig::default(),
            work_mode: WorkMode::default(),
            esim: EsimConfig::default(),
        }
    }
}

fn migrate_legacy_webhook_config(config: &mut AppConfig) {
    if config.notifications.webhook == WebhookConfig::default()
        && config.webhook != WebhookConfig::default()
    {
        config.notifications.webhook = config.webhook.clone();
    }
    config.webhook = config.notifications.webhook.clone();
}

/// 配置管理器
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(config_path: PathBuf) -> Self {
        let mut config = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        warn!(error = %e, "Failed to parse config file, using defaults");
                        AppConfig::default()
                    }
                },
                Err(e) => {
                    warn!(error = %e, "Failed to read config file, using defaults");
                    AppConfig::default()
                }
            }
        } else {
            info!("No config file found, using defaults");
            AppConfig::default()
        };

        migrate_legacy_webhook_config(&mut config);

        let manager = Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        };

        // 保存默认配置（如果文件不存在）
        if !manager.config_path.exists() {
            let _ = manager.save();
        }

        manager
    }

    /// 获取通知配置
    pub fn get_notifications(&self) -> NotificationConfig {
        self.config.read().unwrap().notifications.clone()
    }

    pub fn get_roaming_allowed(&self) -> bool {
        self.config.read().unwrap().roaming_allowed
    }

    pub fn get_data_enabled(&self) -> bool {
        self.config.read().unwrap().data_enabled
    }

    pub fn get_apn_config(&self) -> ApnConfig {
        self.config.read().unwrap().apn.clone()
    }

    pub fn get_work_mode(&self) -> WorkMode {
        self.config.read().unwrap().work_mode
    }

    pub fn get_esim_config(&self) -> EsimConfig {
        self.config.read().unwrap().esim.clone()
    }

    pub fn get_device_network(&self) -> DeviceNetworkConfig {
        self.config.read().unwrap().device_network.clone()
    }

    pub fn get_ddns_config(&self) -> DdnsConfig {
        self.config.read().unwrap().device_network.ddns.clone()
    }

    pub fn get_version_update_notifications(&self) -> VersionUpdateNotificationConfig {
        self.config
            .read()
            .unwrap()
            .version_update_notifications
            .clone()
    }

    pub fn get_security(&self) -> SecurityConfig {
        self.config.read().unwrap().security.clone()
    }

    pub fn set_security(&self, security: SecurityConfig) -> Result<(), String> {
        {
            let mut c = self.config.write().unwrap();
            c.security = security;
        }
        self.save()
    }

    pub fn set_data_enabled(&self, enabled: bool) -> Result<(), String> {
        {
            let mut c = self.config.write().unwrap();
            c.data_enabled = enabled;
        }
        self.save()
    }

    pub fn set_apn_config(&self, apn: ApnConfig) -> Result<(), String> {
        {
            let mut c = self.config.write().unwrap();
            c.apn = apn;
        }
        self.save()
    }

    pub fn set_work_mode(&self, mode: WorkMode) -> Result<(), String> {
        {
            let mut c = self.config.write().unwrap();
            c.work_mode = mode;
        }
        self.save()
    }

    pub fn set_roaming_allowed(&self, allowed: bool) -> Result<(), String> {
        {
            let mut c = self.config.write().unwrap();
            c.roaming_allowed = allowed;
        }
        self.save()
    }

    pub fn set_ddns_config(&self, ddns: DdnsConfig) -> Result<(), String> {
        {
            let mut c = self.config.write().unwrap();
            c.device_network.ddns = ddns;
        }
        self.save()
    }

    pub fn set_last_notified_update_version(&self, version: String) -> Result<(), String> {
        {
            let mut c = self.config.write().unwrap();
            c.version_update_notifications.last_notified_version = Some(version);
        }
        self.save()
    }

    /// 更新通知配置
    pub fn set_notifications(&self, notifications: NotificationConfig) -> Result<(), String> {
        {
            let mut config = self.config.write().unwrap();
            config.webhook = notifications.webhook.clone();
            config.notifications = notifications;
        }
        self.save()
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), String> {
        let config = self.config.read().unwrap();
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

/// 获取默认配置文件路径
pub fn get_default_config_path() -> PathBuf {
    // 尝试 /data/config.json（设备上的持久化目录）
    let device_path = PathBuf::from("/data/config.json");
    if device_path.parent().map(|p| p.exists()).unwrap_or(false) {
        return device_path;
    }

    // 回退到当前目录
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("config.json")
}
