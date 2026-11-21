use std::path::Path;
use std::sync::Mutex;

use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::banks::ALL_BANKS;

pub struct DbState(pub Mutex<Connection>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name_en: String,
    pub name_th: String,
    pub kind: String,
    pub bank_code: Option<String>,
    pub color: String,
    pub uses_shared_album: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name_en: String,
    pub name_th: String,
    pub kind: String,
    pub icon: String,
    pub color: String,
    pub is_system: bool,
    pub sort_order: i32,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub disabled: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordItem {
    pub id: String,
    pub kind: String,
    pub amount_satang: i64,
    pub currency: String,
    pub local_date: String,
    pub local_time: Option<String>,
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub payee_raw: Option<String>,
    pub note: Option<String>,
    pub source: String,
    pub parse_status: String,
    pub is_reviewed: bool,
    pub recurring_rule_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub slip_ref: Option<String>,
    pub slip_qr_raw: Option<String>,
    pub slip_ocr_text: Option<String>,
    pub slip_image_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordInput {
    pub id: Option<String>,
    pub kind: String,
    pub amount_satang: i64,
    pub local_date: String,
    pub local_time: Option<String>,
    pub account_id: Option<String>,
    pub category_id: Option<String>,
    pub payee_raw: Option<String>,
    pub note: Option<String>,
    pub tag_ids: Vec<String>,
    pub recurring_frequency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FutureRecordItem {
    pub rule_id: String,
    pub kind: String,
    pub amount_satang: i64,
    pub due_date: String,
    pub category_id: Option<String>,
    pub note: Option<String>,
    pub frequency: String,
    pub day_of_month: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub month_start_day: i32,
    pub week_start_day: i32,
    pub default_view: String,
    pub allow_shared_album: bool,
    pub owner_names: Vec<String>,
    pub akson_ocr_api_key: String,
    pub akson_ocr_model: String,
    pub english_ocr_engine: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            month_start_day: 1,
            week_start_day: 0,
            default_view: "monthly".to_string(),
            allow_shared_album: false,
            owner_names: Vec::new(),
            akson_ocr_api_key: String::new(),
            akson_ocr_model: "AksonOCR-preview".to_string(),
            english_ocr_engine: "mlkit_latin".to_string(),
        }
    }
}

pub fn init_db(db_path: &Path) -> Result<Connection, String> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create db dir: {e}"))?;
    }
    let conn = Connection::open(db_path).map_err(|e| format!("open sqlite: {e}"))?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            name_en TEXT NOT NULL,
            name_th TEXT NOT NULL,
            kind TEXT NOT NULL,
            bank_code TEXT,
            color TEXT NOT NULL,
            uses_shared_album INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name_en TEXT NOT NULL,
            name_th TEXT NOT NULL,
            kind TEXT NOT NULL,
            icon TEXT NOT NULL,
            color TEXT NOT NULL,
            is_system INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            archived INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#3b82f6',
            disabled INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS recurring_rules (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            amount_satang INTEGER NOT NULL,
            category_id TEXT,
            account_id TEXT,
            note TEXT,
            frequency TEXT NOT NULL,
            day_of_month INTEGER,
            next_run_date TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS records (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            amount_satang INTEGER NOT NULL,
            currency TEXT NOT NULL DEFAULT 'THB',
            local_date TEXT NOT NULL,
            local_time TEXT,
            account_id TEXT,
            category_id TEXT,
            payee_raw TEXT,
            payee_normalized TEXT,
            note TEXT,
            source TEXT NOT NULL DEFAULT 'manual',
            parse_status TEXT NOT NULL DEFAULT 'ok',
            is_reviewed INTEGER NOT NULL DEFAULT 1,
            recurring_rule_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_records_date ON records(local_date DESC);
        CREATE INDEX IF NOT EXISTS idx_records_kind ON records(kind);
        CREATE INDEX IF NOT EXISTS idx_records_category ON records(category_id);
        CREATE INDEX IF NOT EXISTS idx_records_account ON records(account_id);

        CREATE TABLE IF NOT EXISTS record_tags (
            record_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (record_id, tag_id),
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS slips (
            id TEXT PRIMARY KEY,
            record_id TEXT NOT NULL,
            media_store_id TEXT,
            content_sha256 TEXT NOT NULL UNIQUE,
            sending_bank TEXT,
            trans_ref TEXT,
            qr_raw TEXT,
            ocr_text TEXT,
            image_path TEXT,
            parsed_at TEXT NOT NULL,
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_slips_bank_ref
            ON slips(sending_bank, trans_ref)
            WHERE sending_bank IS NOT NULL AND trans_ref IS NOT NULL;

        CREATE TABLE IF NOT EXISTS payee_rules (
            id TEXT PRIMARY KEY,
            pattern TEXT NOT NULL UNIQUE,
            category_id TEXT NOT NULL,
            kind_override TEXT,
            hit_count INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS budgets (
            id TEXT PRIMARY KEY,
            scope_type TEXT NOT NULL,
            scope_id TEXT,
            period_type TEXT NOT NULL DEFAULT 'monthly',
            limit_satang INTEGER NOT NULL,
            alert_threshold_pct INTEGER NOT NULL DEFAULT 80,
            active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS activity_log (
            local_date TEXT PRIMARY KEY,
            slips_synced INTEGER NOT NULL DEFAULT 0,
            records_categorized INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
    .map_err(|e| format!("run migrations: {e}"))?;

    // Safe additive migration for existing SQLite databases
    let _ = conn.execute("ALTER TABLE slips ADD COLUMN image_path TEXT", []);

    seed_defaults(&conn)?;
    Ok(conn)
}

fn seed_defaults(conn: &Connection) -> Result<(), String> {
    // Seed all 20 Thai banks/wallets + Cash
    for bank in ALL_BANKS {
        conn.execute(
            "INSERT OR IGNORE INTO accounts (id, name_en, name_th, kind, bank_code, color, uses_shared_album, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            params![
                bank.id,
                bank.name_en,
                bank.name_th,
                bank.kind,
                bank.bot_code,
                bank.color,
                if bank.uses_shared_album { 1 } else { 0 }
            ],
        )
        .map_err(|e| format!("seed bank {}: {e}", bank.id))?;
    }

    conn.execute(
        "INSERT OR IGNORE INTO accounts (id, name_en, name_th, kind, bank_code, color, uses_shared_album, enabled)
         VALUES ('cash', 'Cash', 'เงินสด', 'cash', NULL, '#10b981', 0, 1)",
        [],
    )
    .map_err(|e| format!("seed cash: {e}"))?;

    // 15 default Spending categories + 6 default Income categories
    let default_categories: &[(&str, &str, &str, &str, &str, &str, i32)] = &[
        ("cat_food", "Food", "อาหารและเครื่องดื่ม", "spending", "utensils", "#f97316", 1),
        ("cat_transport", "Transport & car", "เดินทางและรถยนต์", "spending", "car", "#06b6d4", 2),
        ("cat_necessities", "Necessities", "ของใช้จำเป็น", "spending", "shopping-basket", "#eab308", 3),
        ("cat_shopping", "Shopping", "ช้อปปิ้ง", "spending", "shopping-bag", "#ec4899", 4),
        ("cat_entertainment", "Entertainment", "บันเทิง", "spending", "film", "#ef4444", 5),
        ("cat_home", "Home & utilities", "บ้านและสาธารณูปโภค", "spending", "home", "#3b82f6", 6),
        ("cat_health", "Health & wellness", "สุขภาพ", "spending", "heart-pulse", "#14b8a6", 7),
        ("cat_family", "Family & pets", "ครอบครัวและสัตว์เลี้ยง", "spending", "users", "#a855f7", 8),
        ("cat_giving", "Giving", "ให้และทำบุญ", "spending", "gift", "#8b5cf6", 9),
        ("cat_travel", "Travel", "ท่องเที่ยว", "spending", "plane", "#0ea5e9", 10),
        ("cat_education", "Education", "การศึกษา", "spending", "graduation-cap", "#6366f1", 11),
        ("cat_work", "Work & business", "งานและธุรกิจ", "spending", "briefcase", "#d97706", 12),
        ("cat_invest", "Save & invest", "ออมและลงทุน", "spending", "trending-up", "#84cc16", 13),
        ("cat_loans", "Loans & cards", "สินเชื่อและบัตรเครดิต", "spending", "credit-card", "#64748b", 14),
        ("cat_other_spend", "Other", "อื่นๆ", "spending", "ellipsis", "#94a3b8", 15),
        ("cat_salary", "Salary", "เงินเดือน", "income", "wallet", "#22c55e", 1),
        ("cat_wages", "Wages", "ค่าจ้าง", "income", "hand-coins", "#10b981", 2),
        ("cat_gifts_in", "Gifts", "ของขวัญ / โบนัส", "income", "gift", "#84cc16", 3),
        ("cat_trade", "Trade & business", "ค้าขาย / ธุรกิจ", "income", "store", "#14b8a6", 4),
        ("cat_cashback", "Cashback", "เงินคืน", "income", "rotate-ccw", "#06b6d4", 5),
        ("cat_other_in", "Other", "อื่นๆ", "income", "ellipsis", "#94a3b8", 6),
    ];

    for (id, en, th, kind, icon, color, order) in default_categories {
        conn.execute(
            "INSERT OR IGNORE INTO categories (id, name_en, name_th, kind, icon, color, is_system, sort_order, archived)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, 0)",
            params![id, en, th, kind, icon, color, order],
        )
        .map_err(|e| format!("seed category {id}: {e}"))?;
    }

    // Starter Thai merchant auto-categorization rules
    let merchant_seeds: &[(&str, &str)] = &[
        ("7-eleven", "cat_food"),
        ("cp all", "cat_food"),
        ("starbucks", "cat_food"),
        ("cafe amazon", "cat_food"),
        ("central restaurants", "cat_food"),
        ("mk restaurant", "cat_food"),
        ("grabfood", "cat_food"),
        ("lineman", "cat_food"),
        ("bangkok smartcard", "cat_transport"),
        ("bts", "cat_transport"),
        ("mrt", "cat_transport"),
        ("bem", "cat_transport"),
        ("ptt", "cat_transport"),
        ("bangchak", "cat_transport"),
        ("grab", "cat_transport"),
        ("bolt", "cat_transport"),
        ("shopee", "cat_shopping"),
        ("lazada", "cat_shopping"),
        ("lotus", "cat_necessities"),
        ("big c", "cat_necessities"),
        ("makro", "cat_necessities"),
        ("netflix", "cat_entertainment"),
        ("spotify", "cat_entertainment"),
        ("youtube", "cat_entertainment"),
        ("major cineplex", "cat_entertainment"),
        ("sf cinema", "cat_entertainment"),
        ("mea", "cat_home"),
        ("pea", "cat_home"),
        ("mwa", "cat_home"),
        ("ais", "cat_home"),
        ("true", "cat_home"),
        ("dtac", "cat_home"),
    ];
    for (pattern, cat_id) in merchant_seeds {
        let id = format!("seed_{}", pattern.replace(' ', "_"));
        conn.execute(
            "INSERT OR IGNORE INTO payee_rules (id, pattern, category_id, hit_count) VALUES (?1, ?2, ?3, 1)",
            params![id, pattern, cat_id],
        )
        .map_err(|e| format!("seed merchant {pattern}: {e}"))?;
    }

    Ok(())
}

pub fn load_settings(conn: &Connection) -> Result<AppSettings, String> {
    let mut s = AppSettings::default();
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (k, v) = row.map_err(|e| e.to_string())?;
        match k.as_str() {
            "language" => s.language = v,
            "month_start_day" => {
                if let Ok(n) = v.parse::<i32>() {
                    s.month_start_day = n.clamp(1, 28);
                }
            }
            "week_start_day" => {
                if let Ok(n) = v.parse::<i32>() {
                    s.week_start_day = n.clamp(0, 6);
                }
            }
            "default_view" => s.default_view = v,
            "allow_shared_album" => s.allow_shared_album = v == "true",
            "owner_names" => {
                if let Ok(names) = serde_json::from_str::<Vec<String>>(&v) {
                    s.owner_names = names;
                }
            }
            "akson_ocr_api_key" => s.akson_ocr_api_key = v,
            "akson_ocr_model" => s.akson_ocr_model = v,
            "english_ocr_engine" => s.english_ocr_engine = v,
            _ => {}
        }
    }
    Ok(s)
}

pub fn save_settings(conn: &Connection, s: &AppSettings) -> Result<(), String> {
    let pairs: Vec<(&str, String)> = vec![
        ("language", s.language.clone()),
        ("month_start_day", s.month_start_day.clamp(1, 28).to_string()),
        ("week_start_day", s.week_start_day.clamp(0, 6).to_string()),
        ("default_view", s.default_view.clone()),
        ("allow_shared_album", s.allow_shared_album.to_string()),
        (
            "owner_names",
            serde_json::to_string(&s.owner_names).unwrap_or_else(|_| "[]".to_string()),
        ),
        ("akson_ocr_api_key", s.akson_ocr_api_key.clone()),
        ("akson_ocr_model", s.akson_ocr_model.clone()),
        ("english_ocr_engine", s.english_ocr_engine.clone()),
    ];
    for (k, v) in pairs {
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, v],
        )
        .map_err(|e| format!("save setting {k}: {e}"))?;
    }
    Ok(())
}

pub fn normalize_payee(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn guess_category_for_payee(conn: &Connection, payee_raw: &str) -> Option<String> {
    let norm = normalize_payee(payee_raw);
    if norm.is_empty() {
        return None;
    }
    // Exact match first
    if let Ok(Some(cat)) = conn
        .query_row(
            "SELECT category_id FROM payee_rules WHERE pattern = ?1 ORDER BY hit_count DESC LIMIT 1",
            params![norm],
            |r| r.get::<_, String>(0),
        )
        .optional()
    {
        return Some(cat);
    }
    // Substring match against rules
    if let Ok(mut stmt) =
        conn.prepare("SELECT pattern, category_id FROM payee_rules ORDER BY hit_count DESC, length(pattern) DESC")
    {
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                if !row.0.is_empty() && norm.contains(&row.0) {
                    return Some(row.1);
                }
            }
        }
    }
    None
}

pub fn learn_payee_category(conn: &Connection, payee_raw: &str, category_id: &str) {
    let norm = normalize_payee(payee_raw);
    if norm.len() < 2 {
        return;
    }
    let id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO payee_rules (id, pattern, category_id, hit_count)
         VALUES (?1, ?2, ?3, 1)
         ON CONFLICT(pattern) DO UPDATE SET category_id = excluded.category_id, hit_count = hit_count + 1",
        params![id, norm, category_id],
    );
}

