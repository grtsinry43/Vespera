import { wrap } from 'svelte-spa-router/wrap';
import { get } from 'svelte/store';
import { isAuthenticated } from './authStore';
import { push } from 'svelte-spa-router';

// Route components
import Dashboard from '../routes/Dashboard.svelte';
import Login from '../routes/Login.svelte';
import ServerDetail from '../routes/ServerDetail.svelte';
import AdminPanel from '../routes/admin/AdminPanel.svelte';
import NodeManagement from '../routes/admin/NodeManagement.svelte';
import UserManagement from '../routes/admin/UserManagement.svelte';
import Settings from '../routes/admin/Settings.svelte';
import NotFound from '../routes/NotFound.svelte';

// Auth guard for protected routes
function requireAuth(detail: any) {
    if (!get(isAuthenticated)) {
        // Redirect to login if not authenticated
        setTimeout(() => push('/login'), 0);
        return false;
    }
    return true;
}

// Route configuration
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
    '/admin/users': wrap({
        component: UserManagement,
        conditions: [requireAuth]
    }),
    '/admin/settings': wrap({
        component: Settings,
        conditions: [requireAuth]
    }),
    '*': NotFound
};
