<script lang="ts">
    import { authStore, isAdmin } from "./authStore";
    import { link, push } from "svelte-spa-router";

    let { onLogout } = $props<{
        onLogout: () => void;
    }>();

    let showMenu = $state(false);

    function toggleMenu() {
        showMenu = !showMenu;
    }

    function handleLogout() {
        showMenu = false;
        onLogout();
    }

    function handleNavigate(path: string) {
        showMenu = false;
        push(path);
    }

    // Click outside to close
    function handleClickOutside(event: MouseEvent) {
        if (showMenu && !(event.target as Element).closest(".user-menu")) {
            showMenu = false;
        }
    }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="relative user-menu">
    <button
        onclick={toggleMenu}
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors"
    >
        <div
            class="w-6 h-6 rounded-full bg-zinc-200 dark:bg-zinc-700 flex items-center justify-center text-xs font-semibold text-zinc-600 dark:text-zinc-300"
        >
            {$authStore.user?.username.charAt(0).toUpperCase() || "?"}
        </div>
        <span class="hidden sm:inline">{$authStore.user?.username}</span>
        <svg
            class="w-4 h-4 transition-transform duration-200 {showMenu
                ? 'rotate-180'
                : ''}"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 9l-7 7-7-7"
            />
        </svg>
    </button>

    {#if showMenu}
        <div
            class="absolute right-0 mt-2 w-56 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-lg shadow-lg overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200 z-50"
        >
            <!-- User Info -->
            <div
                class="px-4 py-3 border-b border-zinc-100 dark:border-zinc-800"
            >
                <p class="text-sm font-medium text-zinc-900 dark:text-white">
                    {$authStore.user?.username}
                </p>
                <p class="text-xs text-zinc-500 dark:text-zinc-400 mt-0.5">
                    {$authStore.user?.email || "No email"}
                </p>
                <div class="mt-2">
                    <span
                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {$isAdmin
                            ? 'bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400'
                            : 'bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400'}"
                    >
                        {$authStore.user?.role}
                    </span>
                </div>
            </div>

            <!-- Menu Items -->
            <div class="py-1">
                <button
                    onclick={() => handleNavigate('/admin')}
                    class="w-full px-4 py-2 text-left text-sm text-zinc-700 dark:text-zinc-300 hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors flex items-center gap-2"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-60"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
                    {$isAdmin ? 'Admin Panel' : 'Dashboard'}
                </button>
                <button
                    onclick={handleLogout}
                    class="w-full px-4 py-2 text-left text-sm text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-900/20 transition-colors flex items-center gap-2"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-60"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path><polyline points="16 17 21 12 16 7"></polyline><line x1="21" y1="12" x2="9" y2="12"></line></svg>
                    Logout
                </button>
            </div>
        </div>
    {/if}
</div>
