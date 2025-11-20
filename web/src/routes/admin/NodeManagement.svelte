<script lang="ts">
    import AdminLayout from "../../lib/admin/AdminLayout.svelte";

    // Mock data
    const nodes = [
        {
            id: "srv-01",
            name: "Production API",
            region: "us-east-1",
            status: "online",
            load: "45%",
            uptime: "14d",
        },
        {
            id: "srv-02",
            name: "Database Primary",
            region: "us-east-1",
            status: "online",
            load: "62%",
            uptime: "45d",
        },
        {
            id: "srv-03",
            name: "Cache Layer",
            region: "us-west-2",
            status: "online",
            load: "12%",
            uptime: "2d",
        },
        {
            id: "srv-04",
            name: "Worker Node A",
            region: "eu-central-1",
            status: "offline",
            load: "-",
            uptime: "-",
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
                    Nodes
                </h1>
                <p class="text-zinc-500 dark:text-zinc-400">
                    Manage your registered servers and agents.
                </p>
            </div>
            <button
                class="px-4 py-2 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
            >
                Add Node
            </button>
        </div>

        <div class="overflow-x-auto">
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="border-b border-zinc-200 dark:border-zinc-800">
                        <th
                            class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                            >Name</th
                        >
                        <th
                            class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                            >Region</th
                        >
                        <th
                            class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                            >Status</th
                        >
                        <th
                            class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                            >Load</th
                        >
                        <th
                            class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                            >Uptime</th
                        >
                        <th
                            class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider text-right"
                            >Actions</th
                        >
                    </tr>
                </thead>
                <tbody class="divide-y divide-zinc-100 dark:divide-zinc-800/50">
                    {#each nodes as node}
                        <tr
                            class="group hover:bg-zinc-50 dark:hover:bg-zinc-800/30 transition-colors"
                        >
                            <td class="py-4 px-4">
                                <div class="flex items-center gap-3">
                                    <div
                                        class="w-2 h-2 rounded-full {node.status ===
                                        'online'
                                            ? 'bg-emerald-500'
                                            : 'bg-zinc-300 dark:bg-zinc-600'}"
                                    ></div>
                                    <span
                                        class="text-sm font-medium text-zinc-900 dark:text-white"
                                        >{node.name}</span
                                    >
                                </div>
                                <div class="text-xs text-zinc-500 pl-5">
                                    {node.id}
                                </div>
                            </td>
                            <td
                                class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400"
                                >{node.region}</td
                            >
                            <td class="py-4 px-4">
                                <span
                                    class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium
                                    {node.status === 'online'
                                        ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400'
                                        : 'bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400'}"
                                >
                                    {node.status}
                                </span>
                            </td>
                            <td
                                class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400 font-mono"
                                >{node.load}</td
                            >
                            <td
                                class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400"
                                >{node.uptime}</td
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
