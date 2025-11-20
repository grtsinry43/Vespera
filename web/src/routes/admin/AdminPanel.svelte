<script lang="ts">
    import { link } from "svelte-spa-router";
</script>

<div class="space-y-8 animate-in fade-in duration-500">
    <div
        class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-8 shadow-[0_8px_30px_rgba(0,0,0,0.04)] dark:shadow-none"
    >
        <h1
            class="text-2xl font-bold text-zinc-900 dark:text-white tracking-tight mb-2"
        >
            Admin Panel
        </h1>
        <p class="text-sm text-zinc-500 mb-8">
            Manage your monitoring system
        </p>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <!-- Node Management -->
            <a
                href="/admin/nodes"
                use:link
                class="group p-6 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg hover:border-zinc-900 dark:hover:border-white transition-all hover:shadow-md"
            >
                <div class="flex items-center gap-3 mb-3">
                    <h3
                        class="text-lg font-semibold text-zinc-900 dark:text-white"
                    >
                        Nodes
                    </h3>
                </div>
                <p class="text-sm text-zinc-500">
                    Manage monitored servers and agents
                </p>
            </a>

            <!-- User Management -->
            <a
                href="/admin/users"
                use:link
                class="group p-6 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg hover:border-zinc-900 dark:hover:border-white transition-all hover:shadow-md"
            >
                <div class="flex items-center gap-3 mb-3">
                    <h3
                        class="text-lg font-semibold text-zinc-900 dark:text-white"
                    >
                        Users
                    </h3>
                </div>
                <p class="text-sm text-zinc-500">
                    Manage user accounts and permissions
                </p>
            </a>

            <!-- Settings -->
            <a
                href="/admin/settings"
                use:link
                class="group p-6 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg hover:border-zinc-900 dark:hover:border-white transition-all hover:shadow-md"
            >
                <div class="flex items-center gap-3 mb-3">
                    <h3
                        class="text-lg font-semibold text-zinc-900 dark:text-white"
                    >
                        Settings
                    </h3>
                </div>
                <p class="text-sm text-zinc-500">
                    Configure system settings and alerts
                </p>
            </a>
        </div>
    </div>
</div>
