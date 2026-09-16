// 标签六条 Command 实现，严格按 Planner 2.3 C1–C6

use super::{lock_conn, now_iso8601};
use crate::commands::memo::{escape_like, memo_exists, row_to_memo, SELECT_COLS};
use crate::db::Db;
use crate::error::AppError;
use crate::models::{
    CreateTagInput, DeleteTagInput, DeleteTagResult, ListMemoTagsInput, ListMemosByTagInput, Memo,
    SetMemoTagsInput, Tag, TagWithCount,
};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use tauri::State;

/// 校验标签名：trim 后非空、长度 ≤ 30、不含换行
fn validate_tag_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        return Err(AppError::validation("标签名不能为空"));
    }
    if name.chars().count() > 30 {
        return Err(AppError::validation("标签不能超过 30 个字符"));
    }
    if name.contains('\n') || name.contains('\r') {
        return Err(AppError::validation("标签名不能包含换行"));
    }
    Ok(())
}

fn row_to_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
    })
}

/// 标签是否存在
fn tag_exists(conn: &Connection, id: i64) -> Result<bool, AppError> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM tag WHERE id = ?1)",
        params![id],
        |row| row.get::<_, i64>(0),
    )
    .map(|n| n == 1)
    .map_err(AppError::from)
}

/// 按名称（NOCASE）查询标签 id，命中复用
fn find_tag_id(conn: &Connection, name: &str) -> Result<Option<i64>, AppError> {
    conn.query_row(
        "SELECT id FROM tag WHERE name = ?1 COLLATE NOCASE",
        params![name],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(AppError::from)
}

/// 查询某备忘录的标签列表（按名称排序）
fn query_tags_for_memo(conn: &Connection, memo_id: i64) -> Result<Vec<Tag>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.created_at \
             FROM tag t JOIN memo_tags mt ON mt.tag_id = t.id \
             WHERE mt.memo_id = ?1 ORDER BY t.name",
        )
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map(params![memo_id], row_to_tag)
        .map_err(AppError::from)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(AppError::from)?);
    }
    Ok(result)
}

/// C1 列出全部标签（含引用计数）
#[tauri::command]
pub fn list_tags(state: State<'_, Db>) -> Result<Vec<TagWithCount>, AppError> {
    let conn = lock_conn(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.created_at, COUNT(mt.memo_id) AS memo_count \
             FROM tag t LEFT JOIN memo_tags mt ON mt.tag_id = t.id \
             GROUP BY t.id ORDER BY t.name",
        )
        .map_err(AppError::from)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TagWithCount {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                memo_count: row.get(3)?,
            })
        })
        .map_err(AppError::from)?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(AppError::from)?);
    }
    Ok(result)
}

/// C2 创建标签（管理入口，NOCASE 重名拦截）
#[tauri::command]
pub fn create_tag(state: State<'_, Db>, input: CreateTagInput) -> Result<Tag, AppError> {
    let name = input.name.trim().to_string();
    validate_tag_name(&name)?;

    let conn = lock_conn(&state)?;
    if find_tag_id(&conn, &name)?.is_some() {
        return Err(AppError::validation("标签已存在"));
    }

    let now = now_iso8601();
    conn.execute(
        "INSERT INTO tag (name, created_at) VALUES (?1, ?2)",
        params![name, now],
    )
    .map_err(AppError::from)?;

    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name,
        created_at: now,
    })
}

/// C3 删除标签（仅解除关联，不删备忘录）
#[tauri::command]
pub fn delete_tag(state: State<'_, Db>, input: DeleteTagInput) -> Result<DeleteTagResult, AppError> {
    let conn = lock_conn(&state)?;
    if !tag_exists(&conn, input.id)? {
        return Err(AppError::not_found("标签不存在"));
    }
    conn.execute(
        "DELETE FROM memo_tags WHERE tag_id = ?1",
        params![input.id],
    )
    .map_err(AppError::from)?;
    conn.execute("DELETE FROM tag WHERE id = ?1", params![input.id])
        .map_err(AppError::from)?;
    Ok(DeleteTagResult { id: input.id })
}

