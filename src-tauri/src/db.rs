// SQLite 连接封装与建表迁移

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

/// 全局共享的数据库连接（单进程单用户，无需连接池）
pub struct Db(pub Mutex<Connection>);

/// 备忘录建表 SQL（严格对齐 Planner 1.1：5 字段 + 1 个 CHECK + 1 个索引；remind_at 由迁移补列）
pub const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS memo (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    title      TEXT    NOT NULL,
    content    TEXT    NOT NULL DEFAULT '',
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL,
    CONSTRAINT chk_title_len CHECK (length(title) <= 100)
);

CREATE INDEX IF NOT EXISTS idx_memo_updated_at ON memo (updated_at DESC);
";

/// 轻量强化包增量 SQL（标签/关系/配置表），对齐 Planner 1.2
pub const MIGRATION_SQL: &str = "
CREATE TABLE IF NOT EXISTS tag (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    CONSTRAINT chk_tag_name_len CHECK (length(name) <= 30)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_tag_name_unique ON tag (name COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS memo_tags (
    memo_id INTEGER NOT NULL,
    tag_id  INTEGER NOT NULL,
    PRIMARY KEY (memo_id, tag_id),
    FOREIGN KEY (memo_id) REFERENCES memo (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)  REFERENCES tag  (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_memo_tags_tag ON memo_tags (tag_id);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

/// 打开数据库连接并执行迁移（幂等）。调度线程复用此函数建立第二连接。
pub fn open(path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    // SQLite 外键默认关闭，必须每连接开启
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    // 双连接并发下的写竞争规避
    conn.busy_timeout(Duration::from_secs(5))?;
    migrate(&conn)?;
    Ok(conn)
}

/// 首次启动：创建数据目录 → 打开 my-echo.db → 执行建表迁移
pub fn init(data_dir: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(data_dir)?;
    open(&data_dir.join("my-echo.db"))
}

/// 迁移：建表 + memo 存量列补 remind_at + 记录 user_version（幂等）
fn migrate(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(SCHEMA_SQL)?;
    conn.execute_batch(MIGRATION_SQL)?;

    if !column_exists(conn, "memo", "remind_at")? {
        conn.execute_batch("ALTER TABLE memo ADD COLUMN remind_at TEXT NOT NULL DEFAULT '';")?;
    }

    conn.execute_batch("PRAGMA user_version = 1;")?;
    Ok(())
}

/// 检测表是否已存在某列（列存在性迁移，幂等）
fn column_exists(
    conn: &Connection,
    table: &str,
    column: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}