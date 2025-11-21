<script lang="ts">
    import type { ServiceStatusOverview } from './types';

    let { overview } = $props<{ overview: ServiceStatusOverview }>();

    // Calculate uptime percentage from history
    function calculateUptime(history: typeof overview.history): number {
        const validPoints = history.filter(p => p.status !== 'unknown');
        if (validPoints.length === 0) return 0;

        const upCount = validPoints.filter(p => p.status === 'up').length;
        return Number(((upCount / validPoints.length) * 100).toFixed(2));
    }

    // Calculate average response time
    function calculateAvgLatency(history: typeof overview.history): number {
        const validTimes = history
            .filter(p => p.response_time !== null && p.response_time !== undefined)
            .map(p => p.response_time!);

        if (validTimes.length === 0) return 0;
        return Math.round(validTimes.reduce((a, b) => a + b, 0) / validTimes.length);
    }

    // Convert status to display format (for history bar)
    // 1 = up, 0 = down, 2 = degraded/timeout/error
    function statusToNumber(status: typeof overview.current_status): number {
        if (status === 'up') return 1;
        if (status === 'down') return 0;
        return 2; // timeout, error, unknown
    }

    // Derived values
    const uptime = $derived(calculateUptime(overview.history));
    const avgLatency = $derived(calculateAvgLatency(overview.history));
    const history = $derived(overview.history.map(p => statusToNumber(p.status)));
    const displayUrl = $derived(
        overview.service.type === 'http'
            ? overview.service.target.replace(/^https?:\/\//, '')
            : overview.service.target
    );
</script>

<div
    class="group relative overflow-hidden bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-5 hover:border-zinc-300/50 dark:hover:border-zinc-700 transition-all duration-300 shadow-[0_2px_10px_rgba(0,0,0,0.02)] hover:shadow-[0_8px_30px_rgba(0,0,0,0.06)] hover:-translate-y-0.5"
>
    <div class="flex justify-between items-start mb-4">
        <div class="flex items-center gap-3">
            <div class="relative">
                <div
                    class="w-2.5 h-2.5 rounded-full {overview.current_status === 'up'
                        ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]'
                        : overview.current_status === 'down'
                          ? 'bg-rose-500 shadow-[0_0_8px_rgba(244,63,94,0.4)]'
                          : 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.4)]'}"
                ></div>
                {#if overview.current_status === "up"}
                    <div
                        class="absolute inset-0 rounded-full bg-emerald-500 animate-ping opacity-20"
                    ></div>
                {/if}
            </div>
            <div>
                <div class="flex items-center gap-1.5">
                    <h3
                        class="font-bold text-sm text-zinc-900 dark:text-white leading-none"
                    >
                        {overview.service.name}
                    </h3>
                    {#if !overview.service.is_public}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="11"
                            height="11"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2.5"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="text-zinc-400/70 dark:text-zinc-500/70 shrink-0"
                            title="Private (team only)"
                            ><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg
                        >
                    {/if}
                </div>
                {#if overview.service.type === 'http'}
                    <a
                        href={overview.service.target}
                        target="_blank"
                        class="text-[10px] text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors font-mono flex items-center gap-1"
                    >
                        {displayUrl}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="10"
                            height="10"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            class="opacity-0 group-hover:opacity-100 transition-opacity"
                            ><path
                                d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
                            /><polyline points="15 3 21 3 21 9" /><line
                                x1="10"
                                y1="14"
                                x2="21"
                                y2="3"
                            /></svg
                        >
                    </a>
                {:else}
                    <span
                        class="text-[10px] text-zinc-400 font-mono"
                    >
                        {displayUrl}
                    </span>
                {/if}
            </div>
        </div>
        <div class="text-right">
            <div
                class="text-sm font-bold font-mono {overview.current_status === 'up'
                    ? 'text-emerald-600 dark:text-emerald-400'
                    : overview.current_status === 'down'
                      ? 'text-rose-600 dark:text-rose-400'
                      : 'text-amber-600 dark:text-amber-400'}"
            >
                {uptime}%
            </div>
            <div
                class="text-[10px] font-medium text-zinc-400 uppercase tracking-wider"
            >
                Uptime
            </div>
        </div>
    </div>

    <!-- Uptime History Bar (Uptime Kuma style) -->
    <div class="flex items-center gap-[2px] h-1.5 mt-4 mb-2 opacity-80">
        {#each history as status, i}
            <div
                class="flex-1 rounded-full h-full transition-all duration-300 hover:scale-y-150 hover:opacity-100 cursor-help"
                class:bg-emerald-500={status === 1}
                class:bg-rose-500={status === 0}
                class:bg-amber-500={status === 2}
                title={`Check ${i + 1}: ${status === 1 ? "Up" : status === 0 ? "Down" : "Degraded"}`}
            ></div>
        {/each}
    </div>

    <div
        class="flex items-center justify-between pt-2 border-t border-black/5 dark:border-zinc-800/50"
    >
        <div class="flex items-center gap-2">
            <span
                class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-zinc-50 dark:bg-zinc-800 text-zinc-500 border border-zinc-100 dark:border-zinc-700/50"
            >
                {overview.service.type}
            </span>
        </div>
        <div
            class="flex items-center gap-1.5 text-[10px] font-medium text-zinc-400"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><circle cx="12" cy="12" r="10" /><polyline
                    points="12 6 12 12 16 14"
                /></svg
            >
            {avgLatency > 0 ? `${avgLatency}ms` : 'N/A'}
        </div>
    </div>
</div>
