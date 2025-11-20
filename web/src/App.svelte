<script lang="ts">
    import { onMount } from "svelte";
    import Router from "svelte-spa-router";
    import { link, push, location } from "svelte-spa-router";
    import ThemeToggle from "./lib/ThemeToggle.svelte";
    import UserMenu from "./lib/UserMenu.svelte";
    import { authStore, isAuthenticated } from "./lib/authStore";
    import { routes } from "./lib/router";

    // State
    let authInitialized = $state(false);

    // Global stats (placeholder for now, can be moved to a store)
    let globalStats = $state({
        active: 0,
        total: 0,
        traffic_in: "0",
        traffic_out: "0",
    });

    onMount(() => {
        // 初始化认证状态
        authStore.init().finally(() => {
            authInitialized = true;
        });
    });

    async function handleLogout() {
        await authStore.logout();
        push("/login");
    }
</script>

{#if !authInitialized}
    <div
        class="flex items-center justify-center min-h-screen bg-[#fafafa] dark:bg-[#09090b] text-zinc-500"
    >
        Loading...
    </div>
{:else if $location && $location.startsWith("/admin")}
    <Router {routes} />
{:else}
    <div
        class="min-h-screen pb-20 relative overflow-x-hidden bg-[#fafafa] dark:bg-[#09090b] text-zinc-900 dark:text-zinc-300 transition-colors duration-300"
    >
        <!-- Subtle Ambient Background -->
        <div class="fixed inset-0 pointer-events-none">
            <!-- Light Mode Glow -->
            <div
                class="absolute top-[-10%] right-[-5%] w-[40%] h-[40%] bg-indigo-200/30 blur-[100px] rounded-full dark:hidden"
            ></div>
            <div
                class="absolute bottom-[-10%] left-[-5%] w-[40%] h-[40%] bg-emerald-200/30 blur-[100px] rounded-full dark:hidden"
            ></div>

            <!-- Dark Mode Glow -->
            <div
                class="absolute top-[-20%] left-[-10%] w-[50%] h-[50%] bg-indigo-900/10 blur-[120px] rounded-full hidden dark:block"
            ></div>
            <div
                class="absolute bottom-[-20%] right-[-10%] w-[50%] h-[50%] bg-emerald-900/10 blur-[120px] rounded-full hidden dark:block"
            ></div>
        </div>

        <!-- Navbar -->
        <nav
            class="sticky top-0 z-30 border-b border-black/5 dark:border-zinc-800 bg-white/80 dark:bg-[#09090b]/80 backdrop-blur-2xl supports-[backdrop-filter]:bg-white/60"
        >
            <div
                class="max-w-7xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between"
            >
                <a
                    href="/"
                    use:link
                    class="flex items-center gap-3 cursor-pointer group"
                >
                    <!-- Logo: Just a simple dot -->
                    <div
                        class="w-1.5 h-1.5 rounded-full bg-zinc-900 dark:bg-white shadow-[0_0_10px_rgba(0,0,0,0.1)] group-hover:scale-125 transition-transform duration-300"
                    ></div>
                    <span
                        class="text-sm font-bold tracking-tight text-zinc-900 dark:text-white"
                    >
                        Vespera <span
                            class="text-zinc-400 dark:text-zinc-500 font-normal ml-1"
                            >by grtsinry43</span
                        >
                    </span>
                </a>

                <div class="flex items-center gap-4 sm:gap-6">
                    <!-- Desktop Stats (始终显示) -->
                    <div
                        class="hidden md:flex items-center gap-8 text-xs font-medium text-zinc-500"
                    >
                        <div class="flex items-center gap-2">
                            <span class="relative flex h-2 w-2">
                                <span
                                    class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"
                                ></span>
                                <span
                                    class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"
                                ></span>
                            </span>
                            <span class="text-zinc-700 dark:text-zinc-200"
                                >{globalStats.active} / {globalStats.total}</span
                            >
                            <span class="text-zinc-400">Online</span>
                        </div>
                        <div
                            class="h-4 w-px bg-zinc-200 dark:bg-zinc-800"
                        ></div>
                        <div class="flex items-center gap-4">
                            <div class="flex items-center gap-1.5">
                                <span
                                    class="text-zinc-700 dark:text-zinc-200 font-mono"
                                    >{globalStats.traffic_in}</span
                                >
                                <span
                                    class="text-[10px] uppercase tracking-wider"
                                    >Mbps In</span
                                >
                            </div>
                            <div class="flex items-center gap-1.5">
                                <span
                                    class="text-zinc-700 dark:text-zinc-200 font-mono"
                                    >{globalStats.traffic_out}</span
                                >
                                <span
                                    class="text-[10px] uppercase tracking-wider"
                                    >Mbps Out</span
                                >
                            </div>
                        </div>
                    </div>

                    <div
                        class="h-4 w-px bg-zinc-200 dark:bg-zinc-800 hidden md:block"
                    ></div>

                    <ThemeToggle />

                    <!-- User Menu or Login Button -->
                    {#if $isAuthenticated}
                        <UserMenu onLogout={handleLogout} />
                    {:else}
                        <a
                            href="/login"
                            use:link
                            class="px-3 py-1.5 text-xs font-medium text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800 rounded-lg transition-colors"
                        >
                            Login
                        </a>
                    {/if}
                </div>
            </div>
        </nav>

        <!-- Main Content -->
        <main class="relative z-10">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 py-8 sm:py-12">
                <Router {routes} />
            </div>
        </main>
    </div>
{/if}

<style>
    /* Custom scrollbar for webkit */
    :global(::-webkit-scrollbar) {
        width: 6px;
    }

    :global(::-webkit-scrollbar-track) {
        background: transparent;
    }

    :global(::-webkit-scrollbar-thumb) {
        background: #d4d4d8;
        border-radius: 3px;
    }

    :global(.dark ::-webkit-scrollbar-thumb) {
        background: #27272a;
    }

    :global(::-webkit-scrollbar-thumb:hover) {
        background: #a1a1aa;
    }

    :global(.dark ::-webkit-scrollbar-thumb:hover) {
        background: #3f3f46;
    }
</style>
