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
  is_public: boolean;
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

// ==================== 用户认证相关 ====================

// 用户角色
export type UserRole = 'admin' | 'user';

// 用户信息
export interface User {
  id: number;
  username: string;
  email?: string;
  role: UserRole;
  avatar_url?: string;
  is_active: boolean;
  created_at: number;
  updated_at: number;
  last_login_at?: number;
}

// 登录请求
export interface LoginRequest {
  username: string;
  password: string;
}

// 注册请求
export interface RegisterRequest {
  username: string;
  email?: string;
  password: string;
  is_admin?: boolean;
}

// 登录响应
export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  user: User;
  expires_at: number;
}

// Refresh Token 请求
export interface RefreshTokenRequest {
  refresh_token: string;
}

// Refresh Token 响应
export interface RefreshTokenResponse {
  access_token: string;
  refresh_token?: string;
  expires_at: number;
}

// 修改密码请求
export interface ChangePasswordRequest {
  old_password: string;
  new_password: string;
}

// 创建用户请求（管理员）
export interface CreateUserRequest {
  username: string;
  email?: string;
  password: string;
  role: UserRole;
}

// 更新用户请求（管理员）
export interface UpdateUserRequest {
  email?: string;
  avatar_url?: string;
  is_active?: boolean;
  role?: UserRole;
}

// 重置密码请求（管理员）
export interface ResetPasswordRequest {
  new_password: string;
}

// ==================== WebSocket ====================

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
  is_public: boolean;
  triggered_at: number;
}

// ==================== 管理员节点信息 ====================

export interface AdminNode {
  id: number;
  uuid: string;
  name: string;
  ip_address: string;
  agent_version: string;
  os_type: string;
  os_version?: string;
  cpu_cores: number;
  total_memory: number;
  status: NodeStatus;
  last_seen: number;
  created_at: number;
  updated_at: number;
  is_public: boolean;
  tags?: string[];
}

export interface UpdateNodeRequest {
  name?: string;
  tags?: string[];
  is_public?: boolean;
}

export interface UpdateNodeVisibilityRequest {
  is_public: boolean;
}

// ==================== 服务监控相关 ====================

// 服务类型
export type ServiceType = 'http' | 'tcp';

// 服务状态
export type ServiceStatus = 'up' | 'down' | 'timeout' | 'error' | 'unknown';

// 服务配置
export interface Service {
  id: number;
  node_id?: number;
  name: string;
  type: ServiceType;
  target: string;
  check_interval: number;
  timeout: number;
  method?: string;
  expected_code?: number;
  expected_body?: string;
  headers?: Record<string, string>;
  enabled: boolean;
  is_public: boolean;
  created_at: number;
  updated_at: number;
}

// 服务状态历史点
export interface ServiceStatusPoint {
  timestamp: number;
  status: ServiceStatus;
  response_time?: number;
}

// 服务状态概览
export interface ServiceStatusOverview {
  service: Service;
  current_status: ServiceStatus;
  history: ServiceStatusPoint[];
}

// 创建服务请求
export interface ServiceCreate {
  node_id?: number;
  name: string;
  type: ServiceType;
  target: string;
  check_interval?: number;
  timeout?: number;
  method?: string;
  expected_code?: number;
  expected_body?: string;
  headers?: Record<string, string>;
  enabled?: boolean;
  is_public?: boolean;
}

// 更新服务请求
export interface ServiceUpdate {
  name?: string;
  target?: string;
  check_interval?: number;
  timeout?: number;
  method?: string;
  expected_code?: number;
  expected_body?: string;
  headers?: Record<string, string>;
  enabled?: boolean;
  is_public?: boolean;
}

export interface UpdateServiceVisibilityRequest {
  is_public: boolean;
}

// ==================== 系统健康检查 ====================

export interface HealthCheckData {
  status: string;
  uptime_secs: number;
  version: string;
}
