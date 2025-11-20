<script lang="ts">
    import { pop } from "svelte-spa-router";
</script>

<div class="space-y-8 animate-in fade-in duration-500">
    <!-- Navigation -->
    <button
        onclick={() => pop()}
        class="flex items-center gap-2 text-sm font-medium text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors"
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
            stroke-linejoin="round"><path d="m15 18-6-6 6-6" /></svg
        >
        Back to Admin Panel
    </button>

    <div
        class="bg-white/60 dark:bg-[#18181b]/80 backdrop-blur-xl border border-black/5 dark:border-zinc-800 rounded-xl p-8 shadow-[0_8px_30px_rgba(0,0,0,0.04)] dark:shadow-none"
    >
        <h1
            class="text-2xl font-bold text-zinc-900 dark:text-white tracking-tight mb-2"
        >
            User Management
        </h1>
        <p class="text-sm text-zinc-500 mb-8">
            Manage user accounts and permissions
        </p>

        <div class="text-center py-20">
            <p class="text-zinc-500">Coming Soon</p>
            <p class="text-sm text-zinc-400 mt-2">
                User management features will be available here
            </p>
        </div>
    </div>
</div>
