// 备忘录/标签/配置实体与命令入参/返回结构，字段名与顺序严格对齐 Planner 契约

use serde::{Deserialize, Serialize};

/// 备忘录实体（字段顺序与前端契约一致；remind_at 为 F09 新增）
#[derive(Debug, Serialize, Deserialize)]
pub struct Memo {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub remind_at: String,
}

/// create_memo 入参
#[derive(Debug, Deserialize)]
pub struct CreateMemoInput {
    pub title: String,
    pub content: String,
}

/// get_memo 入参
#[derive(Debug, Deserialize)]
pub struct GetMemoInput {
    pub id: i64,
}

/// update_memo 入参
#[derive(Debug, Deserialize)]
pub struct UpdateMemoInput {
    pub id: i64,
    pub title: String,
    pub content: String,
}

/// delete_memo 入参
#[derive(Debug, Deserialize)]
pub struct DeleteMemoInput {
    pub id: i64,
}

/// delete_memo 返回
#[derive(Debug, Serialize)]
pub struct DeleteMemoResult {
    pub id: i64,
}

/// search_memos 入参
#[derive(Debug, Deserialize)]
pub struct SearchMemosInput {
    pub keyword: String,
}

/// 标签实体
#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

/// list_tags 返回：标签 + 被引用计数
#[derive(Debug, Serialize, Deserialize)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub memo_count: i64,
}

/// create_tag 入参
#[derive(Debug, Deserialize)]
pub struct CreateTagInput {
    pub name: String,
}

/// delete_tag 入参
#[derive(Debug, Deserialize)]
pub struct DeleteTagInput {
    pub id: i64,
}

/// delete_tag 返回
#[derive(Debug, Serialize)]
pub struct DeleteTagResult {
    pub id: i64,
}

/// list_memo_tags 入参
#[derive(Debug, Deserialize)]
pub struct ListMemoTagsInput {
    pub memo_id: i64,
}

/// set_memo_tags 入参
#[derive(Debug, Deserialize)]
pub struct SetMemoTagsInput {
    pub memo_id: i64,
    pub tag_names: Vec<String>,
}

/// list_memos_by_tag 入参（keyword 可选，与标签筛选叠加）
#[derive(Debug, Deserialize)]
pub struct ListMemosByTagInput {
    pub tag_id: i64,
    #[serde(default)]
    pub keyword: Option<String>,
}

/// set_memo_reminder 入参
#[derive(Debug, Deserialize)]
pub struct SetMemoReminderInput {
    pub memo_id: i64,
    pub remind_at: String,
}

/// clear_memo_reminder 入参
#[derive(Debug, Deserialize)]
pub struct ClearMemoReminderInput {
    pub memo_id: i64,
}

/// 快捷键绑定（应用内 4 项 + 全局速记 1 项）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutBindings {
    pub new_memo: String,
    pub save: String,
    pub delete: String,
    pub focus_search: String,
    pub quick_note: String,
}

/// 应用配置完整值（get_settings 返回、update_settings 合并后返回）
#[derive(Debug, Serialize, Clone)]
pub struct AppSettings {
    pub theme: String,
    pub close_behavior: String,
    pub shortcuts: ShortcutBindings,
}

/// update_settings 入参（部分字段）
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsInput {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub close_behavior: Option<String>,
    #[serde(default)]
    pub shortcuts: Option<ShortcutBindings>,
}