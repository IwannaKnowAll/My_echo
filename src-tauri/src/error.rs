// 统一错误结构，经 Command 返回给前端用于分支提示

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

/// 错误码：仅三类，与 Planner 契约严格一致
#[derive(Debug)]
pub enum ErrorCode {
    /// 参数校验失败
    Validation,
    /// 目标不存在
    NotFound,
    /// 内部错误
    Internal,
}

impl ErrorCode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Validation => "E_VALIDATION",
            Self::NotFound => "E_NOT_FOUND",
            Self::Internal => "E_INTERNAL",
        }
    }
}

/// 序列化后前端 catch (err) 直接拿到 { code, message }
#[derive(Debug)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Validation,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::NotFound,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Internal,
            message: message.into(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code.as_str())?;
        state.serialize_field("message", &self.message)?;
        state.end()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(_: rusqlite::Error) -> Self {
        Self::internal("数据库操作失败")
    }
}