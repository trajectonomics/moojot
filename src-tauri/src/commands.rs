use base64::Engine as _;
use chrono::Local;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::analytics::{
    build_budgets, build_compare_payload, build_home_payload, build_summary_payload, export_csv,
    BudgetItem, ComparePayload, HomePayload, SummaryPayload,
};
use crate::banks::ALL_BANKS;
use crate::categorizer::{
    auto_categorize_uncategorized as cat_auto_categorize, classify_local, delete_user_rule,
    list_user_rules, upsert_user_rule, PayeeRule,
};
use crate::db::{
    fetch_tags_for_record, list_accounts, list_categories, list_tags,
    list_known_media_ids, load_settings, normalize_payee, record_activity_synced,
    remember_scanned_media as remember_scanned_media_rows, save_settings, upsert_record, Account,
    AppSettings, Category, DbState, RecordInput, RecordItem, Tag,
};
use crate::slip_parser::{call_akson_ocr, parse_slip_qr, parse_slip_text};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlipScanCandidate {
    pub media_store_id: String,
    pub content_sha256: String,
    pub album_name: String,
    pub qr_raw: Option<String>,
    pub ocr_text: String,
    pub image_path: Option<String>,
    pub image_base64: Option<String>,
    pub mime_type: Option<String>,
    pub date_added_sec: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlipScanReport {
    pub scanned: usize,
    pub imported: usize,
    pub duplicates: usize,
    pub skipped_non_slip: usize,
    pub needs_review: usize,
    pub akson_calls: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetInput {
    pub id: Option<String>,
    pub scope_type: String,
    pub scope_id: Option<String>,
    pub period_type: String,
    pub limit_satang: i64,
    pub alert_threshold_pct: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInput {
    pub name_en: String,
    pub name_th: String,
    pub kind: String,
    pub icon: String,
    pub color: String,
}

#[tauri::command]
pub fn get_home_data(
    state: State<'_, DbState>,
    period_offset: i32,
    account_ids: Option<Vec<String>>,
) -> Result<HomePayload, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    build_home_payload(&conn, period_offset, account_ids.as_deref())
}

#[tauri::command]
pub fn get_summary_data(
    state: State<'_, DbState>,
    period_offset: i32,
    kind: String,
    account_ids: Option<Vec<String>>,
) -> Result<SummaryPayload, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    build_summary_payload(&conn, period_offset, &kind, account_ids.as_deref())
}

#[tauri::command]
pub fn get_compare_data(
    state: State<'_, DbState>,
    period_offset: i32,
    kind: String,
    months: Option<i32>,
) -> Result<ComparePayload, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    build_compare_payload(&conn, period_offset, &kind, months.unwrap_or(6))
}

#[tauri::command]
pub fn get_budgets(
    state: State<'_, DbState>,
    period_offset: i32,
) -> Result<Vec<BudgetItem>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    build_budgets(&conn, period_offset)
}

#[tauri::command]
pub fn upsert_budget(state: State<'_, DbState>, input: BudgetInput) -> Result<String, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let id = input
        .id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    conn.execute(
        "INSERT INTO budgets (id, scope_type, scope_id, period_type, limit_satang, alert_threshold_pct, active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)
         ON CONFLICT(id) DO UPDATE SET
           scope_type = excluded.scope_type,
           scope_id = excluded.scope_id,
           period_type = excluded.period_type,
           limit_satang = excluded.limit_satang,
           alert_threshold_pct = excluded.alert_threshold_pct,
           active = 1",
        params![
            id,
            input.scope_type,
            input.scope_id,
            input.period_type,
            input.limit_satang,
            input.alert_threshold_pct.clamp(10, 100)
        ],
    )
    .map_err(|e| format!("upsert budget: {e}"))?;
    Ok(id)
}

