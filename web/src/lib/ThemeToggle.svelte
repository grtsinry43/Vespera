<script lang="ts">
    import { onMount } from "svelte";

    let isDark = $state(false);

    onMount(() => {
        // Check localStorage or system preference
        if (
            localStorage.theme === "dark" ||
            (!("theme" in localStorage) &&
                window.matchMedia("(prefers-color-scheme: dark)").matches)
        ) {
            setDark(true);
        } else {
            setDark(false);
        }
    });

    function setDark(value: boolean) {
        isDark = value;
        if (isDark) {
            document.documentElement.classList.add("dark");
            document.documentElement.style.colorScheme = "dark";
            localStorage.theme = "dark";
        } else {
            document.documentElement.classList.remove("dark");
            document.documentElement.style.colorScheme = "light";
            localStorage.theme = "light";
        }
    }

    function toggleTheme() {
        setDark(!isDark);
    }
</script>

<button
    onclick={toggleTheme}
    class="p-2 rounded-lg bg-white dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors border border-zinc-200 dark:border-zinc-700 shadow-sm"
    aria-label="Toggle theme"
>
    {#if isDark}
        <!-- Sun Icon -->
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
    {:else}
        <!-- Moon Icon -->
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
            ><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" /></svg
        >
    {/if}
</button>
