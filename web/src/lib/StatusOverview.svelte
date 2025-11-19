<script lang="ts">
    let { globalStats } = $props();

    // Mock incident data
    let activeIncidents = $state([
        // {
        //     title: "Blog Backend Service Exception",
        //     time: "Detected 5m ago",
        //     description:
        //         "Due to backend service issues, the blog is currently operating in static mode. We are working to resolve this as soon as possible.",
        // },
    ]);

    // // Mock incident data
    // let activeIncidents = $state([
    //     {
    //         title: "Blog Backend Service Exception",
    //         time: "Detected 5m ago",
    //         description:
    //             "Due to backend service issues, the blog is currently operating in static mode. We are working to resolve this as soon as possible.",
    //     },
    // ]);

    let systemStatus = $derived(
        activeIncidents.length > 0 ? "degraded" : "operational",
    );
</script>

<div class="mb-12 animate-in fade-in slide-in-from-top-4 duration-700">
    <div
        class="relative overflow-hidden rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-white/80 dark:bg-[#18181b]/80 backdrop-blur-md shadow-sm p-6 sm:p-8"
    >
        <!-- Background Decor -->
        <div
            class="absolute top-0 right-0 w-96 h-96 bg-gradient-to-br from-emerald-500/5 to-transparent dark:from-emerald-500/10 dark:to-transparent rounded-bl-full pointer-events-none opacity-50"
        ></div>

        <div
            class="relative flex flex-col sm:flex-row items-start sm:items-center justify-between gap-6"
        >
            <div class="flex items-start gap-5">
                <div
                    class="relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-zinc-50 dark:bg-zinc-900 border border-zinc-100 dark:border-zinc-800 shadow-inner"
                >
                    {#if systemStatus === "operational"}
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
                            class="text-emerald-500"
                            ><path d="M20 6 9 17l-5-5" /></svg
                        >
                        <span
                            class="absolute inset-0 rounded-xl ring-1 ring-inset ring-emerald-500/20"
                        ></span>
                    {:else}
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
                            class="text-amber-500"
                            ><path
                                d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
                            /><line x1="12" y1="9" x2="12" y2="13" /><line
                                x1="12"
                                y1="17"
                                x2="12.01"
                                y2="17"
                            /></svg
                        >
                        <span
                            class="absolute inset-0 rounded-xl ring-1 ring-inset ring-amber-500/20"
                        ></span>
                    {/if}
                </div>

                <div>
                    <h2
                        class="text-xl font-bold text-zinc-900 dark:text-white tracking-tight"
                    >
                        {#if systemStatus === "operational"}
                            All Systems Operational
                        {:else}
                            System Issues Detected
                        {/if}
                    </h2>
                    <p
                        class="text-sm text-zinc-500 mt-1 max-w-md leading-relaxed"
                    >
                        {#if systemStatus === "operational"}
                            All services are running normally. No active
                            incidents reported in the last 24 hours.
                        {:else}
                            {activeIncidents[0].description}
                        {/if}
                    </p>
                </div>
            </div>

            <!-- Mini Stats -->
            <div
                class="flex items-center gap-8 sm:border-l border-zinc-100 dark:border-zinc-800 sm:pl-8"
            >
                <div>
                    <div
                        class="text-2xl font-mono font-bold text-zinc-900 dark:text-white tracking-tight"
                    >
                        99.98%
                    </div>
                    <div
                        class="text-[10px] font-semibold text-zinc-400 uppercase tracking-wider mt-0.5"
                    >
                        Uptime (30d)
                    </div>
                </div>
                <div>
                    <div
                        class="text-2xl font-mono font-bold text-zinc-900 dark:text-white tracking-tight"
                    >
                        {globalStats.active}/{globalStats.total}
                    </div>
                    <div
                        class="text-[10px] font-semibold text-zinc-400 uppercase tracking-wider mt-0.5"
                    >
                        Nodes Active
                    </div>
                </div>
            </div>
        </div>

        <!-- Incident List (Collapsible or Always visible if active) -->
        {#if activeIncidents.length > 0}
            <div
                class="mt-6 pt-6 border-t border-zinc-100 dark:border-zinc-800"
            >
                <h3
                    class="text-xs font-bold text-zinc-900 dark:text-white uppercase tracking-wider mb-3"
                >
                    Active Incidents
                </h3>
                <div class="space-y-3">
                    {#each activeIncidents as incident}
                        <div
                            class="flex items-center justify-between p-3 rounded-lg bg-amber-50 dark:bg-amber-900/10 border border-amber-100 dark:border-amber-900/20"
                        >
                            <div class="flex items-center gap-3">
                                <span class="relative flex h-2 w-2">
                                    <span
                                        class="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75"
                                    ></span>
                                    <span
                                        class="relative inline-flex rounded-full h-2 w-2 bg-amber-500"
                                    ></span>
                                </span>
                                <span
                                    class="text-sm font-medium text-zinc-900 dark:text-zinc-200"
                                    >{incident.title}</span
                                >
                            </div>
                            <span class="text-xs text-zinc-500"
                                >{incident.time}</span
                            >
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
    </div>
</div>
