// models 模块测试：实体/入参/返回结构的字段名与契约一致（小写下划线）。

use crate::models::{
    CreateMemoInput, DeleteMemoInput, DeleteMemoResult, GetMemoInput, Memo, SearchMemosInput,
    UpdateMemoInput,
};
use serde_json::json;

#[test]
fn memo_serializes_snake_case_fields_matching_contract() {
    let memo = Memo {
        id: 1,
        title: "标题".to_string(),
        content: "正文".to_string(),
        created_at: "2026-09-14T03:15:00.000Z".to_string(),
        updated_at: "2026-09-14T03:15:00.000Z".to_string(),
        remind_at: "".to_string(),
    };

    let value = serde_json::to_value(&memo).unwrap();
    let obj = value.as_object().unwrap();
    assert_eq!(obj.len(), 6, "Memo 应恰好序列化 6 个字段");
    assert_eq!(obj["id"], json!(1));
    assert_eq!(obj["title"], json!("标题"));
    assert_eq!(obj["content"], json!("正文"));
    assert_eq!(obj["created_at"], json!("2026-09-14T03:15:00.000Z"));
    assert_eq!(obj["updated_at"], json!("2026-09-14T03:15:00.000Z"));
    assert_eq!(obj["remind_at"], json!(""));

    for key in obj.keys() {
        assert!(
            key.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
            "字段名应为小写下划线: {key}"
        );
    }
}

#[test]
fn input_structs_deserialize_from_snake_case_json() {
    let create: CreateMemoInput =
        serde_json::from_value(json!({ "title": "t", "content": "c" })).unwrap();
    assert_eq!(create.title, "t");
    assert_eq!(create.content, "c");

    let get: GetMemoInput = serde_json::from_value(json!({ "id": 7 })).unwrap();
    assert_eq!(get.id, 7);

    let update: UpdateMemoInput =
        serde_json::from_value(json!({ "id": 7, "title": "t", "content": "c" })).unwrap();
    assert_eq!(update.id, 7);
    assert_eq!(update.title, "t");
    assert_eq!(update.content, "c");

    let delete: DeleteMemoInput = serde_json::from_value(json!({ "id": 7 })).unwrap();
    assert_eq!(delete.id, 7);

    let search: SearchMemosInput = serde_json::from_value(json!({ "keyword": "k" })).unwrap();
    assert_eq!(search.keyword, "k");
}

#[test]
fn delete_result_serializes_id_only() {
    let value = serde_json::to_value(DeleteMemoResult { id: 9 }).unwrap();
    assert_eq!(value["id"], json!(9));
    assert_eq!(value.as_object().unwrap().len(), 1);
}