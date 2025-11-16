use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use thiserror::Error;

use super::response::Response as ApiResponse;

/// Server 端统一错误类型
#[derive(Error, Debug)]
pub enum ServerError {
    /// 内部服务器错误
    #[error("Internal server error: {0}")]
    Internal(String),

    /// 数据库错误
    #[error("Database error: {0}")]
    Database(String),

    /// 请求参数错误
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// 未找到资源
    #[error("Not found: {0}")]
    NotFound(String),

    /// 未授权
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 自定义错误
    #[error("{0}")]
    Custom(String),
}

impl ServerError {
    /// 获取错误对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            ServerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ServerError::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 获取错误对应的业务错误码
    pub fn error_code(&self) -> i32 {
        match self {
            ServerError::Internal(_) => 5000,
            ServerError::Database(_) => 5001,
            ServerError::BadRequest(_) => 4000,
            ServerError::NotFound(_) => 4004,
            ServerError::Unauthorized(_) => 4001,
            ServerError::Custom(_) => 5999,
        }
    }
}

/// 实现 IntoResponse trait，将错误转换为 HTTP 响应
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_code = self.error_code();
        let error_msg = self.to_string();

        let body = Json(ApiResponse::<()>::error(error_code, error_msg));

        (status_code, body).into_response()
    }
}

/// 从 anyhow::Error 转换
impl From<anyhow::Error> for ServerError {
    fn from(err: anyhow::Error) -> Self {
        ServerError::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ServerError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ServerError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ServerError::Internal("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(ServerError::BadRequest("test".to_string()).error_code(), 4000);
        assert_eq!(ServerError::NotFound("test".to_string()).error_code(), 4004);
        assert_eq!(ServerError::Internal("test".to_string()).error_code(), 5000);
    }
}
