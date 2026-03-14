import Dashboard from "../routes/Dashboard.svelte";
import Login from "../routes/Login.svelte";
import ServerDetail from "../routes/ServerDetail.svelte";
import NotFound from "../routes/NotFound.svelte";
import AdminPanel from "../routes/admin/AdminPanel.svelte";
import NodeManagement from "../routes/admin/NodeManagement.svelte";
import UserManagement from "../routes/admin/UserManagement.svelte";
import ServiceManagement from "../routes/admin/ServiceManagement.svelte";
import Settings from "../routes/admin/Settings.svelte";
import { wrap } from "svelte-spa-router/wrap";
import { isAuthenticated, authStore } from "./authStore";
import { get } from "svelte/store";

const guardedRoute = (component: any, conditions: Array<() => boolean>) =>
    wrap({
        component,
        conditions,
    }) as any;

// 路由守卫：检查是否已登录
const requireAuth = () => {
    if (!get(isAuthenticated)) {
        return false;
    }
    return true;
};

// 路由守卫：检查是否为管理员
const requireAdmin = () => {
    const auth = get(authStore);
    if (!auth.user) {
        return false;
    }
    if (auth.user.role !== 'admin') {
        return false;
    }
    return true;
};

export const routes: Record<string, any> = {
    '/': Dashboard as any,
    '/login': Login as any,
    '/servers/:id': ServerDetail as any,
    '/admin': guardedRoute(AdminPanel as any, [requireAuth]),
    '/admin/nodes': guardedRoute(NodeManagement as any, [requireAuth]),
    '/admin/services': guardedRoute(ServiceManagement as any, [requireAuth]),
    '/admin/users': guardedRoute(UserManagement as any, [requireAdmin]),
    '/admin/settings': guardedRoute(Settings as any, [requireAdmin]),
    '*': NotFound as any
};
