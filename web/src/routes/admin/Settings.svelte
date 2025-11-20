<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import AdminLayout from "../../lib/admin/AdminLayout.svelte";

    let activeTab = "general";

    const tabs = [
        { id: "general", label: "General" },
        { id: "appearance", label: "Appearance" },
        { id: "notifications", label: "Notifications" },
        { id: "security", label: "Security" },
        { id: "system", label: "System" },
    ];
</script>

<AdminLayout>
    <div class="max-w-7xl mx-auto p-8 md:p-12 animate-in fade-in duration-500">
        <div class="mb-12">
            <h1
                class="text-3xl font-bold text-zinc-900 dark:text-white tracking-tight mb-2"
            >
                Settings
            </h1>
            <p class="text-zinc-500 dark:text-zinc-400">
                Manage your workspace preferences and system configurations.
            </p>
        </div>

        <div class="flex flex-col lg:flex-row gap-12">
            <!-- Minimal Sidebar -->
            <nav class="w-full lg:w-48 shrink-0 space-y-1">
                {#each tabs as tab}
                    <button
                        onclick={() => (activeTab = tab.id)}
                        class="w-full text-left px-3 py-2 text-sm font-medium rounded-md transition-colors
                        {activeTab === tab.id
                            ? 'text-zinc-900 dark:text-white bg-zinc-100 dark:bg-zinc-800/50'
                            : 'text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white hover:bg-zinc-50 dark:hover:bg-zinc-800/30'}"
                    >
                        {tab.label}
                    </button>
                {/each}
            </nav>

            <!-- Content -->
            <div class="flex-1 min-w-0">
                {#key activeTab}
                    <div
                        in:fly={{ y: 10, duration: 300, delay: 50 }}
                        out:fade={{ duration: 100 }}
                        class="space-y-12"
                    >
                        {#if activeTab === "general"}
                            <section class="space-y-6 max-w-2xl">
                                <div>
                                    <h2
                                        class="text-lg font-medium text-zinc-900 dark:text-white"
                                    >
                                        Workspace Profile
                                    </h2>
                                    <p
                                        class="text-sm text-zinc-500 dark:text-zinc-400 mt-1"
                                    >
                                        This information will be displayed
                                        publicly.
                                    </p>
                                </div>

                                <div class="space-y-6">
                                    <div class="grid gap-2">
                                        <label
                                            for="ws-name"
                                            class="text-sm font-medium text-zinc-700 dark:text-zinc-300"
                                            >Workspace Name</label
                                        >
                                        <input
                                            id="ws-name"
                                            type="text"
                                            value="Vespera Production"
                                            class="w-full px-0 py-2 bg-transparent border-b border-zinc-200 dark:border-zinc-800 text-zinc-900 dark:text-white focus:outline-none focus:border-zinc-900 dark:focus:border-white transition-colors placeholder-zinc-400"
                                        />
                                    </div>

                                    <div class="grid gap-2">
                                        <label
                                            for="ws-url"
                                            class="text-sm font-medium text-zinc-700 dark:text-zinc-300"
                                            >Workspace URL</label
                                        >
                                        <div
                                            class="flex items-center border-b border-zinc-200 dark:border-zinc-800 focus-within:border-zinc-900 dark:focus-within:border-white transition-colors"
                                        >
                                            <span
                                                class="py-2 text-zinc-500 dark:text-zinc-500 select-none"
                                                >vespera.app/</span
                                            >
                                            <input
                                                id="ws-url"
                                                type="text"
                                                value="production"
                                                class="flex-1 px-0 py-2 bg-transparent border-none focus:ring-0 text-zinc-900 dark:text-white placeholder-zinc-400"
                                            />
                                        </div>
                                    </div>
                                </div>
                            </section>

                            <section class="space-y-6 max-w-2xl">
                                <div>
                                    <h2
                                        class="text-lg font-medium text-zinc-900 dark:text-white"
                                    >
                                        Region & Language
                                    </h2>
                                    <p
                                        class="text-sm text-zinc-500 dark:text-zinc-400 mt-1"
                                    >
                                        Set your preferred language and
                                        timezone.
                                    </p>
                                </div>

                                <div
                                    class="grid grid-cols-1 md:grid-cols-2 gap-8"
                                >
                                    <div class="grid gap-2">
                                        <label
                                            for="lang"
                                            class="text-sm font-medium text-zinc-700 dark:text-zinc-300"
                                            >Language</label
                                        >
                                        <select
                                            id="lang"
                                            class="w-full px-0 py-2 bg-transparent border-b border-zinc-200 dark:border-zinc-800 text-zinc-900 dark:text-white focus:outline-none focus:border-zinc-900 dark:focus:border-white transition-colors cursor-pointer"
                                        >
                                            <option>English</option>
                                            <option>中文 (简体)</option>
                                            <option>日本語</option>
                                        </select>
                                    </div>
                                    <div class="grid gap-2">
                                        <label
                                            for="timezone"
                                            class="text-sm font-medium text-zinc-700 dark:text-zinc-300"
                                            >Timezone</label
                                        >
                                        <select
                                            id="timezone"
                                            class="w-full px-0 py-2 bg-transparent border-b border-zinc-200 dark:border-zinc-800 text-zinc-900 dark:text-white focus:outline-none focus:border-zinc-900 dark:focus:border-white transition-colors cursor-pointer"
                                        >
                                            <option
                                                >UTC (Coordinated Universal
                                                Time)</option
                                            >
                                            <option
                                                >PST (Pacific Standard Time)</option
                                            >
                                            <option
                                                >EST (Eastern Standard Time)</option
                                            >
                                        </select>
                                    </div>
                                </div>
                            </section>
                        {:else if activeTab === "appearance"}
                            <section class="space-y-6">
                                <div>
                                    <h2
                                        class="text-lg font-medium text-zinc-900 dark:text-white"
                                    >
                                        Interface Theme
                                    </h2>
                                    <p
                                        class="text-sm text-zinc-500 dark:text-zinc-400 mt-1"
                                    >
                                        Customize how Vespera looks on your
                                        device.
                                    </p>
                                </div>

                                <div class="grid grid-cols-3 gap-6 max-w-3xl">
                                    {#each ["Light", "Dark", "System"] as theme}
                                        <button class="group text-left">
                                            <div
                                                class="w-full aspect-video bg-zinc-100 dark:bg-zinc-900 rounded-lg mb-3 border border-zinc-200 dark:border-zinc-800 group-hover:border-zinc-400 dark:group-hover:border-zinc-600 transition-all overflow-hidden relative"
                                            >
                                                <!-- Mock UI inside -->
                                                <div
                                                    class="absolute top-2 left-2 right-2 h-2 bg-zinc-200 dark:bg-zinc-800 rounded-sm"
                                                ></div>
                                                <div
                                                    class="absolute top-6 left-2 w-8 h-16 bg-zinc-200 dark:bg-zinc-800 rounded-sm"
                                                ></div>
                                                <div
                                                    class="absolute top-6 left-12 right-2 bottom-2 bg-white dark:bg-black rounded-sm border border-zinc-200 dark:border-zinc-800"
                                                ></div>
                                            </div>
                                            <span
                                                class="text-sm font-medium text-zinc-900 dark:text-white block"
                                                >{theme}</span
                                            >
                                        </button>
                                    {/each}
                                </div>
                            </section>

                            <section class="space-y-6">
                                <div>
                                    <h2
                                        class="text-lg font-medium text-zinc-900 dark:text-white"
                                    >
                                        Density
                                    </h2>
                                    <p
                                        class="text-sm text-zinc-500 dark:text-zinc-400 mt-1"
                                    >
                                        Adjust the compactness of the interface.
                                    </p>
                                </div>

                                <div class="flex items-center gap-6">
                                    <label
                                        class="flex items-center gap-3 text-sm text-zinc-900 dark:text-white cursor-pointer group"
                                    >
                                        <div
                                            class="w-4 h-4 rounded-full border border-zinc-300 dark:border-zinc-600 group-hover:border-zinc-500 flex items-center justify-center"
                                        >
                                            <div
                                                class="w-2 h-2 rounded-full bg-zinc-900 dark:bg-white"
                                            ></div>
                                        </div>
                                        Default
                                    </label>
                                    <label
                                        class="flex items-center gap-3 text-sm text-zinc-500 dark:text-zinc-400 cursor-pointer group"
                                    >
                                        <div
                                            class="w-4 h-4 rounded-full border border-zinc-300 dark:border-zinc-600 group-hover:border-zinc-500 flex items-center justify-center"
                                        ></div>
                                        Compact
                                    </label>
                                </div>
                            </section>
                        {:else if activeTab === "notifications"}
                            <section class="space-y-6 max-w-2xl">
                                <div>
                                    <h2
                                        class="text-lg font-medium text-zinc-900 dark:text-white"
                                    >
                                        Email Notifications
                                    </h2>
                                    <p
                                        class="text-sm text-zinc-500 dark:text-zinc-400 mt-1"
                                    >
                                        Choose what you want to be notified
                                        about.
                                    </p>
                                </div>

                                <div
                                    class="space-y-0 divide-y divide-zinc-100 dark:divide-zinc-800/50"
                                >
                                    {#each ["Security alerts", "System updates", "Maintenance windows", "Weekly reports"] as item}
                                        <div
                                            class="flex items-center justify-between py-4"
                                        >
                                            <span
                                                class="text-sm text-zinc-700 dark:text-zinc-300"
                                                >{item}</span
                                            >
                                            <button
                                                class="w-10 h-5 bg-zinc-200 dark:bg-zinc-700 rounded-full relative transition-colors hover:bg-zinc-300 dark:hover:bg-zinc-600"
                                            >
                                                <span
                                                    class="absolute left-1 top-1 w-3 h-3 bg-white rounded-full shadow-sm transition-transform"
                                                ></span>
                                            </button>
                                        </div>
                                    {/each}
                                </div>
                            </section>
                        {:else}
                            <div class="py-20">
                                <h3
                                    class="text-lg font-medium text-zinc-900 dark:text-white mb-1"
                                >
                                    Under Construction
                                </h3>
                                <p
                                    class="text-sm text-zinc-500 dark:text-zinc-400"
                                >
                                    This section is currently being built.
                                </p>
                            </div>
                        {/if}

                        {#if activeTab !== "system" && activeTab !== "security"}
                            <div class="pt-8">
                                <button
                                    class="px-5 py-2.5 bg-zinc-900 dark:bg-white text-white dark:text-zinc-900 text-sm font-medium rounded-md hover:bg-zinc-800 dark:hover:bg-zinc-100 transition-colors"
                                >
                                    Save Changes
                                </button>
                            </div>
                        {/if}
                    </div>
                {/key}
            </div>
        </div>
    </div>
</AdminLayout>
