# My Echo 轻量强化包（F06–F11）增量规划

> 范围：在 P0 已交付的 6 命令 + 单窗口之上，增量实现标签、待办勾选、全局速记、本地提醒、三态主题、快捷键。
> 事实源优先级：AGENTS.md（唯一）> PRD > 本规划；最终以 AGENTS.md 5.2/5.3/5.4 为准。
> 硬约束沿用：纯本地、无账号、无网络、Tauri 2 + rusqlite（bundled）+ chrono、契约优先、禁 any、错误码仅 E_VALIDATION / E_NOT_FOUND / E_INTERNAL、invoke 第二参数统一 `{ input }`。
> 时间戳约定不改：所有时间字段统一存 UTC RFC3339 毫秒、Z 结尾（文本字典序 === 时间序）。

## 一、数据库增量设计

### 1.1 迁移机制选型（决策）

选「列存在性检测 + `ALTER TABLE ADD COLUMN`」+ 引入 `PRAGMA user_version` 占位，不引入重型迁移框架。

理由：
- 本期迁移只有两件事：给存量 `memo` 表补 1 列（`remind_at`）、新建 3 张新表（`CREATE TABLE IF NOT EXISTS` 天然幂等）。补列是 SQLite 最轻量迁移，`PRAGMA table_info(memo)` 检测列缺失即补，重启重跑不重复、天然幂等，无需版本号跳转逻辑。
- `user_version` 本期仅落 `=1` 作为「已执行轻量迁移」的记录与未来 F04（软删 `deleted_at` + 数据回填）、F05（加密列迁移）复杂迁移的入口；本期不加迁移链框架，避免过度设计。
- 执行方式：`db::init` 打开连接后，顺序执行 ① `PRAGMA foreign_keys = ON`；② 建新表 SQL；③ 若 `memo` 无 `remind_at` 列则 `ALTER TABLE memo ADD COLUMN ...`；④ `PRAGMA user_version = 1`。

### 1.2 增量 SQL

