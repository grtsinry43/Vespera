<script lang="ts">
    import type { PublicNode } from "./types";

    let { server, onclick } = $props<{
        server: PublicNode;
        onclick?: () => void;
    }>();

    const getStatusColor = (status: string) => {
        switch (status) {
            case "online":
                return "bg-emerald-500";
            case "warning":
                return "bg-amber-500";
            case "offline":
                return "bg-rose-500";
            default:
                return "bg-zinc-400";
        }
    };

    const getBarColor = (val: number) => {
        if (val > 80) return "bg-rose-500";
        if (val > 50) return "bg-amber-500";
        return "bg-emerald-500";
    };

    // Format bytes to GB
    function formatMemory(bytes: number): string {
        const gb = bytes / (1024 * 1024 * 1024);
        return gb.toFixed(1);
    }

    // Get time since last seen
    function getTimeSince(timestamp: number): string {
        const now = Math.floor(Date.now() / 1000);
        const diff = now - timestamp;
        if (diff < 60) return `${diff}s ago`;
        if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
        if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
        return `${Math.floor(diff / 86400)}d ago`;
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    {onclick}
    class="group relative overflow-hidden bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-2xl p-6 hover:border-zinc-300/50 dark:hover:border-zinc-700 transition-all duration-300 shadow-[0_2px_10px_rgba(0,0,0,0.02)] hover:shadow-[0_8px_30px_rgba(0,0,0,0.06)] cursor-pointer hover:-translate-y-0.5"
>
    <!-- Header -->
    <div class="flex justify-between items-start mb-4">
        <div class="flex items-center gap-4">
            <div class="relative flex items-center justify-center w-2.5 h-2.5">
                <div
                    class="w-2.5 h-2.5 rounded-full {getStatusColor(
                        server.status,
                    )} shadow-sm"
                ></div>
            </div>

            <div>
                <div class="flex items-center gap-2">
                    <h3
                        class="text-sm font-semibold text-zinc-900 dark:text-white tracking-tight"
                    >
                        {server.name}
                    </h3>
                </div>
                <p
                    class="text-[10px] font-medium uppercase tracking-wider text-zinc-500 mt-0.5"
                >
                    {server.os_type}
                </p>
            </div>
        </div>

        <div class="text-right">
            <div
                class="text-[11px] font-mono font-medium text-zinc-500 bg-zinc-50 dark:bg-zinc-900 px-1.5 py-0.5 rounded border border-zinc-100 dark:border-zinc-800"
            >
                {getTimeSince(server.last_seen)}
            </div>
        </div>
    </div>

    <!-- Metrics -->
    {#if server.status !== "offline"}
        <div class="space-y-5">
            <!-- CPU Usage -->
            {#if server.cpu_usage !== undefined}
                <div class="space-y-2">
                    <div
                        class="flex justify-between text-[10px] font-medium uppercase tracking-wider text-zinc-500"
                    >
                        <span>CPU</span>
                        <span
                            class="text-zinc-900 dark:text-zinc-200 font-mono"
                        >
                            {server.cpu_usage.toFixed(1)}%
                        </span>
                    </div>
                    <div
                        class="h-1.5 w-full bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full {getBarColor(
                                server.cpu_usage,
                            )} transition-all duration-500 ease-out"
                            style="width: {server.cpu_usage}%"
                        ></div>
                    </div>
                </div>
            {/if}

            <!-- Memory Usage -->
            {#if server.memory_usage !== undefined}
                <div class="space-y-2">
                    <div
                        class="flex justify-between text-[10px] font-medium uppercase tracking-wider text-zinc-500"
                    >
                        <span>MEM</span>
                        <span
                            class="text-zinc-900 dark:text-zinc-200 font-mono"
                        >
                            {server.memory_usage.toFixed(1)}%
                        </span>
                    </div>
                    <div
                        class="h-1.5 w-full bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full {getBarColor(
                                server.memory_usage,
                            )} transition-all duration-500 ease-out"
                            style="width: {server.memory_usage}%"
                        ></div>
                    </div>
                </div>
            {/if}

            <!-- Network (if available) -->
            {#if server.net_in !== undefined && server.net_out !== undefined}
                <div
                    class="pt-4 border-t border-zinc-100 dark:border-zinc-800 grid grid-cols-2 gap-4"
                >
                    <div>
                        <div
                            class="text-[10px] uppercase tracking-wider text-zinc-500 font-medium mb-0.5"
                        >
                            ↓ Total
                        </div>
                        <div
                            class="text-xs font-mono font-medium text-zinc-700 dark:text-zinc-300"
                        >
                            {server.net_in.toFixed(2)}
                            <span class="text-[9px] text-zinc-400">GB</span>
                        </div>
                    </div>
                    <div>
                        <div
                            class="text-[10px] uppercase tracking-wider text-zinc-500 font-medium mb-0.5"
                        >
                            ↑ Total
                        </div>
                        <div
                            class="text-xs font-mono font-medium text-zinc-700 dark:text-zinc-300"
                        >
                            {server.net_out.toFixed(2)}
                            <span class="text-[9px] text-zinc-400">GB</span>
                        </div>
                    </div>
                </div>
            {:else}
                <!-- System Info when no realtime metrics -->
                <div
                    class="pt-4 border-t border-zinc-100 dark:border-zinc-800 space-y-2"
                >
                    <div
                        class="flex justify-between text-[10px] font-medium uppercase tracking-wider text-zinc-500"
                    >
                        <span>System</span>
                        <span
                            class="text-zinc-900 dark:text-zinc-200 font-mono"
                        >
                            {server.cpu_cores} Cores
                        </span>
                    </div>
                    <div
                        class="flex justify-between text-[10px] font-medium uppercase tracking-wider text-zinc-500"
                    >
                        <span>Memory</span>
                        <span
                            class="text-zinc-900 dark:text-zinc-200 font-mono"
                        >
                            {formatMemory(server.total_memory)} GB
                        </span>
                    </div>
                </div>
            {/if}

            <!-- Tags -->
            {#if server.tags && server.tags.length > 0}
                <div class="pt-4 border-t border-zinc-100 dark:border-zinc-800">
                    <div class="flex flex-wrap gap-2">
                        {#each server.tags as tag}
                            <span
                                class="px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 rounded"
                            >
                                {tag}
                            </span>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    {:else}
        <!-- Offline State -->
        <div class="h-[106px] flex items-center justify-center">
            <div
                class="text-[10px] text-zinc-400 font-medium uppercase tracking-widest"
            >
                System Offline
            </div>
        </div>
    {/if}
</div>
