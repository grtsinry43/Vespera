<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fade, scale } from "svelte/transition";
    import type {
        Service,
        ServiceCreate,
        ServiceUpdate,
        PublicNode,
    } from "../types";
    import { api } from "../api";

    let { show = $bindable(false), service = $bindable(null) }: { show?: boolean; service?: Service | null } = $props();

    const dispatch = createEventDispatcher();

    let loading = $state(false);
    let nodes = $state<PublicNode[]>([]);
    let nodesLoaded = $state(false);

    // Form data
    let formData = $state<ServiceCreate>({
        name: "",
        type: "http",
        target: "",
        check_interval: 60,
        timeout: 10,
        enabled: true,
        method: "GET",
        expected_code: 200,
        node_id: undefined,
    });

    // Reset form when opening for create, or fill when editing
    $effect(() => {
        if (show) {
            // Load nodes only once
            if (!nodesLoaded) {
                loadNodes();
            }

            // Reset or fill form
            if (service) {
                formData = {
                    name: service.name,
                    type: service.type,
                    target: service.target,
                    check_interval: service.check_interval,
                    timeout: service.timeout,
                    enabled: service.enabled,
                    method: service.method || "GET",
                    expected_code: service.expected_code || 200,
                    node_id: service.node_id,
                };
            } else {
                formData = {
                    name: "",
                    type: "http",
                    target: "",
                    check_interval: 60,
                    timeout: 10,
                    enabled: true,
                    method: "GET",
                    expected_code: 200,
                    node_id: undefined,
                };
            }
        }
    });

    async function loadNodes() {
        try {
            nodes = await api.nodes.list(100, 0);
            nodesLoaded = true;
        } catch (e) {
            console.error("Failed to load nodes", e);
        }
    }

    async function handleSubmit() {
        loading = true;
        try {
            if (service) {
                // Update
                const updateData: ServiceUpdate = {
                    name: formData.name,
                    target: formData.target,
                    check_interval: formData.check_interval,
                    timeout: formData.timeout,
                    method:
                        formData.type === "http" ? formData.method : undefined,
                    expected_code:
                        formData.type === "http"
                            ? formData.expected_code
                            : undefined,
                    enabled: formData.enabled,
                };
                await api.services.update(service.id, updateData);
            } else {
                // Create
                await api.services.create(formData);
            }
            dispatch("save");
            show = false;
        } catch (e) {
            console.error("Failed to save service", e);
            alert(
                "Failed to save service: " +
                    (e instanceof Error ? e.message : String(e)),
            );
        } finally {
            loading = false;
        }
    }

    function close() {
        show = false;
        dispatch("close");
    }
</script>

