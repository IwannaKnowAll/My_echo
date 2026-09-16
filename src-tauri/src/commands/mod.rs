pub mod memo;
pub mod reminder;
pub mod tag;

use crate::db::Db;
use crate::error::AppError;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

/// 生成 UTC RFC3339 毫秒精度、Z 结尾的时间戳（保证文本字典序 === 时间序）
pub(crate) fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// 获取全局数据库连接的互斥守卫
pub(crate) fn lock_conn<'a>(
    state: &'a State<'a, Db>,
) -> Result<MutexGuard<'a, Connection>, AppError> {
    state
        .0
        .lock()
        .map_err(|_| AppError::internal("数据库连接异常"))
}