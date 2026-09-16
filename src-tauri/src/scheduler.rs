// 提醒到点调度：后台线程定期扫描最近到期时间，到点发通知并清空 remind_at

use crate::db;
use crate::notification;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tauri::AppHandle;

/// 调度器状态：命令侧通过 poke 唤醒线程重新扫描
pub struct Scheduler {
    wake: Arc<(Mutex<bool>, Condvar)>,
}

impl Scheduler {
    /// 启动后台调度线程，持有独立的数据库连接
    pub fn start(db_path: PathBuf, app: AppHandle) -> Self {
        let wake = Arc::new((Mutex::new(false), Condvar::new()));
        let w = wake.clone();
        std::thread::Builder::new()
            .name("reminder-scheduler".into())
            .spawn(move || scheduler_loop(db_path, app, w))
            .ok();
        Self { wake }
    }

    /// 唤醒调度线程（设置/清除提醒后调用）
    pub fn poke(&self) {
        let (flag, cv) = &*self.wake;
        if let Ok(mut g) = flag.lock() {
            *g = true;
            cv.notify_all();
        }
    }
}

fn scheduler_loop(db_path: PathBuf, app: AppHandle, wake: Arc<(Mutex<bool>, Condvar)>) {
    let conn = match db::open(&db_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    loop {
        let next_wait = scan_due(&conn, &app);

        let (flag, cv) = &*wake;
        let mut guard = match flag.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if *guard {
            *guard = false;
            continue;
        }
        let wait = next_wait.unwrap_or_else(|| Duration::from_secs(3600));
        match cv.wait_timeout(guard, wait) {
            Ok(_) => {}
            Err(_) => return,
        }
    }
}

/// 扫描并触发所有已到期提醒，返回距下一个未到期提醒的等待时长
fn scan_due(conn: &rusqlite::Connection, app: &AppHandle) -> Option<Duration> {
    loop {
        let row = conn
            .query_row(
                "SELECT id, title, remind_at FROM memo WHERE remind_at <> '' \
                 ORDER BY remind_at ASC LIMIT 1",
                [],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .ok();

        let Some((id, title, remind_at)) = row else {
            return None;
        };

        let due = match chrono::DateTime::parse_from_rfc3339(&remind_at) {
            Ok(d) => d.with_timezone(&chrono::Utc),
            // 异常数据：清除提醒，继续扫描
            Err(_) => {
                let _ = conn.execute("UPDATE memo SET remind_at = '' WHERE id = ?1", [id]);
                continue;
            }
        };

        let now = chrono::Utc::now();
        if due <= now {
            notification::send_reminder(app.clone(), id, title);
            let _ = conn.execute("UPDATE memo SET remind_at = '' WHERE id = ?1", [id]);
            continue;
        } else {
            return Some((due - now).to_std().unwrap_or(Duration::ZERO));
        }
    }
}