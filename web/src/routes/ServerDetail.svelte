<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { pop } from "svelte-spa-router";
    import { api } from "../lib/api";
    import { WebSocketManager } from "../lib/websocket";
    import MetricsChart from "../lib/MetricsChart.svelte";
    import type { PublicNode, NodeMetrics, ServerMessage } from "../lib/types";

    let { params } = $props<{ params: { id: string } }>();

    // 当前节点的详细信息和指标
    let server = $state<PublicNode | null>(null);
    let latestMetrics = $state<NodeMetrics | null>(null);
    let loading = $state(true);
    let ws: WebSocketManager | null = null;

    // 图表模式：24h 或 realtime
    let chartMode = $state<"24h" | "realtime">("24h");

    // 图表数据
    let cpuChartData = $state<{ timestamp: number; value: number }[]>([]);
    let memChartData = $state<{ timestamp: number; value: number }[]>([]);

    // Realtime 模式的数据缓冲区（最多保留 60 个点，约 5 分钟）
    const MAX_REALTIME_POINTS = 60;

    // 加载节点详情
    async function loadNodeDetail() {
        try {
            loading = true;
            const nodeId = parseInt(params.id, 10);
            const detail = await api.nodes.get(nodeId);
            server = detail.node;
            latestMetrics = detail.latest_metrics || null;
        } catch (err) {
            console.error("Failed to load node detail:", err);
        } finally {
            loading = false;
        }
    }

    // 加载 24h 历史数据
    async function load24hData() {
        if (!server) return;

        try {
            const now = Math.floor(Date.now() / 1000);
            const start = now - 86400; // 24 小时前

            const metrics = await api.nodes.getMetrics(server.id, {
                start,
                end: now,
                limit: 24, // 采样 24 个点
            });

            cpuChartData = metrics.map((m) => ({
                timestamp: m.timestamp,
                value: m.cpu_usage,
            }));

            memChartData = metrics.map((m) => ({
                timestamp: m.timestamp,
                value: m.memory_usage,
            }));
        } catch (err) {
            console.error("Failed to load 24h data:", err);
        }
    }

    // 初始化 WebSocket
    async function initWebSocket() {
        const token = localStorage.getItem("token");
        if (!token || !server) {
            console.warn("No token or server found, skipping WebSocket connection");
            return;
        }

        try {
            ws = new WebSocketManager(token);
            await ws.connect();
            console.log("WebSocket connected");

            // 订阅该节点的更新
            ws.subscribe([server.id]);
            ws.addHandler(handleWsMessage);
        } catch (err) {
            console.error("Failed to connect WebSocket:", err);
        }
    }

    // 切换图表模式
    function switchMode(mode: "24h" | "realtime") {
        chartMode = mode;
        if (mode === "24h") {
            load24hData();
        } else {
            // 切换到 realtime 时，清空数据，等待 WebSocket 推送
            cpuChartData = [];
            memChartData = [];

            // 如果有最新指标，添加为第一个点
            if (latestMetrics) {
                cpuChartData = [
                    {
                        timestamp: latestMetrics.timestamp,
                        value: latestMetrics.cpu_usage,
                    },
                ];
                memChartData = [
                    {
                        timestamp: latestMetrics.timestamp,
                        value: latestMetrics.memory_usage,
                    },
                ];
            }
        }
    }

    // WebSocket 消息处理器
    function handleWsMessage(message: ServerMessage) {
        if (!server) return;

        if (
            message.type === "metrics_update" &&
            message.data.node_id === server.id
        ) {
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

            // Realtime 模式下更新图表
            if (chartMode === "realtime") {
                // 更新 CPU 数据
                cpuChartData = [
                    ...cpuChartData,
                    {
                        timestamp: message.data.timestamp,
                        value: message.data.cpu_usage,
                    },
                ].slice(-MAX_REALTIME_POINTS);

                // 更新内存数据
                memChartData = [
                    ...memChartData,
                    {
                        timestamp: message.data.timestamp,
                        value: message.data.memory_usage,
                    },
                ].slice(-MAX_REALTIME_POINTS);
            }
        }
    }

    onMount(async () => {
        await loadNodeDetail();
        // 只在初始加载时执行一次
        await load24hData();
        await initWebSocket();
    });

    onDestroy(() => {
        // 取消订阅
        if (ws && server) {
            ws.unsubscribe([server.id]);
            ws.removeHandler(handleWsMessage);
            ws.disconnect();
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

    function goBack() {
        pop();
    }
</script>

{#if loading}
    <div class="flex items-center justify-center min-h-[50vh]">
        <div class="text-sm text-zinc-500">Loading...</div>
    </div>
{:else if !server}
    <div class="flex items-center justify-center min-h-[50vh]">
        <div class="text-sm text-zinc-500">Server not found</div>
    </div>
{:else}
    <div class="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-300">
        <!-- Navigation -->
        <button
            onclick={goBack}
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
            <div
                class="flex flex-col md:flex-row justify-between items-start md:items-center gap-6"
            >
                <div class="flex items-center gap-6">
                    <div class="text-4xl filter drop-shadow-sm">🖥️</div>
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
                                {server.cpu_cores} Cores / {formatBytes(
                                    server.total_memory,
                                )}
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
                <div
                    class="grid grid-cols-1 md:grid-cols-3 gap-6 mt-8 pt-8 border-t border-black/5 dark:border-zinc-800"
                >
                    <!-- CPU -->
                    <div>
                        <div
                            class="text-xs text-zinc-500 uppercase tracking-wider mb-2"
                        >
                            CPU Usage
                        </div>
                        <div
                            class="text-2xl font-mono font-bold text-zinc-900 dark:text-white"
                        >
                            {latestMetrics.cpu_usage.toFixed(1)}%
                        </div>
                    </div>

                    <!-- Memory -->
                    <div>
                        <div
                            class="text-xs text-zinc-500 uppercase tracking-wider mb-2"
                        >
                            Memory
                        </div>
                        <div
                            class="text-2xl font-mono font-bold text-zinc-900 dark:text-white"
                        >
                            {latestMetrics.memory_usage.toFixed(1)}%
                        </div>
                        <div class="text-xs text-zinc-500 mt-1">
                            {formatBytes(latestMetrics.memory_used)} / {formatBytes(
                                latestMetrics.memory_total,
                            )}
                        </div>
                    </div>

                    <!-- Network -->
                    <div>
                        <div
                            class="text-xs text-zinc-500 uppercase tracking-wider mb-2"
                        >
                            Network
                        </div>
                        <div class="text-sm font-mono text-zinc-900 dark:text-white">
                            ↓ {formatBytes(latestMetrics.net_in_bytes)}/s
                        </div>
                        <div
                            class="text-sm font-mono text-zinc-900 dark:text-white mt-1"
                        >
                            ↑ {formatBytes(latestMetrics.net_out_bytes)}/s
                        </div>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Mode Toggle -->
        <div class="flex justify-center gap-2 mb-6">
            <button
                onclick={() => switchMode("24h")}
                class="px-4 py-2 text-xs font-medium rounded-lg transition-all duration-200 {chartMode ===
                '24h'
                    ? 'bg-zinc-900 dark:bg-white text-white dark:text-zinc-900'
                    : 'bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700'}"
            >
                24h History
            </button>
            <button
                onclick={() =>
                    server.status !== "offline" && switchMode("realtime")}
                disabled={server.status === "offline"}
                class="px-4 py-2 text-xs font-medium rounded-lg transition-all duration-200 {server.status ===
                'offline'
                    ? 'bg-zinc-100 dark:bg-zinc-800 text-zinc-400 dark:text-zinc-600 cursor-not-allowed opacity-50'
                    : chartMode === 'realtime'
                        ? 'bg-zinc-900 dark:bg-white text-white dark:text-zinc-900'
                        : 'bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700'}"
            >
                <span class="flex items-center gap-1.5">
                    {#if server.status !== "offline"}
                        <span class="relative flex h-2 w-2">
                            <span
                                class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"
                            ></span>
                            <span
                                class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"
                            ></span>
                        </span>
                    {/if}
                    {server.status === "offline" ? "Unavailable" : "Realtime"}
                </span>
            </button>
        </div>

        <!-- Charts Section -->
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <!-- CPU Chart -->
            <div
                class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-6 shadow-[0_2px_10px_rgba(0,0,0,0.02)]"
            >
                <div class="flex items-center justify-between mb-4">
                    <h3
                        class="text-sm font-semibold text-zinc-900 dark:text-white"
                    >
                        CPU Usage
                    </h3>
                    <span
                        class="text-[10px] font-medium text-zinc-500 uppercase tracking-wider"
                    >
                        {chartMode === "24h" ? "24h" : "Live"}
                    </span>
                </div>
                {#if cpuChartData.length > 0}
                    <MetricsChart
                        title="CPU Usage"
                        data={cpuChartData}
                        color="#3b82f6"
                    />
                {:else}
                    <div
                        class="h-[200px] flex items-center justify-center border border-dashed border-black/5 dark:border-zinc-800 rounded-lg bg-zinc-50/50 dark:bg-zinc-900/50"
                    >
                        <span class="text-xs text-zinc-400 font-medium"
                            >Loading...</span
                        >
                    </div>
                {/if}
            </div>

            <!-- Memory Chart -->
            <div
                class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-6 shadow-[0_2px_10px_rgba(0,0,0,0.02)]"
            >
                <div class="flex items-center justify-between mb-4">
                    <h3
                        class="text-sm font-semibold text-zinc-900 dark:text-white"
                    >
                        Memory Usage
                    </h3>
                    <span
                        class="text-[10px] font-medium text-zinc-500 uppercase tracking-wider"
                    >
                        {chartMode === "24h" ? "24h" : "Live"}
                    </span>
                </div>
                {#if memChartData.length > 0}
                    <MetricsChart
                        title="Memory Usage"
                        data={memChartData}
                        color="#10b981"
                    />
                {:else}
                    <div
                        class="h-[200px] flex items-center justify-center border border-dashed border-black/5 dark:border-zinc-800 rounded-lg bg-zinc-50/50 dark:bg-zinc-900/50"
                    >
                        <span class="text-xs text-zinc-400 font-medium"
                            >Loading...</span
                        >
                    </div>
                {/if}
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
                                <div
                                    class="flex justify-between text-xs font-medium mb-2"
                                >
                                    <span class="text-zinc-500"
                                        >{disk.mount}</span
                                    >
                                    <span
                                        class="text-zinc-900 dark:text-white font-mono"
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
                                    {formatBytes(disk.used)} / {formatBytes(
                                        disk.total,
                                    )}
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
                <h3
                    class="text-sm font-semibold text-zinc-900 dark:text-white"
                >
                    System Info
                </h3>
            </div>
            <div class="p-6">
                {#if latestMetrics}
                    <div class="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <div class="text-zinc-500 mb-1">Last Update</div>
                            <div
                                class="text-zinc-900 dark:text-white font-mono"
                            >
                                {formatTime(latestMetrics.timestamp)}
                            </div>
                        </div>
                        <div>
                            <div class="text-zinc-500 mb-1">Load Average</div>
                            <div
                                class="text-zinc-900 dark:text-white font-mono"
                            >
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
{/if}
