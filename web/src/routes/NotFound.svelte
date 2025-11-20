<script lang="ts">
    import { link } from "svelte-spa-router";
</script>

<div
    class="flex items-center justify-center min-h-[70vh] w-full px-4 relative overflow-hidden"
>
    <div class="text-center">
        <div class="text-6xl mb-6">404</div>
        <h1
            class="text-2xl font-bold text-zinc-900 dark:text-white tracking-tight mb-2"
        >
            Page Not Found
        </h1>
        <p class="text-sm text-zinc-500 mb-8">
            The page you're looking for doesn't exist.
        </p>
        <a
            href="/"
            use:link
            class="inline-block px-6 py-3 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 font-medium text-sm rounded-lg hover:bg-zinc-800 dark:hover:bg-zinc-200 transition-colors"
        >
            Go Home
        </a>
    </div>
</div>
