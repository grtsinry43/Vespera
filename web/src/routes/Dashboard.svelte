<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import ServerCard from "../lib/ServerCard.svelte";
    import ServiceCard from "../lib/ServiceCard.svelte";
    import StatusOverview from "../lib/StatusOverview.svelte";
    import { api } from "../lib/api";
    import { authStorage } from "../lib/authStorage";
    import { WebSocketManager } from "../lib/websocket";
    import { wsStore } from "../lib/wsStore";
    import type { PublicNode, ServerMessage, ServiceStatusOverview, HealthCheckData } from "../lib/types";

    // State
    let loading = $state(true);
    let error = $state<string | null>(null);
    let ws: WebSocketManager | null = null;
    let healthData = $state<HealthCheckData | null>(null);

    // Real data for servers
    let servers = $state<PublicNode[]>([]);

    // Real data for services (替换 mock 数据)
    let services = $state<ServiceStatusOverview[]>([]);

    // Derived global stats
    let globalStats = $derived({
        active: servers.filter((s) => s.status !== "offline").length,
        total: servers.length,
        traffic_in: "0", // 从实际指标计算
        traffic_out: "0", // 从实际指标计算
    });

    // Format uptime
    function formatUptime(secs: number): string {
        const days = Math.floor(secs / 86400);
        const hours = Math.floor((secs % 86400) / 3600);
        const minutes = Math.floor((secs % 3600) / 60);

        if (days > 0) {
            return `${days}d ${hours}h ${minutes}m`;
        } else if (hours > 0) {
            return `${hours}h ${minutes}m`;
        } else {
            return `${minutes}m`;
        }
    }

    // 加载节点列表
    async function loadNodes() {
        try {
            loading = true;
            error = null;
            servers = await api.nodes.list(100);
        } catch (err: any) {
            error = err.message || "Failed to load nodes";
            console.error("Failed to load nodes:", err);
        } finally {
            loading = false;
        }
    }

    // 加载服务列表
    async function loadServices() {
        try {
            services = await api.services.getAllOverviews();
        } catch (err: any) {
            console.error("Failed to load services:", err);
            // 不设置全局 error，让服务加载失败不影响整体页面
        }
    }

    // 加载健康状态
    async function loadHealth() {
        try {
            healthData = await api.system.health();
        } catch (err: any) {
            console.error("Failed to load health:", err);
        }
    }

    // 初始化 WebSocket
    async function initWebSocket() {
        const token = authStorage.getAccessToken();
        if (!token) {
            console.warn("No token found, skipping WebSocket connection");
            return;
        }

        try {
            ws = new WebSocketManager(token);
            await ws.connect();
            console.log("WebSocket connected");

            // 监听指标更新
            ws.addHandler((message: ServerMessage) => {
                if (message.type === "metrics_update") {
                    // 更新对应节点的信息
                    const nodeId = message.data.node_id;
                    servers = servers.map((s) => {
                        if (s.id === nodeId) {
                            return {
                                ...s,
                                status:
                                    message.data.cpu_usage > 80 ||
                                    message.data.memory_usage > 90
                                        ? "warning"
                                        : "online",
                                cpu_usage: message.data.cpu_usage,
                                memory_usage: message.data.memory_usage,
                                // 累计流量，转换为 GB
                                net_in:
                                    message.data.network_in /
                                    (1024 * 1024 * 1024),
                                net_out:
                                    message.data.network_out /
                                    (1024 * 1024 * 1024),
                            };
                        }
                        return s;
                    });
                } else if (message.type === "node_online") {
                    servers = servers.map((s) => {
                        if (s.id === message.node_id) {
                            return { ...s, status: "online" };
                        }
                        return s;
                    });
                } else if (message.type === "node_offline") {
                    servers = servers.map((s) => {
                        if (s.id === message.node_id) {
                            return { ...s, status: "offline" };
                        }
                        return s;
                    });
                }
            });
        } catch (err) {
            console.error("Failed to connect WebSocket:", err);
        }
    }

    onMount(() => {
        loadNodes();
        loadServices();
        loadHealth();
        initWebSocket();

        // 每 60 秒刷新一次服务状态和健康状态
        const serviceInterval = setInterval(loadServices, 60000);
        const healthInterval = setInterval(loadHealth, 60000);

        return () => {
            clearInterval(serviceInterval);
            clearInterval(healthInterval);
        };
    });

    onDestroy(() => {
        if (ws) {
            ws.disconnect();
        }
    });
</script>

<div class="space-y-12 animate-in fade-in duration-500">
    <!-- Status Overview -->
    <StatusOverview {globalStats} />

    <!-- Services Section -->
    <section>
        <div class="flex items-center justify-between mb-6">
            <h2
                class="text-sm font-semibold text-zinc-900 dark:text-white uppercase tracking-wider flex items-center gap-2"
            >
                Services
                <span
                    class="px-2 py-0.5 rounded-full bg-zinc-100 dark:bg-zinc-800 text-[10px] text-zinc-500"
                    >{services.length}</span
                >
            </h2>
        </div>
        {#if services.length === 0}
            <div class="text-center py-8 text-zinc-500 dark:text-zinc-400">
                No services configured yet.
            </div>
        {:else}
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {#each services as overview}
                    <ServiceCard {overview} />
                {/each}
            </div>
        {/if}
    </section>

    <!-- Servers Section -->
    <section>
        <div class="flex items-center justify-between mb-6">
            <h2
                class="text-sm font-semibold text-zinc-900 dark:text-white uppercase tracking-wider flex items-center gap-2"
            >
                Infrastructure
                <span
                    class="px-2 py-0.5 rounded-full bg-zinc-100 dark:bg-zinc-800 text-[10px] text-zinc-500"
                    >{servers.length}</span
                >
            </h2>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
            {#each servers as server}
                <ServerCard {server} />
            {/each}
        </div>
    </section>

    <!-- Footer -->
    <footer class="mt-16 pt-6 border-t border-zinc-200 dark:border-zinc-800">
        <div class="flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-zinc-500 dark:text-zinc-400">
            <div class="flex items-center gap-4">
                <span class="font-mono">Vespera LightMonitor v{healthData?.version ?? '--'}</span>
                {#if healthData}
                    <span class="hidden sm:inline text-zinc-300 dark:text-zinc-700">|</span>
                    <span class="hidden sm:inline">
                        Uptime: <span class="font-mono text-zinc-700 dark:text-zinc-300">{formatUptime(healthData.uptime_secs)}</span>
                    </span>
                {/if}
            </div>
            <div class="flex items-center gap-2">
                <span class="text-zinc-400 dark:text-zinc-500">WebSocket:</span>
                {#if $wsStore.status === 'connected'}
                    <span class="inline-flex items-center gap-1.5">
                        <span class="relative flex h-2 w-2">
                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                            <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                        </span>
                        <span class="text-emerald-600 dark:text-emerald-400">Connected</span>
                    </span>
                {:else if $wsStore.status === 'connecting'}
                    <span class="inline-flex items-center gap-1.5">
                        <span class="h-2 w-2 rounded-full bg-amber-500 animate-pulse"></span>
                        <span class="text-amber-600 dark:text-amber-400">Connecting...</span>
                    </span>
                {:else}
                    <span class="inline-flex items-center gap-1.5">
                        <span class="h-2 w-2 rounded-full bg-zinc-400"></span>
                        <span class="text-zinc-500 dark:text-zinc-400">Disconnected</span>
                    </span>
                {/if}
            </div>
        </div>
    </footer>
</div>
