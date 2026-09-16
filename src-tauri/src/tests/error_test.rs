// error 模块测试：三码字符串、序列化为 {code, message}、rusqlite 错误映射。

use crate::error::{AppError, ErrorCode};
use rusqlite::Connection;

#[test]
fn error_code_as_str_matches_contract() {
    assert_eq!(ErrorCode::Validation.as_str(), "E_VALIDATION");
    assert_eq!(ErrorCode::NotFound.as_str(), "E_NOT_FOUND");
    assert_eq!(ErrorCode::Internal.as_str(), "E_INTERNAL");
}

#[test]
fn app_error_serializes_to_code_and_message_only() {
    let cases = [
        (AppError::validation("标题不能为空"), "E_VALIDATION", "标题不能为空"),
        (AppError::not_found("备忘录不存在"), "E_NOT_FOUND", "备忘录不存在"),
        (AppError::internal("数据库操作失败"), "E_INTERNAL", "数据库操作失败"),
    ];

    for (err, code, message) in cases {
        let value = serde_json::to_value(&err).unwrap();
        assert_eq!(value["code"], code, "错误码序列化不符");
        assert_eq!(value["message"], message, "错误文案序列化不符");
        assert_eq!(
            value.as_object().unwrap().len(),
            2,
            "AppError 只应序列化 code/message 两字段"
        );
    }
}

#[test]
fn rusqlite_error_maps_to_internal() {
    let conn = Connection::open_in_memory().unwrap();
    let err = conn.execute("SELECT * FROM missing_table", []).unwrap_err();
    let app_err = AppError::from(err);

    assert_eq!(app_err.code.as_str(), "E_INTERNAL");
    let value = serde_json::to_value(&app_err).unwrap();
    assert_eq!(value["code"], "E_INTERNAL");
    assert_eq!(value["message"], "数据库操作失败");
}