```sql
-- 每次连接必须开启外键（SQLite 默认关闭，per-connection，rusqlite 打开后立即执行）
PRAGMA foreign_keys = ON;

-- memo 存量列迁移（新库也可直接并入建表；这里用迁移补列）
-- 仅当 PRAGMA table_info(memo) 无 remind_at 列时执行：
ALTER TABLE memo ADD COLUMN remind_at TEXT NOT NULL DEFAULT '';

-- 标签表
CREATE TABLE IF NOT EXISTS tag (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    CONSTRAINT chk_tag_name_len CHECK (length(name) <= 30)
);
-- 不区分大小写去重：NOCASE 唯一索引（仅 ASCII 大小写折叠，覆盖验收「英文重名去重」）
CREATE UNIQUE INDEX IF NOT EXISTS idx_tag_name_unique ON tag (name COLLATE NOCASE);

-- 备忘录-标签关系表（多对多，联合主键）
CREATE TABLE IF NOT EXISTS memo_tags (
    memo_id INTEGER NOT NULL,
    tag_id  INTEGER NOT NULL,
    PRIMARY KEY (memo_id, tag_id),
    FOREIGN KEY (memo_id) REFERENCES memo (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)  REFERENCES tag  (id) ON DELETE CASCADE
);
-- 按标签反查备忘录的索引
CREATE INDEX IF NOT EXISTS idx_memo_tags_tag ON memo_tags (tag_id);

-- 应用配置表（key-value，与备忘录数据物理隔离）
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

外键策略说明：
- 采用「外键声明 + `ON DELETE CASCADE` + 每连接 `PRAGMA foreign_keys=ON`」再加应用层显式删除兜底（`delete_memo` 内 `DELETE FROM memo_tags WHERE memo_id=?`；`delete_tag` 内 `DELETE FROM memo_tags WHERE tag_id=?`）。
- 外键只做一致性兜底，主路径走应用层显式删，避免「某处忘开 PRAGMA 导致孤儿关系」。
- 兼容 F04 软删：F04 改软删是 `UPDATE ... SET deleted_at` 而非 `DELETE`，外键 CASCADE 不被触发、关系与提醒随行保留，恢复即还原，与未来方案不冲突。

### 1.3 标签去重策略（决策）

选「`COLLATE NOCASE` 唯一索引（DB 兜底）+ 应用层 trim/规范化后查询 (get-or-create)」，双保险。

取舍理由：
- 唯一索引提供原子去重与并发兜底，防「应用层查询再插入」的竞态。
- `COLLATE NOCASE` 仅折叠 ASCII 大小写（A–Z）；需求中的「大小写不敏感」在中文标签场景正是针对英文字母，覆盖验收。全角/非 ASCII 大小写不在验收范围，不加复杂度。
- 应用层 get-or-create 语义正好支撑「编辑器输入已有标签名 → 直接挂上该标签」（PRD 3.1）的落地：`set_memo_tags` 对每个名称 `SELECT id FROM tag WHERE name = ?1 COLLATE NOCASE`，命中复用、未中插入。管理入口 `create_tag` 命中则报「标签已存在」。

### 1.4 应用配置存储选型（决策）

选 SQLite `settings` 表，不用本地 JSON 文件。

理由：
- 与既有 SQLite 统一后端，读写全走 Command 契约（前端不碰文件系统），单一存储来源降低复杂度。
- 「不进入备忘录数据」由「独立 `settings` 表」（非 `memo` 表）天然满足，无需额外文件隔离。
- 结构 key-value 易扩展：三个 key —— `theme`（`system|light|dark`）、`close_behavior`（`quit|hide`）、`shortcuts`（序列化后的 `ShortcutBindings` JSON）。

## 二、Command 契约增量

### 2.0 统一约定（沿用 P0 不变）

- `invoke('cmd_name', { input: {...} })`；无参命令不传第二参数。
- 返回 `Result<T, AppError>`；错误码仅三类。
- 时间字段 UTC RFC3339 毫秒、Z 结尾；`remind_at` 空串表示「不提醒」。

### 2.1 最终 TS 类型全集（严格对齐 5.3）

```ts
// ===== 实体 =====
export interface Memo {
  id: number;
  title: string;
  content: string;
  created_at: string; // ISO8601 UTC
  updated_at: string; // ISO8601 UTC
  remind_at: string;  // ISO8601 UTC 或 ''，空 = 不提醒（F09 新增）
}

export interface Tag {
  id: number;
  name: string;
  created_at: string; // ISO8601 UTC
}

/** list_tags 返回：Tag + 被引用计数（标签栏「随用随现」用 memo_count>0 过滤） */
export interface TagWithCount extends Tag {
  memo_count: number;
}

// ===== 备忘录（P0 输入不变，仅返回增益 remind_at）=====
export interface CreateMemoInput { title: string; content: string; }
export interface GetMemoInput { id: number; }
export interface UpdateMemoInput { id: number; title: string; content: string; }
export interface DeleteMemoInput { id: number; }
export interface DeleteMemoOutput { id: number; }
export interface SearchMemosInput { keyword: string; }

// ===== 标签 =====
export interface CreateTagInput { name: string; }
export interface DeleteTagInput { id: number; }
export interface DeleteTagOutput { id: number; }
export interface ListMemoTagsInput { memo_id: number; }
export interface SetMemoTagsInput { memo_id: number; tag_names: string[]; }
export interface ListMemosByTagInput { tag_id: number; keyword?: string; }

// ===== 提醒 =====
export interface SetMemoReminderInput { memo_id: number; remind_at: string; }
export interface ClearMemoReminderInput { memo_id: number; }

// ===== 应用配置 =====
export type ThemeMode = 'system' | 'light' | 'dark';
export type CloseBehavior = 'quit' | 'hide';

