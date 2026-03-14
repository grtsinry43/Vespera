import type {
  ApiResponse,
  PublicNode,
  AdminNode,
  UpdateNodeRequest,
  NodeDetail,
  NodeMetrics,
  MetricsRangeQuery,
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  User,
  RefreshTokenRequest,
  ChangePasswordRequest,
  CreateUserRequest,
  UpdateUserRequest,
  ResetPasswordRequest,
  Service,
  ServiceCreate,
  ServiceUpdate,
  ServiceStatusPoint,
  ServiceStatusOverview,
  HealthCheckData
} from './types';
import { authStorage } from './authStorage';

const API_BASE = '/api/v1';

class ApiError extends Error {
  constructor(public code: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const token = authStorage.getAccessToken();

  const headers = new Headers(options?.headers);
  if (options?.body !== undefined) {
    headers.set('Content-Type', 'application/json');
  }

  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  }

  const response = await fetch(url, {
    ...options,
    credentials: 'same-origin',
    headers,
  });

  const data: ApiResponse<T> = await response.json();

  if (data.code !== 0) {
    throw new ApiError(data.code, data.msg || 'Unknown error');
  }

  return data.data!;
}

export const api = {
  // 系统健康检查
  system: {
    /**
     * 获取系统健康状态（包含版本和运行时长）
     */
    health: async (): Promise<HealthCheckData> => {
      return request<HealthCheckData>('/health');
    },
  },

  // 节点相关 API
  nodes: {
    /**
     * 获取节点列表
     */
    list: async (limit = 20, offset = 0): Promise<PublicNode[]> => {
      return request<PublicNode[]>(`${API_BASE}/nodes?limit=${limit}&offset=${offset}`);
    },

    /**
     * 获取节点详情
     */
    get: async (id: number): Promise<NodeDetail> => {
      return request<NodeDetail>(`${API_BASE}/nodes/${id}`);
    },

    /**
     * 获取节点历史指标
     */
    getMetrics: async (id: number, query: MetricsRangeQuery): Promise<NodeMetrics[]> => {
      const params = new URLSearchParams({
        start: query.start.toString(),
        end: query.end.toString(),
        ...(query.limit && { limit: query.limit.toString() }),
      });
      return request<NodeMetrics[]>(`${API_BASE}/nodes/${id}/metrics?${params}`);
    },

    /**
     * 管理员：获取所有节点（完整信息）
     */
    adminList: async (limit = 20, offset = 0): Promise<AdminNode[]> => {
      return request<AdminNode[]>(`${API_BASE}/admin/nodes?limit=${limit}&offset=${offset}`);
    },

    /**
     * 管理员：更新节点
     */
    adminUpdate: async (id: number, req: UpdateNodeRequest): Promise<AdminNode> => {
      return request<AdminNode>(`${API_BASE}/admin/nodes/${id}`, {
        method: 'PUT',
        body: JSON.stringify(req),
      });
    },

    /**
     * 管理员：删除节点
     */
    adminDelete: async (id: number): Promise<void> => {
      return request<void>(`${API_BASE}/admin/nodes/${id}`, {
        method: 'DELETE',
      });
    },
  },

  // 认证相关 API
  auth: {
    /**
     * 登录
     */
    login: async (username: string, password: string): Promise<LoginResponse> => {
      const data = await request<LoginResponse>(`${API_BASE}/auth/login`, {
        method: 'POST',
        body: JSON.stringify({ username, password }),
      });
      authStorage.setTokens(data.access_token);
      return data;
    },

    /**
     * 注册
     */
    register: async (req: RegisterRequest): Promise<LoginResponse> => {
      const data = await request<LoginResponse>(`${API_BASE}/auth/register`, {
        method: 'POST',
        body: JSON.stringify(req),
      });
      authStorage.setTokens(data.access_token);
      return data;
    },

    /**
     * 登出
     */
    logout: async (): Promise<void> => {
      try {
        await request<void>(`${API_BASE}/auth/logout`, {
          method: 'POST',
          body: JSON.stringify({}),
        });
      } catch (e) {
        console.error('Logout API failed:', e);
      }
      // 清除本地存储
      authStorage.clear();
    },

    /**
     * 获取当前用户信息
     */
    me: async (): Promise<User> => {
      return request<User>(`${API_BASE}/auth/me`);
    },

    /**
     * 修改密码
     */
    changePassword: async (req: ChangePasswordRequest): Promise<void> => {
      return request<void>(`${API_BASE}/auth/change-password`, {
        method: 'POST',
        body: JSON.stringify(req),
      });
    },

    /**
     * 刷新 Token
     */
    refreshToken: async (): Promise<void> => {
      const data = await request<{ access_token: string; refresh_token?: string; expires_at: number }>(`${API_BASE}/auth/refresh`, {
        method: 'POST',
        body: JSON.stringify({}),
      });
      authStorage.setAccessToken(data.access_token);
    },
  },

  // 用户管理 API (管理员)
  users: {
    /**
     * 获取用户列表
     */
    list: async (): Promise<User[]> => {
      return request<User[]>(`${API_BASE}/users`);
    },

    /**
     * 获取用户详情
     */
    get: async (id: number): Promise<User> => {
      return request<User>(`${API_BASE}/users/${id}`);
    },

    /**
     * 创建用户
     */
    create: async (req: CreateUserRequest): Promise<User> => {
      return request<User>(`${API_BASE}/users`, {
        method: 'POST',
        body: JSON.stringify(req),
      });
    },

    /**
     * 更新用户
     */
    update: async (id: number, req: UpdateUserRequest): Promise<User> => {
      return request<User>(`${API_BASE}/users/${id}`, {
        method: 'PUT',
        body: JSON.stringify(req),
      });
    },

    /**
     * 删除用户
     */
    delete: async (id: number): Promise<void> => {
      return request<void>(`${API_BASE}/users/${id}`, {
        method: 'DELETE',
      });
    },

    /**
     * 重置用户密码（管理员）
     */
    resetPassword: async (id: number, req: ResetPasswordRequest): Promise<void> => {
      return request<void>(`${API_BASE}/users/${id}/reset-password`, {
        method: 'POST',
        body: JSON.stringify(req),
      });
    },
  },

  // 服务监控 API
  services: {
    /**
     * 获取所有服务列表
     */
    list: async (): Promise<Service[]> => {
      return request<Service[]>(`${API_BASE}/services`);
    },

    /**
     * 获取单个服务详情
     */
    get: async (id: number): Promise<Service> => {
      return request<Service>(`${API_BASE}/services/${id}`);
    },

    /**
     * 创建服务（管理员）
     */
    create: async (req: ServiceCreate): Promise<Service> => {
      return request<Service>(`${API_BASE}/services`, {
        method: 'POST',
        body: JSON.stringify(req),
      });
    },

    /**
     * 更新服务（管理员）
     */
    update: async (id: number, req: ServiceUpdate): Promise<Service> => {
      return request<Service>(`${API_BASE}/services/${id}`, {
        method: 'PUT',
        body: JSON.stringify(req),
      });
    },

    /**
     * 删除服务（管理员）
     */
    delete: async (id: number): Promise<void> => {
      return request<void>(`${API_BASE}/services/${id}`, {
        method: 'DELETE',
      });
    },

    /**
     * 获取服务状态历史（最近30小时）
     */
    getStatusHistory: async (id: number): Promise<ServiceStatusPoint[]> => {
      return request<ServiceStatusPoint[]>(`${API_BASE}/services/${id}/status`);
    },

    /**
     * 获取服务状态概览（服务信息 + 当前状态 + 历史）
     */
    getOverview: async (id: number): Promise<ServiceStatusOverview> => {
      return request<ServiceStatusOverview>(`${API_BASE}/services/${id}/overview`);
    },

    /**
     * 获取所有服务状态概览（前端监控面板用）
     */
    getAllOverviews: async (): Promise<ServiceStatusOverview[]> => {
      return request<ServiceStatusOverview[]>(`${API_BASE}/services/all/overview`);
    },
  },
};

export { ApiError };
