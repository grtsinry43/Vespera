import { writable, derived } from 'svelte/store';
import type { User } from './types';
import { api } from './api';
import { authStorage } from './authStorage';

// 认证状态
interface AuthState {
  user: User | null;
  loading: boolean;
  error: string | null;
}

// 创建 store
function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    user: null,
    loading: false,
    error: null,
  });

  return {
    subscribe,

    // 初始化：检查登录状态
    async init() {
      const token = authStorage.getAccessToken();
      if (!token) {
        set({ user: null, loading: false, error: null });
        return;
      }

      update((state) => ({ ...state, loading: true }));

      try {
        const user = await api.auth.me();
        set({ user, loading: false, error: null });
      } catch (error: any) {
        // Token 可能已过期，尝试刷新
        try {
          await api.auth.refreshToken();
          const user = await api.auth.me();
          set({ user, loading: false, error: null });
        } catch (refreshError) {
          // 刷新失败，清除认证
          authStorage.clear();
          set({ user: null, loading: false, error: null });
        }
      }
    },

    // 登录
    async login(username: string, password: string) {
      update((state) => ({ ...state, loading: true, error: null }));

      try {
        const response = await api.auth.login(username, password);
        set({ user: response.user, loading: false, error: null });
        return response;
      } catch (error: any) {
        const errorMsg = error.message || 'Login failed';
        update((state) => ({ ...state, loading: false, error: errorMsg }));
        throw error;
      }
    },

    // 注册
    async register(username: string, password: string, email?: string) {
      update((state) => ({ ...state, loading: true, error: null }));

      try {
        const response = await api.auth.register({ username, password, email });
        set({ user: response.user, loading: false, error: null });
        return response;
      } catch (error: any) {
        const errorMsg = error.message || 'Registration failed';
        update((state) => ({ ...state, loading: false, error: errorMsg }));
        throw error;
      }
    },

    // 登出
    async logout() {
      try {
        await api.auth.logout();
      } catch (error) {
        console.error('Logout failed:', error);
      }
      set({ user: null, loading: false, error: null });
    },

    // 清除错误
    clearError() {
      update((state) => ({ ...state, error: null }));
    },
  };
}

export const authStore = createAuthStore();

// 派生状态
export const isAuthenticated = derived(authStore, ($auth) => $auth.user !== null);
export const isAdmin = derived(authStore, ($auth) => $auth.user?.role === 'admin');
