// API 响应类型
export interface ApiResponse<T> {
  code: number;
  data: T | null;
  msg: string | null;
  timestamp: string;
}

// 节点状态
export type NodeStatus = 'online' | 'offline' | 'warning';

// 公开节点信息
export interface PublicNode {
  id: number;
  name: string;
  status: NodeStatus;
  os_type: string;
  cpu_cores: number;
  total_memory: number;
  last_seen: number;
  tags?: string[];
  // 前端扩展字段 - 用于显示实时指标
  cpu_usage?: number;
  memory_usage?: number;
  net_in?: number;
  net_out?: number;
}

// 节点详情
export interface NodeDetail {
  node: PublicNode;
  latest_metrics?: NodeMetrics;
}

// 节点指标
export interface NodeMetrics {
  timestamp: number;
  cpu_usage: number;
  memory_used: number;
  memory_total: number;
  memory_usage: number;
  disk_info: DiskMetric[];
  net_in_bytes: number;
  net_out_bytes: number;
  load_1?: number;
  load_5?: number;
  load_15?: number;
}

// 磁盘指标
export interface DiskMetric {
  mount: string;
  used: number;
  total: number;
  usage: number;
}

// 指标历史查询参数
export interface MetricsRangeQuery {
  start: number;
  end: number;
  limit?: number;
}

// WebSocket 消息类型
export type ClientMessage =
  | { type: 'auth'; token: string }
  | { type: 'pong' }
  | { type: 'subscribe'; node_ids: number[] }
  | { type: 'unsubscribe'; node_ids: number[] };

export type ServerMessage =
  | { type: 'auth_success' }
  | { type: 'auth_failed'; message: string }
  | { type: 'ping' }
  | { type: 'error'; message: string }
  | { type: 'metrics_update'; data: MetricsUpdate }
  | { type: 'node_online'; node_id: number; node_name: string }
  | { type: 'node_offline'; node_id: number; node_name: string }
  | { type: 'alert'; data: AlertData };

export interface MetricsUpdate {
  node_id: number;
  node_uuid: string;
  node_name: string;
  timestamp: number;
  cpu_usage: number;
  memory_usage: number;
  memory_used: number;
  memory_total: number;
  disk_info: DiskMetric[];
  network_in: number;
  network_out: number;
  load_1?: number;
  load_5?: number;
  load_15?: number;
}

export interface AlertData {
  id: number;
  node_id: number;
  alert_type: string;
  severity: string;
  message: string;
  triggered_at: number;
}