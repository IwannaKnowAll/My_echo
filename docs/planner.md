# My Echo 首版（P0）架构规划

> 范围：仅 F01（增删改查）+ F02（搜索与排序）。P1/P2 一律不实现、不预留字段。
> 事实源：AGENTS.md 5.2/5.3/5.4/5.5；运行约束：config.toml（on-request / workspace-write / high）。

## 一、SQLite 建表 SQL

### 1.1 设计决策说明

- **时间戳统一存 UTC 的 ISO8601（RFC3339）文本**，形如 `2026-09-14T03:15:00.123Z`（毫秒精度、`Z` 结尾）。理由：同格式下文本字典序 === 时间序，`ORDER BY updated_at DESC` 结果严格正确；若存本地时区会因夏令时/正负时区导致字符串排序错乱。前端展示时用 `new Date(iso)` 转本地时区。
- **标题 ≤100 与必填**：数据库层 `NOT NULL` + `CHECK(length(title) <= 100)` 兜底；业务层"标题必填"（trim 后非空）由 Command 校验，因为"空字符串"是合法默认值、不能靠 DB 约束表达"业务不可为空"。
- **搜索大小写不敏感**：用**默认 `LIKE`**（SQLite 对 ASCII 已大小写不敏感），**不使用 `COLLATE NOCASE`**。取舍：需求仅为标题+正文模糊匹配，中文无大小写、ASCII 默认 LIKE 已满足验收；`COLLATE NOCASE` 会改变列的比较/排序语义且只影响 ASCII，收益为零、徒增复杂度，故不用。
- **索引**：仅建 `updated_at` 普通索引（列表与搜索均按 `updated_at` 倒序 → 走索引反向扫描，有序返回）。`id` 因 `INTEGER PRIMARY KEY` 已有隐式唯一索引，无需再建。`LIKE '%kw%'`（前缀通配）无法利用 B 树索引，故标题/正文不建索引；P0 单机数据量小，可接受。

```sql
-- 备忘录表（仅 P0 五个字段，严格对齐 AGENTS.md 5.3，禁止新增字段）
CREATE TABLE IF NOT EXISTS memo (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,      -- 主键，自增，唯一标识
    title      TEXT    NOT NULL,                       -- 标题，必填；业务层校验非空，长度上限 100
    content    TEXT    NOT NULL DEFAULT '',            -- 正文，允许为空（空字符串表示空）
    created_at TEXT    NOT NULL,                       -- 创建时间，ISO8601 UTC 文本（毫秒，Z 结尾）
    updated_at TEXT    NOT NULL,                       -- 更新时间，ISO8601 UTC 文本（毫秒，Z 结尾）
    CONSTRAINT chk_title_len CHECK (length(title) <= 100)  -- 标题长度 ≤ 100 兜底约束
);

-- 更新时间索引：列表/搜索均按 updated_at 倒序，反向扫描有序返回
CREATE INDEX IF NOT EXISTS idx_memo_updated_at ON memo (updated_at DESC);
```

验收锚点：表含恰好 5 个业务字段 + 1 个 CHECK 约束 + 1 个索引；无 `is_pinned`、`deleted_at` 等任何 P1/P2 字段。

## 二、Tauri Command 契约

### 2.0 统一约定

- 命令命名：Rust 命令函数名与 `invoke` 字符串均为 **snake_case**。
- **invoke 调用约定（联调硬规则）**：Tauri 2 按参数名反序列化。当 Rust 命令含业务入参时，JS 第二参数为 `{ <Rust 参数名>: <TS Input> }`。本项目所有命令的业务参数名统一为 `input`，故形如 `invoke('create_memo', { input: { title, content } })`；无业务入参的命令（`list_memos`）不传第二参数。**禁止展平传参**（如 `{ ...input }`），否则 Rust 反序列化失败（集成验证已实证该坑）。
- 所有命令返回 `Result<T, AppError>`；`AppError` 为结构体，序列化后前端 `catch (err)` 直接拿到对象。
- 错误码枚举（仅三类，前端据此分支提示）：

| code | 含义 | 触发场景 |
|---|---|---|
| `E_VALIDATION` | 参数校验失败 | 标题为空、标题超 100、keyword 非法等 |
| `E_NOT_FOUND` | 目标不存在 | get/update/delete 时 id 不存在或已被删 |
| `E_INTERNAL` | 内部错误 | SQLite 读写异常等，含人类可读 message |

```ts
// 统一错误结构（前端 catch 到的对象）
interface AppError {
  code: 'E_VALIDATION' | 'E_NOT_FOUND' | 'E_INTERNAL';
  message: string; // 人类可读的中文提示，直接用于界面展示
}
```

