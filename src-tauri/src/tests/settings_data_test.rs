// 应用配置数据层测试：默认值、部分合并（save/load）、非法值回退默认、默认快捷键绑定。

use crate::db::init;
use crate::settings::{default_shortcuts, load_settings, save_settings};
use crate::models::AppSettings;
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

#[test]
fn default_shortcuts_match_binding_protocol() {
    let s = default_shortcuts();
    assert_eq!(s.new_memo, "Cmd+N");
    assert_eq!(s.save, "Cmd+S");
    assert_eq!(s.delete, "Cmd+Backspace");
    assert_eq!(s.focus_search, "Cmd+F");
    assert_eq!(s.quick_note, "Cmd+Shift+Space");
}

#[test]
fn load_settings_returns_defaults_on_empty_db() {
    let dir = temp_dir("settings-default");
    let conn = init(&dir).expect("init 失败");

    let s = load_settings(&conn);
    assert_eq!(s.theme, "system");
    assert_eq!(s.close_behavior, "quit");
    assert_eq!(s.shortcuts.delete, "Cmd+Backspace");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn save_then_load_roundtrips_and_merges() {
    let dir = temp_dir("settings-roundtrip");
    let conn = init(&dir).expect("init 失败");

    let mut s = load_settings(&conn);
    s.theme = "dark".to_string();
    save_settings(&conn, &s).expect("保存失败");

    let loaded = load_settings(&conn);
    assert_eq!(loaded.theme, "dark");
    assert_eq!(loaded.close_behavior, "quit", "未写的键应保留默认值");
    assert_eq!(loaded.shortcuts.save, "Cmd+S");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn invalid_stored_values_fall_back_to_defaults() {
    let dir = temp_dir("settings-invalid");
    let conn = init(&dir).expect("init 失败");

    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('theme', 'bogus')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('close_behavior', 'bogus')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('shortcuts', 'not-json')",
        [],
    )
    .unwrap();

    let s = load_settings(&conn);
    assert_eq!(s.theme, "system", "非法主题应回退 system");
    assert_eq!(s.close_behavior, "quit", "非法关闭行为应回退 quit");
    assert_eq!(s.shortcuts.delete, "Cmd+Backspace", "非法 shortcuts JSON 应回退默认");

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn settings_table_shares_no_fields_with_memo() {
    // 应用配置与备忘录数据物理隔离：settings 为独立 key-value 表，不含备忘录字段。
    let dir = temp_dir("settings-isolation");
    let conn = init(&dir).expect("init 失败");

    let mut stmt = conn.prepare("PRAGMA table_info(settings)").unwrap();
    let cols: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(cols, vec!["key", "value"], "settings 表应仅为 key/value 两列");
    drop(stmt);

    drop(conn);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn app_settings_serializes_expected_keys() {
    let s = AppSettings {
        theme: "dark".to_string(),
        close_behavior: "hide".to_string(),
        shortcuts: default_shortcuts(),
    };
    let v = serde_json::to_value(&s).unwrap();
    assert_eq!(v["theme"], "dark");
    assert_eq!(v["close_behavior"], "hide");
    assert_eq!(v["shortcuts"]["delete"], "Cmd+Backspace");
}