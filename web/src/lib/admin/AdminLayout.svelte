<script lang="ts">
    import { link, location } from "svelte-spa-router";
    import ThemeToggle from "../ThemeToggle.svelte";
    import { authStore } from "../authStore";

    // Check if user is admin
    const isAdmin = $derived($authStore.user?.role === 'admin');

    const navItems = [
        {
            href: "/admin",
            label: "Overview",
            icon: "M3 3v18h18V3H3zm8 16H5v-6h6v6zm0-8H5V5h6v6zm8 8h-6v-6h6v6zm0-8h-6V5h6v6z",
            adminOnly: false,
        },
        {
            href: "/admin/nodes",
            label: "Nodes",
            icon: "M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z",
            adminOnly: false,
        },
        {
            href: "/admin/services",
            label: "Services",
            icon: "M11.99 18.54l-7.37-5.73L3 14.07l9 7 9-7-1.63-1.27-7.38 5.74zM12 16l7.36-5.73L21 9l-9-7-9 7 1.63 1.27L12 16z",
            adminOnly: false,
        },
        {
            href: "/admin/users",
            label: "Users",
            icon: "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z",
            adminOnly: true,
        },
        {
            href: "/admin/settings",
            label: "Settings",
            icon: "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.488.488 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84a.484.484 0 0 0-.48.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0 .59-.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.27.41.48.41h3.84c.24 0 .44-.17.48-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z",
            adminOnly: true,
        },
    ];

    // Filter nav items based on user role
    const visibleNavItems = $derived(navItems.filter(item => !item.adminOnly || isAdmin));

    function isActive(path: string) {
        if (path === "/admin") {
            return $location === "/admin";
        }
        return $location.startsWith(path);
    }
</script>

<div class="min-h-screen bg-zinc-50 dark:bg-black flex">
    <!-- Sidebar -->
    <aside
        class="w-64 shrink-0 bg-white dark:bg-zinc-900 border-r border-zinc-200 dark:border-zinc-800 flex flex-col"
    >
        <div class="p-6 border-b border-zinc-200 dark:border-zinc-800">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <!-- Logo: Just a simple dot (Matched with Frontend) -->
                    <div
                        class="w-1.5 h-1.5 rounded-full bg-zinc-900 dark:bg-white shadow-[0_0_10px_rgba(0,0,0,0.1)]"
                    ></div>
                    <span
                        class="text-sm font-bold tracking-tight text-zinc-900 dark:text-white"
                    >
                        Vespera <span
                            class="text-zinc-400 dark:text-zinc-500 font-normal ml-1"
                            >by grtsinry43</span
                        >
                    </span>
                </div>
                <ThemeToggle />
            </div>
        </div>

        <nav class="flex-1 p-4 space-y-1">
            {#each visibleNavItems as item}
                <a
                    href={item.href}
                    use:link
                    class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all
                    {isActive(item.href)
                        ? 'bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-white'
                        : 'text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-50 dark:hover:bg-zinc-800/50'}"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                        class="opacity-70"
                    >
                        <path d={item.icon} />
                    </svg>
                    {item.label}
                </a>
            {/each}
        </nav>

        <div
            class="p-4 border-t border-zinc-200 dark:border-zinc-800 space-y-2"
        >
            <a
                href="/"
                use:link
                class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-all"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="opacity-70"
                    ><path
                        d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"
                    /><polyline points="16 17 21 12 16 7" /><line
                        x1="21"
                        x2="9"
                        y1="12"
                        y2="12"
                    /></svg
                >
                Back to App
            </a>

            <div
                class="flex items-center gap-3 px-3 py-2 pt-4 border-t border-zinc-100 dark:border-zinc-800/50"
            >
                <div
                    class="w-8 h-8 rounded-full bg-zinc-200 dark:bg-zinc-700 flex items-center justify-center text-xs font-medium text-zinc-600 dark:text-zinc-300"
                >
                    {$authStore.user?.username?.charAt(0).toUpperCase() || 'U'}
                </div>
                <div class="flex-1 min-w-0">
                    <p
                        class="text-sm font-medium text-zinc-900 dark:text-white truncate"
                    >
                        {$authStore.user?.username || 'User'}
                    </p>
                    <p
                        class="text-xs text-zinc-500 dark:text-zinc-400 truncate"
                    >
                        {$authStore.user?.role === 'admin' ? 'Administrator' : 'Team Member'}
                    </p>
                </div>
            </div>
        </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 h-screen overflow-y-auto">
        <slot></slot>
    </main>
</div>