```json
// 错误 JSON 示例
{ "code": "E_VALIDATION", "message": "标题不能为空" }
```

- 公共实体（前端与 Rust `Memo` 结构体字段/顺序严格一致）：

```ts
interface Memo {
  id: number;
  title: string;
  content: string;
  created_at: string; // ISO8601 UTC，形如 2026-09-14T03:15:00.123Z
  updated_at: string; // ISO8601 UTC
}
```

### 2.1 新建 `create_memo`

- 用途：新建一条备忘录，后台生成 `id`、`created_at`、`updated_at`。
- 入参 TS：

```ts
interface CreateMemoInput {
  title: string;   // 必填，trim 后非空，长度 ≤ 100
  content: string; // 允许为空
}
```

- 返回 JSON（完整新建备忘录）：

```json
{
  "id": 1,
  "title": "会议纪要",
  "content": "与产品对齐 P0 排期",
  "created_at": "2026-09-14T03:15:00.000Z",
  "updated_at": "2026-09-14T03:15:00.000Z"
}
```

- 校验：`title.trim()` 为空 → `E_VALIDATION`「标题不能为空」；`title.length > 100` → `E_VALIDATION`「标题不能超过 100 个字符」。

### 2.2 列表 `list_memos`

- 用途：取全部备忘录，按 `updated_at` 倒序（最新在前）。
- 入参：无。
- 返回 JSON（示例为倒序序列，最上面 = 最新）：

```json
[
  { "id": 3, "title": "最新的一条", "content": "", "created_at": "2026-09-14T05:00:00.000Z", "updated_at": "2026-09-14T05:00:00.000Z" },
  { "id": 1, "title": "会议纪要", "content": "与产品对齐 P0 排期", "created_at": "2026-09-14T03:15:00.000Z", "updated_at": "2026-09-14T04:30:00.000Z" },
  { "id": 2, "title": "购物清单", "content": "牛奶、鸡蛋", "created_at": "2026-09-13T10:00:00.000Z", "updated_at": "2026-09-13T10:00:00.000Z" }
]
```

- 契约要求：Shell 保证返回已按 `updated_at` 降序；前端信任该顺序直接渲染，不本地二次排序（一致性由契约背书的 `ORDER BY updated_at DESC` 保证）。

### 2.3 单条查询 `get_memo`

- 用途：编辑回填时按 id 拉取单条完整内容。
- 入参 TS：

```ts
interface GetMemoInput { id: number; }
```

- 返回 JSON：`Memo`（结构同 2.1）。
- 异常：id 不存在 → `E_NOT_FOUND`「备忘录不存在」。

### 2.4 更新 `update_memo`

- 用途：修改标题/正文，**后台自动刷新 `updated_at`**（`created_at` 不变）。
- 入参 TS：

```ts
interface UpdateMemoInput {
  id: number;
  title: string;   // 必填，校验同 create
  content: string; // 允许为空
}
```

- 返回 JSON（更新后的完整 `Memo`，`updated_at` 为本次时间）：

```json
{
  "id": 1,
  "title": "会议纪要（已改）",
  "content": "更新了内容",
  "created_at": "2026-09-14T03:15:00.000Z",
  "updated_at": "2026-09-14T06:00:00.000Z"
}
```

- 校验：标题规则同 create；id 不存在 → `E_NOT_FOUND`。

### 2.5 删除 `delete_memo`

- 用途：P0 硬删（物理 `DELETE`），删除前由前端弹确认框。
- 入参 TS：`interface DeleteMemoInput { id: number; }`
- 返回 JSON：

```json
{ "id": 1 }
```

- 异常：id 不存在 → `E_NOT_FOUND`「备忘录不存在」。

### 2.6 搜索 `search_memos`

- 用途：标题 + 正文模糊匹配（`LIKE`），结果按 `updated_at` 倒序。
- 入参 TS：

```ts
interface SearchMemosInput { keyword: string; } // 必须 trim 后非空
```

