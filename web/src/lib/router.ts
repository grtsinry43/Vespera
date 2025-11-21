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

export const routes = {
    '/': Dashboard,
    '/login': Login,
    '/servers/:id': ServerDetail,
    '/admin': wrap({
        component: AdminPanel,
        conditions: [requireAuth]
    }),
    '/admin/nodes': wrap({
        component: NodeManagement,
        conditions: [requireAuth]
    }),
    '/admin/services': wrap({
        component: ServiceManagement,
        conditions: [requireAuth]
    }),
    '/admin/users': wrap({
        component: UserManagement,
        conditions: [requireAdmin]
    }),
    '/admin/settings': wrap({
        component: Settings,
        conditions: [requireAdmin]
    }),
    '*': NotFound
};
