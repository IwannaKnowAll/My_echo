// 增量模型序列化契约测试：Tag/TagWithCount/ShortcutBindings/AppSettings 及新输入结构。

use crate::models::{
    AppSettings, ListMemosByTagInput, SetMemoTagsInput, ShortcutBindings, Tag, TagWithCount,
    UpdateSettingsInput,
};
use serde_json::json;

#[test]
fn tag_serializes_three_snake_case_fields() {
    let tag = Tag {
        id: 1,
        name: "工作".to_string(),
        created_at: "2026-09-14T03:15:00.000Z".to_string(),
    };
    let v = serde_json::to_value(&tag).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 3, "Tag 应恰好 3 字段");
    assert_eq!(obj["id"], json!(1));
    assert_eq!(obj["name"], json!("工作"));
    assert_eq!(obj["created_at"], json!("2026-09-14T03:15:00.000Z"));
}

#[test]
fn tag_with_count_serializes_four_fields_with_memo_count() {
    let t = TagWithCount {
        id: 2,
        name: "生活".to_string(),
        created_at: "2026-09-14T03:15:00.000Z".to_string(),
        memo_count: 5,
    };
    let v = serde_json::to_value(&t).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 4, "TagWithCount 应恰好 4 字段");
    assert_eq!(obj["memo_count"], json!(5));
}

#[test]
fn shortcut_bindings_roundtrip() {
    let b = ShortcutBindings {
        new_memo: "Cmd+N".to_string(),
        save: "Cmd+S".to_string(),
        delete: "Cmd+Backspace".to_string(),
        focus_search: "Cmd+F".to_string(),
        quick_note: "Cmd+Shift+Space".to_string(),
    };
    let v = serde_json::to_value(&b).unwrap();
    assert_eq!(v["delete"], json!("Cmd+Backspace"));
    assert_eq!(v.as_object().unwrap().len(), 5);

    let back: ShortcutBindings = serde_json::from_value(v).unwrap();
    assert_eq!(back.new_memo, "Cmd+N");
}

#[test]
fn app_settings_serializes_nested_shortcuts() {
    let s = AppSettings {
        theme: "system".to_string(),
        close_behavior: "quit".to_string(),
        shortcuts: ShortcutBindings {
            new_memo: "Cmd+N".to_string(),
            save: "Cmd+S".to_string(),
            delete: "Cmd+Backspace".to_string(),
            focus_search: "Cmd+F".to_string(),
            quick_note: "Cmd+Shift+Space".to_string(),
        },
    };
    let v = serde_json::to_value(&s).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 3);
    assert_eq!(obj["theme"], json!("system"));
    assert_eq!(obj["shortcuts"]["quick_note"], json!("Cmd+Shift+Space"));
}

#[test]
fn update_settings_input_accepts_partial_fields() {
    let partial: UpdateSettingsInput = serde_json::from_value(json!({ "theme": "dark" })).unwrap();
    assert_eq!(partial.theme.as_deref(), Some("dark"));
    assert!(partial.close_behavior.is_none());
    assert!(partial.shortcuts.is_none());

    let empty: UpdateSettingsInput = serde_json::from_value(json!({})).unwrap();
    assert!(empty.theme.is_none());
    assert!(empty.close_behavior.is_none());
    assert!(empty.shortcuts.is_none());
}

#[test]
fn list_memos_by_tag_input_defaults_keyword_to_none() {
    let with_kw: ListMemosByTagInput =
        serde_json::from_value(json!({ "tag_id": 3, "keyword": "k" })).unwrap();
    assert_eq!(with_kw.tag_id, 3);
    assert_eq!(with_kw.keyword.as_deref(), Some("k"));

    let without_kw: ListMemosByTagInput =
        serde_json::from_value(json!({ "tag_id": 3 })).unwrap();
    assert_eq!(without_kw.tag_id, 3);
    assert!(without_kw.keyword.is_none(), "keyword 缺省应为 None");
}

#[test]
fn set_memo_tags_input_deserializes_tag_names() {
    let input: SetMemoTagsInput = serde_json::from_value(json!({
        "memo_id": 7,
        "tag_names": ["工作", "生活"]
    }))
    .unwrap();
    assert_eq!(input.memo_id, 7);
    assert_eq!(input.tag_names, vec!["工作", "生活"]);
}