export interface ShortcutBindings {
  new_memo: string;
  save: string;
  delete: string;
  focus_search: string;
  quick_note: string;
}

export interface AppSettings {
  theme: ThemeMode;
  close_behavior: CloseBehavior;
  shortcuts: ShortcutBindings;
}

export interface UpdateSettingsInput {
  theme?: ThemeMode;
  close_behavior?: CloseBehavior;
  shortcuts?: ShortcutBindings;
}

// ===== 错误（沿用）=====
export type AppErrorCode = 'E_VALIDATION' | 'E_NOT_FOUND' | 'E_INTERNAL';
export interface AppError { code: AppErrorCode; message: string; }
```

### 2.2 变更命令（P0 六命令，仅返回结构增益）

| 命令 | 变更点 |
|---|---|
| `create_memo` | 输入不变（title+content）；返回 `Memo` 增 `remind_at`（默认 `''`） |
| `list_memos` | 返回 `Memo[]` 增 `remind_at` |
| `get_memo` | 返回 `Memo` 增 `remind_at` |
| `update_memo` | 输入不变；返回 `Memo` 增 `remind_at`（更新不改 remind_at） |
| `delete_memo` | 签名不变；内部先 `DELETE FROM memo_tags WHERE memo_id=?` 再 `DELETE memo`（外键 CASCADE 兜底） |
| `search_memos` | 返回 `Memo[]` 增 `remind_at` |

`SELECT_COLS` 常量升级为 `id, title, content, created_at, updated_at, remind_at`；`Memo`、`row_to_memo`、`CreateMemoInput`/`UpdateMemoInput` 反序列化同步（create/update 不接收 remind_at）。

### 2.3 新增命令清单

#### 标签

**C1 `list_tags`** — 列出全部标签（含引用计数）
- 入参：无（不传第二参数）
- 返回：`TagWithCount[]`
```json
[ { "id": 1, "name": "工作", "created_at": "2026-09-15T02:00:00.000Z", "memo_count": 3 } ]
```
- SQL：`SELECT t.id, t.name, t.created_at, COUNT(mt.memo_id) AS memo_count FROM tag t LEFT JOIN memo_tags mt ON mt.tag_id = t.id GROUP BY t.id ORDER BY t.name`
- 异常：仅 `E_INTERNAL`。

**C2 `create_tag`** — 创建标签（管理入口）
- 入参：`{ name: string }`
- 返回：`Tag`
- 校验：trim 空 → `E_VALIDATION`「标签名不能为空」；`chars().count() > 30` → 「标签不能超过 30 个字符」；含 `\n`/`\r` → 「标签名不能包含换行」；NOCASE 下已存在 → `E_VALIDATION`「标签已存在」

**C3 `delete_tag`** — 删除标签（仅解除关联，不删备忘录）
- 入参：`{ id: number }`；返回：`{ id }`
- 逻辑：删 `memo_tags WHERE tag_id=?` → 删 `tag WHERE id=?`；id 不存在 → `E_NOT_FOUND`「标签不存在」

**C4 `list_memo_tags`** — 查询某备忘录的标签（编辑器回填）
- 入参：`{ memo_id: number }`；返回：`Tag[]`
- memo 不存在 → `E_NOT_FOUND`「备忘录不存在」

**C5 `set_memo_tags`** — 整体设置某备忘录标签集合（覆盖式）
- 入参：`{ memo_id: number; tag_names: string[] }`；返回：`Tag[]`（设置后该 memo 的标签）
- 逻辑（单事务）：校验 memo 存在 → 对每个 `tag_name` 先 trim 校验（空/超30/换行 → `E_VALIDATION`），再 NOCASE get-or-create 得 tag_id（同名复用、去重 tag_names）→ `DELETE memo_tags WHERE memo_id=?` → 批量 `INSERT` → 回查返回 `Tag[]`
- 异常：memo 不存在 → `E_NOT_FOUND`；单个名称非法 → `E_VALIDATION`

**C6 `list_memos_by_tag`** — 按标签筛选（可与关键词叠加）
- 入参：`{ tag_id: number; keyword?: string }`；返回：`Memo[]`（倒序）
- SQL 语义：
```sql
SELECT m.id, m.title, m.content, m.created_at, m.updated_at, m.remind_at
FROM memo m JOIN memo_tags mt ON mt.memo_id = m.id
WHERE mt.tag_id = ?1
  [AND (m.title LIKE ?2 ESCAPE '\' OR m.content LIKE ?2 ESCAPE '\')]
ORDER BY m.updated_at DESC
```
- 说明：`memo_tags` 联合主键保证 JOIN 不产生重复行；`keyword` 为空/缺省时仅按标签过滤；keyword 仍需 `%`/`_`/`\` 转义（同 `search_memos`）
- 异常：tag 不存在 → `E_NOT_FOUND`「标签不存在」

#### 提醒（独立命令）

**C7 `set_memo_reminder`** — 设置提醒时间
- 入参：`{ memo_id: number; remind_at: string }`；返回：`Memo`（`remind_at` 已更新，`updated_at` 不变）
- 校验：memo 不存在 → `E_NOT_FOUND`；`remind_at` 非 ISO8601 可解析 → `E_VALIDATION`「提醒时间格式不正确」；解析后（截断到分钟）早于当前时间 → `E_VALIDATION`「提醒时间不能早于当前时间」
- 注意：不刷新 `updated_at`（提醒不改变内容排序，不顶置）

**C8 `clear_memo_reminder`** — 清除提醒
- 入参：`{ memo_id: number }`；返回：`Memo`（`remind_at=''`）
- 幂等：已无提醒也返回成功；memo 不存在 → `E_NOT_FOUND`

#### 应用配置

**C9 `get_settings`** — 读配置
- 入参：无；返回：`AppSettings`
- 首读无记录时返回默认值并幂等回写（`theme='system'`、`close_behavior='quit'`、`shortcuts`=默认绑定）

**C10 `update_settings`** — 更新配置（部分字段）
- 入参：`UpdateSettingsInput`（至少一项非空，否则 `E_VALIDATION`「没有需要更新的配置」）；返回：`AppSettings`（合并后的完整值）
- 校验：
  - `shortcuts` 组合合法性：每项须含修饰键（Cmd/Option/Control/Shift）或为功能键（F1–F12 等单键），否则 `E_VALIDATION`「快捷键必须包含修饰键或为功能键」
  - 同上下文重复：应用内 4 项（`new_memo`/`save`/`delete`/`focus_search`）与全局项 `quick_note` 各自上下文内不得重复，重复 → `E_VALIDATION`「该组合已被『XX』使用」；跨上下文（应用内 vs 全局）不硬拦
- 副作用（在命令内同步执行，见 2.5）：`theme` 无壳层副作用（前端 CSS 处理）；`close_behavior` 更新运行时缓存；`shortcuts` 更新前端内存态 + 重注册全局 `quick_note` 快捷键

> **默认绑定协议（主 Agent 裁定，前后端统一）**：`new_memo=Cmd+N`、`save=Cmd+S`、`delete=Cmd+Backspace`、`focus_search=Cmd+F`、`quick_note=Cmd+Shift+Space`。删除键字符串一律 `Cmd+Backspace`（对齐 AGENTS.md 5.4，键位为 macOS 退格键）；settings 默认值、校验、持久化、前端展示均使用该格式。

**C11 `open_notification_settings`** — 打开系统通知设置（主 Agent 补充契约）
- 入参：无（不传第二参数）；返回：`null`
- Rust 侧 `std::process::Command::new("open")` 打开 `x-apple.systempreferences:com.apple.preference.notifications`，纯本地系统调用、无网络；注册进 invoke_handler
- 用途：设置面板与提醒区「去系统设置」按钮

### 2.4 提醒并入 create/update 还是独立命令（决策）

选独立命令 `set_memo_reminder` / `clear_memo_reminder`。
- 提醒是独立操作入口（PRD 3.4 编辑器提醒区独立「添加/修改/清除」），生命周期与内容编辑正交。
- 独立性让「过期拦截」「不刷新 updated_at」「权限降级提示」等提醒专属逻辑不污染 `create_memo`/`update_memo`，也避免新建/编辑必须携带可选时间字段的耦合。
- `Memo.remind_at` 仍入实体（5.3 强制），创建/更新默认 `''`，提醒另走命令。

### 2.5 壳层非命令能力（无前端命令，要点简述）

**提醒到点调度**
- 选型（已变更）：**`osascript` 直发系统通知**，纯本地、无网络。原规划选用 `tauri-plugin-notification`，实现时确认其 2.x 已移除 Rust 侧 on_action/identifier 能力，无法满足「点击通知打开备忘录」；改用 osascript `display notification`（不引入第三方依赖，debug/release 均可真实弹出、无权限弹窗），并移除 tauri-plugin-notification 与 notify-rust 依赖及其 capabilities 权限。
- 调度：Rust 维护后台线程，启动及 `set_memo_reminder` 后扫描 `SELECT id,title,remind_at FROM memo WHERE remind_at <> ''`，取最近未到期时间，`std::thread` 定时到点 → 发通知（title=memo.title）→ 将该条 `remind_at` 置空（实现「已触发 → 无提醒」状态机）。
- 已知限制（如实标注）：真正「应用完全退出后仍由系统弹通知」需 macOS `UNUserNotificationCenter` 的 scheduled trigger（预约）；osascript 即时通知仅在应用运行期/重启后运行期触发，满足 F09 验收锚点「重启后未到期仍触发」。若产品强要求「完全退出后仍弹」，列为 P2 增强（见风险节）。

**托盘（菜单栏图标）**
- `tauri::tray::TrayIconBuilder`（Cargo 增 feature `tray-icon`），图标常驻；菜单「打开 My Echo」（show 主窗口）、「退出」（`app.exit()` 真正退出）。
- 关联关窗策略：setup 读 `close_behavior`，监听主窗口 `WindowEvent::CloseRequested` —— `quit` 放行默认关闭即退出；`hide` 则 `prevent_default()` + `hide()`。

**全局快捷键**
- `tauri-plugin-global-shortcut`（Rust 注册，系统级）：仅注册 `quick_note`（默认 `Cmd+Shift+Space`，从 settings 读，可改键）。Rust 持有运行时缓存 `Mutex<RuntimeConfig>`，`update_settings` 改键后注销重注册。
- 应用内 4 快捷键（`new_memo`/`save`/`delete`/`focus_search`）由前端 keydown 监听实现（必须应用内，绝不能全局，否则干扰其他应用）。

**速记窗（独立 WebviewWindow）**
- 触发（托盘图标入口或全局快捷键）→ 若速记窗存在则 `focus`，否则 `WebviewWindowBuilder` 建小窗（label=`quick-note`，`always_on_top`、`focused`、无边框或轻量边框），URL 带 `?mode=quick`；应用未启动时由系统唤起应用再走同一流程。
- 前端据 `mode=quick` 渲染速记组件；保存走 `create_memo` 后 `emit('quick_note_saved')`，主窗口 `listen` 后 `loadList()` 刷新置顶；关窗由前端 `close()` 该窗口，未保存内容二次确认在前端本地完成。

**通知点击打开备忘录 + 窗口间事件**
- 实现已由 notify-rust 变更为 osascript：osascript `display notification` 无点击回调，无法捕获用户点击，故「点通知打开对应备忘录」降级为「到点发通知 + 1 秒后自动打开对应备忘录」——`notification.rs` 到点发通知后延时 1 秒 show 主窗口 → `emit('open_memo', { memo_id })`。
- 前端主窗口 `listen('open_memo', ...)` 切到对应备忘录（调 `get_memo` + `list_memo_tags` 回填）。
- 事件名约定（前端 `@tauri-apps/api/event` 的 `listen`）：`open_memo`（payload `{ memo_id }`）、`quick_note_saved`（payload 空）。
- 系统通知设置跳转：C11 `open_notification_settings`（`open x-apple.systempreferences:...`，本地系统调用）。

### 2.6 待办勾选（确认无新命令）

- 纯前端 text 变换：`- [ ]` ↔ `- [x]`，改后调既有 `update_memo` 保存；进度「2/5」前端实时解析 `content` 计算，不入库、不新增字段、不新建命令。
- 待办行文本天然参与 `search_memos`（title/content LIKE），无需额外处理。

## 三、Frontend / Shell 原子任务清单

> 依赖 `→` 表示须先完成；「增量」标注 P0 不重做内容。契约束引用 §2.x / §1.x。

### Shell 侧（src-tauri/）

| 编号 | 任务 | 依赖 | 验收标准（引用契约） |
|---|---|---|---|
| S-01 | Cargo.toml 增依赖 `tauri` feature `tray-icon`、`tauri-plugin-notification`、`tauri-plugin-global-shortcut`；`lib.rs` 注册两个 plugin + 十新增命令 | 无 | `cargo build` 通过；新增命令可被 invoke 触达（§2.3 C1–C10） |
| S-02 | `db.rs` 升级：`PRAGMA foreign_keys=ON`、建 `tag`/`memo_tags`/`settings`、remind_at 列检测迁移、`user_version=1`（§1.1/1.2）；单测 `db_test` 断言新表/索引/迁移幂等 | S-01 | 存量 5 字段库启动后自动补 6 列；新库直接 6 列；`idx_tag_name_unique` 存在 |
| S-03 | `models.rs` 增 `Memo.remind_at`、`Tag`、`TagWithCount`、各 Input/Output、`AppSettings`/`UpdateSettingsInput`/`ShortcutBindings`（§2.1） | S-02 | 字段与 §2.1 逐字一致；`models_test` 序列化契约一致 |
| S-04 | 变更六命令：`SELECT_COLS` 与 `row_to_memo` 增 `remind_at`；`delete_memo` 先删 memo_tags（§2.2） | S-03 | create/list/get/update/search 返回均含 `remind_at`；删 memo 后 `memo_tags` 无残留 |
| S-05 | 标签命令 `list_tags`/`create_tag`/`delete_tag`/`list_memo_tags`/`set_memo_tags`/`list_memos_by_tag`（§2.3 C1–C6） | S-04 | 去重/长度/换行校验生效；NOCASE 重名拦截；set 覆盖式 + get-or-create；tag 筛选与 keyword 叠加正确；删 memo/tag 关系无孤儿 |
| S-06 | 提醒命令 `set_memo_reminder`/`clear_memo_reminder` + 调度线程（§2.3 C7/C8、§2.4） | S-04 | 过期拦截返回 E_VALIDATION；设置不刷新 updated_at；到点发通知并清 remind_at；重启扫描未到期仍触发 |
| S-07 | 配置命令 `get_settings`/`update_settings` + `settings.rs`（RuntimeConfig 缓存 + 默认值 + 校验/冲突检测 + 副作用）（§2.3 C9/C10） | S-03 | 首读返回默认；部分更新合并；同上下文重复绑定被拒；改 close_behavior/quick_note 即时生效 |
| S-08 | 托盘 + 关窗策略：`tray.rs`（TrayIconBuilder + 菜单）、`WindowEvent::CloseRequested` 分支 quit/hide（§2.5） | S-07 | 图标常驻；点击打开主窗口；`quit` 关窗退出、`hide` 关窗隐藏；托盘「退出」真退 |
| S-09 | 全局快捷键 + 速记窗：`shortcut.rs`（注册 quick_note）、`quicknote.rs`（建/focus 速记窗）（§2.5） | S-07 | 全局键唤起速记窗；不叠加新窗；缺窗时创建置顶聚焦窗 |
| S-10 | 通知点击回调 + 事件：`notification.rs`（osascript 到点发通知 → 延时 1 秒 show 主窗 → emit `open_memo`）（§2.5） | S-06 | 到点发通知并自动打开对应备忘录、emit 事件 |
| S-11 | capabilities：新增 `src-tauri/capabilities/default.json` 授予 `core:event:default`（前端 listen/emit 所需），其余走 Rust 侧无需前端权限（§2.5） | S-01 | 前端 `listen('open_memo')` 可用 |
| S-12 | 打包配置校验：version 0.2.0、通知 usage description、签名状态清单 | S-05–S-11 | `tauri build` 出 .app/.dmg，发布清单如实标注签名与通知权限 |

### Frontend 侧（client/）

| 编号 | 任务 | 依赖 | 验收标准（引用契约） |
|---|---|---|---|
| F-01 | `types/memo.ts` 增全部新类型（Memo.remind_at、Tag、TagWithCount、各 Input、AppSettings 等，§2.1）；`lib/api.ts` 增十条前端包装函数 + 事件 `listen` 封装 | 无（类型先行） | 类型与 §2.1 逐字一致、无 any；api 函数签名与命令一一对应 |
| F-02 | 主题令牌扩展：base.css 增 `[data-theme='light']`/`[data-theme='dark']` 强制块 + 保留 `@media` system 路径；主题管理器读取 `get_settings` 设置 `<html data-theme>`，`system` 用 matchMedia 实时跟随 | F-01（联调依赖 S-07/C9） | 三态即时生效 + 持久化；深色下对比度可读（无白字白底） |
| F-03 | 标签栏组件（列表视图左侧/工具栏下）：`list_tags` 拉取、`memo_count>0` 过滤展示、`全部` 项、选中态、空标签「暂无标签」；点击切换筛选（`list_memos_by_tag` + keyword 叠加） | F-01（联调依赖 S-05） | 点标签过滤正确且与搜索叠加；空态「该标签下暂无备忘录」 |
| F-04 | 编辑器标签区：已挂标签 chip（× 移除）、输入联想已有标签、回车挂载/新建；保存前 `set_memo_tags`（覆盖式）；编辑回填并行 `get_memo`+`list_memo_tags` | F-01（联调依赖 S-05/C4/C5） | 挂/删标签即时正确；重名复用不报错；回填展示已挂标签 |
| F-05 | 待办渲染：正文行解析 `- [ ]`/`- [x]` → checkbox，点击切换 `[ ]`/`[x]` 并 `update_memo` 保存；列表项进度「2/5」（仅含待办时显示） | F-01（复用 P0 update_memo） | 切换正文精确变更；进度计算正确；无待办不显示 |
| F-06 | 提醒 UI：编辑器提醒区（显示当前/添加入口/修改/清除）、`<datetime-local>` 选择器转 UTC ISO8601、过期拦截提示；列表项铃铛标记 | F-01（联调依赖 S-06） | 设/清均落库；过期被拦截；列表显示提醒标记 |
| F-07 | 速记窗组件 `QuickNote.vue`：`?mode=quick` 下渲染，标题必填正文可空，回车保存 `create_memo` → emit `quick_note_saved` → 关窗；未存内容 Esc/关闭二次确认 | F-01（联调依赖 S-09） | 保存后置顶且重启仍在；Esc 不落库；二次确认丢弃不落库 |
| F-08 | 设置面板：主题三态、`保留在菜单栏`开关、快捷键录制/冲突提示/恢复默认/全部恢复默认；`update_settings` 保存 + 前端 keydown 监听应用内 4 快捷键 | F-01（联调依赖 S-07/C10） | 三态持久化；开关持久化；改键生效 + 重启保留；同上下文重复被拒（后端）；恢复默认可用 |
| F-09 | 事件接线：监听 `open_memo` 打开对应备忘录；`quick_note_saved` 刷新列表；通知打开链路端到端 | F-01/F-07（联调依赖 S-10/S-11） | 到点自动打开对应备忘；速记保存后主列表刷新 |

**依赖与并行建议**
- 关键路径：S-01→S-02→S-03→S-04→S-05/S-06/S-07（可并行）→S-08/S-09/S-10→S-12；前端 F-01 先行（纯类型），F-02/F-03/F-04/F-05/F-06 可并行，F-07/F-08 依赖各自命令联调，F-09 收尾。
- 联调契约唯一：§2 命令签名 + 字段顺序 + 错误码；任一方不符即回退，不允许单方隐式改契约。

## 四、风险与决策点

1. **存量库迁移**：`ALTER TABLE ADD COLUMN` 幂等但需「先检测列再补」，避免重复执行报错；外键 `PRAGMA foreign_keys` 是 per-connection，必须在 `db::init` 每次连接开启并加应用层显式删兜底，防止「漏开 PRAGMA 产生孤儿关系」。加 `user_version=1` 为 F04/F05 复杂迁移预留入口但本期不做框架。

2. **通知权限降级与退出后通知**：系统通知权限被拒时保存提醒但明示「通知可能不显示」（PRD 已定义），设置页提供跳系统设置的入口。真正「完全退出后系统预约通知」依赖 `UNUserNotificationCenter` scheduled trigger，若插件不暴露时列为 P2 增强，本期以「运行期 + 重启后调度」覆盖验收锚点。

3. **全局快捷键与系统冲突**：`quick_note` 全局注册可能与系统/其他应用冲突；实测冲突时以系统为准并在设置项旁提示「可能与系统快捷键冲突」（PRD 已定，不硬拦）。应用内 4 快捷键必须前端实现，绝不能走 global-shortcut 以免劫持其他应用。

4. **托盘与关窗的 macOS 行为差异**：菜单栏图标左击默认弹菜单、`hide` 关窗后 Dock 仍留图标，`quit` 需真正 `app.exit()`。需在 `WindowEvent::CloseRequested` 正确区分「关窗」与「托盘退出」，避免 hide 后进程常驻但 UI 入口丢失。macOS 无「纯隐藏退出」的统一范式，实测以 macOS 惯例校准。

5. **标签筛选与搜索叠加查询**：`list_memos_by_tag` 的 `JOIN` + 可选 `LIKE` 需统一转义、`memo_tags` 联合主键保证不重复行；关键词为空回退纯标签过滤，避免「标签 + 空关键词」走到 `search_memos` 漏掉标签维度。前端「全部 + 不搜索」仍走 `list_memos`。

6. **5.3 单字段约束 vs PRD「过期弱化」**：按 AGENTS.md 优先，`Memo` 仅增 `remind_at`、无「已触发」标记列，故触发后清空 `remind_at`（已触发→无提醒），不实现 PRD「过期视觉弱化」（需新增 `fired_at` 字段才可表达，属 5.3 变更，走变更约定再议）。

7. **标签与 F04/F05 兼容**：F04 软删用 `UPDATE deleted_at` 不触发外键 CASCADE，标签关系与 `remind_at` 随行保留、恢复即还原，与「删除连带删除关系」在软删语义下统一（软删不物理删行）；F05 导出需包含 tag、memo_tags、remind_at，本期仅登记不实现。

8. **capabilities 空白**：当前 `capabilities.json` 为 `{}`，P0 只靠自定义 command 无需权限；本增量前端新增 `listen/emit` 与可能的前端窗口控制，必须补 `core:event:default`（及按需 `core:window:default`），否则事件监听静默失败——这是集成高频坑，列入 S-11 强制。