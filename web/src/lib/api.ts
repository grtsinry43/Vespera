import type {
  ApiResponse,
  PublicNode,
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
  ResetPasswordRequest
} from './types';

const API_BASE = '/api/v1';

class ApiError extends Error {
  constructor(public code: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const token = localStorage.getItem('token');

  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...options?.headers,
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(url, {
    ...options,
    headers,
  });

  const data: ApiResponse<T> = await response.json();

  if (data.code !== 0) {
    throw new ApiError(data.code, data.msg || 'Unknown error');
  }

  return data.data!;
}

export const api = {
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
      // 保存 token 到 localStorage
      localStorage.setItem('token', data.access_token);
      localStorage.setItem('refresh_token', data.refresh_token);
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
      // 保存 token 到 localStorage
      localStorage.setItem('token', data.access_token);
      localStorage.setItem('refresh_token', data.refresh_token);
      return data;
    },

    /**
     * 登出
     */
    logout: async (): Promise<void> => {
      const refreshToken = localStorage.getItem('refresh_token');
      if (refreshToken) {
        try {
          await request<void>(`${API_BASE}/auth/logout`, {
            method: 'POST',
            body: JSON.stringify({ refresh_token: refreshToken }),
          });
        } catch (e) {
          console.error('Logout API failed:', e);
        }
      }
      // 清除本地存储
      localStorage.removeItem('token');
      localStorage.removeItem('refresh_token');
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
      const refreshToken = localStorage.getItem('refresh_token');
      if (!refreshToken) {
        throw new Error('No refresh token available');
      }
      const data = await request<{ access_token: string; expires_at: number }>(`${API_BASE}/auth/refresh`, {
        method: 'POST',
        body: JSON.stringify({ refresh_token: refreshToken }),
      });
      localStorage.setItem('token', data.access_token);
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
};

export { ApiError };