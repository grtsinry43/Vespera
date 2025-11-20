<script lang="ts">
    import AdminLayout from "../../lib/admin/AdminLayout.svelte";

    // Mock data
    const users = [
        {
            id: "u-01",
            name: "Admin User",
            email: "admin@vespera.app",
            role: "Administrator",
            status: "active",
            lastActive: "2 mins ago",
        },
        {
            id: "u-02",
            name: "John Doe",
            email: "john@example.com",
            role: "Developer",
            status: "active",
            lastActive: "1 hour ago",
        },
        {
            id: "u-03",
            name: "Jane Smith",
            email: "jane@example.com",
            role: "Viewer",
            status: "inactive",
            lastActive: "3 days ago",
        },
    ];
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
                class="px-4 py-2 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
            >
                Invite User
            </button>
        </div>

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
                            >Status</th
                        >
                        <th
                            class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                            >Last Active</th
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
                                        {user.name.charAt(0)}
                                    </div>
                                    <div>
                                        <div
                                            class="text-sm font-medium text-zinc-900 dark:text-white"
                                        >
                                            {user.name}
                                        </div>
                                        <div class="text-xs text-zinc-500">
                                            {user.email}
                                        </div>
                                    </div>
                                </div>
                            </td>
                            <td class="py-4 px-4">
                                <span
                                    class="inline-flex items-center px-2 py-1 rounded-md text-xs font-medium bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300"
                                >
                                    {user.role}
                                </span>
                            </td>
                            <td class="py-4 px-4">
                                <span
                                    class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium
                                    {user.status === 'active'
                                        ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400'
                                        : 'bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400'}"
                                >
                                    {user.status}
                                </span>
                            </td>
                            <td
                                class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400"
                                >{user.lastActive}</td
                            >
                            <td class="py-4 px-4 text-right">
                                <button
                                    class="text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors"
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
                                        ><circle cx="12" cy="12" r="1" /><circle
                                            cx="19"
                                            cy="12"
                                            r="1"
                                        /><circle cx="5" cy="12" r="1" /></svg
                                    >
                                </button>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
</AdminLayout>
