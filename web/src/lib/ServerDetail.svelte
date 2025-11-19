<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { api } from "./api";
    import type { PublicNode, NodeMetrics, ServerMessage } from "./types";

    let { server, onBack, ws } = $props<{
        server: PublicNode;
        onBack: () => void;
        ws: any;
    }>();

    // 当前节点的详细信息和指标
    let latestMetrics = $state<NodeMetrics | null>(null);
    let loading = $state(true);

    // 加载节点详情
    async function loadNodeDetail() {
        try {
            loading = true;
            const detail = await api.nodes.get(server.id);
            latestMetrics = detail.latest_metrics || null;
        } catch (err) {
            console.error("Failed to load node detail:", err);
        } finally {
            loading = false;
        }
    }

    // WebSocket 消息处理器
    function handleWsMessage(message: ServerMessage) {
        if (message.type === "metrics_update" && message.data.node_id === server.id) {
            // 更新最新指标
            latestMetrics = {
                timestamp: message.data.timestamp,
                cpu_usage: message.data.cpu_usage,
                memory_used: message.data.memory_used,
                memory_total: message.data.memory_total,
                memory_usage: message.data.memory_usage,
                disk_info: message.data.disk_info,
                net_in_bytes: message.data.network_in,
                net_out_bytes: message.data.network_out,
                load_1: message.data.load_1,
                load_5: message.data.load_5,
                load_15: message.data.load_15,
            };
        }
    }

    onMount(() => {
        loadNodeDetail();

        // 订阅该节点的更新
        if (ws) {
            ws.subscribe([server.id]);
            ws.addHandler(handleWsMessage);
        }
    });

    onDestroy(() => {
        // 取消订阅
        if (ws) {
            ws.unsubscribe([server.id]);
            ws.removeHandler(handleWsMessage);
        }
    });

    // 格式化字节数
    function formatBytes(bytes: number): string {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
    }

    // 格式化时间戳
    function formatTime(timestamp: number): string {
        return new Date(timestamp * 1000).toLocaleString();
    }
</script>