#[tauri::command]
pub fn delete_budget(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    conn.execute("DELETE FROM budgets WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn save_record(state: State<'_, DbState>, input: RecordInput) -> Result<RecordItem, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    upsert_record(&conn, &input)
}

#[tauri::command]
pub fn delete_record(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    conn.execute("DELETE FROM records WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_recurring_rule(state: State<'_, DbState>, rule_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    conn.execute("DELETE FROM recurring_rules WHERE id = ?1", params![rule_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn search_records(state: State<'_, DbState>, query: String) -> Result<Vec<RecordItem>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let pattern = format!("%{}%", q.to_lowercase());
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.kind, r.amount_satang, r.currency, r.local_date, r.local_time,
                    r.account_id, r.category_id, r.payee_raw, r.note, r.source, r.parse_status,
                    r.is_reviewed, r.recurring_rule_id, s.trans_ref, s.qr_raw, s.ocr_text, s.image_path, r.created_at
             FROM records r
             LEFT JOIN slips s ON s.record_id = r.id
             LEFT JOIN categories c ON c.id = r.category_id
             WHERE lower(COALESCE(r.payee_raw, '')) LIKE ?1
                OR lower(COALESCE(r.note, '')) LIKE ?1
                OR lower(COALESCE(c.name_en, '')) LIKE ?1
                OR lower(COALESCE(c.name_th, '')) LIKE ?1
                OR lower(COALESCE(s.trans_ref, '')) LIKE ?1
             ORDER BY r.local_date DESC, r.created_at DESC
             LIMIT 200",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![pattern], |r| {
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
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for item in rows {
        let mut rec = item.map_err(|e| e.to_string())?;
        rec.tag_ids = fetch_tags_for_record(&conn, &rec.id);
        out.push(rec);
    }
    Ok(out)
}

#[tauri::command]
pub fn get_accounts(state: State<'_, DbState>) -> Result<Vec<Account>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    list_accounts(&conn)
}

#[tauri::command]
pub fn toggle_account_enabled(
    state: State<'_, DbState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    conn.execute(
        "UPDATE accounts SET enabled = ?2 WHERE id = ?1",
        params![id, if enabled { 1 } else { 0 }],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_categories(state: State<'_, DbState>) -> Result<Vec<Category>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    list_categories(&conn)
}

#[tauri::command]
pub fn create_category(
    state: State<'_, DbState>,
    input: CategoryInput,
) -> Result<Category, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let id = format!("cat_custom_{}", Uuid::new_v4().simple());
    conn.execute(
        "INSERT INTO categories (id, name_en, name_th, kind, icon, color, is_system, sort_order, archived)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 100, 0)",
        params![id, input.name_en, input.name_th, input.kind, input.icon, input.color],
    )
    .map_err(|e| e.to_string())?;
    Ok(Category {
        id,
        name_en: input.name_en,
        name_th: input.name_th,
        kind: input.kind,
        icon: input.icon,
        color: input.color,
        is_system: false,
        sort_order: 100,
        archived: false,
    })
}

#[tauri::command]
pub fn delete_category(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    conn.execute(
        "UPDATE records SET category_id = NULL WHERE category_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM payee_rules WHERE category_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM budgets WHERE scope_type = 'category' AND scope_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    let deleted = conn
        .execute(
            "DELETE FROM categories WHERE id = ?1 AND is_system = 0",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    if deleted == 0 {
        conn.execute(
            "UPDATE categories SET archived = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_tags(state: State<'_, DbState>) -> Result<Vec<Tag>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    list_tags(&conn)
}

#[tauri::command]
pub fn create_tag(
    state: State<'_, DbState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let trimmed = name.trim().trim_start_matches('#').to_string();
    if trimmed.is_empty() {
        return Err("Tag name cannot be empty".to_string());
    }
    let id = format!("tag_{}", Uuid::new_v4().simple());
    let c = color.unwrap_or_else(|| "#3b82f6".to_string());
    conn.execute(
        "INSERT INTO tags (id, name, color, disabled, sort_order) VALUES (?1, ?2, ?3, 0, 50)",
        params![id, trimmed, c],
    )
    .map_err(|e| e.to_string())?;
    Ok(Tag {
        id,
        name: trimmed,
        color: c,
        disabled: false,
        sort_order: 50,
    })
}

#[tauri::command]
pub fn toggle_tag_disabled(
    state: State<'_, DbState>,
    id: String,
    disabled: bool,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    conn.execute(
        "UPDATE tags SET disabled = ?2 WHERE id = ?1",
        params![id, if disabled { 1 } else { 0 }],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, DbState>) -> Result<AppSettings, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    load_settings(&conn)
}

#[tauri::command]
pub fn update_settings(state: State<'_, DbState>, settings: AppSettings) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    save_settings(&conn, &settings)
}

#[tauri::command]
pub fn get_album_scan_keywords(state: State<'_, DbState>) -> Result<Vec<String>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let accounts = list_accounts(&conn)?;
    let mut keywords: Vec<String> = Vec::new();
    for bank in ALL_BANKS {
        let is_enabled = accounts
            .iter()
            .find(|a| a.id == bank.id)
            .map(|a| a.enabled)
            .unwrap_or(true);
        if is_enabled {
            for kw in bank.album_keywords {
                keywords.push((*kw).to_string());
            }
        }
    }
    Ok(keywords)
}

#[tauri::command]
pub fn list_known_slip_media_ids(state: State<'_, DbState>) -> Result<Vec<String>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    list_known_media_ids(&conn)
}

#[tauri::command]
pub fn remember_scanned_media(state: State<'_, DbState>, ids: Vec<String>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    remember_scanned_media_rows(&conn, &ids)
}

#[tauri::command]
pub async fn import_slip_candidates(
    state: State<'_, DbState>,
    candidates: Vec<SlipScanCandidate>,
) -> Result<SlipScanReport, String> {
    let settings = {
        let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
        load_settings(&conn)?
    };

    let mut report = SlipScanReport {
        scanned: candidates.len(),
        imported: 0,
        duplicates: 0,
        skipped_non_slip: 0,
        needs_review: 0,
        akson_calls: 0,
    };

    for cand in candidates {
        // 1. Check SHA-256 duplicate before doing any OCR or cloud call!
        let sha_exists: bool = {
            let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
            conn.query_row(
                "SELECT 1 FROM slips WHERE content_sha256 = ?1",
                params![cand.content_sha256],
                |_| Ok(true),
            )
            .optional()
            .unwrap_or(None)
            .unwrap_or(false)
        };
        if sha_exists {
            report.duplicates += 1;
            continue;
        }

        // 2. Validate QR first (BOT Slip Verify Mini-QR or TrueMoney Slip QR)
        let parsed_qr = cand.qr_raw.as_deref().and_then(|raw| parse_slip_qr(raw).ok());

        // Check (sending_bank, trans_ref) duplicate if QR parsed
        if let Some(ref qr) = parsed_qr {
            let ref_exists: bool = {
                let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
                conn.query_row(
                    "SELECT 1 FROM slips WHERE sending_bank = ?1 AND trans_ref = ?2",
                    params![qr.sending_bank.as_deref().unwrap_or("truemoney"), qr.trans_ref],
                    |_| Ok(true),
                )
                .optional()
                .unwrap_or(None)
                .unwrap_or(false)
            };
            if ref_exists {
                report.duplicates += 1;
                continue;
            }
        }

        // 3. Parse local Latin OCR text first (0 AksonOCR credits!)
        let mut combined_ocr = cand.ocr_text.clone();
        let mut parsed = parse_slip_text(
            &combined_ocr,
            parsed_qr.as_ref(),
            &settings.owner_names,
            Some(&cand.album_name),
        );

        // Skip non-slip images: FX currency exchanges, stock buy/sell orders, 0.00 interest notices,
        // or random photos with no valid Slip QR and no clear slip amount.
        if parsed.is_non_slip
            || (parsed_qr.is_none() && parsed.amount_satang.unwrap_or(0) <= 0)
            || (parsed_qr.is_none() && parsed.confidence < 0.35)
        {
            report.skipped_non_slip += 1;
            continue;
        }

        // 4. Smart Credit-Saving Router: only call AksonOCR when Thai / missing fields & API key present!
        let should_call_akson = !settings.akson_ocr_api_key.trim().is_empty()
            && (settings.english_ocr_engine == "akson_only" || parsed.needs_thai_ocr);

        if should_call_akson {
            if let Some(ref b64) = cand.image_base64 {
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                    let mime = cand.mime_type.as_deref().unwrap_or("image/jpeg");
                    let model = if settings.akson_ocr_model.is_empty() {
                        "AksonOCR-preview"
                    } else {
                        &settings.akson_ocr_model
                    };
                    if let Ok(thai_text) =
                        call_akson_ocr(&bytes, mime, &settings.akson_ocr_api_key, model).await
                    {
                        report.akson_calls += 1;
                        combined_ocr = thai_text;
                        parsed = parse_slip_text(
                            &combined_ocr,
                            parsed_qr.as_ref(),
                            &settings.owner_names,
                            Some(&cand.album_name),
                        );
                    }
                }
            }
        }

        // 5. Insert record + slip row into SQLite
        let now = Local::now().to_rfc3339();
        let fallback_date = chrono::DateTime::from_timestamp(cand.date_added_sec, 0)
            .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

        let local_date = parsed.occurred_date.clone().unwrap_or(fallback_date);
        let kind = if parsed.is_self_transfer {
            "transfer"
        } else {
            "spending"
        };
        let amount_satang = parsed.amount_satang.unwrap_or(0);
        let parse_status = if parsed.amount_satang.unwrap_or(0) > 0 && parsed_qr.is_some() {
            "ok"
        } else {
            report.needs_review += 1;
            "needs_review"
        };

        let record_id = Uuid::new_v4().to_string();
        let slip_id = Uuid::new_v4().to_string();

        {
            let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
            let guessed_cat = if kind == "spending" {
                classify_local(
                    &conn,
                    parsed.receiver_name.as_deref(),
                    parsed.memo.as_deref(),
                    Some(&combined_ocr),
                )
            } else {
                None
            };
            let payee_norm = parsed.receiver_name.as_deref().map(normalize_payee);
            let note = parsed.memo.clone().or_else(|| {
                if parsed_qr.is_none() {
                    Some("E-slip has no QR code (OCR fallback)".to_string())
                } else {
                    None
                }
            });

            conn.execute(
                "INSERT INTO records (
                    id, kind, amount_satang, currency, local_date, local_time,
                    account_id, category_id, payee_raw, payee_normalized, note,
                    source, parse_status, is_reviewed, recurring_rule_id, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, 'THB', ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'slip_auto', ?11, 0, NULL, ?12, ?12)",
                params![
                    record_id,
                    kind,
                    amount_satang,
                    local_date,
                    parsed.occurred_time,
                    parsed.bank_id,
                    guessed_cat,
                    parsed.receiver_name,
                    payee_norm,
                    note,
                    parse_status,
                    now
                ],
            )
            .map_err(|e| format!("insert slip record: {e}"))?;

            conn.execute(
                "INSERT INTO slips (
                    id, record_id, media_store_id, content_sha256, sending_bank, trans_ref, qr_raw, ocr_text, image_path, parsed_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    slip_id,
                    record_id,
                    cand.media_store_id,
                    cand.content_sha256,
                    parsed_qr
                        .as_ref()
                        .and_then(|q| q.sending_bank.clone())
                        .or_else(|| parsed.bank_id.clone()),
                    parsed_qr
                        .as_ref()
                        .map(|q| q.trans_ref.clone())
                        .or(parsed.trans_ref),
                    cand.qr_raw,
                    combined_ocr,
                    cand.image_path,
                    now
                ],
            )
            .map_err(|e| format!("insert slip metadata: {e}"))?;
        }

        report.imported += 1;
    }

    if report.imported > 0 || report.scanned > 0 {
        if let Ok(conn) = state.0.lock() {
            record_activity_synced(&conn, report.imported.max(1) as i32);
            // Cross-bank own-account pass so newly imported slips also inherit known own accounts
            let _ = run_reparse_slips_inner(&conn, false);
        }
    }

    Ok(report)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReparseReport {
    pub updated: usize,
    pub removed_non_slips: usize,
    pub transfers_detected: usize,
}

fn run_reparse_slips_inner(
    conn: &rusqlite::Connection,
    prune_non_slips: bool,
) -> Result<ReparseReport, String> {
    let settings = load_settings(conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.local_date, r.category_id, s.qr_raw, s.ocr_text
             FROM records r
             JOIN slips s ON s.record_id = r.id
             WHERE r.source LIKE 'slip%'",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, String, Option<String>, Option<String>, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);

    // Pass 1: parse every slip & collect user's own account last-4 digits and own sender names
    let mut parsed_rows = Vec::with_capacity(rows.len());
    let mut own_last4s: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut learned_owners: Vec<String> = settings.owner_names.clone();

    for (rec_id, orig_date, orig_cat, qr_raw, ocr_text) in rows {
        let qr = qr_raw.as_deref().and_then(|raw| parse_slip_qr(raw).ok());
        let ocr = ocr_text.as_deref().unwrap_or("");
        let parsed = parse_slip_text(ocr, qr.as_ref(), &settings.owner_names, None);

        if !parsed.is_non_slip {
            if let Some(ref last4) = parsed.sender_acct_last4 {
                own_last4s.insert(last4.clone());
            }
            if parsed.is_self_transfer {
                if let Some(ref last4) = parsed.receiver_acct_last4 {
                    own_last4s.insert(last4.clone());
                }
            }
            if let Some(ref sender) = parsed.sender_name {
                if !learned_owners.iter().any(|o| o.eq_ignore_ascii_case(sender)) {
                    learned_owners.push(sender.clone());
                }
            }
        }
        parsed_rows.push((rec_id, orig_date, orig_cat, qr, ocr.to_string(), parsed));
    }

    // Pass 2: re-evaluate with learned own names + cross-bank own-account last-4 digits
    let now = Local::now().to_rfc3339();
    let mut report = ReparseReport {
        updated: 0,
        removed_non_slips: 0,
        transfers_detected: 0,
    };

    for (rec_id, orig_date, orig_cat, qr, ocr, _) in parsed_rows {
        let mut parsed = parse_slip_text(&ocr, qr.as_ref(), &learned_owners, None);
        if !parsed.is_self_transfer {
            if let Some(ref recv_last4) = parsed.receiver_acct_last4 {
                if own_last4s.contains(recv_last4) {
                    parsed.is_self_transfer = true;
                }
            }
        }

        let should_prune = parsed.is_non_slip
            || (qr.is_none() && parsed.amount_satang.unwrap_or(0) <= 0);
        if prune_non_slips && should_prune {
            conn.execute("DELETE FROM records WHERE id = ?1", params![rec_id])
                .map_err(|e| e.to_string())?;
            report.removed_non_slips += 1;
            continue;
        }

        let kind = if parsed.is_self_transfer {
            report.transfers_detected += 1;
            "transfer"
        } else {
            "spending"
        };
        let amount_satang = parsed.amount_satang.unwrap_or(0);
        let local_date = parsed.occurred_date.clone().unwrap_or(orig_date);
        let parse_status = if amount_satang > 0 && qr.is_some() {
            "ok"
        } else {
            "needs_review"
        };
        let cat_id = if kind == "transfer" {
            None
        } else {
            orig_cat.or_else(|| {
                classify_local(
                    conn,
                    parsed.receiver_name.as_deref(),
                    parsed.memo.as_deref(),
                    Some(&ocr),
                )
            })
        };
        let payee_norm = parsed.receiver_name.as_deref().map(normalize_payee);
        let note = parsed.memo.clone().or_else(|| {
            if qr.is_none() {
                Some("E-slip has no QR code (OCR fallback)".to_string())
            } else {
                None
            }
        });

        conn.execute(
            "UPDATE records
             SET kind = ?2, amount_satang = ?3, local_date = ?4, local_time = ?5,
                 account_id = COALESCE(?6, account_id), category_id = ?7,
                 payee_raw = ?8, payee_normalized = ?9, note = ?10,
                 parse_status = ?11, updated_at = ?12
             WHERE id = ?1",
            params![
                rec_id,
                kind,
                amount_satang,
                local_date,
                parsed.occurred_time,
                parsed.bank_id,
                cat_id,
                parsed.receiver_name,
                payee_norm,
                note,
                parse_status,
                now
            ],
        )
        .map_err(|e| e.to_string())?;
        report.updated += 1;
    }

    Ok(report)
}

#[tauri::command]
pub fn reparse_all_slips(state: State<'_, DbState>) -> Result<ReparseReport, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    run_reparse_slips_inner(&conn, true)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillTarget {
    pub media_store_id: String,
    pub content_sha256: String,
    pub needs_thai_ocr: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillResultItem {
    pub content_sha256: String,
    pub image_path: Option<String>,
    pub ocr_text: Option<String>,
}

#[tauri::command]
pub fn get_backfill_targets(state: State<'_, DbState>) -> Result<Vec<BackfillTarget>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT media_store_id, content_sha256, ocr_text, image_path
             FROM slips
             WHERE media_store_id IS NOT NULL AND media_store_id != ''",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            let media_id: String = r.get(0)?;
            let sha: String = r.get(1)?;
            let ocr: String = r.get::<_, Option<String>>(2)?.unwrap_or_default();
            let img_path: Option<String> = r.get(3)?;
            let parsed = parse_slip_text(&ocr, None, &[], None);
            Ok((media_id, sha, img_path.is_none(), parsed.needs_thai_ocr))
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for item in rows {
        let (media_store_id, content_sha256, missing_img, needs_thai_ocr) =
            item.map_err(|e| e.to_string())?;
        if missing_img || needs_thai_ocr {
            out.push(BackfillTarget {
                media_store_id,
                content_sha256,
                needs_thai_ocr,
            });
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn apply_slip_backfill(
    state: State<'_, DbState>,
    items: Vec<BackfillResultItem>,
) -> Result<usize, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let mut count = 0usize;
    for item in items {
        if let Some(ref img_path) = item.image_path {
            conn.execute(
                "UPDATE slips SET image_path = ?2 WHERE content_sha256 = ?1",
                params![item.content_sha256, img_path],
            )
            .map_err(|e| e.to_string())?;
            count += 1;
        }
        if let Some(ref ocr_text) = item.ocr_text {
            if !ocr_text.trim().is_empty() {
                conn.execute(
                    "UPDATE slips SET ocr_text = ?2 WHERE content_sha256 = ?1",
                    params![item.content_sha256, ocr_text],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn list_payee_rules(state: State<'_, DbState>) -> Result<Vec<PayeeRule>, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    list_user_rules(&conn)
}

#[tauri::command]
pub fn upsert_payee_rule(
    state: State<'_, DbState>,
    pattern: String,
    category_id: String,
) -> Result<PayeeRule, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    upsert_user_rule(&conn, &pattern, &category_id)
}

#[tauri::command]
pub fn delete_payee_rule(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    delete_user_rule(&conn, &id)
}

#[tauri::command]
pub fn auto_categorize_uncategorized(state: State<'_, DbState>) -> Result<usize, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    cat_auto_categorize(&conn)
}

#[tauri::command]
pub fn export_records_csv(state: State<'_, DbState>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    export_csv(&conn)
}

#[tauri::command]
pub fn export_backup_json(state: State<'_, DbState>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    let accounts = list_accounts(&conn)?;
    let categories = list_categories(&conn)?;
    let tags = list_tags(&conn)?;
    let records = crate::analytics::query_records_in_range(&conn, "1970-01-01", "2999-12-31", None)?;
    let payload = serde_json::json!({
        "app": "MooJot",
        "version": 1,
        "exportedAt": Local::now().to_rfc3339(),
        "accounts": accounts,
        "categories": categories,
        "tags": tags,
        "records": records,
    });
    serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStatementInput {
    pub pdf_base64: String,
    /// Used only to decrypt this PDF. Never stored or returned.
    pub password: Option<String>,
}

#[tauri::command]
pub fn import_statement_pdf(
    state: State<'_, DbState>,
    input: ImportStatementInput,
) -> Result<crate::statement::ImportStatementReport, String> {
    let encoded = input
        .pdf_base64
        .trim()
        .rsplit_once(',')
        .map(|(_, data)| data)
        .unwrap_or(input.pdf_base64.trim());
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| "invalid pdf data".to_string())?;
    let conn = state.0.lock().map_err(|_| "db lock poisoned".to_string())?;
    crate::statement::import_statement_pdf(&conn, &bytes, input.password.as_deref())
}
