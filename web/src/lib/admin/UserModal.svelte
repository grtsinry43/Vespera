<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fade, scale } from "svelte/transition";
    import type { User, CreateUserRequest, UpdateUserRequest } from "../types";
    import { api } from "../api";

    let { show = $bindable(false), user = $bindable(null) }: { show?: boolean; user?: User | null } = $props();

    const dispatch = createEventDispatcher();

    let loading = $state(false);

    type UserFormData = {
        username: string;
        password: string;
        email: string;
        is_admin: boolean;
    };

    // Form data
    let formData = $state<UserFormData>({
        username: "",
        password: "",
        email: "",
        is_admin: false,
    });

    // Reset form when opening
    $effect(() => {
        if (show) {
            if (user) {
                // Edit mode
                formData = {
                    username: user.username,
                    password: "",
                    email: user.email ?? "",
                    is_admin: user.role === "admin",
                };
            } else {
                // Create mode
                formData = {
                    username: "",
                    password: "",
                    email: "",
                    is_admin: false,
                };
            }
        }
    });

    async function handleSubmit() {
        loading = true;
        try {
            if (user) {
                // Update
                const request: UpdateUserRequest = {
                    email: formData.email || undefined,
                    role: formData.is_admin ? "admin" : "user",
                };
                await api.users.update(user.id, request);
            } else {
                // Create
                const request: CreateUserRequest = {
                    username: formData.username,
                    password: formData.password,
                    email: formData.email || undefined,
                    role: formData.is_admin ? "admin" : "user",
                };
                await api.users.create(request);
            }
            dispatch("save");
            show = false;
        } catch (e) {
            console.error("Failed to save user", e);
            alert(
                "Failed to save user: " +
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
                    {user ? "Edit User" : "Create New User"}
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
                    {#if user}
                        <!-- User Info (Read-only) -->
                        <div class="p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg space-y-2">
                            <div class="text-xs font-medium text-zinc-500 dark:text-zinc-400">
                                User Information
                            </div>
                            <div class="grid grid-cols-2 gap-2 text-xs">
                                <div>
                                    <span class="text-zinc-500">Username:</span>
                                    <span class="text-zinc-900 dark:text-white font-mono ml-2">{user.username}</span>
                                </div>
                                <div>
                                    <span class="text-zinc-500">ID:</span>
                                    <span class="text-zinc-900 dark:text-white font-mono ml-2">{user.id}</span>
                                </div>
                            </div>
                        </div>
                    {:else}
                        <!-- Username (Create only) -->
                        <div>
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Username</label
                            >
                            <input
                                type="text"
                                bind:value={formData.username}
                                required
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                                placeholder="john_doe"
                            />
                        </div>

                        <!-- Password (Create only) -->
                        <div>
                            <label
                                class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                                >Password</label
                            >
                            <input
                                type="password"
                                bind:value={formData.password}
                                required
                                minlength="6"
                                class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                                placeholder="••••••••"
                            />
                            <p class="mt-1 text-[10px] text-zinc-500">Minimum 6 characters</p>
                        </div>
                    {/if}

                    <!-- Email -->
                    <div>
                        <label
                            class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 mb-1.5"
                            >Email (Optional)</label
                        >
                        <input
                            type="email"
                            bind:value={formData.email}
                            class="w-full px-3 py-2 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all"
                            placeholder="john@example.com"
                        />
                    </div>

                    <!-- Admin Toggle -->
                    <div class="flex items-center gap-3 pt-2">
                        <button
                            type="button"
                            class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {formData.is_admin
                                ? 'bg-indigo-500'
                                : 'bg-zinc-200 dark:bg-zinc-700'}"
                            on:click={() =>
                                (formData.is_admin = !formData.is_admin)}
                        >
                            <span
                                class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {formData.is_admin
                                    ? 'translate-x-5'
                                    : 'translate-x-0'}"
                            ></span>
                        </button>
                        <span
                            class="text-sm font-medium text-zinc-700 dark:text-zinc-300"
                            >Administrator</span
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
                            {user ? "Save Changes" : "Create User"}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}
