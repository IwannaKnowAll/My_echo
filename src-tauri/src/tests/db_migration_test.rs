// 轻量强化包迁移幂等测试：存量 5 列 → 6 列、新库 6 列、三张新表与索引、user_version。

use crate::db::init;
use rusqlite::Connection;
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

fn memo_columns(conn: &Connection) -> Vec<String> {
    let mut stmt = conn.prepare("PRAGMA table_info(memo)").unwrap();
    stmt.query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn table_exists(conn: &Connection, name: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |row| row.get(0),
        )
        .unwrap();
    count == 1
}

fn index_exists(conn: &Connection, name: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
            [name],
            |row| row.get(0),
        )
        .unwrap();
    count == 1
}

#[test]
fn new_db_init_has_exactly_six_memo_columns_in_order() {
    let dir = temp_dir("migration-new");
    let conn = init(&dir).expect("init 新库失败");

    assert_eq!(
        memo_columns(&conn),
        vec!["id", "title", "content", "created_at", "updated_at", "remind_at"],
        "新库应直接为 6 字段且 remind_at 在末尾"
    );

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn legacy_five_column_db_gets_remind_at_appended() {
    let dir = temp_dir("migration-legacy");
    let db_path = dir.join("my-echo.db");

    // 模拟 P0 已交付的 5 列存量库
    {
        let conn = Connection::open(&db_path).expect("打开旧库失败");
        conn.execute_batch(
            "CREATE TABLE memo (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                title      TEXT    NOT NULL,
                content    TEXT    NOT NULL DEFAULT '',
                created_at TEXT    NOT NULL,
                updated_at TEXT    NOT NULL,
                CONSTRAINT chk_title_len CHECK (length(title) <= 100)
            );
            CREATE INDEX idx_memo_updated_at ON memo (updated_at DESC);",
        )
        .expect("建旧表失败");
    }

    let conn = init(&dir).expect("init 迁移失败");

    assert_eq!(
        memo_columns(&conn),
        vec!["id", "title", "content", "created_at", "updated_at", "remind_at"],
        "存量 5 列库应补出 remind_at 且位于末尾"
    );

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn migration_creates_tag_memo_tags_settings_tables() {
    let dir = temp_dir("migration-tables");
    let conn = init(&dir).expect("init 失败");

    assert!(table_exists(&conn, "tag"), "应存在 tag 表");
    assert!(table_exists(&conn, "memo_tags"), "应存在 memo_tags 表");
    assert!(table_exists(&conn, "settings"), "应存在 settings 表");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn migration_creates_required_indexes() {
    let dir = temp_dir("migration-indexes");
    let conn = init(&dir).expect("init 失败");

    assert!(index_exists(&conn, "idx_tag_name_unique"), "应存在标签唯一索引");
    assert!(index_exists(&conn, "idx_memo_tags_tag"), "应存在 memo_tags 反查索引");
    assert!(index_exists(&conn, "idx_memo_updated_at"), "应存在备忘录时间索引");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn migration_sets_user_version_one() {
    let dir = temp_dir("migration-version");
    let conn = init(&dir).expect("init 失败");

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1, "迁移后 user_version 应为 1");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn migration_is_idempotent() {
    let dir = temp_dir("migration-idempotent");
    let conn = init(&dir).expect("首次 init 失败");
    drop(conn);

    // 第二次打开迁移路径应无重跑报错，字段不变
    let conn2 = init(&dir).expect("二次 init 失败");
    assert_eq!(memo_columns(&conn2).len(), 6, "重开后 memo 仍应 6 列");

    drop(conn2);
    let _ = fs::remove_dir_all(&dir);
}