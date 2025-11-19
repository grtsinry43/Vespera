import type { ServerMessage, ClientMessage, MetricsUpdate } from './types';

export type MessageHandler = (message: ServerMessage) => void;

export class WebSocketManager {
  private ws: WebSocket | null = null;
  private token: string;
  private url: string;
  private handlers: Set<MessageHandler> = new Set();
  private reconnectTimer: number | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 3000;
  private authenticated = false;
  private subscribedNodes: Set<number> = new Set();

  constructor(token: string, url = '/api/v1/ws') {
    this.token = token;

    // 在开发环境使用相对路径（走 Vite 代理），生产环境构造完整 URL
    if (import.meta.env.DEV) {
      // 开发环境：使用相对路径，Vite 会代理到后端
      // 需要将 http:// 替换为 ws://
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      this.url = `${protocol}//${window.location.host}${url}`;
    } else {
      // 生产环境：根据当前协议构造完整 WebSocket URL
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      this.url = `${protocol}//${window.location.host}${url}`;
    }
  }

  /**
   * 连接 WebSocket
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
          console.log('[WS] Connected');
          this.reconnectAttempts = 0;

          // 发送认证消息
          this.send({
            type: 'auth',
            token: this.token,
          });

          // 等待认证结果（但即使失败也继续，因为后端支持匿名访问）
          const authTimeout = setTimeout(() => {
            console.log('[WS] Auth timeout, continuing in anonymous mode');
            resolve(); // 超时也算成功，继续以匿名模式运行
          }, 5000);

          const handleAuth = (message: ServerMessage) => {
            clearTimeout(authTimeout);
            if (message.type === 'auth_success') {
              this.authenticated = true;
              console.log('[WS] Authenticated');
              // 重新订阅节点
              if (this.subscribedNodes.size > 0) {
                this.subscribe(Array.from(this.subscribedNodes));
              }
              resolve();
            } else if (message.type === 'auth_failed') {
              console.warn('[WS] Auth failed, continuing in anonymous mode');
              // 认证失败但继续连接（匿名模式）
              this.authenticated = false;
              resolve(); // 不要 reject，继续使用匿名模式
            }
          };

          this.addHandler(handleAuth);
        };

        this.ws.onmessage = (event) => {
          try {
            const message: ServerMessage = JSON.parse(event.data);

            // 处理 Ping 消息
            if (message.type === 'ping') {
              this.send({ type: 'pong' });
              return;
            }

            // 通知所有监听器
            this.handlers.forEach((handler) => handler(message));
          } catch (err) {
            console.error('[WS] Failed to parse message:', err);
          }
        };

        this.ws.onerror = (error) => {
          console.error('[WS] Error:', error);
          // 不要 reject，因为后续可能会重连
        };

        this.ws.onclose = () => {
          console.log('[WS] Disconnected');
          this.authenticated = false;
          this.attemptReconnect();
        };
      } catch (err) {
        reject(err);
      }
    });
  }

  /**
   * 尝试重连
   */
  private attemptReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WS] Max reconnect attempts reached');
      return;
    }

    this.reconnectAttempts++;
    console.log(`[WS] Reconnecting (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);

    this.reconnectTimer = window.setTimeout(() => {
      this.connect().catch((err) => {
        console.error('[WS] Reconnect failed:', err);
      });
    }, this.reconnectDelay * this.reconnectAttempts);
  }

  /**
   * 发送消息
   */
  private send(message: ClientMessage) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  /**
   * 订阅节点
   */
  subscribe(nodeIds: number[]) {
    nodeIds.forEach((id) => this.subscribedNodes.add(id));

    if (this.authenticated) {
      this.send({
        type: 'subscribe',
        node_ids: nodeIds,
      });
    }
  }

  /**
   * 取消订阅节点
   */
  unsubscribe(nodeIds: number[]) {
    nodeIds.forEach((id) => this.subscribedNodes.delete(id));

    if (this.authenticated) {
      this.send({
        type: 'unsubscribe',
        node_ids: nodeIds,
      });
    }
  }

  /**
   * 添加消息处理器
   */
  addHandler(handler: MessageHandler) {
    this.handlers.add(handler);
  }

  /**
   * 移除消息处理器
   */
  removeHandler(handler: MessageHandler) {
    this.handlers.delete(handler);
  }

  /**
   * 关闭连接
   */
  disconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.authenticated = false;
    this.handlers.clear();
  }

  /**
   * 获取连接状态
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN && this.authenticated;
  }
}