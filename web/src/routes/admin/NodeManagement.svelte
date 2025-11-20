<script lang="ts">
    import { onMount } from "svelte";
    import AdminLayout from "../../lib/admin/AdminLayout.svelte";
    import NodeModal from "../../lib/admin/NodeModal.svelte";
    import { api } from "../../lib/api";
    import type { AdminNode } from "../../lib/types";

    let nodes = $state<AdminNode[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    // Modal state
    let showModal = $state(false);
    let editingNode = $state<AdminNode | null>(null);

    // Get time since last seen
    function getTimeSince(timestamp: number): string {
        const now = Math.floor(Date.now() / 1000);
        const diff = now - timestamp;
        if (diff < 60) return `${diff}s ago`;
        if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
        if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
        return `${Math.floor(diff / 86400)}d ago`;
    }

    // Format memory
    function formatMemory(bytes: number): string {
        const gb = bytes / (1024 * 1024 * 1024);
        return `${gb.toFixed(1)} GB`;
    }

    async function loadNodes() {
        try {
            if (nodes.length === 0) loading = true;
            error = null;
            nodes = await api.nodes.adminList(100);
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load nodes";
            console.error("Failed to load nodes:", e);
        } finally {
            loading = false;
        }
    }

    function handleEdit(node: AdminNode) {
        editingNode = node;
        showModal = true;
    }

    async function handleDelete(node: AdminNode) {
        if (
            !confirm(
                `Are you sure you want to delete node "${node.name}"? This will also delete all related metrics and data. This action cannot be undone.`,
            )
        ) {
            return;
        }

        try {
            await api.nodes.adminDelete(node.id);
            await loadNodes();
        } catch (e) {
            console.error("Failed to delete node:", e);
            alert("Failed to delete node");
        }
    }

    function handleSave() {
        loadNodes();
    }

    onMount(() => {
        loadNodes();
        // Refresh every 60 seconds
        const interval = setInterval(loadNodes, 60000);
        return () => clearInterval(interval);
    });
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
        </div>

        {#if loading && nodes.length === 0}
            <div class="flex justify-center items-center py-12">
                <div class="text-zinc-500">Loading nodes...</div>
            </div>
        {:else if error}
            <div class="flex justify-center items-center py-12">
                <div class="text-red-500">Error: {error}</div>
            </div>
        {:else if nodes.length === 0}
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
                        ><rect x="2" y="2" width="20" height="8" rx="2" ry="2"
                        ></rect><rect
                            x="2"
                            y="14"
                            width="20"
                            height="8"
                            rx="2"
                            ry="2"
                        ></rect><line x1="6" y1="6" x2="6.01" y2="6"
                        ></line><line x1="6" y1="18" x2="6.01" y2="18"
                        ></line></svg
                    >
                </div>
                <h3
                    class="text-lg font-medium text-zinc-900 dark:text-white mb-1"
                >
                    No nodes registered
                </h3>
                <p class="text-zinc-500 dark:text-zinc-400">
                    Install and configure an agent to get started.
                </p>
            </div>
        {:else}
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="border-b border-zinc-200 dark:border-zinc-800">
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Node</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Status</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >IP Address</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >System</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Last Seen</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider text-right"
                                >Actions</th
                            >
                        </tr>
                    </thead>
                    <tbody
                        class="divide-y divide-zinc-100 dark:divide-zinc-800/50"
                    >
                        {#each nodes as node}
                            <tr
                                class="group hover:bg-zinc-50 dark:hover:bg-zinc-800/30 transition-colors"
                            >
                                <td class="py-4 px-4">
                                    <div class="flex items-center gap-3">
                                        <div
                                            class="w-2 h-2 rounded-full {node.status ===
                                            'online'
                                                ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]'
                                                : node.status === 'warning'
                                                  ? 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.4)]'
                                                  : 'bg-zinc-300 dark:bg-zinc-600'}"
                                        ></div>
                                        <div>
                                            <div
                                                class="text-sm font-medium text-zinc-900 dark:text-white"
                                            >
                                                {node.name}
                                            </div>
                                            <div class="text-[10px] text-zinc-500 font-mono">
                                                {node.uuid.slice(0, 8)}
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td class="py-4 px-4">
                                    <span
                                        class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium
                                        {node.status === 'online'
                                            ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400'
                                            : node.status === 'warning'
                                              ? 'bg-amber-100 text-amber-700 dark:bg-amber-500/10 dark:text-amber-400'
                                              : 'bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400'}"
                                    >
                                        {node.status}
                                    </span>
                                </td>
                                <td
                                    class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400 font-mono"
                                    >{node.ip_address}</td
                                >
                                <td class="py-4 px-4">
                                    <div class="text-sm text-zinc-900 dark:text-white">
                                        {node.os_type}
                                    </div>
                                    <div class="text-xs text-zinc-500">
                                        {node.cpu_cores} cores • {formatMemory(
                                            node.total_memory,
                                        )}
                                    </div>
                                </td>
                                <td
                                    class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400"
                                    >{getTimeSince(node.last_seen)}</td
                                >
                                <td class="py-4 px-4 text-right">
                                    <div class="flex items-center justify-end gap-2">
                                        <button
                                            on:click={() => handleEdit(node)}
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
                                            on:click={() => handleDelete(node)}
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

    <NodeModal
        bind:show={showModal}
        bind:node={editingNode}
        on:save={handleSave}
        on:close={() => (showModal = false)}
    />
</AdminLayout>
