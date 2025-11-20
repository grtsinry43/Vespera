<script lang="ts">
    import { authStore } from "../lib/authStore";
    import { fade, fly, slide } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { push } from "svelte-spa-router";

    let mode = $state<"login" | "register">("login");
    let username = $state("");
    let password = $state("");
    let email = $state("");
    let error = $state<string | null>(null);
    let loading = $state(false);

    async function handleSubmit() {
        if (!username || !password) {
            error = "Please fill in all fields";
            return;
        }

        loading = true;
        error = null;

        try {
            if (mode === "login") {
                await authStore.login(username, password);
                // 登录成功后跳转到首页
                push('/');
            } else {
                await authStore.register(
                    username,
                    password,
                    email || undefined,
                );
                // 注册成功后跳转到首页
                push('/');
            }
        } catch (err: any) {
            error =
                err.message ||
                `${mode === "login" ? "Login" : "Registration"} failed`;
        } finally {
            loading = false;
        }
    }

    function switchMode() {
        mode = mode === "login" ? "register" : "login";
        error = null;
    }
</script>

<div
    class="flex items-center justify-center min-h-[80vh] w-full px-4 relative overflow-hidden"
>
    <!-- Technical Grid Background -->
    <div
        class="absolute inset-0 pointer-events-none"
        style="background-image: radial-gradient(circle at 1px 1px, var(--tw-prose-headings) 1px, transparent 0); background-size: 24px 24px; opacity: 0.03;"
    ></div>

    <!-- Main Card -->
    <div
        class="relative w-full max-w-[380px]"
        in:fly={{ y: 10, duration: 600, delay: 100, easing: cubicOut }}
    >
        <!-- Clean Minimalist Container -->
        <div
            class="bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-8 sm:p-10 shadow-sm"
        >
            <!-- Header -->
            <div class="mb-10">
                <div class="flex items-center gap-3 mb-6">
                    <div class="w-2 h-2 bg-zinc-900 dark:bg-white"></div>
                    <span
                        class="text-lg font-bold tracking-tight text-zinc-900 dark:text-white uppercase"
                    >
                        Vespera
                    </span>
                </div>
                <h1
                    class="text-2xl font-semibold text-zinc-900 dark:text-white mb-2 tracking-tight"
                >
                    {mode === "login" ? "Authentication" : "Registration"}
                </h1>
                <p
                    class="text-xs text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                >
                    {mode === "login"
                        ? "System Access Control"
                        : "New User Provisioning"}
                </p>
            </div>

            <!-- Error Message -->
            {#if error}
                <div
                    transition:slide={{ duration: 200, axis: "y" }}
                    class="mb-6 p-3 bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-800 rounded-md text-xs text-rose-600 dark:text-rose-400 font-mono"
                >
                    Error: {error}
                </div>
            {/if}

            <!-- Form -->
            <form
                onsubmit={(e) => {
                    e.preventDefault();
                    handleSubmit();
                }}
                class="space-y-6"
            >
                <!-- Username -->
                <div class="space-y-2">
                    <label
                        for="username"
                        class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >
                        Username
                    </label>
                    <input
                        id="username"
                        type="text"
                        bind:value={username}
                        disabled={loading}
                        autocomplete="username"
                        required
                        class="w-full px-0 py-2 bg-transparent border-b border-zinc-200 dark:border-zinc-800 text-sm text-zinc-900 dark:text-white placeholder-zinc-300 dark:placeholder-zinc-700 focus:outline-none focus:border-zinc-900 dark:focus:border-white transition-colors disabled:opacity-50 font-mono"
                        placeholder="user@system"
                    />
                </div>

                <!-- Email (Register only) -->
                {#if mode === "register"}
                    <div class="space-y-2" transition:slide={{ duration: 200 }}>
                        <label
                            for="email"
                            class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                        >
                            Email <span
                                class="text-zinc-300 dark:text-zinc-700 normal-case tracking-normal"
                                >(Optional)</span
                            >
                        </label>
                        <input
                            id="email"
                            type="email"
                            bind:value={email}
                            disabled={loading}
                            autocomplete="email"
                            class="w-full px-0 py-2 bg-transparent border-b border-zinc-200 dark:border-zinc-800 text-sm text-zinc-900 dark:text-white placeholder-zinc-300 dark:placeholder-zinc-700 focus:outline-none focus:border-zinc-900 dark:focus:border-white transition-colors disabled:opacity-50 font-mono"
                            placeholder="contact@domain"
                        />
                    </div>
                {/if}

                <!-- Password -->
                <div class="space-y-2">
                    <label
                        for="password"
                        class="block text-xs font-medium text-zinc-500 dark:text-zinc-400 uppercase tracking-wider"
                    >
                        Password
                    </label>
                    <input
                        id="password"
                        type="password"
                        bind:value={password}
                        disabled={loading}
                        autocomplete={mode === "login"
                            ? "current-password"
                            : "new-password"}
                        required
                        class="w-full px-0 py-2 bg-transparent border-b border-zinc-200 dark:border-zinc-800 text-sm text-zinc-900 dark:text-white placeholder-zinc-300 dark:placeholder-zinc-700 focus:outline-none focus:border-zinc-900 dark:focus:border-white transition-colors disabled:opacity-50 font-mono"
                        placeholder="••••••••"
                    />
                </div>

                <!-- Submit Button -->
                <button
                    type="submit"
                    disabled={loading}
                    class="w-full mt-4 py-3 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 font-medium text-xs uppercase tracking-widest hover:bg-zinc-800 dark:hover:bg-zinc-200 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                    {#if loading}
                        <span class="animate-pulse">Processing...</span>
                    {:else}
                        {mode === "login" ? "Authenticate" : "Register System"}
                    {/if}
                </button>
            </form>

            <!-- Footer -->
            <div
                class="mt-8 pt-6 border-t border-zinc-100 dark:border-zinc-900 flex justify-between items-center"
            >
                <span
                    class="text-[10px] text-zinc-400 dark:text-zinc-600 uppercase tracking-widest"
                >
                    Vespera Sys v1.0
                </span>
                <button
                    onclick={switchMode}
                    class="text-[10px] font-medium text-zinc-900 dark:text-white hover:underline uppercase tracking-widest"
                >
                    {mode === "login" ? "Create ID" : "Login"}
                </button>
            </div>
        </div>
    </div>
</div>
