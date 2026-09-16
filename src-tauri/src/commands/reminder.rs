// 提醒两条 Command 实现，严格按 Planner 2.3 C7/C8

use super::lock_conn;
use crate::commands::memo::{memo_exists, query_by_id};
use crate::db::Db;
use crate::error::AppError;
use crate::models::{ClearMemoReminderInput, Memo, SetMemoReminderInput};
use crate::scheduler::Scheduler;
use chrono::Timelike;
use rusqlite::params;
use tauri::State;

/// 解析 ISO8601 并截断到分钟；无法解析返回 E_VALIDATION
fn parse_to_utc_minute(raw: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    let parsed = chrono::DateTime::parse_from_rfc3339(raw)
        .map_err(|_| AppError::validation("提醒时间格式不正确"))?;
    let utc = parsed.with_timezone(&chrono::Utc);
    utc.with_second(0)
        .and_then(|d| d.with_nanosecond(0))
        .ok_or_else(|| AppError::validation("提醒时间格式不正确"))
}

/// C7 设置提醒时间（不刷新 updated_at）
#[tauri::command]
pub fn set_memo_reminder(
    state: State<'_, Db>,
    scheduler: State<'_, Scheduler>,
    input: SetMemoReminderInput,
) -> Result<Memo, AppError> {
    let conn = lock_conn(&state)?;
    if !memo_exists(&conn, input.memo_id)? {
        return Err(AppError::not_found("备忘录不存在"));
    }

    let due = parse_to_utc_minute(&input.remind_at)?;
    if due < chrono::Utc::now() {
        return Err(AppError::validation("提醒时间不能早于当前时间"));
    }

    let stored = due.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    conn.execute(
        "UPDATE memo SET remind_at = ?1 WHERE id = ?2",
        params![stored, input.memo_id],
    )
    .map_err(AppError::from)?;

    // 唤醒调度线程重新扫描最近到期时间
    scheduler.poke();

    query_by_id(&conn, input.memo_id)
}

/// C8 清除提醒（幂等）
#[tauri::command]
pub fn clear_memo_reminder(
    state: State<'_, Db>,
    scheduler: State<'_, Scheduler>,
    input: ClearMemoReminderInput,
) -> Result<Memo, AppError> {
    let conn = lock_conn(&state)?;
    if !memo_exists(&conn, input.memo_id)? {
        return Err(AppError::not_found("备忘录不存在"));
    }

    conn.execute(
        "UPDATE memo SET remind_at = '' WHERE id = ?1",
        params![input.memo_id],
    )
    .map_err(AppError::from)?;

    scheduler.poke();

    query_by_id(&conn, input.memo_id)
}