<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fade, scale } from "svelte/transition";
    import type { AdminNode, UpdateNodeRequest } from "../types";
    import { api } from "../api";

    let { show = $bindable(false), node = $bindable(null) }: { show?: boolean; node?: AdminNode | null } = $props();

    const dispatch = createEventDispatcher();

    let loading = $state(false);

    // Form data
    let formData = $state<UpdateNodeRequest>({
        name: "",
        tags: [],
    });

    // Tag input
    let tagInput = $state("");

    // Reset form when opening for editing
    $effect(() => {
        if (show && node) {
            formData = {
                name: node.name,
                tags: node.tags || [],
            };
        }
    });

    async function handleSubmit() {
        if (!node) return;

        loading = true;
        try {
            await api.nodes.adminUpdate(node.id, formData);
            dispatch("save");
            show = false;
        } catch (e) {
            console.error("Failed to update node", e);
            alert(
                "Failed to update node: " +
                    (e instanceof Error ? e.message : String(e)),
            );
        } finally {
            loading = false;
        }
    }

    function addTag() {
        const trimmed = tagInput.trim();
        if (trimmed && !formData.tags?.includes(trimmed)) {
            formData.tags = [...(formData.tags || []), trimmed];
            tagInput = "";
        }
    }

    function removeTag(tag: string) {
        formData.tags = formData.tags?.filter((t) => t !== tag);
    }

    function close() {
        show = false;
        dispatch("close");
    }
</script>

{#if show && node}
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
                    Edit Node
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
                    <!-- Node Info (Read-only) -->
                    <div class="p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg space-y-2">
                        <div class="text-xs font-medium text-zinc-500 dark:text-zinc-400">
                            Node Information
                        </div>
                        <div class="grid grid-cols-2 gap-2 text-xs">
                            <div>
                                <span class="text-zinc-500">UUID:</span>
                                <span class="text-zinc-900 dark:text-white font-mono ml-2">{node.uuid.slice(0, 8)}...</span>
                            </div>
                            <div>
                                <span class="text-zinc-500">IP:</span>
                                <span class="text-zinc-900 dark:text-white font-mono ml-2">{node.ip_address}</span>
                            </div>
                            <div>
                                <span class="text-zinc-500">OS:</span>
                                <span class="text-zinc-900 dark:text-white ml-2">{node.os_type}</span>
                            </div>
                            <div>
                                <span class="text-zinc-500">Agent:</span>
                                <span class="text-zinc-900 dark:text-white font-mono ml-2">{node.agent_version}</span>
                            </div>
                        </div>
                    </div>

                    <!-- Name -->
                    <div>
                        <label
                            class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                            >Node Name</label
                        >
                        <input
                            type="text"
                            bind:value={formData.name}
                            required
                            class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                            placeholder="My Server"
                        />
                    </div>

                    <!-- Tags -->
                    <div>
                        <label
                            class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                            >Tags</label
                        >
                        <div class="space-y-2">
                            <div class="flex gap-2">
                                <input
                                    type="text"
                                    bind:value={tagInput}
                                    on:keypress={(e) =>
                                        e.key === "Enter" &&
                                        (e.preventDefault(), addTag())}
                                    class="flex-1 px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                                    placeholder="Add tag..."
                                />
                                <button
                                    type="button"
                                    on:click={addTag}
                                    class="px-3 py-2 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-lg hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
                                >
                                    Add
                                </button>
                            </div>
                            {#if formData.tags && formData.tags.length > 0}
                                <div class="flex flex-wrap gap-2">
                                    {#each formData.tags as tag}
                                        <span
                                            class="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300 rounded-md"
                                        >
                                            {tag}
                                            <button
                                                type="button"
                                                on:click={() => removeTag(tag)}
                                                class="hover:text-red-600 dark:hover:text-red-400"
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
                                                    ><line x1="18" y1="6" x2="6" y2="18"></line><line
                                                        x1="6"
                                                        y1="6"
                                                        x2="18"
                                                        y2="18"
                                                    ></line></svg
                                                >
                                            </button>
                                        </span>
                                    {/each}
                                </div>
                            {/if}
                        </div>
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
                            Save Changes
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}