/// C4 查询某备忘录的标签（编辑器回填）
#[tauri::command]
pub fn list_memo_tags(state: State<'_, Db>, input: ListMemoTagsInput) -> Result<Vec<Tag>, AppError> {
    let conn = lock_conn(&state)?;
    if !memo_exists(&conn, input.memo_id)? {
        return Err(AppError::not_found("备忘录不存在"));
    }
    query_tags_for_memo(&conn, input.memo_id)
}

/// C5 整体设置某备忘录标签集合（覆盖式，单事务，get-or-create）
#[tauri::command]
pub fn set_memo_tags(state: State<'_, Db>, input: SetMemoTagsInput) -> Result<Vec<Tag>, AppError> {
    // 先 trim 并去重（NOCASE 语义，去重保留首个）
    let mut seen: HashSet<String> = HashSet::new();
    let mut names: Vec<String> = Vec::new();
    for raw in &input.tag_names {
        let name = raw.trim().to_string();
        validate_tag_name(&name)?;
        // 数据库唯一索引仅折叠 ASCII 大小写，应用层用小写去重对齐
        if seen.insert(name.to_lowercase()) {
            names.push(name);
        }
    }

    let mut conn = lock_conn(&state)?;
    if !memo_exists(&conn, input.memo_id)? {
        return Err(AppError::not_found("备忘录不存在"));
    }

    let tx = conn.transaction().map_err(AppError::from)?;

    let now = now_iso8601();
    let mut tag_ids: Vec<i64> = Vec::new();
    for name in &names {
        let existing = find_tag_id(&tx, name)?;
        let id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO tag (name, created_at) VALUES (?1, ?2)",
                    params![name, now],
                )
                .map_err(AppError::from)?;
                tx.last_insert_rowid()
            }
        };
        tag_ids.push(id);
    }

    tx.execute(
        "DELETE FROM memo_tags WHERE memo_id = ?1",
        params![input.memo_id],
    )
    .map_err(AppError::from)?;
    for tag_id in &tag_ids {
        tx.execute(
            "INSERT INTO memo_tags (memo_id, tag_id) VALUES (?1, ?2)",
            params![input.memo_id, tag_id],
        )
        .map_err(AppError::from)?;
    }
    tx.commit().map_err(AppError::from)?;

    // 使用同一连接回查（事务已提交）
    query_tags_for_memo(&conn, input.memo_id)
}

/// C6 按标签筛选（可与关键词叠加），结果按 updated_at 倒序
#[tauri::command]
pub fn list_memos_by_tag(
    state: State<'_, Db>,
    input: ListMemosByTagInput,
) -> Result<Vec<Memo>, AppError> {
    let conn = lock_conn(&state)?;
    if !tag_exists(&conn, input.tag_id)? {
        return Err(AppError::not_found("标签不存在"));
    }

    let keyword = input.keyword.map(|k| k.trim().to_string());
    let mut result: Vec<Memo> = Vec::new();

    match keyword.as_deref() {
        Some("") | None => {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT m.{cols} FROM memo m \
                     JOIN memo_tags mt ON mt.memo_id = m.id \
                     WHERE mt.tag_id = ?1 ORDER BY m.updated_at DESC",
                    cols = SELECT_COLS
                ))
                .map_err(AppError::from)?;
            let rows = stmt
                .query_map(params![input.tag_id], row_to_memo)
                .map_err(AppError::from)?;
            for row in rows {
                result.push(row.map_err(AppError::from)?);
            }
        }
        Some(kw) => {
            let pattern = format!("%{}%", escape_like(kw));
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT m.{cols} FROM memo m \
                     JOIN memo_tags mt ON mt.memo_id = m.id \
                     WHERE mt.tag_id = ?1 \
                       AND (m.title LIKE ?2 ESCAPE '\\' OR m.content LIKE ?2 ESCAPE '\\') \
                     ORDER BY m.updated_at DESC",
                    cols = SELECT_COLS
                ))
                .map_err(AppError::from)?;
            let rows = stmt
                .query_map(params![input.tag_id, pattern], row_to_memo)
                .map_err(AppError::from)?;
            for row in rows {
                result.push(row.map_err(AppError::from)?);
            }
        }
    }

    Ok(result)
}