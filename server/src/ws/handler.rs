//! WebSocket 连接处理器
//!
//! 负责处理 WebSocket 连接的完整生命周期:
//! 1. 可选认证 (可以匿名访问公开信息)
//! 2. 消息循环 (广播接收 + 客户端消息处理)
//! 3. 心跳检测 (30 秒 ping/pong)
//!
//! # 安全
//! - **公开模式**: 匿名用户可以接收所有节点的公开信息更新
//! - **认证模式**: 已认证用户获得完整访问权限
//! - 使用 JWT 验证用户身份
//! - 支持节点订阅过滤 (用户只能看到有权限的节点)

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{header::AUTHORIZATION, header::COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use vespera_common::{ClientMessage, ServerMessage, UserRole};

use crate::middleware::auth::AuthUser;
use crate::state::AppState;

/// 初始消息超时时间 (秒) - 用于等待可选的认证消息
const INITIAL_MESSAGE_TIMEOUT_SECS: u64 = 5;

/// 心跳间隔 (秒)
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// 单条客户端消息最大长度
const MAX_CLIENT_MESSAGE_SIZE_BYTES: usize = 16 * 1024;

/// 每个会话最多订阅的节点数量
const MAX_SUBSCRIPTIONS_PER_SESSION: usize = 512;

const ACCESS_TOKEN_COOKIE: &str = "vespera_access_token";

/// WebSocket 升级处理器
///
/// GET /api/v1/ws
///
/// # 公开访问
/// - **无需认证**: 匿名用户可以连接并接收公开的节点信息
/// - **可选认证**: 发送 Auth 消息后获得认证用户权限
///
/// # 协议
/// 1. 客户端连接
/// 2. (可选) 客户端发送 `{"type":"auth","token":"<JWT>"}` 进行认证
/// 3. 服务器响应 `{"type":"auth_success"}` 或继续以匿名模式运行
/// 4. 进入正常消息循环 (接收节点指标更新、告警等)
///
/// # 匿名模式限制
/// - 只能接收公开的节点信息 (MetricsUpdate, NodeOnline, NodeOffline)
/// - 可以订阅/取消订阅节点
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let permit = match state.ws_connection_limiter.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "WebSocket connection limit reached",
            )
                .into_response();
        }
    };

    let auth_user = authenticate_from_headers(&headers);

    ws.on_upgrade(|socket| handle_socket(socket, state, auth_user, permit))
}

