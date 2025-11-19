//! WebSocket 连接处理器
//!
//! 负责处理 WebSocket 连接的完整生命周期:
//! 1. 认证 (首次消息必须是 Auth)
//! 2. 消息循环 (广播接收 + 客户端消息处理)
//! 3. 心跳检测 (30 秒 ping/pong)
//!
//! # 安全
//! - 连接建立后 5 秒内必须完成认证
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
    response::Response,
};
use tokio::sync::broadcast;
use futures_util::{StreamExt, SinkExt};

use vespera_common::{ClientMessage, ServerMessage, UserRole};

use crate::middleware::auth::AuthUser;
use crate::state::AppState;

/// 认证超时时间 (秒)
const AUTH_TIMEOUT_SECS: u64 = 5;

/// 心跳间隔 (秒)
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// WebSocket 升级处理器
///
/// GET /api/v1/ws
///
/// # 安全
/// - 不在 URL 参数中传递 token (避免日志泄露)
/// - 要求首次 WebSocket 消息包含认证信息
///
/// # 协议
/// 1. 客户端连接
/// 2. 客户端发送 `{"type":"auth","token":"<JWT>"}`
/// 3. 服务器验证 JWT 并响应 `{"type":"auth_success"}` 或 `{"type":"auth_failed"}`
/// 4. 进入正常消息循环
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// 处理单个 WebSocket 连接
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    // 分离读写流
    let (mut sender, mut receiver) = socket.split();

    // 1. 等待认证消息 (超时 5 秒)
    let auth_user = match tokio::time::timeout(
        Duration::from_secs(AUTH_TIMEOUT_SECS),
        authenticate(&mut receiver),
    )
    .await
    {
        Ok(Ok(user)) => {
            // 认证成功
            tracing::info!(user_id = user.id, username = %user.username, "WebSocket authenticated");

            // 发送成功响应
            let success_msg = ServerMessage::AuthSuccess;
            if let Ok(json) = serde_json::to_string(&success_msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }

            user
        }
        Ok(Err(e)) => {
            // 认证失败
            tracing::warn!("WebSocket authentication failed: {}", e);

            let error_msg = ServerMessage::AuthFailed {
                message: e.to_string(),
            };
            if let Ok(json) = serde_json::to_string(&error_msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }

            // 关闭连接
            let _ = sender.close().await;
            return;
        }
        Err(_) => {
            // 超时
            tracing::warn!("WebSocket authentication timeout");

            let error_msg = ServerMessage::AuthFailed {
                message: "Authentication timeout".to_string(),
            };
            if let Ok(json) = serde_json::to_string(&error_msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }

            let _ = sender.close().await;
            return;
        }
    };

    // 2. 创建会话
    let session = WsSession::new(auth_user);

    // 3. 订阅广播通道
    let mut broadcast_rx = state.broadcaster.subscribe();

    tracing::debug!(
        user_id = session.user.id,
        "WebSocket session started, receiver_count = {}",
        state.broadcaster.receiver_count()
    );

    // 4. 创建心跳定时器
    let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

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
                            user_id = session.user.id,
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
                        if let Err(e) = handle_client_message(msg, &mut session.clone()).await {
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
                        tracing::debug!(user_id = session.user.id, "WebSocket connection closed");
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

    tracing::info!(
        user_id = session.user.id,
        username = %session.user.username,
        "WebSocket connection terminated"
    );
}

