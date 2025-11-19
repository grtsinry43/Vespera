<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import ThemeToggle from "./lib/ThemeToggle.svelte";
    import ServerCard from "./lib/ServerCard.svelte";
    import ServiceCard from "./lib/ServiceCard.svelte";
    import ServerDetail from "./lib/ServerDetail.svelte";
    import StatusOverview from "./lib/StatusOverview.svelte";
    import { api } from "./lib/api";
    import { WebSocketManager } from "./lib/websocket";
    import type { PublicNode, ServerMessage } from "./lib/types";

    // State
    let currentView = $state("dashboard"); // 'dashboard' | 'detail'
    let selectedServer = $state<PublicNode | null>(null);
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
        traffic_in: "0",  // 从实际指标计算
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
                console.log("[App] Received WS message:", message.type);

                if (message.type === "metrics_update") {
                    console.log("[App] Metrics update for node:", message.data.node_id, message.data);
                    // 更新对应节点的信息
                    const nodeId = message.data.node_id;
                    servers = servers.map((s) => {
                        if (s.id === nodeId) {
                            return {
                                ...s,
                                status: message.data.cpu_usage > 80 || message.data.memory_usage > 90 ? "warning" : "online",
                                cpu_usage: message.data.cpu_usage,
                                memory_usage: message.data.memory_usage,
                                // 累计流量，转换为 GB
                                net_in: message.data.network_in / (1024 * 1024 * 1024),
                                net_out: message.data.network_out / (1024 * 1024 * 1024),
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

    function selectServer(server) {
        selectedServer = server;
        currentView = "detail";
        window.scrollTo(0, 0);
    }

    function goBack() {
        selectedServer = null;
        currentView = "dashboard";
    }
</script>

<div
    class="min-h-screen pb-20 relative overflow-x-hidden bg-[#fafafa] dark:bg-[#09090b] text-zinc-900 dark:text-zinc-300 transition-colors duration-300"
>
    <!-- Subtle Ambient Background -->
    <div class="fixed inset-0 pointer-events-none">
        <!-- Light Mode Glow -->
        <div
            class="absolute top-[-10%] right-[-5%] w-[40%] h-[40%] bg-indigo-200/30 blur-[100px] rounded-full dark:hidden"
        ></div>
        <div
            class="absolute bottom-[-10%] left-[-5%] w-[40%] h-[40%] bg-emerald-200/30 blur-[100px] rounded-full dark:hidden"
        ></div>

        <!-- Dark Mode Glow -->
        <div
            class="absolute top-[-20%] left-[-10%] w-[50%] h-[50%] bg-indigo-900/10 blur-[120px] rounded-full hidden dark:block"
        ></div>
        <div
            class="absolute bottom-[-20%] right-[-10%] w-[50%] h-[50%] bg-emerald-900/10 blur-[120px] rounded-full hidden dark:block"
        ></div>
    </div>

    <!-- Navbar -->
    <nav
        class="sticky top-0 z-30 border-b border-black/5 dark:border-zinc-800 bg-white/80 dark:bg-[#09090b]/80 backdrop-blur-2xl supports-[backdrop-filter]:bg-white/60"
    >
        <div
            class="max-w-7xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between"
        >
            <div
                class="flex items-center gap-3 cursor-pointer group"
                onclick={goBack}
            >
                <!-- Logo: Just a simple dot -->
                <div
                    class="w-1.5 h-1.5 rounded-full bg-zinc-900 dark:bg-white shadow-[0_0_10px_rgba(0,0,0,0.1)] group-hover:scale-125 transition-transform duration-300"
                ></div>
                <span
                    class="text-sm font-bold tracking-tight text-zinc-900 dark:text-white"
                >
                    Vespera <span
                        class="text-zinc-400 dark:text-zinc-500 font-normal ml-1"
                        >by grtsinry43</span
                    >
                </span>
            </div>

            <div class="flex items-center gap-4 sm:gap-6">
                <!-- Desktop Stats -->
                <div
                    class="hidden md:flex items-center gap-8 text-xs font-medium text-zinc-500"
                >
                    <div class="flex items-center gap-2">
                        <span class="relative flex h-2 w-2">
                            <span
                                class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"
                            ></span>
                            <span
                                class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"
                            ></span>
                        </span>
                        <span class="text-zinc-700 dark:text-zinc-200"
                            >{globalStats.active} / {globalStats.total}</span
                        >
                        <span class="text-zinc-400">Online</span>
                    </div>
                    <div class="h-4 w-px bg-zinc-200 dark:bg-zinc-800"></div>
                    <div class="flex items-center gap-4">
                        <div class="flex items-center gap-1.5">
                            <span
                                class="text-zinc-700 dark:text-zinc-200 font-mono"
                                >{globalStats.traffic_in}</span
                            >
                            <span class="text-[10px] uppercase tracking-wider"
                                >Mbps In</span
                            >
                        </div>
                        <div class="flex items-center gap-1.5">
                            <span
                                class="text-zinc-700 dark:text-zinc-200 font-mono"
                                >{globalStats.traffic_out}</span
                            >
                            <span class="text-[10px] uppercase tracking-wider"
                                >Mbps Out</span
                            >
                        </div>
                    </div>
                </div>

                <div
                    class="h-4 w-px bg-zinc-200 dark:bg-zinc-800 hidden md:block"
                ></div>

                <ThemeToggle />
            </div>
        </div>
    </nav>

    <!-- Main Content -->
    <main class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 py-8 sm:py-12">
        {#if currentView === "dashboard"}
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
                    <div
                        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4"
                    >
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
                    <div
                        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6"
                    >
                        {#each servers as server}
                            <ServerCard
                                {server}
                                onclick={() => selectServer(server)}
                            />
                        {/each}
                    </div>
                </section>
            </div>
        {:else if currentView === "detail" && selectedServer}
            <ServerDetail server={selectedServer} onBack={goBack} {ws} />
        {/if}
    </main>
</div>

<style>
    /* Custom scrollbar for webkit */
    :global(::-webkit-scrollbar) {
        width: 6px;
    }

    :global(::-webkit-scrollbar-track) {
        background: transparent;
    }

    :global(::-webkit-scrollbar-thumb) {
        background: #d4d4d8;
        border-radius: 3px;
    }

    :global(.dark ::-webkit-scrollbar-thumb) {
        background: #27272a;
    }

    :global(::-webkit-scrollbar-thumb:hover) {
        background: #a1a1aa;
    }

    :global(.dark ::-webkit-scrollbar-thumb:hover) {
        background: #3f3f46;
    }
</style>
