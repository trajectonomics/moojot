use std::collections::BTreeMap;

use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::db::{
    compute_streaks, fetch_tags_for_record, list_categories, list_tags, load_settings,
    FutureRecordItem, RecordItem,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodRange {
    pub start_date: String,
    pub end_date: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayGroup {
    pub local_date: String,
    pub day_of_week: String,
    pub day_number: u32,
    pub spending_satang: i64,
    pub income_satang: i64,
    pub transfer_count: usize,
    pub records: Vec<RecordItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomePayload {
    pub period: PeriodRange,
    pub total_spending_satang: i64,
    pub total_income_satang: i64,
    pub total_transfer_satang: i64,
    pub remaining_satang: i64,
    pub day_groups: Vec<DayGroup>,
    pub future_records: Vec<FutureRecordItem>,
    pub future_total_satang: i64,
    pub needs_review_count: usize,
    pub today_jotted_count: usize,
    pub sync_streak_days: i32,
    pub categorize_streak_days: i32,
    pub best_streak_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryBreakdown {
    pub category_id: Option<String>,
    pub name_en: String,
    pub name_th: String,
    pub icon: String,
    pub color: String,
    pub amount_satang: i64,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagBreakdown {
    pub tag_id: String,
    pub name: String,
    pub color: String,
    pub amount_satang: i64,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryPayload {
    pub period: PeriodRange,
    pub income_satang: i64,
    pub spending_satang: i64,
    pub transfer_satang: i64,
    pub remaining_satang: i64,
    pub by_category: Vec<CategoryBreakdown>,
    pub by_tag: Vec<TagBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendBar {
    pub label: String,
    pub start_date: String,
    pub end_date: String,
    pub amount_satang: i64,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparePayload {
    pub bars: Vec<TrendBar>,
    pub average_satang: i64,
    pub current_satang: i64,
    pub delta_vs_average_satang: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetItem {
    pub id: String,
    pub scope_type: String,
    pub scope_id: Option<String>,
    pub scope_label: String,
    pub scope_icon: Option<String>,
    pub scope_color: String,
    pub period_type: String,
    pub limit_satang: i64,
    pub spent_satang: i64,
    pub remaining_satang: i64,
    pub spent_pct: f64,
    pub today_pace_pct: f64,
    pub avg_daily_spent_satang: i64,
    pub avg_daily_remaining_satang: i64,
    pub days_remaining: i64,
    pub alert_threshold_pct: i32,
}

const MONTH_SHORT_EN: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub fn compute_month_period(offset: i32, month_start_day: i32) -> (NaiveDate, NaiveDate, String) {
    let today = Local::now().date_naive();
    let start_day = month_start_day.clamp(1, 28) as u32;

    // Anchor month based on whether today >= start_day
    let mut base_year = today.year();
    let mut base_month = today.month() as i32;
    if today.day() < start_day {
        base_month -= 1;
        if base_month < 1 {
            base_month = 12;
            base_year -= 1;
        }
    }

    let total_months = base_year * 12 + (base_month - 1) + offset;
    let y = total_months.div_euclid(12);
    let m = (total_months.rem_euclid(12) + 1) as u32;

    let start = NaiveDate::from_ymd_opt(y, m, start_day).unwrap_or(today);
    let next_total = total_months + 1;
    let ny = next_total.div_euclid(12);
    let nm = (next_total.rem_euclid(12) + 1) as u32;
    let next_start = NaiveDate::from_ymd_opt(ny, nm, start_day).unwrap_or(start + Duration::days(30));
    let end = next_start - Duration::days(1);

    let label = if start_day == 1 {
        format!("{} {}", MONTH_SHORT_EN[(m - 1) as usize], y % 100)
    } else {
        format!(
            "{} {} - {} {}",
            start.day(),
            MONTH_SHORT_EN[(start.month() - 1) as usize],
            end.day(),
            MONTH_SHORT_EN[(end.month() - 1) as usize]
        )
    };
    (start, end, label)
}

pub fn query_records_in_range(
    conn: &Connection,
    start_ymd: &str,
    end_ymd: &str,
    account_ids: Option<&[String]>,
) -> Result<Vec<RecordItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.kind, r.amount_satang, r.currency, r.local_date, r.local_time,
                    r.account_id, r.category_id, r.payee_raw, r.note, r.source, r.parse_status,
                    r.is_reviewed, r.recurring_rule_id, s.trans_ref, s.qr_raw, s.ocr_text, s.image_path, r.created_at
             FROM records r
             LEFT JOIN slips s ON s.record_id = r.id
             WHERE r.local_date >= ?1 AND r.local_date <= ?2
             ORDER BY r.local_date DESC, COALESCE(r.local_time, '00:00') DESC, r.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![start_ymd, end_ymd], |r| {
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
        if let Some(filter) = account_ids {
            if !filter.is_empty() {
                let matched = rec
                    .account_id
                    .as_ref()
                    .map(|a| filter.iter().any(|f| f == a))
                    .unwrap_or(false);
                if !matched {
                    continue;
                }
            }
        }
        rec.tag_ids = fetch_tags_for_record(conn, &rec.id);
        out.push(rec);
    }
    Ok(out)
}

pub fn build_home_payload(
    conn: &Connection,
    period_offset: i32,
    account_ids: Option<&[String]>,
) -> Result<HomePayload, String> {
    let settings = load_settings(conn)?;
    let (start, end, label) = compute_month_period(period_offset, settings.month_start_day);
    let start_ymd = start.format("%Y-%m-%d").to_string();
    let end_ymd = end.format("%Y-%m-%d").to_string();

    let records = query_records_in_range(conn, &start_ymd, &end_ymd, account_ids)?;

    let mut total_spending = 0i64;
    let mut total_income = 0i64;
    let mut total_transfer = 0i64;
    let mut needs_review_count = 0usize;
    let mut today_jotted_count = 0usize;
    let today_ymd = Local::now().format("%Y-%m-%d").to_string();

    let mut by_day: BTreeMap<String, Vec<RecordItem>> = BTreeMap::new();
    for rec in records {
        match rec.kind.as_str() {
            "spending" => total_spending += rec.amount_satang,
            "income" => total_income += rec.amount_satang,
            "transfer" => total_transfer += rec.amount_satang,
            _ => {}
        }
        if rec.parse_status != "ok" {
            needs_review_count += 1;
        }
        if rec.created_at.starts_with(&today_ymd) && rec.source.starts_with("slip") {
            today_jotted_count += 1;
        }
        by_day.entry(rec.local_date.clone()).or_default().push(rec);
    }

    let mut day_groups: Vec<DayGroup> = Vec::new();
    for (date_str, recs) in by_day.into_iter().rev() {
        let parsed = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().date_naive());
        let dow = if date_str == today_ymd {
            "Today".to_string()
        } else {
            parsed.format("%a").to_string()
        };
        let spending_satang = recs
            .iter()
            .filter(|r| r.kind == "spending")
            .map(|r| r.amount_satang)
            .sum();
        let income_satang = recs
            .iter()
            .filter(|r| r.kind == "income")
            .map(|r| r.amount_satang)
            .sum();
        let transfer_count = recs.iter().filter(|r| r.kind == "transfer").count();

        day_groups.push(DayGroup {
            local_date: date_str,
            day_of_week: dow,
            day_number: parsed.day(),
            spending_satang,
            income_satang,
            transfer_count,
            records: recs,
        });
    }

    // Future recurring records due in this period or upcoming 30 days
    let mut future_stmt = conn
        .prepare(
            "SELECT id, kind, amount_satang, next_run_date, category_id, note, frequency, day_of_month
             FROM recurring_rules WHERE active = 1 ORDER BY next_run_date ASC",
        )
        .map_err(|e| e.to_string())?;
    let future_records: Vec<FutureRecordItem> = future_stmt
        .query_map([], |r| {
            Ok(FutureRecordItem {
                rule_id: r.get(0)?,
                kind: r.get(1)?,
                amount_satang: r.get(2)?,
                due_date: r.get(3)?,
                category_id: r.get(4)?,
                note: r.get(5)?,
                frequency: r.get(6)?,
                day_of_month: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let future_total_satang: i64 = future_records
        .iter()
        .filter(|f| f.kind == "spending")
        .map(|f| f.amount_satang)
        .sum();

    let (sync_streak, cat_streak, best_streak) = compute_streaks(conn);

    Ok(HomePayload {
        period: PeriodRange {
            start_date: start_ymd,
            end_date: end_ymd,
            label,
        },
        total_spending_satang: total_spending,
        total_income_satang: total_income,
        total_transfer_satang: total_transfer,
        remaining_satang: total_income - total_spending,
        day_groups,
        future_records,
        future_total_satang,
        needs_review_count,
        today_jotted_count,
        sync_streak_days: sync_streak,
        categorize_streak_days: cat_streak,
        best_streak_days: best_streak,
    })
}

pub fn build_summary_payload(
    conn: &Connection,
    period_offset: i32,
    kind: &str,
    account_ids: Option<&[String]>,
) -> Result<SummaryPayload, String> {
    let settings = load_settings(conn)?;
    let (start, end, label) = compute_month_period(period_offset, settings.month_start_day);
    let start_ymd = start.format("%Y-%m-%d").to_string();
    let end_ymd = end.format("%Y-%m-%d").to_string();

    let records = query_records_in_range(conn, &start_ymd, &end_ymd, account_ids)?;
    let categories = list_categories(conn)?;
    let tags = list_tags(conn)?;

    let mut income = 0i64;
    let mut spending = 0i64;
    let mut transfer = 0i64;

    let mut cat_totals: BTreeMap<Option<String>, (i64, usize)> = BTreeMap::new();
    let mut tag_totals: BTreeMap<String, (i64, usize)> = BTreeMap::new();

    for rec in &records {
        match rec.kind.as_str() {
            "income" => income += rec.amount_satang,
            "spending" => spending += rec.amount_satang,
            "transfer" => transfer += rec.amount_satang,
            _ => {}
        }
        if rec.kind == kind {
            let entry = cat_totals.entry(rec.category_id.clone()).or_insert((0, 0));
            entry.0 += rec.amount_satang;
            entry.1 += 1;
            for tid in &rec.tag_ids {
                let te = tag_totals.entry(tid.clone()).or_insert((0, 0));
                te.0 += rec.amount_satang;
                te.1 += 1;
            }
        }
    }

    let denom = match kind {
        "income" => income,
        "transfer" => transfer,
        _ => spending,
    };

    let mut by_category: Vec<CategoryBreakdown> = cat_totals
        .into_iter()
        .map(|(cid, (amt, count))| {
            let meta = cid
                .as_deref()
                .and_then(|id| categories.iter().find(|c| c.id == id));
            let pct = if denom > 0 {
                (amt as f64 / denom as f64) * 100.0
            } else {
                0.0
            };
            CategoryBreakdown {
                category_id: cid,
                name_en: meta.map(|m| m.name_en.clone()).unwrap_or_else(|| "Uncategorized".into()),
                name_th: meta.map(|m| m.name_th.clone()).unwrap_or_else(|| "ยังไม่ระบุหมวด".into()),
                icon: meta.map(|m| m.icon.clone()).unwrap_or_else(|| "leaf".into()),
                color: meta.map(|m| m.color.clone()).unwrap_or_else(|| "#64748b".into()),
                amount_satang: amt,
                count,
                percentage: (pct * 10.0).round() / 10.0,
            }
        })
        .collect();
    by_category.sort_by(|a, b| b.amount_satang.cmp(&a.amount_satang));

    let mut by_tag: Vec<TagBreakdown> = tag_totals
        .into_iter()
        .map(|(tid, (amt, count))| {
            let meta = tags.iter().find(|t| t.id == tid);
            let pct = if denom > 0 {
                (amt as f64 / denom as f64) * 100.0
            } else {
                0.0
            };
            TagBreakdown {
                tag_id: tid,
                name: meta.map(|m| m.name.clone()).unwrap_or_else(|| "tag".into()),
                color: meta.map(|m| m.color.clone()).unwrap_or_else(|| "#3b82f6".into()),
                amount_satang: amt,
                count,
                percentage: (pct * 10.0).round() / 10.0,
            }
        })
        .collect();
    by_tag.sort_by(|a, b| b.amount_satang.cmp(&a.amount_satang));

    Ok(SummaryPayload {
        period: PeriodRange {
            start_date: start_ymd,
            end_date: end_ymd,
            label,
        },
        income_satang: income,
        spending_satang: spending,
        transfer_satang: transfer,
        remaining_satang: income - spending,
        by_category,
        by_tag,
    })
}

pub fn build_compare_payload(
    conn: &Connection,
    period_offset: i32,
    kind: &str,
    months: i32,
) -> Result<ComparePayload, String> {
    let settings = load_settings(conn)?;
    let count = months.clamp(3, 12);
    let mut bars = Vec::new();
    let mut prior_sum = 0i64;
    let mut prior_count = 0i64;
    let mut current_satang = 0i64;

    for i in (0..count).rev() {
        let off = period_offset - i;
        let (start, end, label) = compute_month_period(off, settings.month_start_day);
        let start_ymd = start.format("%Y-%m-%d").to_string();
        let end_ymd = end.format("%Y-%m-%d").to_string();
        let amt: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(amount_satang), 0) FROM records
                 WHERE kind = ?1 AND local_date >= ?2 AND local_date <= ?3",
                params![kind, start_ymd, end_ymd],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let is_current = i == 0;
        if is_current {
            current_satang = amt;
        } else {
            prior_sum += amt;
            prior_count += 1;
        }
        bars.push(TrendBar {
            label,
            start_date: start_ymd,
            end_date: end_ymd,
            amount_satang: amt,
            is_current,
        });
    }

    let average_satang = if prior_count > 0 {
        prior_sum / prior_count
    } else {
        current_satang
    };
    Ok(ComparePayload {
        bars,
        average_satang,
        current_satang,
        delta_vs_average_satang: current_satang - average_satang,
    })
}

pub fn build_budgets(conn: &Connection, period_offset: i32) -> Result<Vec<BudgetItem>, String> {
    let settings = load_settings(conn)?;
    let (start, end, _) = compute_month_period(period_offset, settings.month_start_day);
    let start_ymd = start.format("%Y-%m-%d").to_string();
    let end_ymd = end.format("%Y-%m-%d").to_string();
    let today = Local::now().date_naive();

    let total_days = ((end - start).num_days() + 1).max(1);
    let elapsed_days = if today < start {
        0
    } else if today > end {
        total_days
    } else {
        (today - start).num_days() + 1
    };
    let days_remaining = (total_days - elapsed_days).max(0);
    let today_pace_pct = ((elapsed_days as f64 / total_days as f64) * 100.0).clamp(0.0, 100.0);

    let categories = list_categories(conn)?;
    let tags = list_tags(conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, scope_type, scope_id, period_type, limit_satang, alert_threshold_pct
             FROM budgets WHERE active = 1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i32>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for row in rows {
        let (id, scope_type, scope_id, period_type, limit_satang, alert_threshold_pct) =
            row.map_err(|e| e.to_string())?;
        let spent_satang: i64 = match (scope_type.as_str(), scope_id.as_deref()) {
            ("category", Some(cid)) => conn
                .query_row(
                    "SELECT COALESCE(SUM(amount_satang), 0) FROM records
                     WHERE kind = 'spending' AND category_id = ?1 AND local_date >= ?2 AND local_date <= ?3",
                    params![cid, start_ymd, end_ymd],
                    |r| r.get(0),
                )
                .unwrap_or(0),
            ("tag", Some(tid)) => conn
                .query_row(
                    "SELECT COALESCE(SUM(r.amount_satang), 0)
                     FROM records r JOIN record_tags rt ON rt.record_id = r.id
                     WHERE r.kind = 'spending' AND rt.tag_id = ?1 AND r.local_date >= ?2 AND r.local_date <= ?3",
                    params![tid, start_ymd, end_ymd],
                    |r| r.get(0),
                )
                .unwrap_or(0),
            _ => conn
                .query_row(
                    "SELECT COALESCE(SUM(amount_satang), 0) FROM records
                     WHERE kind = 'spending' AND local_date >= ?1 AND local_date <= ?2",
                    params![start_ymd, end_ymd],
                    |r| r.get(0),
                )
                .unwrap_or(0),
        };

        let (scope_label, scope_icon, scope_color) = match (scope_type.as_str(), scope_id.as_deref())
        {
            ("category", Some(cid)) => categories
                .iter()
                .find(|c| c.id == cid)
                .map(|c| {
                    let lbl = if settings.language == "th" {
                        c.name_th.clone()
                    } else {
                        c.name_en.clone()
                    };
                    (lbl, Some(c.icon.clone()), c.color.clone())
                })
                .unwrap_or_else(|| ("Category".into(), None, "#f59e0b".into())),
            ("tag", Some(tid)) => tags
                .iter()
                .find(|t| t.id == tid)
                .map(|t| (format!("#{}", t.name), Some("hash".into()), t.color.clone()))
                .unwrap_or_else(|| ("#Tag".into(), Some("hash".into()), "#3b82f6".into())),
            _ => (
                if settings.language == "th" {
                    "งบประมาณรวม".into()
                } else {
                    "Total Monthly Budget".into()
                },
                Some("wallet".into()),
                "#22c55e".into(),
            ),
        };

        let remaining_satang = limit_satang - spent_satang;
        let spent_pct = if limit_satang > 0 {
            (spent_satang as f64 / limit_satang as f64) * 100.0
        } else {
            0.0
        };
        let avg_daily_spent_satang = if elapsed_days > 0 {
            spent_satang / elapsed_days
        } else {
            0
        };
        let avg_daily_remaining_satang = if days_remaining > 0 {
            remaining_satang.max(0) / days_remaining
        } else {
            remaining_satang.max(0)
        };

        out.push(BudgetItem {
            id,
            scope_type,
            scope_id,
            scope_label,
            scope_icon,
            scope_color,
            period_type,
            limit_satang,
            spent_satang,
            remaining_satang,
            spent_pct: (spent_pct * 10.0).round() / 10.0,
            today_pace_pct: (today_pace_pct * 10.0).round() / 10.0,
            avg_daily_spent_satang,
            avg_daily_remaining_satang,
            days_remaining,
            alert_threshold_pct,
        });
    }
    Ok(out)
}

/// First characters that make a spreadsheet app read a cell as a formula
/// (OWASP "CSV Injection").
const CSV_FORMULA_PREFIXES: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

/// Makes a text cell safe to open in a spreadsheet: if it starts with a
/// formula character, a leading `'` makes the app show it as plain text.
/// Only for text columns; the date, time and amount columns are written as is.
fn csv_text_cell(text: String) -> String {
    if text.starts_with(CSV_FORMULA_PREFIXES) {
        format!("'{text}")
    } else {
        text
    }
}

pub fn export_csv(conn: &Connection) -> Result<String, String> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record([
        "date",
        "time",
        "kind",
        "amount_thb",
        "currency",
        "category",
        "account",
        "payee",
        "note",
        "source",
        "slip_ref",
    ])
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT r.local_date, COALESCE(r.local_time, ''), r.kind, r.amount_satang, r.currency,
                    COALESCE(c.name_en, 'Uncategorized'), COALESCE(a.name_en, ''),
                    COALESCE(r.payee_raw, ''), COALESCE(r.note, ''), r.source, COALESCE(s.trans_ref, '')
             FROM records r
             LEFT JOIN categories c ON c.id = r.category_id
             LEFT JOIN accounts a ON a.id = r.account_id
             LEFT JOIN slips s ON s.record_id = r.id
             ORDER BY r.local_date DESC, r.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |r| {
            let satang: i64 = r.get(3)?;
            Ok([
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                csv_text_cell(r.get(2)?),
                format!("{:.2}", satang as f64 / 100.0),
                csv_text_cell(r.get(4)?),
                csv_text_cell(r.get(5)?),
                csv_text_cell(r.get(6)?),
                csv_text_cell(r.get(7)?),
                csv_text_cell(r.get(8)?),
                csv_text_cell(r.get(9)?),
                csv_text_cell(r.get(10)?),
            ])
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let rec = row.map_err(|e| e.to_string())?;
        wtr.write_record(rec).map_err(|e| e.to_string())?;
    }
    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod csv_export_tests {
    use super::*;
    use crate::db::init_db;

    fn temp_db() -> (Connection, std::path::PathBuf) {
        let path =
            std::env::temp_dir().join(format!("moojot-csv-{}.sqlite3", uuid::Uuid::new_v4()));
        let conn = init_db(&path).expect("init db");
        (conn, path)
    }

    fn cleanup(path: &std::path::Path) {
        for suffix in ["", "-wal", "-shm"] {
            let mut file = path.as_os_str().to_os_string();
            file.push(suffix);
            let _ = std::fs::remove_file(file);
        }
    }

    fn add_record(conn: &Connection, id: &str, date: &str, payee: &str, note: &str) {
        conn.execute(
            "INSERT INTO records (id, kind, amount_satang, local_date, account_id, category_id,
                                  payee_raw, note, created_at, updated_at)
             VALUES (?1, 'spending', 1250, ?2, 'test_acct', 'test_cat', ?3, ?4, ?2, ?2)",
            params![id, date, payee, note],
        )
        .unwrap();
    }

    /// Exported rows, keyed by date, parsed back with a CSV reader.
    fn export_rows(conn: &Connection) -> BTreeMap<String, csv::StringRecord> {
        let text = export_csv(conn).unwrap();
        csv::Reader::from_reader(text.as_bytes())
            .records()
            .map(|row| {
                let row = row.unwrap();
                (row[0].to_string(), row)
            })
            .collect()
    }

    #[test]
    fn text_cells_that_look_like_formulas_are_exported_as_text() {
        let (conn, path) = temp_db();
        conn.execute(
            "INSERT INTO accounts (id, name_en, name_th, kind, color)
             VALUES ('test_acct', '-Test account', '-Test account', 'bank', '#000000')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO categories (id, name_en, name_th, kind, icon, color)
             VALUES ('test_cat', '+Test category', '+Test category', 'spending', 'leaf', '#000000')",
            [],
        )
        .unwrap();
        add_record(&conn, "r1", "2026-01-01", "=1+2", "@A1");
        add_record(&conn, "r2", "2026-01-02", "\t1", "\r1");
        add_record(&conn, "r3", "2026-01-03", "Cafe 1+1", "");

        let rows = export_rows(&conn);
        // Columns: date, time, kind, amount_thb, currency, category, account, payee, note, ...
        let first = &rows["2026-01-01"];
        assert_eq!(&first[3], "12.50");
        assert_eq!(&first[5], "'+Test category");
        assert_eq!(&first[6], "'-Test account");
        assert_eq!(&first[7], "'=1+2");
        assert_eq!(&first[8], "'@A1");

        let second = &rows["2026-01-02"];
        assert_eq!(&second[7], "'\t1");
        assert_eq!(&second[8], "'\r1");

        // Text that doesn't start with a formula character is unchanged.
        let third = &rows["2026-01-03"];
        assert_eq!(&third[7], "Cafe 1+1");
        assert_eq!(&third[8], "");

        cleanup(&path);
    }
}
