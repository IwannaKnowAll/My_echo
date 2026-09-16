// 应用配置：运行时缓存 + 默认值 + 校验/冲突检测 + get_settings/update_settings 命令

use crate::db::Db;
use crate::error::AppError;
use crate::models::{AppSettings, ShortcutBindings, UpdateSettingsInput};
use crate::shortcut;
use rusqlite::{params, Connection, OptionalExtension};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::State;

/// 运行时配置缓存：close_behavior 供关窗策略读取、quick_note 供快捷键重注册
#[derive(Clone, Default)]
pub struct RuntimeConfig {
    pub close_behavior: Arc<Mutex<String>>,
    pub quick_note: Arc<Mutex<String>>,
}

/// 快捷键默认绑定（应用内 4 项 + 全局速记 1 项，对齐 AGENTS.md 5.4）
pub fn default_shortcuts() -> ShortcutBindings {
    ShortcutBindings {
        new_memo: "Cmd+N".to_string(),
        save: "Cmd+S".to_string(),
        delete: "Cmd+Backspace".to_string(),
        focus_search: "Cmd+F".to_string(),
        quick_note: "Cmd+Shift+Space".to_string(),
    }
}

fn default_settings() -> AppSettings {
    AppSettings {
        theme: "system".to_string(),
        close_behavior: "quit".to_string(),
        shortcuts: default_shortcuts(),
    }
}

/// 读取单条配置值
fn read_value(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(AppError::from)
}

/// 覆盖写单条配置值
fn write_value(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(AppError::from)?;
    Ok(())
}

/// 读取配置（缺失/非法回退默认值）
pub fn load_settings(conn: &Connection) -> AppSettings {
    let mut s = default_settings();
    if let Ok(Some(v)) = read_value(conn, "theme") {
        if is_valid_theme(&v) {
            s.theme = v;
        }
    }
    if let Ok(Some(v)) = read_value(conn, "close_behavior") {
        if is_valid_close_behavior(&v) {
            s.close_behavior = v;
        }
    }
    if let Ok(Some(v)) = read_value(conn, "shortcuts") {
        if let Ok(bindings) = serde_json::from_str::<ShortcutBindings>(&v) {
            s.shortcuts = bindings;
        }
    }
    s
}

/// 持久化配置
pub fn save_settings(conn: &Connection, s: &AppSettings) -> Result<(), AppError> {
    write_value(conn, "theme", &s.theme)?;
    write_value(conn, "close_behavior", &s.close_behavior)?;
    write_value(conn, "shortcuts", &serde_json::to_string(&s.shortcuts).unwrap_or_default())?;
    Ok(())
}

fn is_valid_theme(v: &str) -> bool {
    matches!(v, "system" | "light" | "dark")
}

fn is_valid_close_behavior(v: &str) -> bool {
    matches!(v, "quit" | "hide")
}

/// 快捷键合法性：含修饰键或为功能键（F1–F12）
fn is_valid_shortcut(binding: &str) -> bool {
    let b = binding.trim();
    if b.is_empty() {
        return false;
    }
    let upper = b.to_uppercase();
    let is_fn_key = (1..=12).any(|n| upper == format!("F{n}"));
    if is_fn_key {
        return true;
    }
    b.split('+').map(|p| p.trim().to_uppercase()).any(|p| {
        matches!(
            p.as_str(),
            "CMD"
                | "COMMAND"
                | "CTRL"
                | "CONTROL"
                | "OPTION"
                | "ALT"
                | "SHIFT"
                | "SUPER"
                | "CMDORCTRL"
                | "COMMANDORCONTROL"
        )
    })
}

/// 快捷键显示名（冲突提示用）
fn shortcut_label(key: &str) -> &str {
    match key {
        "new_memo" => "新建",
        "save" => "保存",
        "delete" => "删除",
        "focus_search" => "搜索",
        "quick_note" => "全局速记",
        _ => key,
    }
}

