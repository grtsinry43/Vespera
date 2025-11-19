<script lang="ts">
    let { server, onclick } = $props();

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
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="group relative bg-white/80 dark:bg-[#18181b]/80 backdrop-blur-sm border border-zinc-200 dark:border-zinc-800 rounded-xl p-6 hover:border-zinc-300 dark:hover:border-zinc-700 transition-all duration-200 cursor-pointer shadow-sm hover:shadow-md hover:-translate-y-0.5"
    {onclick}
>
    <!-- Header -->
    <div class="flex justify-between items-start mb-8">
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
                    <span
                        class="text-base leading-none opacity-50 group-hover:opacity-100 transition-opacity"
                        >{server.flag}</span
                    >
                </div>
                <p
                    class="text-[10px] font-medium uppercase tracking-wider text-zinc-500 mt-0.5"
                >
                    {server.region}
                </p>
            </div>
        </div>

        <div class="text-right">
            <div
                class="text-[11px] font-mono font-medium text-zinc-500 bg-zinc-50 dark:bg-zinc-900 px-1.5 py-0.5 rounded border border-zinc-100 dark:border-zinc-800"
            >
                {#if server.status === "online" || server.status === "warning"}
                    {server.ping}ms
                {:else}
                    TIMEOUT
                {/if}
            </div>
        </div>
    </div>

    <!-- Metrics -->
    {#if server.status !== "offline"}
        <div class="space-y-5">
            <!-- CPU & RAM Row -->
            <div class="grid grid-cols-2 gap-6">
                <!-- CPU -->
                <div class="space-y-2">
                    <div
                        class="flex justify-between text-[10px] font-medium uppercase tracking-wider text-zinc-500"
                    >
                        <span>CPU</span>
                        <span class="text-zinc-900 dark:text-zinc-200 font-mono"
                            >{server.cpu.toFixed(0)}%</span
                        >
                    </div>
                    <div
                        class="h-1.5 w-full bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full {getBarColor(
                                server.cpu,
                            )} transition-all duration-500 ease-out"
                            style="width: {server.cpu}%"
                        ></div>
                    </div>
                </div>
                <!-- RAM -->
                <div class="space-y-2">
                    <div
                        class="flex justify-between text-[10px] font-medium uppercase tracking-wider text-zinc-500"
                    >
                        <span>MEM</span>
                        <span class="text-zinc-900 dark:text-zinc-200 font-mono"
                            >{server.memory.toFixed(0)}%</span
                        >
                    </div>
                    <div
                        class="h-1.5 w-full bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden"
                    >
                        <div
                            class="h-full {getBarColor(
                                server.memory,
                            )} transition-all duration-500 ease-out"
                            style="width: {server.memory}%"
                        ></div>
                    </div>
                </div>
            </div>

            <!-- Network -->
            <div
                class="pt-4 border-t border-zinc-100 dark:border-zinc-800 grid grid-cols-2 gap-4"
            >
                <div>
                    <div
                        class="text-[10px] uppercase tracking-wider text-zinc-500 font-medium mb-0.5"
                    >
                        Net In
                    </div>
                    <div
                        class="text-xs font-mono font-medium text-zinc-700 dark:text-zinc-300"
                    >
                        {server.net_in.toFixed(1)}
                        <span class="text-[9px] text-zinc-400">M</span>
                    </div>
                </div>
                <div>
                    <div
                        class="text-[10px] uppercase tracking-wider text-zinc-500 font-medium mb-0.5"
                    >
                        Net Out
                    </div>
                    <div
                        class="text-xs font-mono font-medium text-zinc-700 dark:text-zinc-300"
                    >
                        {server.net_out.toFixed(1)}
                        <span class="text-[9px] text-zinc-400">M</span>
                    </div>
                </div>
            </div>
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
