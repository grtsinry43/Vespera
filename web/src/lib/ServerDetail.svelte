<script lang="ts">
    let { server, onBack } = $props();

    // Mock historical data for charts
    let history = Array(40)
        .fill(0)
        .map(() => Math.floor(Math.random() * 60) + 20);
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
        <div
            class="flex flex-col md:flex-row justify-between items-start md:items-center gap-6"
        >
            <div class="flex items-center gap-6">
                <div class="text-4xl filter drop-shadow-sm">
                    {server.flag || "🏳️"}
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
                        <span class="font-medium"
                            >{server.region}, {server.country}</span
                        >
                        <span
                            class="w-1 h-1 rounded-full bg-zinc-300 dark:bg-zinc-700"
                        ></span>
                        <span
                            class="font-mono bg-zinc-50 dark:bg-zinc-900 px-1.5 py-0.5 rounded border border-zinc-100 dark:border-zinc-800"
                            >{server.ip || "192.168.1.1"}</span
                        >
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
                        Uptime
                    </div>
                    <div
                        class="text-xl font-mono font-semibold text-zinc-900 dark:text-white"
                    >
                        {server.uptime}
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
                        {server.load}
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Charts Section -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- CPU History -->
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
            <div class="h-40 flex items-end gap-[2px]">
                {#each history as val}
                    <div
                        class="flex-1 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors rounded-t-sm relative group"
                    >
                        <div
                            class="absolute bottom-0 w-full bg-zinc-900 dark:bg-white rounded-t-sm opacity-80"
                            style="height: {val}%"
                        ></div>
                    </div>
                {/each}
            </div>
        </div>

        <!-- Network History -->
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
                    >Chart Visualization</span
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
            <div class="space-y-8">
                <div>
                    <div class="flex justify-between text-xs font-medium mb-2">
                        <span class="text-zinc-500">/ (root)</span>
                        <span class="text-zinc-900 dark:text-white font-mono"
                            >45%</span
                        >
                    </div>
                    <div
                        class="w-full bg-zinc-100 dark:bg-zinc-800 rounded-full h-2 overflow-hidden"
                    >
                        <div
                            class="bg-zinc-900 dark:bg-white h-full rounded-full opacity-80"
                            style="width: 45%"
                        ></div>
                    </div>
                </div>
                <div>
                    <div class="flex justify-between text-xs font-medium mb-2">
                        <span class="text-zinc-500">/data</span>
                        <span class="text-zinc-900 dark:text-white font-mono"
                            >78%</span
                        >
                    </div>
                    <div
                        class="w-full bg-zinc-100 dark:bg-zinc-800 rounded-full h-2 overflow-hidden"
                    >
                        <div
                            class="bg-zinc-900 dark:bg-white h-full rounded-full opacity-80"
                            style="width: 78%"
                        ></div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Process List -->
    <div
        class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl overflow-hidden shadow-[0_2px_10px_rgba(0,0,0,0.02)]"
    >
        <div
            class="px-6 py-4 border-b border-black/5 dark:border-zinc-800 flex justify-between items-center"
        >
            <h3 class="text-sm font-semibold text-zinc-900 dark:text-white">
                Top Processes
            </h3>
        </div>
        <div class="overflow-x-auto">
            <table class="w-full text-sm text-left whitespace-nowrap">
                <thead
                    class="text-xs text-zinc-500 uppercase bg-zinc-50 dark:bg-zinc-900/50 font-medium"
                >
                    <tr>
                        <th class="px-6 py-3">User</th>
                        <th class="px-6 py-3">PID</th>
                        <th class="px-6 py-3">CPU</th>
                        <th class="px-6 py-3">MEM</th>
                        <th class="px-6 py-3">Command</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-black/5 dark:divide-zinc-800">
                    <tr
                        class="hover:bg-zinc-50 dark:hover:bg-zinc-900/50 transition-colors"
                    >
                        <td
                            class="px-6 py-4 text-zinc-900 dark:text-white font-medium"
                            >root</td
                        >
                        <td class="px-6 py-4 text-zinc-500 font-mono">1234</td>
                        <td class="px-6 py-4 text-zinc-500 font-mono">2.4%</td>
                        <td class="px-6 py-4 text-zinc-500 font-mono">1.2%</td>
                        <td
                            class="px-6 py-4 text-zinc-700 dark:text-zinc-300 font-mono"
                            >/usr/bin/dockerd</td
                        >
                    </tr>
                    <tr
                        class="hover:bg-zinc-50 dark:hover:bg-zinc-900/50 transition-colors"
                    >
                        <td
                            class="px-6 py-4 text-zinc-900 dark:text-white font-medium"
                            >postgres</td
                        >
                        <td class="px-6 py-4 text-zinc-500 font-mono">2345</td>
                        <td class="px-6 py-4 text-zinc-500 font-mono">1.8%</td>
                        <td class="px-6 py-4 text-zinc-500 font-mono">8.5%</td>
                        <td
                            class="px-6 py-4 text-zinc-700 dark:text-zinc-300 font-mono"
                            >postgres: writer process</td
                        >
                    </tr>
                    <tr
                        class="hover:bg-zinc-50 dark:hover:bg-zinc-900/50 transition-colors"
                    >
                        <td
                            class="px-6 py-4 text-zinc-900 dark:text-white font-medium"
                            >www-data</td
                        >
                        <td class="px-6 py-4 text-zinc-500 font-mono">3456</td>
                        <td class="px-6 py-4 text-zinc-500 font-mono">0.5%</td>
                        <td class="px-6 py-4 text-zinc-500 font-mono">2.1%</td>
                        <td
                            class="px-6 py-4 text-zinc-700 dark:text-zinc-300 font-mono"
                            >nginx: worker process</td
                        >
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</div>
