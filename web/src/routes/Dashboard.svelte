<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import ServerCard from "../lib/ServerCard.svelte";
    import ServiceCard from "../lib/ServiceCard.svelte";
    import StatusOverview from "../lib/StatusOverview.svelte";
    import { api } from "../lib/api";
    import { WebSocketManager } from "../lib/websocket";
    import type { PublicNode, ServerMessage } from "../lib/types";

    // State
    let loading = $state(true);
    let error = $state<string | null>(null);
    let ws: WebSocketManager | null = null;

    // Real data for servers
    let servers = $state<PublicNode[]>([]);

    // Mock data for services (保留 mock，因为后端暂时没有 service API)
    let services = $state([
        {
            id: 1,
            name: "Main Website",
            url: "https://vespera.io",
            type: "http",
            status: "up",
            uptime: 99.99,
            latency: 45,
        },
        {
            id: 2,
            name: "API Gateway",
            url: "https://api.vespera.io",
            type: "http",
            status: "up",
            uptime: 99.95,
            latency: 120,
        },
        {
            id: 3,
            name: "Database Cluster",
            url: "tcp://db.vespera.io:5432",
            type: "tcp",
            status: "up",
            uptime: 100,
            latency: 2,
        },
        {
            id: 4,
            name: "Redis Cache",
            url: "tcp://redis.vespera.io:6379",
            type: "tcp",
            status: "degraded",
            uptime: 98.5,
            latency: 15,
        },
        {
            id: 5,
            name: "Auth Service",
            url: "https://auth.vespera.io",
            type: "http",
            status: "down",
            uptime: 95.2,
            latency: 0,
        },
    ]);

    // Derived global stats
    let globalStats = $derived({
        active: servers.filter((s) => s.status !== "offline").length,
        total: servers.length,
        traffic_in: "0", // 从实际指标计算
        traffic_out: "0", // 从实际指标计算
    });

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

    // 初始化 WebSocket
    async function initWebSocket() {
        const token = localStorage.getItem("token");
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
        initWebSocket();
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
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each services as service}
                <ServiceCard {service} />
            {/each}
        </div>
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
</div>
