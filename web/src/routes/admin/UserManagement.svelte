<script lang="ts">
    import { onMount } from "svelte";
    import AdminLayout from "../../lib/admin/AdminLayout.svelte";
    import UserModal from "../../lib/admin/UserModal.svelte";
    import { api } from "../../lib/api";
    import type { User } from "../../lib/types";

    let users = $state<User[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    // Modal state
    let showModal = $state(false);
    let editingUser = $state<User | null>(null);

    // Get time since last login
    function getTimeSince(timestamp: number | null): string {
        if (!timestamp) return "Never";
        const now = Math.floor(Date.now() / 1000);
        const diff = now - timestamp;
        if (diff < 60) return `${diff}s ago`;
        if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
        if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
        return `${Math.floor(diff / 86400)}d ago`;
    }

    async function loadUsers() {
        try {
            if (users.length === 0) loading = true;
            error = null;
            users = await api.users.list();
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load users";
            console.error("Failed to load users:", e);
        } finally {
            loading = false;
        }
    }

    function handleCreate() {
        editingUser = null;
        showModal = true;
    }

    function handleEdit(user: User) {
        editingUser = user;
        showModal = true;
    }

    async function handleDelete(user: User) {
        if (
            !confirm(
                `Are you sure you want to delete user "${user.username}"? This action cannot be undone.`,
            )
        ) {
            return;
        }

        try {
            await api.users.delete(user.id);
            await loadUsers();
        } catch (e) {
            console.error("Failed to delete user:", e);
            alert("Failed to delete user");
        }
    }

    async function handleResetPassword(user: User) {
        const newPassword = prompt(
            `Enter new password for user "${user.username}":`,
        );
        if (!newPassword) return;

        if (newPassword.length < 6) {
            alert("Password must be at least 6 characters");
            return;
        }

        try {
            await api.users.resetPassword(user.id, { new_password: newPassword });
            alert("Password reset successfully");
        } catch (e) {
            console.error("Failed to reset password:", e);
            alert("Failed to reset password");
        }
    }

    function handleSave() {
        loadUsers();
    }

    onMount(() => {
        loadUsers();
    });
</script>

<AdminLayout>
    <div class="max-w-7xl mx-auto p-8 md:p-12 animate-in fade-in duration-500">
        <div class="flex items-center justify-between mb-12">
            <div>
                <h1
                    class="text-3xl font-bold text-zinc-900 dark:text-white tracking-tight mb-2"
                >
                    Users
                </h1>
                <p class="text-zinc-500 dark:text-zinc-400">
                    Manage user access and permissions.
                </p>
            </div>
            <button
                on:click={handleCreate}
                class="px-4 py-2 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
            >
                Create User
            </button>
        </div>

        {#if loading && users.length === 0}
            <div class="flex justify-center items-center py-12">
                <div class="text-zinc-500">Loading users...</div>
            </div>
        {:else if error}
            <div class="flex justify-center items-center py-12">
                <div class="text-red-500">Error: {error}</div>
            </div>
        {:else if users.length === 0}
            <div
                class="flex flex-col justify-center items-center py-20 border-2 border-dashed border-zinc-200 dark:border-zinc-800 rounded-2xl"
            >
                <div
                    class="w-12 h-12 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center mb-4 text-zinc-400"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M22 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg
                    >
                </div>
                <h3
                    class="text-lg font-medium text-zinc-900 dark:text-white mb-1"
                >
                    No users created
                </h3>
                <p class="text-zinc-500 dark:text-zinc-400 mb-6">
                    Get started by creating your first user.
                </p>
                <button
                    on:click={handleCreate}
                    class="px-4 py-2 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
                >
                    Create User
                </button>
            </div>
        {:else}
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="border-b border-zinc-200 dark:border-zinc-800">
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >User</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Role</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Email</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Last Login</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider text-right"
                                >Actions</th
                            >
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-zinc-100 dark:divide-zinc-800/50">
                        {#each users as user}
                            <tr
                                class="group hover:bg-zinc-50 dark:hover:bg-zinc-800/30 transition-colors"
                            >
                                <td class="py-4 px-4">
                                    <div class="flex items-center gap-3">
                                        <div
                                            class="w-8 h-8 rounded-full bg-zinc-200 dark:bg-zinc-700 flex items-center justify-center text-xs font-medium text-zinc-600 dark:text-zinc-300"
                                        >
                                            {user.username.charAt(0).toUpperCase()}
                                        </div>
                                        <div>
                                            <div
                                                class="text-sm font-medium text-zinc-900 dark:text-white"
                                            >
                                                {user.username}
                                            </div>
                                            <div class="text-[10px] text-zinc-500">
                                                ID: {user.id}
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td class="py-4 px-4">
                                    <span
                                        class="inline-flex items-center px-2 py-1 rounded-md text-xs font-medium
                                        {user.role === 'admin'
                                            ? 'bg-indigo-100 text-indigo-700 dark:bg-indigo-500/10 dark:text-indigo-400'
                                            : 'bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300'}"
                                    >
                                        {user.role === 'admin' ? "Administrator" : "User"}
                                    </span>
                                </td>
                                <td class="py-4 px-4">
                                    <span class="text-sm text-zinc-500 dark:text-zinc-400">
                                        {user.email || "—"}
                                    </span>
                                </td>
                                <td
                                    class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400"
                                    >{getTimeSince(user.last_login_at)}</td
                                >
                                <td class="py-4 px-4 text-right">
                                    <div class="flex items-center justify-end gap-2">
                                        <button
                                            on:click={() => handleEdit(user)}
                                            class="opacity-0 group-hover:opacity-100 p-1.5 text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-all"
                                            title="Edit"
                                        >
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                width="16"
                                                height="16"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                ><path
                                                    d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                                                ></path><path
                                                    d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
                                                ></path></svg
                                            >
                                        </button>
                                        <button
                                            on:click={() => handleResetPassword(user)}
                                            class="opacity-0 group-hover:opacity-100 p-1.5 text-zinc-400 hover:text-amber-600 dark:hover:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-900/20 rounded transition-all"
                                            title="Reset Password"
                                        >
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                width="16"
                                                height="16"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                ><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg
                                            >
                                        </button>
                                        <button
                                            on:click={() => handleDelete(user)}
                                            class="opacity-0 group-hover:opacity-100 p-1.5 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-all"
                                            title="Delete"
                                        >
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                width="16"
                                                height="16"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                ><polyline points="3 6 5 6 21 6"
                                                ></polyline><path
                                                    d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                                                ></path><line
                                                    x1="10"
                                                    y1="11"
                                                    x2="10"
                                                    y2="17"
                                                ></line><line
                                                    x1="14"
                                                    y1="11"
                                                    x2="14"
                                                    y2="17"
                                                ></line></svg
                                            >
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </div>

    <UserModal
        bind:show={showModal}
        bind:user={editingUser}
        on:save={handleSave}
        on:close={() => (showModal = false)}
    />
</AdminLayout>
