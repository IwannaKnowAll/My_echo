// My Echo 壳层入口：注册命令 + 初始化 SQLite + 管理全局状态

mod commands;
mod db;
mod error;
mod models;
mod notification;
mod quicknote;
mod scheduler;
mod settings;
mod shortcut;
mod tray;

use db::Db;
use settings::RuntimeConfig;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // 数据文件：~/Library/Application Support/com.myecho.app/my-echo.db
            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("my-echo.db");
            let conn = db::init(&data_dir)?;

            // 运行时配置缓存：关闭行为 + 全局快捷键
            let runtime = RuntimeConfig::default();
            {
                let loaded = settings::load_settings(&conn);
                if let Ok(mut cb) = runtime.close_behavior.lock() {
                    *cb = loaded.close_behavior.clone();
                }
                if let Ok(mut qn) = runtime.quick_note.lock() {
                    *qn = loaded.shortcuts.quick_note.clone();
                }
                shortcut::register_global(
                    app.handle(),
                    &loaded.shortcuts.quick_note.clone(),
                );
            }

            app.manage(Db(Mutex::new(conn)));
            app.manage(runtime.clone());

            // 菜单栏托盘
            tray::setup_tray(app)?;

            // 关窗策略：hide 时仅隐藏，quit 放行默认关闭即退出
            if let Some(main) = app.get_webview_window("main") {
                let main_for_event = main.clone();
                let close_behavior = runtime.close_behavior.clone();
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let hide = close_behavior
                            .lock()
                            .map(|g| *g == "hide")
                            .unwrap_or(false);
                        if hide {
                            api.prevent_close();
                            let _ = main_for_event.hide();
                        }
                    }
                });
            }

            // 提醒到点调度
            let scheduler = scheduler::Scheduler::start(db_path, app.handle().clone());
            app.manage(scheduler);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::memo::create_memo,
            commands::memo::list_memos,
            commands::memo::get_memo,
            commands::memo::update_memo,
            commands::memo::delete_memo,
            commands::memo::search_memos,
            commands::tag::list_tags,
            commands::tag::create_tag,
            commands::tag::delete_tag,
            commands::tag::list_memo_tags,
            commands::tag::set_memo_tags,
            commands::tag::list_memos_by_tag,
            commands::reminder::set_memo_reminder,
            commands::reminder::clear_memo_reminder,
            settings::get_settings,
            settings::update_settings,
            settings::open_notification_settings,
        ])
        .run(tauri::generate_context!())
        .expect("My Echo 启动失败");
}

#[cfg(test)]
mod tests;