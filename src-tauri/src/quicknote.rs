// 速记窗：存在则聚焦，否则构建置顶聚焦的小窗（URL 带 ?mode=quick）

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// 唤起速记窗：已存在则聚焦，否则创建
pub fn show_quick_note(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("quick-note") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let url = quick_note_url(app);
    let _ = WebviewWindowBuilder::new(app, "quick-note", url)
        .title("速记")
        .inner_size(420.0, 340.0)
        .min_inner_size(320.0, 240.0)
        .always_on_top(true)
        .focused(true)
        .build();
}

/// 复用主窗口 URL 并追加 ?mode=quick，兼容 dev（devUrl）与生产（frontendDist）
fn quick_note_url(app: &AppHandle) -> WebviewUrl {
    if let Some(main) = app.get_webview_window("main") {
        if let Ok(mut url) = main.url() {
            url.set_query(Some("mode=quick"));
            return WebviewUrl::External(url);
        }
    }
    WebviewUrl::App("index.html".into())
}