<script lang="ts">
    import { onMount } from "svelte";
    import AdminLayout from "../../lib/admin/AdminLayout.svelte";
    import ServiceModal from "../../lib/admin/ServiceModal.svelte";
    import { api } from "../../lib/api";
    import type { ServiceStatusOverview, Service } from "../../lib/types";

    let services = $state<ServiceStatusOverview[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    // Modal state
    let showModal = $state(false);
    let editingService = $state<Service | null>(null);

    // Calculate uptime percentage from history
    function calculateUptime(overview: ServiceStatusOverview): string {
        const validPoints = overview.history.filter(
            (p) => p.status !== "unknown",
        );
        if (validPoints.length === 0) return "0";

        const upCount = validPoints.filter((p) => p.status === "up").length;
        return ((upCount / validPoints.length) * 100).toFixed(2);
    }

    // Map status to display status
    function getDisplayStatus(status: string): string {
        if (status === "up") return "online";
        if (status === "down") return "offline";
        if (status === "timeout" || status === "error") return "degraded";
        return "unknown";
    }

    // Convert status to number for history bar (1 = up, 0 = down, 2 = degraded)
    function statusToNumber(status: string): number {
        if (status === "up") return 1;
        if (status === "down") return 0;
        return 2; // timeout, error, unknown
    }

    async function loadServices() {
        try {
            // Don't set loading to true on refresh to avoid flickering
            if (services.length === 0) loading = true;
            error = null;
            services = await api.services.getAllOverviews();
        } catch (e) {
            error = e instanceof Error ? e.message : "Failed to load services";
            console.error("Failed to load services:", e);
        } finally {
            loading = false;
        }
    }

    function handleCreate() {
        editingService = null;
        showModal = true;
    }

    function handleEdit(service: Service) {
        editingService = service;
        showModal = true;
    }

    async function handleDelete(service: Service) {
        if (
            !confirm(
                `Are you sure you want to delete service "${service.name}"? This action cannot be undone.`,
            )
        ) {
            return;
        }

        try {
            await api.services.delete(service.id);
            await loadServices();
        } catch (e) {
            console.error("Failed to delete service:", e);
            alert("Failed to delete service");
        }
    }

    function handleSave() {
        loadServices();
    }

    onMount(() => {
        loadServices();
        // Refresh every 60 seconds
        const interval = setInterval(loadServices, 60000);
        return () => clearInterval(interval);
    });
</script>

<AdminLayout>
    <div class="max-w-7xl mx-auto p-8 md:p-12 animate-in fade-in duration-500">
        <div class="flex items-center justify-between mb-12">
            <div>
                <h1
                    class="text-3xl font-bold text-zinc-900 dark:text-white tracking-tight mb-2"
                >
                    Services
                </h1>
                <p class="text-zinc-500 dark:text-zinc-400">
                    Monitor and manage your external services.
                </p>
            </div>
            <button
                on:click={handleCreate}
                class="px-4 py-2 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
            >
                Add Service
            </button>
        </div>

        {#if loading && services.length === 0}
            <div class="flex justify-center items-center py-12">
                <div class="text-zinc-500">Loading services...</div>
            </div>
        {:else if error}
            <div class="flex justify-center items-center py-12">
                <div class="text-red-500">Error: {error}</div>
            </div>
        {:else if services.length === 0}
            <div
                class="flex flex-col justify-center items-center py-20 border-2 border-dashed border-zinc-200 dark:border-zinc-800 rounded-2xl"
            >
                <div
                    class="w-12 h-12 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center mb-4 text-zinc-400"
                >
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
                        ><rect x="2" y="2" width="20" height="8" rx="2" ry="2"
                        ></rect><rect
                            x="2"
                            y="14"
                            width="20"
                            height="8"
                            rx="2"
                            ry="2"
                        ></rect><line x1="6" y1="6" x2="6.01" y2="6"
                        ></line><line x1="6" y1="18" x2="6.01" y2="18"
                        ></line></svg
                    >
                </div>
                <h3
                    class="text-lg font-medium text-zinc-900 dark:text-white mb-1"
                >
                    No services configured
                </h3>
                <p class="text-zinc-500 dark:text-zinc-400 mb-6">
                    Get started by adding your first service to monitor.
                </p>
                <button
                    on:click={handleCreate}
                    class="px-4 py-2 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
                >
                    Add Service
                </button>
            </div>
        {:else}
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="border-b border-zinc-200 dark:border-zinc-800">
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Service Name</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Type</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Target</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Status</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider"
                                >Uptime (30h)</th
                            >
                            <th
                                class="py-4 px-4 text-xs font-medium text-zinc-500 uppercase tracking-wider text-right"
                                >Actions</th
                            >
                        </tr>
                    </thead>
                    <tbody
                        class="divide-y divide-zinc-100 dark:divide-zinc-800/50"
                    >
                        {#each services as overview}
                            {@const displayStatus = getDisplayStatus(
                                overview.current_status,
                            )}
                            {@const uptime = calculateUptime(overview)}
                            <tr
                                class="group hover:bg-zinc-50 dark:hover:bg-zinc-800/30 transition-colors"
                            >
                                <td class="py-4 px-4">
                                    <div class="flex items-center gap-3">
                                        <div
                                            class="w-2 h-2 rounded-full {displayStatus ===
                                            'online'
                                                ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]'
                                                : displayStatus === 'degraded'
                                                  ? 'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.4)]'
                                                  : 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.4)]'}"
                                        ></div>
                                        <div class="flex-1">
                                            <div
                                                class="text-sm font-medium text-zinc-900 dark:text-white"
                                            >
                                                {overview.service.name}
                                            </div>
                                            {#if !overview.service.enabled}
                                                <div
                                                    class="text-[10px] text-zinc-400 uppercase tracking-wider mt-0.5"
                                                >
                                                    Disabled
                                                </div>
                                            {/if}
                                            <!-- Uptime History Bar -->
                                            <div class="mt-2">
                                                <div class="flex items-center gap-[2px] h-1.5">
                                                    {#each overview.history as point, i}
                                                        {@const status = statusToNumber(point.status)}
                                                        <div
                                                            class="flex-1 rounded-full h-full transition-all duration-300 hover:scale-y-150 cursor-help"
                                                            class:bg-emerald-500={status === 1}
                                                            class:bg-rose-500={status === 0}
                                                            class:bg-amber-500={status === 2}
                                                            title={`Check ${i + 1}: ${status === 1 ? "Up" : status === 0 ? "Down" : "Degraded"}`}
                                                        ></div>
                                                    {/each}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td class="py-4 px-4">
                                    <span
                                        class="inline-flex items-center px-2 py-1 rounded-md text-xs font-medium bg-zinc-100 text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300"
                                    >
                                        {overview.service.type.toUpperCase()}
                                    </span>
                                </td>
                                <td
                                    class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400 font-mono"
                                    >{overview.service.target}</td
                                >
                                <td class="py-4 px-4">
                                    <span
                                        class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium
                                        {displayStatus === 'online'
                                            ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/10 dark:text-emerald-400'
                                            : displayStatus === 'degraded'
                                              ? 'bg-amber-100 text-amber-700 dark:bg-amber-500/10 dark:text-amber-400'
                                              : 'bg-red-100 text-red-700 dark:bg-red-500/10 dark:text-red-400'}"
                                    >
                                        {displayStatus}
                                    </span>
                                </td>
                                <td
                                    class="py-4 px-4 text-sm text-zinc-500 dark:text-zinc-400"
                                    >{uptime}%</td
                                >
                                <td class="py-4 px-4 text-right">
                                    <div class="flex items-center justify-end gap-2">
                                        <button
                                            on:click={() =>
                                                handleEdit(overview.service)}
                                            class="opacity-0 group-hover:opacity-100 p-1.5 text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded transition-all"
                                            title="Edit"
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
                                                stroke-linejoin="round"
                                                ><path
                                                    d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                                                ></path><path
                                                    d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
                                                ></path></svg
                                            >
                                        </button>
                                        <button
                                            on:click={() =>
                                                handleDelete(overview.service)}
                                            class="opacity-0 group-hover:opacity-100 p-1.5 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-all"
                                            title="Delete"
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
                                                stroke-linejoin="round"
                                                ><polyline points="3 6 5 6 21 6"
                                                ></polyline><path
                                                    d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                                                ></path><line
                                                    x1="10"
                                                    y1="11"
                                                    x2="10"
                                                    y2="17"
                                                ></line><line
                                                    x1="14"
                                                    y1="11"
                                                    x2="14"
                                                    y2="17"
                                                ></line></svg
                                            >
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </div>

    <ServiceModal
        bind:show={showModal}
        service={editingService}
        on:save={handleSave}
        on:close={() => (showModal = false)}
    />
</AdminLayout>