/// 应用内四键互查：返回冲突的按键名
fn find_app_conflict(name: &str, value: &str, b: &ShortcutBindings) -> Option<&'static str> {
    let norm = value.trim().to_lowercase();
    let candidates: [(&'static str, &str); 4] = [
        ("new_memo", &b.new_memo),
        ("save", &b.save),
        ("delete", &b.delete),
        ("focus_search", &b.focus_search),
    ];
    for (other_name, other_val) in candidates {
        if other_name != name && other_val.trim().to_lowercase() == norm {
            return Some(other_name);
        }
    }
    None
}

/// 校验一整组快捷键绑定
fn validate_shortcuts(b: &ShortcutBindings) -> Result<(), AppError> {
    for (_, value) in [
        ("new_memo", &b.new_memo),
        ("save", &b.save),
        ("delete", &b.delete),
        ("focus_search", &b.focus_search),
        ("quick_note", &b.quick_note),
    ] {
        if !is_valid_shortcut(value) {
            return Err(AppError::validation("快捷键必须包含修饰键或为功能键"));
        }
    }
    for name in ["new_memo", "save", "delete", "focus_search"] {
        let value = match name {
            "new_memo" => &b.new_memo,
            "save" => &b.save,
            "delete" => &b.delete,
            "focus_search" => &b.focus_search,
            _ => unreachable!(),
        };
        if let Some(c) = find_app_conflict(name, value, b) {
            return Err(AppError::validation(format!(
                "该组合已被『{}』使用",
                shortcut_label(c)
            )));
        }
    }
    Ok(())
}

/// C9 读配置（首读无记录时幂等回写默认值）
#[tauri::command]
pub fn get_settings(state: State<'_, Db>) -> Result<AppSettings, AppError> {
    let conn = crate::commands::lock_conn(&state)?;
    let s = load_settings(&conn);
    // 首读无记录：回写默认值，保证后续 update_settings 有合并基线
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
        .map_err(AppError::from)?;
    if count == 0 {
        save_settings(&conn, &s)?;
    }
    Ok(s)
}

/// C10 更新配置（部分字段合并，含副作用）
#[tauri::command]
pub fn update_settings(
    app: tauri::AppHandle,
    state: State<'_, Db>,
    runtime: State<'_, RuntimeConfig>,
    input: UpdateSettingsInput,
) -> Result<AppSettings, AppError> {
    if input.theme.is_none() && input.close_behavior.is_none() && input.shortcuts.is_none() {
        return Err(AppError::validation("没有需要更新的配置"));
    }
    if let Some(t) = &input.theme {
        if !is_valid_theme(t) {
            return Err(AppError::validation("主题配置不合法"));
        }
    }
    if let Some(c) = &input.close_behavior {
        if !is_valid_close_behavior(c) {
            return Err(AppError::validation("关闭行为配置不合法"));
        }
    }
    if let Some(sc) = &input.shortcuts {
        validate_shortcuts(sc)?;
    }

    let conn = crate::commands::lock_conn(&state)?;
    let mut merged = load_settings(&conn);
    if let Some(t) = input.theme {
        merged.theme = t;
    }
    if let Some(c) = input.close_behavior {
        merged.close_behavior = c;
    }
    if let Some(sc) = input.shortcuts {
        merged.shortcuts = sc;
    }
    save_settings(&conn, &merged)?;

    // 副作用：更新运行时缓存 + 重注册全局快捷键
    if let Ok(mut cb) = runtime.close_behavior.lock() {
        *cb = merged.close_behavior.clone();
    }
    if let Ok(mut qn) = runtime.quick_note.lock() {
        let new_qn = merged.shortcuts.quick_note.clone();
        if *qn != new_qn {
            *qn = new_qn.clone();
            shortcut::register_global(&app, &new_qn);
        }
    }

    Ok(merged)
}

/// C11 打开系统通知设置（纯本地系统调用，无网络、无入参）
#[tauri::command]
pub fn open_notification_settings() -> Result<(), AppError> {
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.notifications")
        .spawn()
        .map_err(|_| AppError::internal("打开系统设置失败"))?;
    Ok(())
}