<div class="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-300">
    <!-- Navigation -->
    <button
        onclick={onBack}
        class="flex items-center gap-2 text-sm font-medium text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors"
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
            stroke-linejoin="round"><path d="m15 18-6-6 6-6" /></svg
        >
        Back to Dashboard
    </button>

    <!-- Header Card -->
    <div
        class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-8 shadow-[0_8px_30px_rgba(0,0,0,0.04)] dark:shadow-none"
    >
        {#if loading}
            <div class="flex items-center justify-center py-8">
                <div class="text-sm text-zinc-500">Loading...</div>
            </div>
        {:else}
            <div
                class="flex flex-col md:flex-row justify-between items-start md:items-center gap-6"
            >
                <div class="flex items-center gap-6">
                    <div class="text-4xl filter drop-shadow-sm">
                        🖥️
                    </div>
                    <div>
                        <h1
                            class="text-2xl font-bold text-zinc-900 dark:text-white tracking-tight"
                        >
                            {server.name}
                        </h1>
                        <div
                            class="flex flex-wrap items-center gap-4 text-sm text-zinc-500 mt-1"
                        >
                            <span class="font-medium">{server.os_type}</span>
                            <span
                                class="w-1 h-1 rounded-full bg-zinc-300 dark:bg-zinc-700"
                            ></span>
                            <span
                                class="font-mono bg-zinc-50 dark:bg-zinc-900 px-1.5 py-0.5 rounded border border-zinc-100 dark:border-zinc-800"
                            >
                                {server.cpu_cores} Cores / {formatBytes(server.total_memory)}
                            </span>
                        </div>
                    </div>
                </div>

                <div
                    class="flex gap-12 w-full md:w-auto pt-6 md:pt-0 border-t md:border-0 border-black/5 dark:border-zinc-800"
                >
                    <div class="text-left md:text-right">
                        <div
                            class="text-[10px] font-medium text-zinc-500 uppercase tracking-wider mb-1"
                        >
                            Status
                        </div>
                        <div
                            class="text-xl font-mono font-semibold text-zinc-900 dark:text-white"
                        >
                            {server.status}
                        </div>
                    </div>
                    <div class="text-left md:text-right">
                        <div
                            class="text-[10px] font-medium text-zinc-500 uppercase tracking-wider mb-1"
                        >
                            Load Avg
                        </div>
                        <div
                            class="text-xl font-mono font-semibold text-zinc-900 dark:text-white"
                        >
                            {latestMetrics?.load_1?.toFixed(2) || "N/A"}
                        </div>
                    </div>
                </div>
            </div>

            <!-- Real-time Metrics -->
            {#if latestMetrics}
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mt-8 pt-8 border-t border-black/5 dark:border-zinc-800">
                    <!-- CPU -->
                    <div>
                        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">CPU Usage</div>
                        <div class="text-2xl font-mono font-bold text-zinc-900 dark:text-white">
                            {latestMetrics.cpu_usage.toFixed(1)}%
                        </div>
                    </div>

                    <!-- Memory -->
                    <div>
                        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Memory</div>
                        <div class="text-2xl font-mono font-bold text-zinc-900 dark:text-white">
                            {latestMetrics.memory_usage.toFixed(1)}%
                        </div>
                        <div class="text-xs text-zinc-500 mt-1">
                            {formatBytes(latestMetrics.memory_used)} / {formatBytes(latestMetrics.memory_total)}
                        </div>
                    </div>

                    <!-- Network -->
                    <div>
                        <div class="text-xs text-zinc-500 uppercase tracking-wider mb-2">Network</div>
                        <div class="text-sm font-mono text-zinc-900 dark:text-white">
                            ↓ {formatBytes(latestMetrics.net_in_bytes)}/s
                        </div>
                        <div class="text-sm font-mono text-zinc-900 dark:text-white mt-1">
                            ↑ {formatBytes(latestMetrics.net_out_bytes)}/s
                        </div>
                    </div>
                </div>
            {/if}
        {/if}
    </div>

    <!-- Charts Section -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- CPU History (Placeholder for now) -->
        <div
            class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-6 shadow-[0_2px_10px_rgba(0,0,0,0.02)]"
        >
            <div class="flex items-center justify-between mb-6">
                <h3 class="text-sm font-semibold text-zinc-900 dark:text-white">
                    CPU Usage
                </h3>
                <span
                    class="text-[10px] font-medium text-zinc-500 uppercase tracking-wider"
                    >24h History</span
                >
            </div>
            <div
                class="h-40 flex items-center justify-center border border-dashed border-black/5 dark:border-zinc-800 rounded-lg bg-zinc-50/50 dark:bg-zinc-900/50"
            >
                <span class="text-xs text-zinc-400 font-medium"
                    >Chart Coming Soon</span
                >
            </div>
        </div>

        <!-- Network History (Placeholder) -->
        <div
            class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-6 shadow-[0_2px_10px_rgba(0,0,0,0.02)]"
        >
            <div class="flex items-center justify-between mb-6">
                <h3 class="text-sm font-semibold text-zinc-900 dark:text-white">
                    Network Traffic
                </h3>
                <span
                    class="text-[10px] font-medium text-zinc-500 uppercase tracking-wider"
                    >Total</span
                >
            </div>
            <div
                class="h-40 flex items-center justify-center border border-dashed border-black/5 dark:border-zinc-800 rounded-lg bg-zinc-50/50 dark:bg-zinc-900/50"
            >
                <span class="text-xs text-zinc-400 font-medium"
                    >Chart Coming Soon</span
                >
            </div>
        </div>

        <!-- Disk Info -->
        <div
            class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-6 shadow-[0_2px_10px_rgba(0,0,0,0.02)]"
        >
            <h3
                class="text-sm font-semibold text-zinc-900 dark:text-white mb-6"
            >
                Storage
            </h3>
            {#if latestMetrics && latestMetrics.disk_info.length > 0}
                <div class="space-y-4">
                    {#each latestMetrics.disk_info as disk}
                        <div>
                            <div class="flex justify-between text-xs font-medium mb-2">
                                <span class="text-zinc-500">{disk.mount}</span>
                                <span class="text-zinc-900 dark:text-white font-mono"
                                    >{disk.usage.toFixed(1)}%</span
                                >
                            </div>
                            <div
                                class="w-full bg-zinc-100 dark:bg-zinc-800 rounded-full h-2 overflow-hidden"
                            >
                                <div
                                    class="bg-zinc-900 dark:bg-white h-full rounded-full opacity-80 transition-all duration-500"
                                    style="width: {disk.usage}%"
                                ></div>
                            </div>
                            <div class="text-xs text-zinc-400 mt-1">
                                {formatBytes(disk.used)} / {formatBytes(disk.total)}
                            </div>
                        </div>
                    {/each}
                </div>
            {:else}
                <div class="text-sm text-zinc-500 text-center py-8">
                    No disk information available
                </div>
            {/if}
        </div>
    </div>

    <!-- Process List (Placeholder for now) -->
    <div
        class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl overflow-hidden shadow-[0_2px_10px_rgba(0,0,0,0.02)]"
    >
        <div
            class="px-6 py-4 border-b border-black/5 dark:border-zinc-800 flex justify-between items-center"
        >
            <h3 class="text-sm font-semibold text-zinc-900 dark:text-white">
                System Info
            </h3>
        </div>
        <div class="p-6">
            {#if latestMetrics}
                <div class="grid grid-cols-2 gap-4 text-sm">
                    <div>
                        <div class="text-zinc-500 mb-1">Last Update</div>
                        <div class="text-zinc-900 dark:text-white font-mono">
                            {formatTime(latestMetrics.timestamp)}
                        </div>
                    </div>
                    <div>
                        <div class="text-zinc-500 mb-1">Load Average</div>
                        <div class="text-zinc-900 dark:text-white font-mono">
                            {latestMetrics.load_1?.toFixed(2) || "N/A"} /
                            {latestMetrics.load_5?.toFixed(2) || "N/A"} /
                            {latestMetrics.load_15?.toFixed(2) || "N/A"}
                        </div>
                    </div>
                </div>
            {:else}
                <div class="text-sm text-zinc-500 text-center py-8">
                    No metrics available
                </div>
            {/if}
        </div>
    </div>
</div>