- SQL 语义：`WHERE title LIKE '%kw%' ESCAPE '\' OR content LIKE '%kw%' ESCAPE '\'`，`ORDER BY updated_at DESC`。
  - Shell 须对 `keyword` 中的 `%`、`_`、`\` 做转义（`\%`、`\_`、`\\`），避免通配符注入导致误匹配。
  - 大小写不敏感由默认 `LIKE` 对 ASCII 提供（详见 1.1）。
- 返回 JSON：`Memo[]`，结构与排序同 2.2。
- 异常/边界：`keyword.trim()` 为空 → 返回空数组 `[]`（约定前端只在关键词非空时调用本命令，空关键词走 `list_memos`）。

## 三、Frontend / Shell 原子任务清单

### 3.1 脚手架与选型建议（先于任务落地）

**Tauri 版本**：使用 **Tauri 2.x 最新稳定版**（禁止 1.x）。Rust crate `tauri = "2"`，CLI `@tauri-apps/cli@^2`，两者 2.x 小版本保持一致；落地时以 `cargo search tauri` 与 `npm view @tauri-apps/cli version` 现网稳定值锁定。

**SQLite 方案**：选 **`rusqlite`（crates.io，`features = ["bundled"]`）**，**不用 `tauri-plugin-sql`**。理由：
1. `tauri-plugin-sql` 让前端直接传 SQL 字符串给后端执行，破坏"契约优先"分层，违反类型安全（禁 any），且易拼出注入；
2. AGENTS.md 要求 SQL 封装在 Shell 侧、前端只经 Command 读写；
3. `rusqlite` 成熟、控制力强，单进程单用户用 `Mutex<Connection>` 塞进 Tauri State 即可，无需连接池；
4. `bundled` 特性自带编译 SQLite，规避 macOS 系统 sqlite 版本差异。

**时间库**：Rust 侧用 `chrono`（`default-features = false, features = ["clock"]`）生成 UTC ISO8601（`Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)`）。

**数据文件位置**：`app.path().app_data_dir()`（Mac → `~/Library/Application Support/<identifier>/`），文件名为 `my-echo.db`，首次启动 `std::fs::create_dir_all` 后打开，执行建表迁移。

**标准目录结构（client/ 与 src-tauri/ 硬边界）**：

```
My_Echo/
├── AGENTS.md
├── config.toml
├── agents/
├── client/                          # 【Frontend 专属】
│   ├── index.html
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── src/
│       ├── main.ts
│       ├── App.vue
│       ├── types/memo.ts            # Memo 及各 Input 接口、AppError
│       ├── lib/api.ts               # 封装 invoke + 统一错误规整
│       └── components/              # 列表 / 编辑表单 / 搜索框 等组件
└── src-tauri/                       # 【Shell 专属】
    ├── Cargo.toml
    ├── tauri.conf.json              # build.frontendDist 指向 ../client/dist
    ├── build.rs
    ├── icons/
    └── src/
        ├── main.rs                  # 入口
        ├── lib.rs                   # invoke_handler 注册命令 + manage(AppState)
        ├── error.rs                 # AppError（code/message）
        ├── db.rs                    # 连接管理 + 迁移建表
        ├── models.rs                # Memo 结构体（serde）
        └── commands/memo.rs         # 六个命令实现