/// 认证 WebSocket 连接
///
/// 等待客户端发送 Auth 消息,验证 JWT
async fn authenticate(
    receiver: &mut futures_util::stream::SplitStream<WebSocket>,
) -> Result<AuthUser, WsError> {
    use futures_util::StreamExt;

    // 等待首条消息
    let msg = receiver
        .next()
        .await
        .ok_or(WsError::ConnectionClosed)?
        .map_err(|_| WsError::InvalidMessage)?;

    // 必须是 Text 消息
    let text = match msg {
        Message::Text(t) => t.to_string(),
        _ => return Err(WsError::InvalidMessage),
    };

    // 解析为 ClientMessage
    let client_msg: ClientMessage =
        serde_json::from_str(&text).map_err(|_| WsError::InvalidMessage)?;

    // 必须是 Auth 消息
    let token = match client_msg {
        ClientMessage::Auth { token } => token,
        _ => return Err(WsError::Unauthorized),
    };

    // 验证 JWT
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET not set, using default");
        "change-this-secret-key-at-least-32-characters-long".to_string()
    });

    let claims = crate::utils::verify_jwt(&token, &jwt_secret)
        .map_err(|e| WsError::Unauthorized)?;

    // 解析用户信息
    let id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| WsError::InvalidMessage)?;

    let role = UserRole::from_str(&claims.role).ok_or(WsError::InvalidMessage)?;

    Ok(AuthUser {
        id,
        username: claims.username.unwrap_or_else(|| format!("user_{}", id)),
        role,
    })
}

/// WebSocket 会话
///
/// 保存连接状态和订阅信息
#[derive(Clone)]
struct WsSession {
    user: AuthUser,
    subscribed_nodes: HashSet<i64>,
}

impl WsSession {
    fn new(user: AuthUser) -> Self {
        Self {
            user,
            subscribed_nodes: HashSet::new(),
        }
    }

    /// 检查是否应该发送消息
    ///
    /// 过滤规则:
    /// - 如果未订阅任何节点,发送所有消息
    /// - 如果已订阅节点,只发送订阅节点的消息
    /// - Ping/Error 等全局消息总是发送
    fn should_send_message(&self, msg: &ServerMessage) -> bool {
        match msg {
            ServerMessage::MetricsUpdate(update) => {
                // 如果未订阅,发送所有
                if self.subscribed_nodes.is_empty() {
                    return true;
                }
                // 检查是否订阅该节点
                self.subscribed_nodes.contains(&update.node_id)
            }
            ServerMessage::NodeOnline { node_id, .. } => {
                if self.subscribed_nodes.is_empty() {
                    return true;
                }
                self.subscribed_nodes.contains(node_id)
            }
            ServerMessage::NodeOffline { node_id, .. } => {
                if self.subscribed_nodes.is_empty() {
                    return true;
                }
                self.subscribed_nodes.contains(node_id)
            }
            ServerMessage::Alert(alert) => {
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
async fn handle_client_message(msg: Message, session: &mut WsSession) -> Result<(), WsError> {
    let text = match msg {
        Message::Text(t) => t.to_string(),
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
            // TODO: 验证用户是否有权限查看这些节点
            tracing::debug!(
                user_id = session.user.id,
                node_count = node_ids.len(),
                "User subscribed to nodes"
            );
            session.subscribed_nodes.extend(node_ids);
            Ok(())
        }
        ClientMessage::Unsubscribe { node_ids } => {
            tracing::debug!(
                user_id = session.user.id,
                node_count = node_ids.len(),
                "User unsubscribed from nodes"
            );
            for node_id in node_ids {
                session.subscribed_nodes.remove(&node_id);
            }
            Ok(())
        }
        ClientMessage::Auth { .. } => {
            // 认证消息不应该在消息循环中出现
            Err(WsError::InvalidMessage)
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

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Permission denied")]
    PermissionDenied,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_send_message_no_subscription() {
        let user = AuthUser {
            id: 1,
            username: "test".to_string(),
            role: UserRole::User,
        };
        let session = WsSession::new(user);

        // 未订阅时应该发送所有消息
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
        let mut session = WsSession::new(user);
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
        let user = AuthUser {
            id: 1,
            username: "test".to_string(),
            role: UserRole::User,
        };
        let mut session = WsSession::new(user);
        session.subscribed_nodes.insert(1);

        // 全局消息总是发送
        assert!(session.should_send_message(&ServerMessage::Ping));
        assert!(session.should_send_message(&ServerMessage::Error {
            message: "test".to_string()
        }));
    }
}
