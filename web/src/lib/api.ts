import type { ApiResponse, PublicNode, NodeDetail, NodeMetrics, MetricsRangeQuery } from './types';

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
    login: async (username: string, password: string): Promise<{ token: string }> => {
      return request<{ token: string }>(`${API_BASE}/auth/login`, {
        method: 'POST',
        body: JSON.stringify({ username, password }),
      });
    },

    /**
     * 注册
     */
    register: async (username: string, password: string): Promise<{ token: string }> => {
      return request<{ token: string }>(`${API_BASE}/auth/register`, {
        method: 'POST',
        body: JSON.stringify({ username, password }),
      });
    },
  },
};

export { ApiError };