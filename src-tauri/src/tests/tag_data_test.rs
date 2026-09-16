// 标签数据层语义测试：NOCASE 去重、长度 CHECK、联合主键、外键级联（删 memo/tag 无孤儿关系）。

use crate::db::init;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时间早于 Unix 纪元")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("my-echo-{tag}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("创建临时目录失败");
    dir
}

fn insert_memo(conn: &Connection, title: &str) -> i64 {
    conn.execute(
        "INSERT INTO memo (title, content, created_at, updated_at) \
         VALUES (?1, '', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z')",
        params![title],
    )
    .expect("插入备忘录失败");
    conn.last_insert_rowid()
}

fn insert_tag(conn: &Connection, name: &str) -> i64 {
    conn.execute(
        "INSERT INTO tag (name, created_at) VALUES (?1, '2026-01-01T00:00:00.000Z')",
        params![name],
    )
    .expect("插入标签失败");
    conn.last_insert_rowid()
}

fn memo_tags_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM memo_tags", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn tag_name_unique_index_is_case_insensitive() {
    let dir = temp_dir("tag-nocase");
    let conn = init(&dir).expect("init 失败");

    insert_tag(&conn, "Work");
    // 大小写变体应被 NOCASE 唯一索引拒绝
    let dup = conn.execute(
        "INSERT INTO tag (name, created_at) VALUES ('work', '2026-01-01T00:00:00.000Z')",
        [],
    );
    assert!(dup.is_err(), "英文重名（大小写不敏感）应被唯一索引拒绝");

    // 区分大小写之外的不同名仍可插入
    insert_tag(&conn, "Life");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tag_name_length_check_rejects_over_30() {
    let dir = temp_dir("tag-len");
    let conn = init(&dir).expect("init 失败");

    let overlong = "字".repeat(31);
    let r = conn.execute(
        "INSERT INTO tag (name, created_at) VALUES (?1, '2026-01-01T00:00:00.000Z')",
        params![overlong],
    );
    assert!(r.is_err(), "超过 30 字符的标签名应被 CHECK 拒绝");

    let ok = "字".repeat(30);
    conn.execute(
        "INSERT INTO tag (name, created_at) VALUES (?1, '2026-01-01T00:00:00.000Z')",
        params![ok],
    )
    .expect("恰好 30 字符应插入成功");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn memo_tags_has_composite_primary_key() {
    let dir = temp_dir("tag-pk");
    let conn = init(&dir).expect("init 失败");

    let memo_id = insert_memo(&conn, "m");
    let tag_id = insert_tag(&conn, "tag");

    conn.execute(
        "INSERT INTO memo_tags (memo_id, tag_id) VALUES (?1, ?2)",
        params![memo_id, tag_id],
    )
    .expect("首次插入关系成功");

    let dup = conn.execute(
        "INSERT INTO memo_tags (memo_id, tag_id) VALUES (?1, ?2)",
        params![memo_id, tag_id],
    );
    assert!(dup.is_err(), "联合主键应拒绝重复关系");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn deleting_memo_cascades_removes_relations() {
    let dir = temp_dir("tag-cascade-memo");
    let conn = init(&dir).expect("init 失败");

    let memo_id = insert_memo(&conn, "m");
    let tag_id = insert_tag(&conn, "tag");
    conn.execute(
        "INSERT INTO memo_tags (memo_id, tag_id) VALUES (?1, ?2)",
        params![memo_id, tag_id],
    )
    .unwrap();
    assert_eq!(memo_tags_count(&conn), 1);

    conn.execute("DELETE FROM memo WHERE id = ?1", params![memo_id])
        .expect("删除备忘录失败");
    assert_eq!(memo_tags_count(&conn), 0, "删除备忘录后关系应无残留");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn deleting_tag_cascades_removes_relations_keeps_memo() {
    let dir = temp_dir("tag-cascade-tag");
    let conn = init(&dir).expect("init 失败");

    let memo_id = insert_memo(&conn, "m");
    let tag_id = insert_tag(&conn, "tag");
    conn.execute(
        "INSERT INTO memo_tags (memo_id, tag_id) VALUES (?1, ?2)",
        params![memo_id, tag_id],
    )
    .unwrap();
    assert_eq!(memo_tags_count(&conn), 1);

    conn.execute("DELETE FROM tag WHERE id = ?1", params![tag_id])
        .expect("删除标签失败");
    assert_eq!(memo_tags_count(&conn), 0, "删除标签后关系应无残留");

    // 备忘录本体仍在（仅解除关联）
    let memo_left: i64 = conn
        .query_row("SELECT COUNT(*) FROM memo WHERE id = ?1", params![memo_id], |r| r.get(0))
        .unwrap();
    assert_eq!(memo_left, 1, "删除标签不应删除备忘录");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}