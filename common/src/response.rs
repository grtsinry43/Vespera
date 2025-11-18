use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;

/// 统一 API 响应格式
#[derive(Serialize, ToSchema)]
pub struct Response<T> {
    /// 状态码：0 表示成功，非 0 表示错误
    pub code: i32,
    /// 响应数据（成功时包含数据，失败时为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 错误消息（失败时包含消息，成功时为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// 时间戳（UTC）
    pub timestamp: String,
}

impl<T> Response<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            data: Some(data),
            msg: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// 创建失败响应
    pub fn error(code: i32, msg: String) -> Response<()> {
        Response {
            code,
            data: None,
            msg: Some(msg),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let resp = Response::success("test data");
        assert_eq!(resp.code, 0);
        assert_eq!(resp.data, Some("test data"));
        assert!(resp.msg.is_none());
    }

    #[test]
    fn test_error_response() {
        let resp = Response::<()>::error(1001, "Test error".to_string());
        assert_eq!(resp.code, 1001);
        assert!(resp.data.is_none());
        assert_eq!(resp.msg, Some("Test error".to_string()));
    }

    #[test]
    fn test_serialization() {
        let resp = Response::success(vec![1, 2, 3]);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"code\":0"));
        assert!(json.contains("\"data\":[1,2,3]"));
    }
}
