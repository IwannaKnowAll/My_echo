// 本地提醒通知：到点经 osascript 发 macOS 系统通知。
//
// 关键决策：`display notification` 无点击回调，无法感知用户是否点击了系统通知，
// 因此「点通知打开对应备忘录」暂以「到点发通知 + 1 秒后自动打开对应备忘录」作为降级实现：
// 提醒触发后无条件 show 主窗口并 emit `open_memo`。待正式方案接入后再改为真点击回调。

use crate::tray::show_main_window;
use serde_json::json;
use std::process::Command;
use tauri::{AppHandle, Emitter};

/// 发送到期提醒：osascript 弹系统通知，并延时 1 秒后打开对应备忘录。
pub fn send_reminder(app: AppHandle, memo_id: i64, title: String) {
    eprintln!("[notification] send_reminder: id={} title={}", memo_id, title);

    // 1) 子线程发 osascript 通知（同步阻塞等待返回，避免占住调度线程）
    std::thread::spawn(move || {
        let script = format!(r#"display notification "备忘提醒" with title "{}""#, title);
        if let Err(e) = Command::new("osascript").arg("-e").arg(&script).output() {
            eprintln!("[notification] osascript error: {:?}", e);
        }
    });

    // 2) 子线程延时 1 秒后打开主窗口并 emit open_memo（见文件头降级说明）
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let h = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            show_main_window(&h);
            let _ = h.emit("open_memo", json!({ "memo_id": memo_id }));
        });
    });
}