pub fn record_activity_categorized(conn: &Connection) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let _ = conn.execute(
        "INSERT INTO activity_log (local_date, slips_synced, records_categorized)
         VALUES (?1, 0, 1)
         ON CONFLICT(local_date) DO UPDATE SET records_categorized = records_categorized + 1",
        params![today],
    );
}

pub fn record_activity_synced(conn: &Connection, count: i32) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let _ = conn.execute(
        "INSERT INTO activity_log (local_date, slips_synced, records_categorized)
         VALUES (?1, ?2, 0)
         ON CONFLICT(local_date) DO UPDATE SET slips_synced = slips_synced + ?2",
        params![today, count.max(1)],
    );
}

pub fn compute_streaks(conn: &Connection) -> (i32, i32, i32) {
    let today = Local::now().date_naive();
    let mut sync_streak = 0;
    let mut cat_streak = 0;

    for i in 0..365 {
        let d = (today - Duration::days(i)).format("%Y-%m-%d").to_string();
        let row: Option<(i32, i32)> = conn
            .query_row(
                "SELECT slips_synced, records_categorized FROM activity_log WHERE local_date = ?1",
                params![d],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .unwrap_or(None);
        match row {
            Some((s, _)) if s > 0 => sync_streak += 1,
            _ if i == 0 => {} // today may not have synced yet
            _ => break,
        }
    }

    for i in 0..365 {
        let d = (today - Duration::days(i)).format("%Y-%m-%d").to_string();
        let row: Option<(i32, i32)> = conn
            .query_row(
                "SELECT slips_synced, records_categorized FROM activity_log WHERE local_date = ?1",
                params![d],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .unwrap_or(None);
        match row {
            Some((_, c)) if c > 0 => cat_streak += 1,
            _ if i == 0 => {}
            _ => break,
        }
    }

    let best = sync_streak.max(cat_streak);
    let stored_best: i32 = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'best_streak'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let final_best = best.max(stored_best);
    if final_best > stored_best {
        let _ = conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('best_streak', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![final_best.to_string()],
        );
    }
    (sync_streak, cat_streak, final_best)
}

pub fn list_accounts(conn: &Connection) -> Result<Vec<Account>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name_en, name_th, kind, bank_code, color, uses_shared_album, enabled
             FROM accounts ORDER BY CASE kind WHEN 'cash' THEN 2 ELSE 1 END, name_en ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Account {
                id: r.get(0)?,
                name_en: r.get(1)?,
                name_th: r.get(2)?,
                kind: r.get(3)?,
                bank_code: r.get(4)?,
                color: r.get(5)?,
                uses_shared_album: r.get::<_, i32>(6)? != 0,
                enabled: r.get::<_, i32>(7)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name_en, name_th, kind, icon, color, is_system, sort_order, archived
             FROM categories WHERE archived = 0 ORDER BY kind, sort_order, name_en",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name_en: r.get(1)?,
                name_th: r.get(2)?,
                kind: r.get(3)?,
                icon: r.get(4)?,
                color: r.get(5)?,
                is_system: r.get::<_, i32>(6)? != 0,
                sort_order: r.get(7)?,
                archived: r.get::<_, i32>(8)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn list_tags(conn: &Connection) -> Result<Vec<Tag>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, disabled, sort_order FROM tags ORDER BY disabled, sort_order, name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Tag {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                disabled: r.get::<_, i32>(3)? != 0,
                sort_order: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn fetch_tags_for_record(conn: &Connection, record_id: &str) -> Vec<String> {
    let Ok(mut stmt) = conn.prepare("SELECT tag_id FROM record_tags WHERE record_id = ?1") else {
        return Vec::new();
    };
    stmt.query_map(params![record_id], |r| r.get::<_, String>(0))
        .map(|it| it.flatten().collect())
        .unwrap_or_default()
}

pub fn upsert_record(conn: &Connection, input: &RecordInput) -> Result<RecordItem, String> {
    let now = Local::now().to_rfc3339();
    let id = input
        .id
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let payee_norm = input.payee_raw.as_deref().map(normalize_payee);

    // Optional recurring rule creation
    let mut recurring_rule_id: Option<String> = None;
    if let Some(freq) = input.recurring_frequency.as_deref().filter(|f| !f.is_empty() && *f != "none") {
        let rule_id = Uuid::new_v4().to_string();
        let base_date = NaiveDate::parse_from_str(&input.local_date, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().date_naive());
        let next_date = match freq {
            "weekly" => base_date + Duration::days(7),
            "biweekly" => base_date + Duration::days(14),
            "yearly" => base_date.with_year(base_date.year() + 1).unwrap_or(base_date + Duration::days(365)),
            _ => add_one_month(base_date),
        };
        conn.execute(
            "INSERT INTO recurring_rules (id, kind, amount_satang, category_id, account_id, note, frequency, day_of_month, next_run_date, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1)",
            params![
                rule_id,
                input.kind,
                input.amount_satang,
                input.category_id,
                input.account_id,
                input.note,
                freq,
                base_date.day() as i32,
                next_date.format("%Y-%m-%d").to_string()
            ],
        )
        .map_err(|e| format!("insert recurring rule: {e}"))?;
        recurring_rule_id = Some(rule_id);
    }

    let exists: bool = conn
        .query_row("SELECT 1 FROM records WHERE id = ?1", params![id], |_| Ok(true))
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or(false);

    if exists {
        conn.execute(
            "UPDATE records
             SET kind = ?2, amount_satang = ?3, local_date = ?4, local_time = ?5,
                 account_id = ?6, category_id = ?7, payee_raw = ?8, payee_normalized = ?9,
                 note = ?10, parse_status = 'ok', is_reviewed = 1, updated_at = ?11
             WHERE id = ?1",
            params![
                id,
                input.kind,
                input.amount_satang,
                input.local_date,
                input.local_time,
                input.account_id,
                input.category_id,
                input.payee_raw,
                payee_norm,
                input.note,
                now
            ],
        )
        .map_err(|e| format!("update record: {e}"))?;
    } else {
        conn.execute(
            "INSERT INTO records (
                id, kind, amount_satang, currency, local_date, local_time,
                account_id, category_id, payee_raw, payee_normalized, note,
                source, parse_status, is_reviewed, recurring_rule_id, created_at, updated_at
             ) VALUES (?1, ?2, ?3, 'THB', ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'manual', 'ok', 1, ?11, ?12, ?12)",
            params![
                id,
                input.kind,
                input.amount_satang,
                input.local_date,
                input.local_time,
                input.account_id,
                input.category_id,
                input.payee_raw,
                payee_norm,
                input.note,
                recurring_rule_id,
                now
            ],
        )
        .map_err(|e| format!("insert record: {e}"))?;
    }

    // Sync tags
    conn.execute("DELETE FROM record_tags WHERE record_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    for tag_id in &input.tag_ids {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
            params![id, tag_id],
        );
    }

    // Learn payee -> category mapping for auto-categorization
    if let (Some(payee), Some(cat)) = (input.payee_raw.as_deref(), input.category_id.as_deref()) {
        learn_payee_category(conn, payee, cat);
    }
    if input.category_id.is_some() {
        record_activity_categorized(conn);
    }

    get_record_by_id(conn, &id)?.ok_or_else(|| "record disappeared after upsert".to_string())
}

pub fn get_record_by_id(conn: &Connection, id: &str) -> Result<Option<RecordItem>, String> {
    let row = conn
        .query_row(
            "SELECT r.id, r.kind, r.amount_satang, r.currency, r.local_date, r.local_time,
                    r.account_id, r.category_id, r.payee_raw, r.note, r.source, r.parse_status,
                    r.is_reviewed, r.recurring_rule_id, s.trans_ref, s.qr_raw, s.ocr_text, s.image_path, r.created_at
             FROM records r
             LEFT JOIN slips s ON s.record_id = r.id
             WHERE r.id = ?1",
            params![id],
            |r| {
                Ok(RecordItem {
                    id: r.get(0)?,
                    kind: r.get(1)?,
                    amount_satang: r.get(2)?,
                    currency: r.get(3)?,
                    local_date: r.get(4)?,
                    local_time: r.get(5)?,
                    account_id: r.get(6)?,
                    category_id: r.get(7)?,
                    payee_raw: r.get(8)?,
                    note: r.get(9)?,
                    source: r.get(10)?,
                    parse_status: r.get(11)?,
                    is_reviewed: r.get::<_, i32>(12)? != 0,
                    recurring_rule_id: r.get(13)?,
                    tag_ids: Vec::new(),
                    slip_ref: r.get(14)?,
                    slip_qr_raw: r.get(15)?,
                    slip_ocr_text: r.get(16)?,
                    slip_image_path: r.get(17)?,
                    created_at: r.get(18)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(row.map(|mut rec| {
        rec.tag_ids = fetch_tags_for_record(conn, &rec.id);
        rec
    }))
}

pub fn add_one_month(date: NaiveDate) -> NaiveDate {
    let (y, m) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    let mut day = date.day();
    while day >= 28 {
        if let Some(d) = NaiveDate::from_ymd_opt(y, m, day) {
            return d;
        }
        day -= 1;
    }
    NaiveDate::from_ymd_opt(y, m, day).unwrap_or(date + Duration::days(30))
}
