<script lang="ts">
    import AdminLayout from "../../lib/admin/AdminLayout.svelte";

    // Mock data for the dashboard
    const stats = [
        { label: "Total Nodes", value: "12", change: "+2", trend: "up" },
        { label: "Active Users", value: "1,234", change: "+15%", trend: "up" },
        { label: "System Load", value: "45%", change: "-5%", trend: "down" },
        { label: "Uptime", value: "99.9%", change: "0%", trend: "neutral" },
    ];

    const recentActivity = [
        {
            action: "New node registered",
            target: "server-01",
            time: "2 mins ago",
            user: "System",
        },
        {
            action: "User login",
            target: "john.doe",
            time: "15 mins ago",
            user: "john.doe",
        },
        {
            action: "Configuration updated",
            target: "Global Settings",
            time: "1 hour ago",
            user: "admin",
        },
        {
            action: "Alert triggered",
            target: "High CPU Usage",
            time: "3 hours ago",
            user: "System",
        },
    ];
</script>

<AdminLayout>
    <div class="max-w-7xl mx-auto p-8 md:p-12 animate-in fade-in duration-500">
        <!-- Header -->
        <div class="mb-12">
            <h1
                class="text-3xl font-bold text-zinc-900 dark:text-white tracking-tight mb-2"
            >
                Overview
            </h1>
            <p class="text-zinc-500 dark:text-zinc-400">
                Welcome back, here's what's happening today.
            </p>
        </div>

        <!-- Stats Grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 mb-16">
            {#each stats as stat}
                <div class="group">
                    <p
                        class="text-sm font-medium text-zinc-500 dark:text-zinc-400 mb-1"
                    >
                        {stat.label}
                    </p>
                    <div class="flex items-baseline gap-3">
                        <span
                            class="text-4xl font-bold text-zinc-900 dark:text-white tracking-tight"
                            >{stat.value}</span
                        >
                        <span
                            class="text-sm font-medium
                            {stat.trend === 'up'
                                ? 'text-emerald-600 dark:text-emerald-400'
                                : stat.trend === 'down'
                                  ? 'text-amber-600 dark:text-amber-400'
                                  : 'text-zinc-500'}"
                        >
                            {stat.change}
                        </span>
                    </div>
                </div>
            {/each}
        </div>

        <!-- Recent Activity -->
        <div>
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-lg font-semibold text-zinc-900 dark:text-white">
                    Recent Activity
                </h2>
                <button
                    class="text-sm font-medium text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors"
                    >View All</button
                >
            </div>

            <div class="border-t border-zinc-200 dark:border-zinc-800">
                {#each recentActivity as activity}
                    <div
                        class="py-4 flex items-center justify-between border-b border-zinc-200 dark:border-zinc-800 hover:bg-zinc-50 dark:hover:bg-zinc-800/30 transition-colors -mx-4 px-4 rounded-lg"
                    >
                        <div class="flex items-center gap-4">
                            <div
                                class="w-2 h-2 rounded-full bg-zinc-300 dark:bg-zinc-600"
                            ></div>
                            <div>
                                <p
                                    class="text-sm font-medium text-zinc-900 dark:text-white"
                                >
                                    {activity.action}
                                </p>
                                <p
                                    class="text-xs text-zinc-500 dark:text-zinc-400"
                                >
                                    on {activity.target}
                                </p>
                            </div>
                        </div>
                        <div class="text-right">
                            <p
                                class="text-xs font-medium text-zinc-900 dark:text-white"
                            >
                                {activity.user}
                            </p>
                            <p class="text-xs text-zinc-500 dark:text-zinc-400">
                                {activity.time}
                            </p>
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    </div>
</AdminLayout>
