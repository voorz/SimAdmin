//! 数据库模块
//!
//! 使用 SQLite 存储短信历史记录和通话记录

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const BEIJING_UTC_OFFSET_SECONDS: i32 = 8 * 60 * 60;
const SMS_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// 短信记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub id: i64,
    pub direction: String,    // "incoming" 或 "outgoing"
    pub phone_number: String, // 发件人或收件人
    pub content: String,      // 短信内容
    pub timestamp: String,    // ISO 8601 格式时间
    pub status: String,       // "pending", "sent", "failed", "received"
    pub pdu: Option<String>,  // 原始 PDU（如果有）
}

/// 通话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    pub id: i64,
    pub direction: String,        // "incoming" / "outgoing" / "missed"
    pub phone_number: String,     // 电话号码
    pub duration: i64,            // 通话时长（秒）
    pub start_time: String,       // 开始时间 ISO 8601
    pub end_time: Option<String>, // 结束时间 ISO 8601
    pub answered: bool,           // 是否接通
}

/// 短信统计
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SmsStats {
    pub total: i64,
    pub incoming: i64,
    pub outgoing: i64,
    #[serde(default)]
    pub pushed: i64,
    #[serde(default)]
    pub push_attempted: i64,
}

/// 通话统计
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CallStats {
    pub total: i64,
    pub incoming: i64,
    pub outgoing: i64,
    pub missed: i64,
    pub total_duration: i64, // 总通话时长（秒）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmscCacheEntry {
    pub sms_center: String,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnNumberCacheEntry {
    pub phone_numbers: Vec<String>,
    pub source: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EsimProfileCacheEntry {
    pub iccid: String,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub profile_class: Option<String>,
    pub imsi: Option<String>,
    pub msisdn: Option<String>,
    pub smsc: Option<String>,
    pub smdp: Option<String>,
    pub isdp_aid: Option<String>,
    pub mcc: Option<String>,
    pub mnc: Option<String>,
    pub updated_at: String,
}

/// 数据库管理器
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

fn beijing_offset() -> FixedOffset {
    FixedOffset::east_opt(BEIJING_UTC_OFFSET_SECONDS).expect("valid Beijing UTC offset")
}

pub fn beijing_sms_now_string() -> String {
    Utc::now()
        .with_timezone(&beijing_offset())
        .format(SMS_TIMESTAMP_FORMAT)
        .to_string()
}

pub fn normalize_sms_timestamp_for_display(timestamp: &str) -> Option<String> {
    let timestamp = timestamp.trim();
    if timestamp.is_empty() {
        return None;
    }

    if let Some(parsed) = parse_sms_timestamp_with_offset(timestamp) {
        return Some(parsed);
    }

    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(timestamp, format) {
            return Some(parsed.format(SMS_TIMESTAMP_FORMAT).to_string());
        }
    }

    None
}

fn parse_sms_timestamp_with_offset(timestamp: &str) -> Option<String> {
    let timestamp = timestamp.replace(' ', "T");

    if let Ok(parsed) = DateTime::parse_from_rfc3339(&timestamp) {
        return Some(
            parsed
                .with_timezone(&beijing_offset())
                .format(SMS_TIMESTAMP_FORMAT)
                .to_string(),
        );
    }

    let offset_start = timestamp
        .char_indices()
        .rev()
        .find_map(|(index, ch)| (index > 10 && matches!(ch, '+' | '-')).then_some(index))?;

    let (datetime, offset) = timestamp.split_at(offset_start);
    let normalized_offset = match offset.len() {
        3 => format!("{offset}:00"),
        5 if !offset.contains(':') => format!("{}:{}", &offset[..3], &offset[3..]),
        _ => offset.to_string(),
    };
    let candidate = format!("{datetime}{normalized_offset}");

    DateTime::parse_from_rfc3339(&candidate).ok().map(|parsed| {
        parsed
            .with_timezone(&beijing_offset())
            .format(SMS_TIMESTAMP_FORMAT)
            .to_string()
    })
}

fn sms_timestamp_for_storage(timestamp: &str) -> String {
    normalize_sms_timestamp_for_display(timestamp).unwrap_or_else(beijing_sms_now_string)
}

fn sms_timestamp_for_display(timestamp: String) -> String {
    normalize_sms_timestamp_for_display(&timestamp).unwrap_or(timestamp)
}

fn sms_message_from_row(row: &Row<'_>) -> Result<SmsMessage> {
    let timestamp: String = row.get(4)?;
    Ok(SmsMessage {
        id: row.get(0)?,
        direction: row.get(1)?,
        phone_number: row.get(2)?,
        content: row.get(3)?,
        timestamp: sms_timestamp_for_display(timestamp),
        status: row.get(5)?,
        pdu: row.get(6)?,
    })
}

fn normalize_existing_sms_timestamps(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, timestamp FROM sms_messages")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut updates = Vec::new();
    for row in rows {
        let (id, timestamp) = row?;
        if let Some(normalized) = normalize_sms_timestamp_for_display(&timestamp) {
            if normalized != timestamp {
                updates.push((id, normalized));
            }
        }
    }
    drop(stmt);

    for (id, timestamp) in updates {
        conn.execute(
            "UPDATE sms_messages SET timestamp = ?1 WHERE id = ?2",
            params![timestamp, id],
        )?;
    }

    Ok(())
}

fn non_empty_option(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn table_has_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

    for row in rows {
        if row? == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}

impl Database {
    /// 创建或打开数据库
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // 创建短信表（如果不存在）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sms_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                direction TEXT NOT NULL,
                phone_number TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                status TEXT NOT NULL,
                notification_status TEXT NOT NULL DEFAULT 'pending',
                pdu TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        if !table_has_column(&conn, "sms_messages", "notification_status")? {
            conn.execute(
                "ALTER TABLE sms_messages
                 ADD COLUMN notification_status TEXT NOT NULL DEFAULT 'pending'",
                [],
            )?;
        }

        // 创建短信索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sms_timestamp ON sms_messages(timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sms_phone ON sms_messages(phone_number)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sms_notification_status ON sms_messages(notification_status)",
            [],
        )?;
        normalize_existing_sms_timestamps(&conn)?;

        // 创建通话记录表（如果不存在）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS call_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                direction TEXT NOT NULL,
                phone_number TEXT NOT NULL,
                duration INTEGER DEFAULT 0,
                start_time TEXT NOT NULL,
                end_time TEXT,
                answered INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 创建通话记录索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_call_start_time ON call_history(start_time DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_call_phone ON call_history(phone_number)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS smsc_cache (
                identity_key TEXT PRIMARY KEY,
                iccid TEXT,
                imsi TEXT,
                operator_id TEXT,
                sms_center TEXT NOT NULL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS own_number_cache (
                identity_key TEXT PRIMARY KEY,
                iccid TEXT,
                imsi TEXT,
                operator_id TEXT,
                phone_numbers TEXT NOT NULL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS esim_profile_cache (
                iccid TEXT PRIMARY KEY,
                name TEXT,
                provider TEXT,
                profile_class TEXT,
                imsi TEXT,
                msisdn TEXT,
                smsc TEXT,
                smdp TEXT,
                isdp_aid TEXT,
                mcc TEXT,
                mnc TEXT,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_sessions (
                session_hash TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_auth_sessions_expires_at ON auth_sessions(expires_at)",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // ==================== 认证相关方法 ====================

    pub fn auth_is_configured(&self) -> Result<bool> {
        Ok(self.get_auth_config_value("admin_password_hash")?.is_some())
    }

    pub fn get_auth_config_value(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value FROM auth_config WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
    }

    pub fn set_auth_config_value(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO auth_config (key, value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn replace_admin_password_hash(&self, password_hash: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().timestamp();
        tx.execute(
            "INSERT INTO auth_config (key, value, updated_at)
             VALUES ('admin_password_hash', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at",
            params![password_hash, now],
        )?;
        tx.execute("DELETE FROM auth_sessions", [])?;
        tx.commit()?;
        Ok(())
    }

    pub fn clear_admin_auth(&self) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM auth_config WHERE key = 'admin_password_hash'",
            [],
        )?;
        tx.execute("DELETE FROM auth_sessions", [])?;
        tx.commit()?;
        Ok(())
    }

    pub fn insert_auth_session(&self, session_hash: &str, ttl_seconds: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "INSERT INTO auth_sessions (session_hash, created_at, expires_at)
             VALUES (?1, ?2, ?3)",
            params![session_hash, now, now + ttl_seconds],
        )?;
        conn.execute(
            "DELETE FROM auth_sessions WHERE expires_at <= ?1",
            params![now],
        )?;
        Ok(())
    }

    pub fn auth_session_valid(&self, session_hash: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "DELETE FROM auth_sessions WHERE expires_at <= ?1",
            params![now],
        )?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM auth_sessions
             WHERE session_hash = ?1 AND expires_at > ?2",
            params![session_hash, now],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn delete_auth_session(&self, session_hash: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM auth_sessions WHERE session_hash = ?1",
            params![session_hash],
        )?;
        Ok(())
    }

    pub fn refresh_auth_session(&self, session_hash: &str, ttl_seconds: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        conn.execute(
            "UPDATE auth_sessions SET expires_at = ?1 WHERE session_hash = ?2",
            params![now + ttl_seconds, session_hash],
        )?;
        Ok(())
    }

    // ==================== 短信相关方法 ====================

    /// 插入新短信
    pub fn insert_sms(
        &self,
        direction: &str,
        phone_number: &str,
        content: &str,
        status: &str,
        pdu: Option<&str>,
    ) -> Result<i64> {
        let timestamp = beijing_sms_now_string();
        self.insert_sms_at(direction, phone_number, content, &timestamp, status, pdu)
    }

    pub fn insert_sms_at(
        &self,
        direction: &str,
        phone_number: &str,
        content: &str,
        timestamp: &str,
        status: &str,
        pdu: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let timestamp = sms_timestamp_for_storage(timestamp);
        conn.execute(
            "INSERT INTO sms_messages (direction, phone_number, content, timestamp, status, pdu)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![direction, phone_number, content, timestamp, status, pdu],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Check whether an SMS marker has already been stored.
    pub fn sms_exists_by_pdu(&self, pdu: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages WHERE pdu = ?1",
            params![pdu],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn incoming_sms_exists_by_timestamp(
        &self,
        phone_number: &str,
        content: &str,
        timestamp: &str,
    ) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let normalized_timestamp = normalize_sms_timestamp_for_display(timestamp)
            .unwrap_or_else(|| timestamp.trim().to_string());
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages
             WHERE direction = 'incoming'
               AND phone_number = ?1
               AND content = ?2
               AND (timestamp = ?3 OR timestamp = ?4)",
            params![phone_number, content, timestamp, normalized_timestamp],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn incoming_sms_exists_by_legacy_content(
        &self,
        phone_number: &str,
        content: &str,
    ) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages
             WHERE direction = 'incoming'
               AND phone_number = ?1
               AND content = ?2
               AND (pdu IS NULL OR pdu NOT LIKE 'mmfp:%')",
            params![phone_number, content],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// 获取所有短信（分页）
    pub fn get_sms_messages(
        &self,
        limit: i64,
        offset: i64,
        direction: Option<&str>,
    ) -> Result<Vec<SmsMessage>> {
        let conn = self.conn.lock().unwrap();
        match direction {
            Some(direction) => {
                let mut stmt = conn.prepare(
                    "SELECT id, direction, phone_number, content, timestamp, status, pdu
                     FROM sms_messages
                     WHERE direction = ?1
                     ORDER BY timestamp DESC
                     LIMIT ?2 OFFSET ?3",
                )?;

                let messages =
                    stmt.query_map(params![direction, limit, offset], sms_message_from_row)?;

                let mut result = Vec::new();
                for message in messages {
                    result.push(message?);
                }

                Ok(result)
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT id, direction, phone_number, content, timestamp, status, pdu
                     FROM sms_messages
                     ORDER BY timestamp DESC
                     LIMIT ?1 OFFSET ?2",
                )?;

                let messages = stmt.query_map(params![limit, offset], sms_message_from_row)?;

                let mut result = Vec::new();
                for message in messages {
                    result.push(message?);
                }

                Ok(result)
            }
        }
    }

    /// 获取与特定号码的对话历史
    pub fn get_sms_conversation(&self, phone_number: &str, limit: i64) -> Result<Vec<SmsMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, direction, phone_number, content, timestamp, status, pdu
             FROM sms_messages
             WHERE phone_number = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let messages = stmt.query_map(params![phone_number, limit], sms_message_from_row)?;

        let mut result = Vec::new();
        for message in messages {
            result.push(message?);
        }

        Ok(result)
    }

    /// 更新短信通知转发状态："pending", "success", "failed", "skipped"
    pub fn update_sms_notification_status(&self, id: i64, status: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sms_messages SET notification_status = ?1 WHERE id = ?2",
            params![status, id],
        )
    }

    /// 获取短信统计
    pub fn get_sms_stats(&self) -> Result<SmsStats> {
        let conn = self.conn.lock().unwrap();

        let total: i64 =
            conn.query_row("SELECT COUNT(*) FROM sms_messages", [], |row| row.get(0))?;

        let incoming: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages
             WHERE direction = 'incoming' AND status = 'received'",
            [],
            |row| row.get(0),
        )?;

        let outgoing: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages WHERE direction = 'outgoing'",
            [],
            |row| row.get(0),
        )?;

        let pushed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages
             WHERE direction = 'incoming'
               AND status = 'received'
               AND notification_status = 'success'",
            [],
            |row| row.get(0),
        )?;

        let push_attempted: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sms_messages
             WHERE direction = 'incoming'
               AND status = 'received'
               AND notification_status IN ('success', 'failed')",
            [],
            |row| row.get(0),
        )?;

        Ok(SmsStats {
            total,
            incoming,
            outgoing,
            pushed,
            push_attempted,
        })
    }

    /// 删除所有短信
    pub fn clear_all_sms(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sms_messages", [])?;
        Ok(())
    }

    /// 删除单条短信
    pub fn delete_sms(&self, id: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sms_messages WHERE id = ?1", params![id])
    }

    /// 删除一个对话的所有短信
    pub fn delete_sms_conversation(&self, phone_number: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM sms_messages WHERE phone_number = ?1",
            params![phone_number],
        )
    }

    /// 按短信 ID 和对话号码批量删除
    pub fn delete_sms_batch(&self, ids: &[i64], phone_numbers: &[String]) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let mut deleted = 0usize;

        for phone_number in phone_numbers {
            deleted += tx.execute(
                "DELETE FROM sms_messages WHERE phone_number = ?1",
                params![phone_number],
            )?;
        }

        for id in ids {
            deleted += tx.execute("DELETE FROM sms_messages WHERE id = ?1", params![id])?;
        }

        tx.commit()?;
        Ok(deleted)
    }

    pub fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("VACUUM")?;
        Ok(())
    }

    // ==================== SMSC cache ====================

    pub fn upsert_smsc_cache(
        &self,
        identity_key: &str,
        iccid: &str,
        imsi: &str,
        operator_id: &str,
        sms_center: &str,
        source: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let updated_at = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO smsc_cache (
                identity_key, iccid, imsi, operator_id, sms_center, source, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(identity_key) DO UPDATE SET
                iccid = excluded.iccid,
                imsi = excluded.imsi,
                operator_id = excluded.operator_id,
                sms_center = excluded.sms_center,
                source = excluded.source,
                updated_at = excluded.updated_at",
            params![
                identity_key,
                iccid,
                imsi,
                operator_id,
                sms_center,
                source,
                updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_smsc_cache(&self, identity_keys: &[String]) -> Result<Option<SmscCacheEntry>> {
        let conn = self.conn.lock().unwrap();
        for key in identity_keys {
            let entry = conn
                .query_row(
                    "SELECT sms_center, source, updated_at
                     FROM smsc_cache
                     WHERE identity_key = ?1",
                    params![key],
                    |row| {
                        Ok(SmscCacheEntry {
                            sms_center: row.get(0)?,
                            source: row.get(1)?,
                            updated_at: row.get(2)?,
                        })
                    },
                )
                .optional()?;
            if entry.is_some() {
                return Ok(entry);
            }
        }
        Ok(None)
    }

    // ==================== Own number cache ====================

    pub fn upsert_own_number_cache(
        &self,
        identity_key: &str,
        iccid: &str,
        imsi: &str,
        operator_id: &str,
        phone_numbers: &[String],
        source: &str,
    ) -> Result<()> {
        if phone_numbers.is_empty() {
            return Ok(());
        }

        let conn = self.conn.lock().unwrap();
        let updated_at = Utc::now().to_rfc3339();
        let phone_numbers = phone_numbers.join("\n");
        conn.execute(
            "INSERT INTO own_number_cache (
                identity_key, iccid, imsi, operator_id, phone_numbers, source, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(identity_key) DO UPDATE SET
                iccid = excluded.iccid,
                imsi = excluded.imsi,
                operator_id = excluded.operator_id,
                phone_numbers = excluded.phone_numbers,
                source = excluded.source,
                updated_at = excluded.updated_at",
            params![
                identity_key,
                iccid,
                imsi,
                operator_id,
                phone_numbers,
                source,
                updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_own_number_cache(
        &self,
        identity_keys: &[String],
    ) -> Result<Option<OwnNumberCacheEntry>> {
        let conn = self.conn.lock().unwrap();
        for key in identity_keys {
            let entry = conn
                .query_row(
                    "SELECT phone_numbers, source, updated_at
                     FROM own_number_cache
                     WHERE identity_key = ?1",
                    params![key],
                    |row| {
                        let phone_numbers: String = row.get(0)?;
                        Ok(OwnNumberCacheEntry {
                            phone_numbers: phone_numbers
                                .lines()
                                .map(str::trim)
                                .filter(|line| !line.is_empty())
                                .map(ToString::to_string)
                                .collect(),
                            source: row.get(1)?,
                            updated_at: row.get(2)?,
                        })
                    },
                )
                .optional()?;
            if entry.is_some() {
                return Ok(entry);
            }
        }
        Ok(None)
    }

    // ==================== eSIM Profile cache ====================

    pub fn upsert_esim_profile_cache(&self, entry: &EsimProfileCacheEntry) -> Result<()> {
        if entry.iccid.trim().is_empty() {
            return Ok(());
        }

        let has_profile_data = [
            entry.name.as_deref(),
            entry.provider.as_deref(),
            entry.profile_class.as_deref(),
            entry.imsi.as_deref(),
            entry.msisdn.as_deref(),
            entry.smsc.as_deref(),
            entry.smdp.as_deref(),
            entry.isdp_aid.as_deref(),
            entry.mcc.as_deref(),
            entry.mnc.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| !value.trim().is_empty());

        if !has_profile_data {
            return Ok(());
        }

        let conn = self.conn.lock().unwrap();
        let updated_at = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO esim_profile_cache (
                iccid, name, provider, profile_class, imsi, msisdn, smsc, smdp,
                isdp_aid, mcc, mnc, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(iccid) DO UPDATE SET
                name = COALESCE(excluded.name, esim_profile_cache.name),
                provider = COALESCE(excluded.provider, esim_profile_cache.provider),
                profile_class = COALESCE(excluded.profile_class, esim_profile_cache.profile_class),
                imsi = COALESCE(excluded.imsi, esim_profile_cache.imsi),
                msisdn = COALESCE(excluded.msisdn, esim_profile_cache.msisdn),
                smsc = COALESCE(excluded.smsc, esim_profile_cache.smsc),
                smdp = COALESCE(excluded.smdp, esim_profile_cache.smdp),
                isdp_aid = COALESCE(excluded.isdp_aid, esim_profile_cache.isdp_aid),
                mcc = COALESCE(excluded.mcc, esim_profile_cache.mcc),
                mnc = COALESCE(excluded.mnc, esim_profile_cache.mnc),
                updated_at = excluded.updated_at",
            params![
                entry.iccid.trim(),
                non_empty_option(entry.name.as_deref()),
                non_empty_option(entry.provider.as_deref()),
                non_empty_option(entry.profile_class.as_deref()),
                non_empty_option(entry.imsi.as_deref()),
                non_empty_option(entry.msisdn.as_deref()),
                non_empty_option(entry.smsc.as_deref()),
                non_empty_option(entry.smdp.as_deref()),
                non_empty_option(entry.isdp_aid.as_deref()),
                non_empty_option(entry.mcc.as_deref()),
                non_empty_option(entry.mnc.as_deref()),
                updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_esim_profile_cache(&self, iccid: &str) -> Result<Option<EsimProfileCacheEntry>> {
        let iccid = iccid.trim();
        if iccid.is_empty() {
            return Ok(None);
        }

        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT iccid, name, provider, profile_class, imsi, msisdn, smsc, smdp,
                    isdp_aid, mcc, mnc, updated_at
             FROM esim_profile_cache
             WHERE iccid = ?1",
            params![iccid],
            |row| {
                Ok(EsimProfileCacheEntry {
                    iccid: row.get(0)?,
                    name: row.get(1)?,
                    provider: row.get(2)?,
                    profile_class: row.get(3)?,
                    imsi: row.get(4)?,
                    msisdn: row.get(5)?,
                    smsc: row.get(6)?,
                    smdp: row.get(7)?,
                    isdp_aid: row.get(8)?,
                    mcc: row.get(9)?,
                    mnc: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .optional()
    }

    pub fn list_esim_profile_cache(&self) -> Result<Vec<EsimProfileCacheEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut statement = conn.prepare(
            "SELECT iccid, name, provider, profile_class, imsi, msisdn, smsc, smdp,
                    isdp_aid, mcc, mnc, updated_at
             FROM esim_profile_cache
             ORDER BY updated_at DESC, iccid ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(EsimProfileCacheEntry {
                iccid: row.get(0)?,
                name: row.get(1)?,
                provider: row.get(2)?,
                profile_class: row.get(3)?,
                imsi: row.get(4)?,
                msisdn: row.get(5)?,
                smsc: row.get(6)?,
                smdp: row.get(7)?,
                isdp_aid: row.get(8)?,
                mcc: row.get(9)?,
                mnc: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        rows.collect()
    }

    pub fn delete_esim_profile_cache(&self, iccid: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM esim_profile_cache WHERE iccid = ?1",
            params![iccid.trim()],
        )?;
        Ok(())
    }

    // ==================== 通话记录相关方法 ====================

    /// 插入新通话记录
    pub fn insert_call(&self, direction: &str, phone_number: &str, answered: bool) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let start_time = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO call_history (direction, phone_number, duration, start_time, answered)
             VALUES (?1, ?2, 0, ?3, ?4)",
            params![direction, phone_number, start_time, answered as i32],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// 更新通话记录（通话结束时调用）
    pub fn update_call_end(&self, id: i64, duration: i64, answered: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let end_time = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE call_history SET duration = ?1, end_time = ?2, answered = ?3 WHERE id = ?4",
            params![duration, end_time, answered as i32, id],
        )?;
        Ok(())
    }

    /// 标记通话为未接来电
    pub fn mark_call_missed(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let end_time = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE call_history SET direction = 'missed', end_time = ?1, answered = 0 WHERE id = ?2",
            params![end_time, id],
        )?;
        Ok(())
    }

    /// 获取通话记录（分页）
    pub fn get_call_history(&self, limit: i64, offset: i64) -> Result<Vec<CallRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, direction, phone_number, duration, start_time, end_time, answered
             FROM call_history
             ORDER BY start_time DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let records = stmt.query_map(params![limit, offset], |row| {
            Ok(CallRecord {
                id: row.get(0)?,
                direction: row.get(1)?,
                phone_number: row.get(2)?,
                duration: row.get(3)?,
                start_time: row.get(4)?,
                end_time: row.get(5)?,
                answered: row.get::<_, i32>(6)? != 0,
            })
        })?;

        let mut result = Vec::new();
        for record in records {
            result.push(record?);
        }

        Ok(result)
    }

    /// 获取通话统计
    pub fn get_call_stats(&self) -> Result<CallStats> {
        let conn = self.conn.lock().unwrap();

        let total: i64 =
            conn.query_row("SELECT COUNT(*) FROM call_history", [], |row| row.get(0))?;

        let incoming: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_history WHERE direction = 'incoming'",
            [],
            |row| row.get(0),
        )?;

        let outgoing: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_history WHERE direction = 'outgoing'",
            [],
            |row| row.get(0),
        )?;

        let missed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM call_history WHERE direction = 'missed'",
            [],
            |row| row.get(0),
        )?;

        let total_duration: i64 = conn.query_row(
            "SELECT COALESCE(SUM(duration), 0) FROM call_history WHERE answered = 1",
            [],
            |row| row.get(0),
        )?;

        Ok(CallStats {
            total,
            incoming,
            outgoing,
            missed,
            total_duration,
        })
    }

    /// 删除单条通话记录
    pub fn delete_call(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM call_history WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 删除所有通话记录
    pub fn clear_all_calls(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM call_history", [])?;
        Ok(())
    }
}
