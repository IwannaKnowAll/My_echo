# My Echo 快照回退机制 与 GitHub 版本同步规范

> 角色：Planner 设计 ｜ 事实源：AGENTS.md 优先
> 现状核实（2026-09-16）：main 6 提交 + 1 tag（baseline→5ebf476）；三处版本号均 0.2.0；HEAD=origin/main=4eb1977。

## 一、机制总览：快照定义分层

快照按「对象」分两层，边界不可混：

| 层 | 快照对象 | 回答的问题 | 载体 | 归属 |
|---|---|---|---|---|
| **代码快照** | 应用源码 + 配置 + 文档 + 图标源文件 | 「某版本应用是怎么写出来的」 | git commit / 附注 tag / 本地产物归档 | git 仓库 + 本地 `releases/` |
| **数据快照** | 用户 SQLite 数据（备忘录/标签/配置） | 「用户写了什么内容」 | `.db` 文件副本（定时 / 手动导出） | 本机用户数据目录（接 F05 / N14，仅简述） |

**边界铁律：**
1. 版本 tag 内绝不塞用户 SQLite 文件、`.dmg`/`.app` 产物——tag 只含可重编译的源码状态。
2. 用户数据备份绝不进 git——数据快照与代码快照物理隔离。
3. 代码快照可 checkout 重编译；数据快照可恢复不可 diff；两者永不进同一辆车。

## 二、代码版本快照规范

### 2.1 tag 规范

- 命名：`vX.Y.Z`，与 `tauri.conf.json` / `Cargo.toml` / `package.json` 三处版本号严格一致。
- 类型：附注 tag（annotated），弃轻量 tag（需要 tagger 元信息与消息）。
- 时机：「Reviewer 发布查验通过」之后、「git push」之前。
- 消息格式：首行 `My Echo vX.Y.Z`，空行后接变更摘要要点。

### 2.2 每版本快照内容清单

必须入库：源码（client/src、src-tauri/src 含 tests）、Cargo.toml/Cargo.lock、tauri.conf.json、capabilities、package.json/package-lock、vite/tsconfig、AGENTS.md/docs/agents/config.toml、icons（含 icon-source.jpg）。

必须不入库：node_modules、target、client/dist、.DS_Store（已 ignore）；`.dmg`/`.app` 归档至本地 `releases/vX.Y.Z/`（`releases/` 加入 .gitignore）。

GitHub Release 附件：网络受限（仅 SSH 443 可用），**不上传 assets、不依赖 gh CLI**；版本历史由 git tag 承载。

### 2.3 历史补课

- v0.2.0 → 已钉 `4eb1977`（附注 tag，已推送 GitHub）。
- v0.1.0 → 已裁定（Melody，2026-09-16）：方案 A，不补 tag。版本历史从 v0.2.0 起算；P0 历史由 `baseline` tag（5ebf476）与提交信息追溯。

## 三、回退机制

### 3.1 场景分级

| 场景 | 手段 | 改写历史 | 适用 |
|---|---|---|---|
| 单提交回滚 | `git revert <commit>` | 否 | 首选 |
| 整版本回退 | `git revert --no-commit <tag>..HEAD` 后 commit | 否 | 发布后缺陷 |
| 配置回退 | revert 单个配置提交 | 否 | 版本号/签名/CSP 配错 |
| 查看旧版本 | `git checkout <tag>`（detached，只读） | 不提交 | 编译/比对 |
| 紧急硬回滚 | `git reset --hard <tag>` + force-with-lease | **是（需 Melody 批准）** | 仅限明确授权 |

### 3.2 操作手册（要点）

- 定位：`git tag -l`、`git log --oneline --decorate`
- 单提交：`git revert <bad>` → `git push origin main`
- 整版本：`git revert --no-commit v0.1.0..v0.2.0` → 手动 commit → 回退后按新版本号（三处一致）重打 tag
- 紧急：`git reset --hard vX.Y.Z` + `git push --force-with-lease`（破坏性，先批准后执行）

### 3.3 回退后同步与验证

```
git push origin main
git push origin <tag>        # 或 git push origin --tags
git ls-remote --tags origin  # 校验远端 tag
git ls-remote origin         # 校验 main 哈希一致
```

tag push 冲突（rejected tag already exists）→ 换新版本号，或经批准删远端 tag（破坏性）。

## 四、GitHub 同步约定

- 通道：固定 SSH over 443（origin = git@github.com:IwannaKnowAll/My_echo.git）。
- 时机：Reviewer 放行后 push commit；打 tag 后 push commit+tag；回退后 push 新状态+新 tag。
- 呈现：权威版本历史 = tag 列表；不启用 Releases 附件；「GitHub 存版本指针与说明，本机存产物实体」。
- 流程挂接：在打包发布+发布查验通过后追加固定步骤——①打附注 tag；②push main+tag；③ls-remote 验证；④归档 .dmg 至 releases/vX.Y.Z/。

## 五、产品侧快照（可选延伸 ⚑ 不展开）

备忘录内容历史版本与回滚（N13）：需新增 `memo_versions` 表，属 5.3 变更，必须先改 AGENTS.md 走变更约定，再出契约。

## 六、风险与决策点

1. 网络受限：唯一通道 SSH 443；禁 https push/gh CLI/Release assets。
2. tag 与三处版本号一致性：人工改三处 + Reviewer 发布查验比对，双保险；不一致即阻断。
3. 数据与代码快照混淆：.gitignore releases/ + 数据目录在工作区外 + Reviewer 查验无大二进制入库。
4. v0.1.0 历史断层：需拍板（方案 A/B）。
5. 破坏性回退：reset/force 先批准后执行，默认只用 revert。