```

**tauri.conf.json 关键指向**：`build.frontendDist = "../client/dist"`、`build.devUrl = "http://localhost:5173"`（Vite 默认）、`beforeDevCommand/beforeBuildCommand` 在 `client/` 目录下执行 `npm run dev`/`npm run build`。

### 3.2 原子任务清单

> 依赖说明：`→` 表示"完成后方可开始"。

#### Shell 侧（src-tauri/）

| 编号 | 任务 | 依赖 | 验收标准 |
|---|---|---|---|
| S1 | 搭建骨架：Cargo.toml（tauri 2.x、rusqlite bundled、chrono、serde/serde_json）、main.rs、lib.rs（注册 6 命令 + `Mutex<Connection>` 进 AppState）、error.rs（AppError 三码） | 无 | `cargo build` 通过；命令能注册、空实现可被 invoke 触达 |
| S2 | 实现 db.rs：app_data_dir 建目录 → 打开 `my-echo.db` → 执行建表 SQL | S1 | 首次启动自动生成 `~/Library/Application Support/<id>/my-echo.db`；表结构含 5 字段 + CHECK + 索引，无多余字段 |
| S3 | 实现 `create_memo`：校验标题 → 生成 UTC ISO8601 时间 → INSERT → 回查返回完整 Memo | S2 | 空标题/超长返回 `E_VALIDATION`；成功后重启应用数据仍在（F01 持久化验收） |
| S4 | 实现 `list_memos`：`ORDER BY updated_at DESC` | S2 | 插入乱序多条后返回严格倒序；断言返回顺序 = 时间递减 |
| S5 | 实现 `get_memo`：按 id 单查 | S2 | 存在的 id 返回完整字段；不存在返回 `E_NOT_FOUND` |
| S6 | 实现 `update_memo`：校验 + UPDATE 并刷新 updated_at（created_at 不变） | S2 | 更新后 updated_at 变大、created_at 不变；再次列表该条排最前 |
| S7 | 实现 `delete_memo`：硬删 DELETE；先查存在性 | S2 | 删除后列表/查询均查不到；重复删同 id 返回 `E_NOT_FOUND`；无残留行 |
| S8 | 实现 `search_memos`：LIKE + ESCAPE 转义 + 倒序 | S2 | 标题命中、正文命中均返回；keyword 含 `%`/`_` 被转义不误配；空关键词返回 `[]`；结果倒序 |
| S9 | 打包配置：identifier、macOS 最低版本、签名占位、CSP | S3–S8 | `tauri build` 产出 .app/.dmg；version、签名状态可出具「发布清单」 |

#### Frontend 侧（client/）

| 编号 | 任务 | 依赖 | 验收标准 |
|---|---|---|---|
| F1 | 搭建骨架：Vite + Vue3 + TS，package.json、vite.config.ts、tsconfig、index.html、main.ts、App.vue | 无 | `npm run dev` 启动空白页无报错；`npm run build` 产出 `client/dist` |
| F2 | 定义 `types/memo.ts`（Memo + 各 Input + AppError，字段与契约严格一致、禁 any）+ `lib/api.ts`（封装 invoke、把 catch 到的 err 规整为 AppError） | 无（纯类型，可先于 Shell 联调先行） | 接口字段与契约逐字一致；所有函数参数/返回值显式类型，无 `any` |
| F3 | 列表组件：调 `list_memos`，渲染标题 + 本地时间，倒序直接渲染不重排 | F2（联调依赖 S4） | 列表按最新在前；时间转本地时区显示；加载/错误态正确处理 |
| F4 | 新建/编辑表单：标题必填（trim 非空）、正文可空；新建调 `create_memo`、编辑调 `update_memo` | F2、F3（联调依赖 S3/S6） | 空标题保存被拦截并提示「标题不能为空」；保存成功后列表刷新且最新在前 |
| F5 | 单条查看/编辑回填：编辑时调 `get_memo` 回填标题与正文 | F4（联调依赖 S5） | 点击某条进入编辑，标题/正文正确回填；E_NOT_FOUND 给出友好提示 |
| F6 | 删除交互：点删除弹确认框 → 调 `delete_memo` → 刷新列表 | F3（联调依赖 S7） | 确认后才删除；取消不删；删除后该条从列表消失 |
| F7 | 搜索框：关键词非空调 `search_memos`、清空回切 `list_memos`；实时/提交过滤 | F3（联调依赖 S8） | 输入关键词即过滤标题+正文匹配项；清空恢复全量；结果倒序 |
| F8 | 时间格式化工具：ISO8601 UTC → 本地可读文本（`YYYY-MM-DD HH:mm`） | F3 | 显示本地时区正确（非原样 Z 字符串） |

#### 依赖与并行建议

- 纵向关键路径：S1→S2→S3/S4/…→S9；F1→F2→F3→F4/F5/F6/F7→F8。
- **并行策略**：F1/F2 是纯前端类型与骨架，可完全不依赖 Shell 先行；Shell 按 S1→S8 自洽推进。两者只以本规划契约对齐，联调在 `cargo build` + `npm run build` 均通过后统一进行。
- 校验规则、错误码、字段顺序为唯一联调契约，任何一方不符即回退修复，不允许单方隐式改契约。

## 四、风险与决策点

1. **Tauri 版本**：定 Tauri 2.x 最新稳定（禁 1.x）。2.0 后 API 已稳定，但 `tauri.conf.json` 结构、CLI 与 Rust crate 大版必须匹配。落地时锁定现网稳定小版本，避免「脚手架拉的 2.x 与手写 Cargo.toml 的 2.x 脱节」。
2. **SQLite 选型**：定 `rusqlite` + `bundled`，弃 `tauri-plugin-sql`（理由见 3.1）。连接用 `Mutex<Connection>`，无需池化。
3. **时间戳格式**：统一 UTC RFC3339 毫秒 `Z` 结尾，保证文本排序=时间序。**这是排序正确性的根基**，禁止改本地时区。
4. **LIKE 大小写与转义**：不做 `COLLATE NOCASE`；`%`/`_` 必须 `ESCAPE '\'` 转义，否则用户输入 `%` 会匹配全部，属隐蔽 bug，Shell S8 强制。
5. **签名与打包**：S9 签名依赖 Apple 开发者证书环境，若本机无签名配置需主 Agent 决策（开发签名 vs ad-hoc vs 延后）。发布清单须如实标注签名状态。
6. **字段边界**：5.3 为本规划唯一建表依据，任何字段新增（含 `is_pinned` 等 P1 字段）属越界，Reviewer 按建表 SQL 逐字段比对验收。
7. **删除语义**：P0 为硬删（物理 DELETE），无回收站字段。P1 软删需走「变更约定」重新推导契约。