/// 处理单个 WebSocket 连接
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    initial_auth_user: Option<AuthUser>,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    // 分离读写流
    let (mut sender, mut receiver) = socket.split();

    // 1. 尝试接收首条消息 (可能是认证消息，也可能是其他消息)
    // 如果是认证消息则进行认证，否则以匿名模式运行
    let auth_user = if let Some(user) = initial_auth_user {
        tracing::info!(user_id = user.id, username = %user.username, "WebSocket authenticated from handshake");
        Some(user)
    } else {
        match tokio::time::timeout(
            Duration::from_secs(INITIAL_MESSAGE_TIMEOUT_SECS),
            try_authenticate(&mut receiver, &mut sender),
        )
        .await
        {
            Ok(Some(user)) => {
                tracing::info!(user_id = user.id, username = %user.username, "WebSocket authenticated");
                Some(user)
            }
            Ok(None) => {
                tracing::info!("WebSocket connected in anonymous mode");
                None
            }
            Err(_) => {
                tracing::info!("WebSocket initial message timeout, running in anonymous mode");
                None
            }
        }
    };

    // 2. 创建会话
    let mut session = WsSession::new(auth_user);

    // 3. 订阅广播通道
    let mut broadcast_rx = state.broadcaster.subscribe();

    let user_id_str = session
        .user
        .as_ref()
        .map(|u| u.id.to_string())
        .unwrap_or_else(|| "anonymous".to_string());

    tracing::debug!(
        user = %user_id_str,
        "WebSocket session started, receiver_count = {}",
        state.broadcaster.receiver_count()
    );

    // 4. 创建心跳定时器
    let mut heartbeat_interval =
        tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

    // 5. 消息循环
    loop {
        tokio::select! {
            // 接收广播消息
            result = broadcast_rx.recv() => {
                match result {
                    Ok(msg) => {
                        // 过滤消息 (检查订阅)
                        if !session.should_send_message(&msg) {
                            continue;
                        }

                        // 序列化并发送
                        let json = match serde_json::to_string(&msg) {
                            Ok(j) => j,
                            Err(e) => {
                                tracing::error!("Failed to serialize message: {}", e);
                                continue;
                            }
                        };

                        if let Err(e) = sender.send(Message::Text(json.into())).await {
                            tracing::warn!("Failed to send message: {}", e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(
                            user = %user_id_str,
                            lagged_count = n,
                            "WebSocket receiver lagged, messages lost"
                        );
                        // 继续运行,慢速客户端丢失部分消息是可接受的
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::error!("Broadcast channel closed");
                        break;
                    }
                }
            }

            // 接收客户端消息
            result = receiver.next() => {
                match result {
                    Some(Ok(msg)) => {
                        if let Err(e) = handle_client_message(msg, &mut session, &mut sender).await {
                            tracing::warn!("Error handling client message: {}", e);
                            // 发送错误响应
                            let error_msg = ServerMessage::Error {
                                message: e.to_string(),
                            };
                            if let Ok(json) = serde_json::to_string(&error_msg) {
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        tracing::warn!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        tracing::debug!(user = %user_id_str, "WebSocket connection closed");
                        break;
                    }
                }
            }

            // 心跳
            _ = heartbeat_interval.tick() => {
                let ping_msg = ServerMessage::Ping;
                if let Ok(json) = serde_json::to_string(&ping_msg) {
                    if let Err(e) = sender.send(Message::Text(json.into())).await {
                        tracing::warn!("Failed to send heartbeat: {}", e);
                        break;
                    }
                }
            }
        }
    }

    let final_user_str = session
        .user
        .as_ref()
        .map(|u| format!("{} ({})", u.username, u.id))
        .unwrap_or_else(|| "anonymous".to_string());

    tracing::info!(
        user = %final_user_str,
        "WebSocket connection terminated"
    );
}

/// 尝试认证 WebSocket 连接（可选）
///
/// 等待客户端首条消息:
/// - 如果是 Auth 消息，则验证 JWT 并返回 Some(AuthUser)
/// - 如果是其他消息，则返回 None (匿名模式)
async fn try_authenticate(
    receiver: &mut futures_util::stream::SplitStream<WebSocket>,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Option<AuthUser> {
    use futures_util::StreamExt;

    // 等待首条消息
    let msg = match receiver.next().await {
        Some(Ok(msg)) => msg,
        _ => return None, // 连接关闭或错误，以匿名模式运行
    };

    // 必须是 Text 消息
    let text = match msg {
        Message::Text(t) => {
            if t.len() > MAX_CLIENT_MESSAGE_SIZE_BYTES {
                return None;
            }
            t.to_string()
        }
        _ => return None, // 非文本消息，以匿名模式运行
    };

    // 尝试解析为 ClientMessage
    let client_msg: ClientMessage = match serde_json::from_str(&text) {
        Ok(msg) => msg,
        Err(_) => return None, // 解析失败，以匿名模式运行
    };

    // 检查是否为 Auth 消息
    let token = match client_msg {
        ClientMessage::Auth { token } => token,
        _ => return None, // 非认证消息，以匿名模式运行
    };

    // 验证 JWT
    let user = match authenticate_token(&token) {
        Ok(user) => user,
        Err(message) => {
            tracing::warn!("WebSocket JWT verification failed: {}", message);
            let error_msg = ServerMessage::AuthFailed { message };
            if let Ok(json) = serde_json::to_string(&error_msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
            return None;
        }
    };

    // 认证成功，发送成功响应
    let success_msg = ServerMessage::AuthSuccess;
    if let Ok(json) = serde_json::to_string(&success_msg) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    Some(user)
}

/// WebSocket 会话
///
/// 保存连接状态和订阅信息
#[derive(Clone)]
struct WsSession {
    user: Option<AuthUser>,
    subscribed_nodes: HashSet<i64>,
}

impl WsSession {
    fn new(user: Option<AuthUser>) -> Self {
        Self {
            user,
            subscribed_nodes: HashSet::new(),
        }
    }

    /// 检查是否已认证
    fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    /// 检查是否应该发送消息
    ///
    /// 过滤规则:
    /// - 匿名用户只能接收公开信息 (MetricsUpdate, NodeOnline, NodeOffline)
    /// - 认证用户可以接收所有消息
    /// - 如果未订阅任何节点,发送所有消息
    /// - 如果已订阅节点,只发送订阅节点的消息
    /// - Ping/Error 等全局消息总是发送
    fn should_send_message(&self, msg: &ServerMessage) -> bool {
        match msg {
            ServerMessage::MetricsUpdate(update) => {
                // 公开信息，所有用户都可以接收
                if self.subscribed_nodes.is_empty() {
                    return true;
                }
                self.subscribed_nodes.contains(&update.node_id)
            }
            ServerMessage::NodeOnline { node_id, .. } => {
                // 公开信息，所有用户都可以接收
                if self.subscribed_nodes.is_empty() {
                    return true;
                }
                self.subscribed_nodes.contains(node_id)
            }
            ServerMessage::NodeOffline { node_id, .. } => {
                // 公开信息，所有用户都可以接收
                if self.subscribed_nodes.is_empty() {
                    return true;
                }
                self.subscribed_nodes.contains(node_id)
            }
            ServerMessage::Alert(alert) => {
                if !self.is_authenticated() && !alert.is_public {
                    return false;
                }
                if self.subscribed_nodes.is_empty() {
                    return true;
                }
                self.subscribed_nodes.contains(&alert.node_id)
            }
            // 全局消息总是发送
            ServerMessage::Ping | ServerMessage::Error { .. } => true,
            // 认证消息不应该出现在这里
            ServerMessage::AuthSuccess | ServerMessage::AuthFailed { .. } => false,
        }
    }
}

/// 处理客户端消息
async fn handle_client_message(
    msg: Message,
    session: &mut WsSession,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
) -> Result<(), WsError> {
    let text = match msg {
        Message::Text(t) => {
            if t.len() > MAX_CLIENT_MESSAGE_SIZE_BYTES {
                return Err(WsError::MessageTooLarge);
            }
            t.to_string()
        }
        Message::Close(_) => return Err(WsError::ConnectionClosed),
        _ => return Ok(()), // 忽略其他消息类型
    };

    let client_msg: ClientMessage =
        serde_json::from_str(&text).map_err(|_| WsError::InvalidMessage)?;

    match client_msg {
        ClientMessage::Pong => {
            // 心跳响应,忽略
            Ok(())
        }
        ClientMessage::Subscribe { node_ids } => {
            if session
                .subscribed_nodes
                .len()
                .saturating_add(node_ids.len())
                > MAX_SUBSCRIPTIONS_PER_SESSION
            {
                return Err(WsError::TooManySubscriptions);
            }
            // 公开操作，匿名用户也可以订阅节点
            tracing::debug!(
                user = session
                    .user
                    .as_ref()
                    .map(|u| u.id.to_string())
                    .unwrap_or_else(|| "anonymous".to_string()),
                node_count = node_ids.len(),
                "User subscribed to nodes"
            );
            session.subscribed_nodes.extend(node_ids);
            Ok(())
        }
        ClientMessage::Unsubscribe { node_ids } => {
            tracing::debug!(
                user = session
                    .user
                    .as_ref()
                    .map(|u| u.id.to_string())
                    .unwrap_or_else(|| "anonymous".to_string()),
                node_count = node_ids.len(),
                "User unsubscribed from nodes"
            );
            for node_id in node_ids {
                session.subscribed_nodes.remove(&node_id);
            }
            Ok(())
        }
        ClientMessage::Auth { token } => {
            // 运行时认证（匿名用户后续可以发送认证消息升级权限）
            if session.is_authenticated() {
                // 已认证，忽略
                return Ok(());
            }

            // 验证 JWT
            match authenticate_token(&token) {
                Ok(user) => {
                    let id = user.id;
                    session.user = Some(user);

                    tracing::info!(user_id = id, "WebSocket upgraded to authenticated mode");

                    // 发送成功响应
                    let success_msg = ServerMessage::AuthSuccess;
                    if let Ok(json) = serde_json::to_string(&success_msg) {
                        let _ = sender.send(Message::Text(json.into())).await;
                    }

                    Ok(())
                }
                Err(e) => {
                    // 认证失败
                    tracing::warn!("WebSocket runtime auth failed: {}", e);
                    let error_msg = ServerMessage::AuthFailed { message: e };
                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = sender.send(Message::Text(json.into())).await;
                    }
                    Ok(())
                }
            }
        }
    }
}

/// WebSocket 错误
#[derive(thiserror::Error, Debug)]
pub enum WsError {
    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Invalid message format")]
    InvalidMessage,

    #[error("Message too large")]
    MessageTooLarge,

    #[error("Too many subscriptions")]
    TooManySubscriptions,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Permission denied")]
    PermissionDenied,
}

fn authenticate_from_headers(headers: &HeaderMap) -> Option<AuthUser> {
    let bearer = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(ToOwned::to_owned);

    let cookie_token = headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(extract_access_token_from_cookie);

    bearer
        .or(cookie_token)
        .and_then(|token| authenticate_token(&token).ok())
}

fn extract_access_token_from_cookie(cookie_header: &str) -> Option<String> {
    cookie_header.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        if name == ACCESS_TOKEN_COOKIE && !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn authenticate_token(token: &str) -> Result<AuthUser, String> {
    let jwt_secret = crate::utils::jwt_secret_from_env()
        .map_err(|e| format!("Server authentication configuration error: {}", e))?;

    let claims = crate::utils::verify_jwt(token, &jwt_secret)
        .map_err(|e| format!("Invalid token: {}", e))?;

    let id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| "Invalid user ID in token".to_string())?;

    let role =
        UserRole::from_str(&claims.role).ok_or_else(|| "Invalid role in token".to_string())?;

    Ok(AuthUser {
        id,
        username: claims.username.unwrap_or_else(|| format!("user_{}", id)),
        role,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_send_message_anonymous_no_subscription() {
        let session = WsSession::new(None);

        // 匿名用户未订阅时应该接收所有公开消息
        let msg = ServerMessage::MetricsUpdate(vespera_common::MetricsUpdate {
            node_id: 1,
            node_uuid: "test".to_string(),
            node_name: "test".to_string(),
            timestamp: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            disk_info: vec![],
            network_in: 0,
            network_out: 0,
            load_1: None,
            load_5: None,
            load_15: None,
        });

        assert!(session.should_send_message(&msg));
    }

    #[test]
    fn test_should_send_message_authenticated_no_subscription() {
        let user = AuthUser {
            id: 1,
            username: "test".to_string(),
            role: UserRole::User,
        };
        let session = WsSession::new(Some(user));

        // 认证用户未订阅时应该接收所有消息
        let msg = ServerMessage::MetricsUpdate(vespera_common::MetricsUpdate {
            node_id: 1,
            node_uuid: "test".to_string(),
            node_name: "test".to_string(),
            timestamp: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            disk_info: vec![],
            network_in: 0,
            network_out: 0,
            load_1: None,
            load_5: None,
            load_15: None,
        });

        assert!(session.should_send_message(&msg));
    }

    #[test]
    fn test_should_send_message_with_subscription() {
        let user = AuthUser {
            id: 1,
            username: "test".to_string(),
            role: UserRole::User,
        };
        let mut session = WsSession::new(Some(user));
        session.subscribed_nodes.insert(1);
        session.subscribed_nodes.insert(2);

        // 订阅了节点 1 和 2
        let msg1 = ServerMessage::NodeOnline {
            node_id: 1,
            node_name: "node1".to_string(),
        };
        let msg3 = ServerMessage::NodeOnline {
            node_id: 3,
            node_name: "node3".to_string(),
        };

        assert!(session.should_send_message(&msg1)); // 应该发送
        assert!(!session.should_send_message(&msg3)); // 不应该发送
    }

    #[test]
    fn test_always_send_global_messages() {
        let session = WsSession::new(None);

        // 全局消息总是发送（即使是匿名用户）
        assert!(session.should_send_message(&ServerMessage::Ping));
        assert!(session.should_send_message(&ServerMessage::Error {
            message: "test".to_string()
        }));
    }

    #[test]
    fn test_anonymous_session() {
        let session = WsSession::new(None);
        assert!(!session.is_authenticated());
    }

    #[test]
    fn test_authenticated_session() {
        let user = AuthUser {
            id: 1,
            username: "test".to_string(),
            role: UserRole::User,
        };
        let session = WsSession::new(Some(user));
        assert!(session.is_authenticated());
    }
}
