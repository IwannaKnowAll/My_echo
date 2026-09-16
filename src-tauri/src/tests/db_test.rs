// db 模块测试：建表字段、CHECK 约束、索引、越界字段静态断言。

use crate::db::{init, SCHEMA_SQL};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// 每个测试独立的临时目录，避免并行测试互相污染。
fn unique_temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时间早于 Unix 纪元")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("my-echo-{tag}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("创建临时目录失败");
    dir
}

#[test]
fn init_creates_memo_table_with_exactly_six_fields_in_order() {
    let dir = unique_temp_dir("init-fields");
    let conn = init(&dir).expect("init 建库建表失败");

    let cols: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(memo)").unwrap();
        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };

    assert_eq!(
        cols,
        vec![
            "id",
            "title",
            "content",
            "created_at",
            "updated_at",
            "remind_at"
        ],
        "表字段必须恰好为契约 6 字段且顺序一致"
    );

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn init_creates_check_constraint_and_updated_at_index() {
    let dir = unique_temp_dir("init-schema");
    let conn = init(&dir).expect("init 建库建表失败");

    let table_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'memo'",
            [],
            |row| row.get(0),
        )
        .expect("应存在 memo 表");
    assert!(table_sql.contains("CHECK"), "建表 SQL 应包含 CHECK 约束");

    let index_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_memo_updated_at'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(index_count, 1, "应存在 idx_memo_updated_at 索引");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_constraint_rejects_title_over_100_chars() {
    let dir = unique_temp_dir("init-check");
    let conn = init(&dir).expect("init 建库建表失败");

    let overlong = "字".repeat(101);
    let insert_over = conn.execute(
        "INSERT INTO memo (title, content, created_at, updated_at) \
         VALUES (?1, '', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z')",
        rusqlite::params![overlong],
    );
    assert!(insert_over.is_err(), "超过 100 字符的标题应被 CHECK 约束拒绝");

    // 恰好 100 字符应允许插入
    let ok = "字".repeat(100);
    conn.execute(
        "INSERT INTO memo (title, content, created_at, updated_at) \
         VALUES (?1, '', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z')",
        rusqlite::params![ok],
    )
    .expect("恰好 100 字符的标题应插入成功");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn schema_sql_has_no_p1_p2_fields() {
    // 越界字段静态断言：P1/P2 功能（置顶/归档/回收站/加密）不得预留字段。
    assert!(!SCHEMA_SQL.contains("is_pinned"));
    assert!(!SCHEMA_SQL.contains("deleted_at"));
    assert!(!SCHEMA_SQL.contains("archived"));
    assert!(!SCHEMA_SQL.contains("encrypt"));
    assert!(!SCHEMA_SQL.contains("pinned"));
}