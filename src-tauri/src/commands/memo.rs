// 备忘录六条 Command 实现，严格按 Planner 契约

use super::{lock_conn, now_iso8601};
use crate::db::Db;
use crate::error::AppError;
use crate::models::{
    CreateMemoInput, DeleteMemoInput, DeleteMemoResult, GetMemoInput, Memo, SearchMemosInput,
    UpdateMemoInput,
};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

/// 查询用到的字段列，顺序与 Memo 结构体字段一致
pub(crate) const SELECT_COLS: &str = "id, title, content, created_at, updated_at, remind_at";

/// 行 → Memo（含 remind_at）
pub(crate) fn row_to_memo(row: &rusqlite::Row<'_>) -> rusqlite::Result<Memo> {
    Ok(Memo {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        remind_at: row.get(5)?,
    })
}

/// 校验标题：trim 后非空、长度不超过 100 个字符（入参为已 trim 的标题）
fn validate_title(title: &str) -> Result<(), AppError> {
    if title.is_empty() {
        return Err(AppError::validation("标题不能为空"));
    }
    if title.chars().count() > 100 {
        return Err(AppError::validation("标题不能超过 100 个字符"));
    }
    Ok(())
}

/// 按 id 查询单条备忘录（供本模块与 reminder/list_memos_by_tag 复用）
pub(crate) fn query_by_id(conn: &Connection, id: i64) -> Result<Memo, AppError> {
    let memo = conn
        .query_row(
            &format!("SELECT {SELECT_COLS} FROM memo WHERE id = ?1"),
            params![id],
            row_to_memo,
        )
        .optional()
        .map_err(AppError::from)?;
    memo.ok_or_else(|| AppError::not_found("备忘录不存在"))
}

/// 存在性检查（delete/update 前调用，语义上区分「不存在」与「执行失败」）
pub(crate) fn memo_exists(conn: &Connection, id: i64) -> Result<bool, AppError> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM memo WHERE id = ?1)",
        params![id],
        |row| row.get::<_, i64>(0),
    )
    .map(|n| n == 1)
    .map_err(AppError::from)
}

/// 读取全部行并按 updated_at 倒序返回
fn query_all(conn: &Connection, sql: &str, pattern: Option<&str>) -> Result<Vec<Memo>, AppError> {
    let mut stmt = conn.prepare(sql).map_err(AppError::from)?;
    let rows = match pattern {
        Some(p) => stmt.query_map(params![p], row_to_memo),
        None => stmt.query_map(params![], row_to_memo),
    }
    .map_err(AppError::from)?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(AppError::from)?);
    }
    Ok(result)
}

/// 转义 LIKE 通配符（%、_、\），避免用户输入导致误匹配
pub(crate) fn escape_like(keyword: &str) -> String {
    let mut escaped = String::with_capacity(keyword.len());
    for c in keyword.chars() {
        match c {
            '\\' | '%' | '_' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

/// 2.1 新建备忘录：后台生成 id、created_at、updated_at（remind_at 默认空）
#[tauri::command]
pub fn create_memo(state: State<'_, Db>, input: CreateMemoInput) -> Result<Memo, AppError> {
    let title = input.title.trim();
    validate_title(title)?;

    let now = now_iso8601();
    let conn = lock_conn(&state)?;
    conn.execute(
        "INSERT INTO memo (title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![title, input.content, now, now],
    )
    .map_err(AppError::from)?;

    let id = conn.last_insert_rowid();
    query_by_id(&conn, id)
}

/// 2.2 全部列表：按 updated_at 倒序（最新在前）
#[tauri::command]
pub fn list_memos(state: State<'_, Db>) -> Result<Vec<Memo>, AppError> {
    let conn = lock_conn(&state)?;
    query_all(
        &conn,
        &format!("SELECT {SELECT_COLS} FROM memo ORDER BY updated_at DESC"),
        None,
    )
}

/// 2.3 单条查询：按 id 拉取完整内容
#[tauri::command]
pub fn get_memo(state: State<'_, Db>, input: GetMemoInput) -> Result<Memo, AppError> {
    let conn = lock_conn(&state)?;
    query_by_id(&conn, input.id)
}

/// 2.4 更新：刷新 updated_at，created_at 与 remind_at 保持不变
#[tauri::command]
pub fn update_memo(state: State<'_, Db>, input: UpdateMemoInput) -> Result<Memo, AppError> {
    let title = input.title.trim();
    validate_title(title)?;

    let conn = lock_conn(&state)?;
    if !memo_exists(&conn, input.id)? {
        return Err(AppError::not_found("备忘录不存在"));
    }

    let now = now_iso8601();
    conn.execute(
        "UPDATE memo SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, input.content, now, input.id],
    )
    .map_err(AppError::from)?;

    query_by_id(&conn, input.id)
}

/// 2.5 删除：P0 硬删；先显式清理标签关系，再删备忘录（外键 CASCADE 兜底）
#[tauri::command]
pub fn delete_memo(
    state: State<'_, Db>,
    input: DeleteMemoInput,
) -> Result<DeleteMemoResult, AppError> {
    let conn = lock_conn(&state)?;
    if !memo_exists(&conn, input.id)? {
        return Err(AppError::not_found("备忘录不存在"));
    }
    conn.execute("DELETE FROM memo_tags WHERE memo_id = ?1", params![input.id])
        .map_err(AppError::from)?;
    conn.execute("DELETE FROM memo WHERE id = ?1", params![input.id])
        .map_err(AppError::from)?;
    Ok(DeleteMemoResult { id: input.id })
}

/// 2.6 搜索：标题 + 正文模糊匹配（LIKE + ESCAPE 转义），结果按 updated_at 倒序
#[tauri::command]
pub fn search_memos(
    state: State<'_, Db>,
    input: SearchMemosInput,
) -> Result<Vec<Memo>, AppError> {
    let keyword = input.keyword.trim();
    if keyword.is_empty() {
        return Ok(Vec::new());
    }

    let pattern = format!("%{}%", escape_like(keyword));
    let conn = lock_conn(&state)?;
    query_all(
        &conn,
        &format!(
            "SELECT {SELECT_COLS} FROM memo WHERE title LIKE ?1 ESCAPE '\\' \
             OR content LIKE ?1 ESCAPE '\\' ORDER BY updated_at DESC"
        ),
        Some(&pattern),
    )
}