<script lang="ts">
    import { onMount } from "svelte";
    import { scale } from "svelte/transition";

    type Theme = "light" | "dark" | "system";

    let theme = $state<Theme>("system");
    let isOpen = $state(false);
    let menuRef: HTMLElement;

    function updateTheme(newTheme: Theme) {
        theme = newTheme;
        localStorage.setItem("theme", newTheme);

        if (newTheme === "system") {
            if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
                document.documentElement.classList.add("dark");
                document.documentElement.style.colorScheme = "dark";
            } else {
                document.documentElement.classList.remove("dark");
                document.documentElement.style.colorScheme = "light";
            }
        } else if (newTheme === "dark") {
            document.documentElement.classList.add("dark");
            document.documentElement.style.colorScheme = "dark";
        } else {
            document.documentElement.classList.remove("dark");
            document.documentElement.style.colorScheme = "light";
        }
    }

    onMount(() => {
        // Initialize theme
        const savedTheme = localStorage.getItem("theme") as Theme | null;
        if (savedTheme) {
            theme = savedTheme;
        } else {
            theme = "system";
        }
        updateTheme(theme);

        // System preference listener
        const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
        const handleChange = (e: MediaQueryListEvent) => {
            if (theme === "system") {
                if (e.matches) {
                    document.documentElement.classList.add("dark");
                    document.documentElement.style.colorScheme = "dark";
                } else {
                    document.documentElement.classList.remove("dark");
                    document.documentElement.style.colorScheme = "light";
                }
            }
        };

        mediaQuery.addEventListener("change", handleChange);

        // Click outside to close
        const handleClickOutside = (event: MouseEvent) => {
            if (menuRef && !menuRef.contains(event.target as Node)) {
                isOpen = false;
            }
        };
        document.addEventListener("click", handleClickOutside);

        return () => {
            mediaQuery.removeEventListener("change", handleChange);
            document.removeEventListener("click", handleClickOutside);
        };
    });

    function selectTheme(t: Theme) {
        updateTheme(t);
        isOpen = false;
    }
</script>

<div class="relative" bind:this={menuRef}>
    <button
        onclick={() => (isOpen = !isOpen)}
        class="p-2 rounded-lg text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-100 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-all duration-200"
        aria-label="Theme settings"
    >
        {#if theme === "light"}
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
                ><circle cx="12" cy="12" r="5" /><path d="M12 1v2" /><path
                    d="M12 21v2"
                /><path d="M4.22 4.22l1.42 1.42" /><path
                    d="M18.36 18.36l1.42 1.42"
                /><path d="M1 12h2" /><path d="M21 12h2" /><path
                    d="M4.22 19.78l1.42-1.42"
                /><path d="M18.36 5.64l1.42-1.42" /></svg
            >
        {:else if theme === "dark"}
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
                ><path
                    d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"
                /></svg
            >
        {:else}
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
                ><rect x="2" y="3" width="20" height="14" rx="2" ry="2" /><line
                    x1="8"
                    y1="21"
                    x2="16"
                    y2="21"
                /><line x1="12" y1="17" x2="12" y2="21" /></svg
            >
        {/if}
    </button>

    {#if isOpen}
        <div
            transition:scale={{ duration: 150, start: 0.95 }}
            class="absolute right-0 mt-2 w-36 bg-white dark:bg-zinc-900 rounded-xl shadow-xl border border-zinc-100 dark:border-zinc-800 overflow-hidden z-50 py-1"
        >
            <button
                onclick={() => selectTheme("light")}
                class="w-full px-4 py-2 text-left text-sm flex items-center gap-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors {theme ===
                'light'
                    ? 'text-emerald-600 dark:text-emerald-400'
                    : 'text-zinc-600 dark:text-zinc-400'}"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><circle cx="12" cy="12" r="5" /><path d="M12 1v2" /><path
                        d="M12 21v2"
                    /><path d="M4.22 4.22l1.42 1.42" /><path
                        d="M18.36 18.36l1.42 1.42"
                    /><path d="M1 12h2" /><path d="M21 12h2" /><path
                        d="M4.22 19.78l1.42-1.42"
                    /><path d="M18.36 5.64l1.42-1.42" /></svg
                >
                Light
            </button>
            <button
                onclick={() => selectTheme("dark")}
                class="w-full px-4 py-2 text-left text-sm flex items-center gap-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors {theme ===
                'dark'
                    ? 'text-emerald-600 dark:text-emerald-400'
                    : 'text-zinc-600 dark:text-zinc-400'}"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path
                        d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"
                    /></svg
                >
                Dark
            </button>
            <button
                onclick={() => selectTheme("system")}
                class="w-full px-4 py-2 text-left text-sm flex items-center gap-3 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 transition-colors {theme ===
                'system'
                    ? 'text-emerald-600 dark:text-emerald-400'
                    : 'text-zinc-600 dark:text-zinc-400'}"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><rect
                        x="2"
                        y="3"
                        width="20"
                        height="14"
                        rx="2"
                        ry="2"
                    /><line x1="8" y1="21" x2="16" y2="21" /><line
                        x1="12"
                        y1="17"
                        x2="12"
                        y2="21"
                    /></svg
                >
                System
            </button>
        </div>
    {/if}
</div>
