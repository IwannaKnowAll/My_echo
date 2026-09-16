// 全局快捷键：仅注册 quick_note，触发即唤起速记窗

use crate::quicknote;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// 注册全局快捷键（先注销全部，再注册当前 quick_note；解析失败/系统占用静默降级）
pub fn register_global(app: &AppHandle, shortcut_str: &str) {
    if shortcut_str.trim().is_empty() {
        return;
    }
    let _ = app.global_shortcut().unregister_all();
    let _ = app
        .global_shortcut()
        .on_shortcut(shortcut_str, |app, _shortcut, event| {
            if matches!(event.state, ShortcutState::Pressed) {
                quicknote::show_quick_note(app);
            }
        });
}