{#if show}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6"
        transition:fade={{ duration: 200 }}
    >
        <div
            class="absolute inset-0 bg-black/50 backdrop-blur-sm"
            on:click={close}
        ></div>

        <div
            class="relative w-full max-w-lg bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border border-zinc-200 dark:border-zinc-800 overflow-hidden"
            transition:scale={{ duration: 200, start: 0.95 }}
        >
            <div
                class="px-6 py-4 border-b border-zinc-100 dark:border-zinc-800 flex items-center justify-between bg-zinc-50/50 dark:bg-zinc-900/50"
            >
                <h3 class="text-lg font-semibold text-zinc-900 dark:text-white">
                    {service ? "Edit Service" : "Add New Service"}
                </h3>
                <button
                    on:click={close}
                    class="text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><line x1="18" y1="6" x2="6" y2="18"></line><line
                            x1="6"
                            y1="6"
                            x2="18"
                            y2="18"
                        ></line></svg
                    >
                </button>
            </div>

            <div class="p-6 max-h-[calc(100vh-200px)] overflow-y-auto">
                <form on:submit|preventDefault={handleSubmit} class="space-y-4">
                    <!-- Basic Info -->
                    <div class="grid grid-cols-2 gap-4">
                        <div class="col-span-2">
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Service Name</label
                            >
                            <input
                                type="text"
                                bind:value={formData.name}
                                required
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                                placeholder="My Website"
                            />
                        </div>

                        <div>
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Type</label
                            >
                            <select
                                bind:value={formData.type}
                                disabled={!!service}
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all disabled:opacity-50"
                            >
                                <option value="http">HTTP(s)</option>
                                <option value="tcp">TCP Port</option>
                            </select>
                        </div>

                        <div>
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Assigned Node (Optional)</label
                            >
                            <select
                                bind:value={formData.node_id}
                                disabled={!!service}
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all disabled:opacity-50"
                            >
                                <option value={undefined}>None (Global)</option>
                                {#each nodes as node}
                                    <option value={node.id}>{node.name}</option>
                                {/each}
                            </select>
                        </div>

                        <div class="col-span-2">
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Target URL / IP:Port</label
                            >
                            <input
                                type="text"
                                bind:value={formData.target}
                                required
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                                placeholder={formData.type === "http"
                                    ? "https://example.com"
                                    : "1.2.3.4:8080"}
                            />
                        </div>
                    </div>

                    <!-- HTTP Options -->
                    {#if formData.type === "http"}
                        <div
                            class="p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg border border-zinc-100 dark:border-zinc-800 space-y-4"
                        >
                            <div
                                class="text-xs font-medium text-zinc-900 dark:text-zinc-300"
                            >
                                HTTP Configuration
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label
                                        class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                        >Method</label
                                    >
                                    <select
                                        bind:value={formData.method}
                                        class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                                    >
                                        <option value="GET">GET</option>
                                        <option value="POST">POST</option>
                                        <option value="HEAD">HEAD</option>
                                    </select>
                                </div>
                                <div>
                                    <label
                                        class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                        >Expected Code</label
                                    >
                                    <input
                                        type="number"
                                        bind:value={formData.expected_code}
                                        class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                                    />
                                </div>
                            </div>
                        </div>
                    {/if}

                    <!-- Monitoring Settings -->
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Interval (seconds)</label
                            >
                            <input
                                type="number"
                                min="10"
                                bind:value={formData.check_interval}
                                required
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                            />
                        </div>
                        <div>
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Timeout (seconds)</label
                            >
                            <input
                                type="number"
                                min="1"
                                bind:value={formData.timeout}
                                required
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                            />
                        </div>
                    </div>

                    <!-- Enabled Toggle -->
                    <div class="flex items-center gap-3 pt-2">
                        <button
                            type="button"
                            class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {formData.enabled
                                ? 'bg-emerald-500'
                                : 'bg-zinc-200 dark:bg-zinc-700'}"
                            on:click={() =>
                                (formData.enabled = !formData.enabled)}
                        >
                            <span
                                class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {formData.enabled
                                    ? 'translate-x-5'
                                    : 'translate-x-0'}"
                            ></span>
                        </button>
                        <span
                            class="text-sm font-medium text-zinc-700 dark:text-zinc-300"
                            >Enable Monitoring</span
                        >
                    </div>

                    <div class="pt-4 flex items-center justify-end gap-3">
                        <button
                            type="button"
                            on:click={close}
                            class="px-4 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            disabled={loading}
                            class="px-4 py-2 text-sm font-medium text-white bg-zinc-900 dark:bg-white dark:text-zinc-900 hover:bg-zinc-800 dark:hover:bg-zinc-100 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                        >
                            {#if loading}
                                <svg
                                    class="animate-spin h-4 w-4"
                                    xmlns="http://www.w3.org/2000/svg"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                >
                                    <circle
                                        class="opacity-25"
                                        cx="12"
                                        cy="12"
                                        r="10"
                                        stroke="currentColor"
                                        stroke-width="4"
                                    ></circle>
                                    <path
                                        class="opacity-75"
                                        fill="currentColor"
                                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                    ></path>
                                </svg>
                            {/if}
                            {service ? "Save Changes" : "Create Service"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}
