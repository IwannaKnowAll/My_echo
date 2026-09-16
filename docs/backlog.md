# My Echo 待办清单（P3 与衔接项）

> 来源：Reviewer 首轮审查 + P2 修复增量复审。均不阻塞当前 P0 交付。

## 1. P3 问题清单

| 编号 | 位置 | 问题 | 建议 |
|---|---|---|---|
| P3-1 | src-tauri/tauri.conf.json | 窗口 960×680，偏离设计建议的 720×560，未设最小尺寸 | 改 720×560 并补 minWidth/minHeight |
| P3-2 | src-tauri/tauri.conf.json | csp 为 null 未配置 | 配 `"csp": "default-src 'self'"` 基线 |
| P3-3 | client/src/components/SearchBox.vue + App.vue | 清空搜索时 list_memos 触发两次（冗余一次查询，有 requestId 守卫不崩） | 清空路径收敛为单次触发 |
| P3-4 | client/src/App.vue | 标题长度前端用 UTF-16 码元、后端用 Unicode 标量，emoji 等增补字符下口径差 1 | 前端改用 `[...value].length` |
| P3-5 | client/src/components/MemoListItem.vue | `<button>` 内嵌 div/p 无效 HTML（渲染正常无功能影响） | 改为 div[role=button] 或内部用 span |
| P3-6 | client/src/App.vue | E_NOT_FOUND 的 listNotice 在搜索/清空搜索时不重置，可能跨查询残留 | doSearch / onSearchClear 中清空 listNotice |

## 2. 测试衔接项（当前已按分工处理）

- 命令层端到端行为（真实 invoke、SQLite 排序、搜索转义、重启持久化）由主 Agent 集成验证负责，已完成/待完成记录见会话。
- escape_like / validate_title 为私有函数不可直测；如需单测需改 pub(crate) 或内嵌 #[cfg(test)]（涉及业务代码，走变更流程）。
- Command 含 tauri::State 无法纯单测；如需可引入 tauri::test 特性（需改 Cargo.toml）。

## 3. 发布相关

- S9 打包前：正式设计应用图标（当前为纯色占位）；macOS 签名策略待主 Agent 决策（开发签名 / ad-hoc / 延后）；发布清单需如实标注签名状态。

## 4. 首次发布查验新增 P3（0.1.0 已打包，Reviewer 放行）

| 编号 | 事项 | 建议 |
|---|---|---|
| P3-7 | bundle identifier 为 `com.myecho.app`，结尾 `.app` 与 macOS 应用包扩展名语义易混淆 | 改为 `com.myecho.desktop` 之类；注意改动会迁移数据目录 `~/Library/Application Support/...` 路径，正式分发前定 |
| P3-8 | 图标为 512×512 纯色占位 | 正式发布前替换为真实品牌图标（多尺寸 ICNS 源） |
| P3-9 | 仅 ad-hoc 签名，dmg 未签名、未公证 | 本机自用可用；对外分发会被 Gatekeeper 拦截，需开发者证书签名+公证 |
| P3-10 | ~~核心代码未入库~~ | 已完成：2026-09-16 分两组提交（eb525d2 配置文档 + db817ec 代码共 79 文件），工作区干净 |

## 5. 轻量强化包首轮审查新增 P3（F06-F11）

| 编号 | 事项 | 建议 |
|---|---|---|
| P3-11 | reminder.rs 过期检查先于 memo 存在性检查，memo 不存在且时间过期时返回 E_VALIDATION 而非 E_NOT_FOUND | 调整检查顺序：先存在性后过期 |
| P3-12 | settings.rs is_valid_shortcut 只判含修饰键，裸 "Cmd"（无主键）也能通过 | 补主键存在校验 |
| P3-13 | 前端标题/标签长度用 JS .length（UTF-16），与 Rust chars().count() 在 emoji 场景不一致 | 前端改 [...value].length |
| P3-14 | SettingsPanel saving prop 与 TodoItem line prop 定义未使用 | 清理冗余 prop |
| P3-15 | ~~权限降级检测用 WebView Notification.permission~~ | 已失效：通知改 osascript 方案（不经 Web 权限体系），前端权限提示为死代码，2026-09-16 与 P3-21 一并清理完毕 |
| P3-16 | ReminderField 前端判过期到毫秒 vs 后端截断分钟判定边界 | 统一口径（前端截断到分钟再判） |
| P3-17 | todo.ts 行尾 `\s?` 残余，会吞全角空格开头文本的前导空格 | 改为 `[ \t]?` 与前半段一致 |
| P3-18 | ~~notification.rs 内层 wait_for_action 线程常驻~~ | 已过时：notify-rust 实现已被 osascript 方案整体替换，此代码路径不存在，2026-09-16 清出清单 |
| P3-19 | 多实例/多版本并存导致 GUI 验证污染（/Applications v0.1.0、release bundle、Dock 入口、debug bundle 均曾与调试实例并存，窗口同名难辨） | 已处理：旧版移出 /Applications、bundle 删除、Dock 清理；今后集成验证前必须确认 pgrep 单实例且路径为目标二进制；v0.2.0 发布时重新安装正式版 |
| P3-20 | notification.rs 中标题直接拼进 AppleScript 双引号字符串，含双引号/反斜杠的标题会破坏脚本导致该次通知不弹 | 拼接前对 title 做转义 |
| P3-21 | ~~权限降级检测在 osascript 方案下失效~~ | 已清理：2026-09-16 移除前端权限检测死代码（App.vue 与 ReminderField 的 permissionDenied 逻辑及测试），111 测试全绿 |

## 7. P3 批量修复销项记录（2026-09-16）

以下各项已修复并经 Reviewer 复审关闭（壳层 34 测试、前端 117 测试全绿；release 实机验证通过）：

- 已关闭：P3-1（窗口 720×560 + min 560×440）、P3-2（csp `default-src 'self'`，release 实机验证无白屏）、P3-3、P3-4、P3-5（含 Enter 键盘可达，Space 待补）、P3-6、P3-11、P3-12、P3-13、P3-14、P3-16、P3-17、P3-20

- P3-24 · 已知行为记录：debug 裸二进制（未走 tauri dev）在 devUrl 存在时回退嵌入资源会白屏——属 Tauri 开发模式边界，不影响 release 产物；开发验证一律用 `tauri dev` 或 release 包（集成验证纪律，与 P3-19 相关）。

## 6. v0.2.0 发布查验新增 P3

| 编号 | 事项 | 建议 |
|---|---|---|
| P3-22 | identifier `com.myecho.app` 警告未处理 | 正式分发前改 `com.myecho.desktop` 并迁移数据目录 |
| P3-23 | ad-hoc 签名 + 未公证 | 对外分发需开发者证书+公证（P3-9 同源，延用） |