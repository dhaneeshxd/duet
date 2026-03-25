<script lang="ts">
    import { onMount } from "svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import PlayerBar from "$lib/components/PlayerBar.svelte";
    import FullscreenPlayer from "$lib/components/FullscreenPlayer.svelte";
    import TitleBar from "$lib/components/TitleBar.svelte";
    import {
        loadTracks,
        loadPlaylists,
        loadFolders,
    } from "$lib/stores/library";

    onMount(async () => {
        await loadTracks();
        await loadPlaylists();
        await loadFolders();
    });
</script>

<div class="app">
    <TitleBar />
    <div class="layout">
        <Sidebar />
        <main class="content">
            <slot />
        </main>
    </div>
    <PlayerBar />
</div>
<FullscreenPlayer />

<style>
    :global(*, *::before, *::after) {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }
    :global(html) { height: 100%; }
    :global(body) {
        height: 100%;
        overflow: hidden;
        background: #0a0a0a;
        color: #e8e8e8;
        font-family: "Segoe UI", system-ui, sans-serif;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
    }
    :global(button) {
        transition:
            opacity 0.15s,
            transform 0.15s,
            background 0.15s,
            color 0.15s,
            box-shadow 0.15s;
    }
    :global(:focus-visible) {
        outline: 2px solid #f59e0b;
        outline-offset: 2px;
    }
    :global(::-webkit-scrollbar)       { width: 5px; }
    :global(::-webkit-scrollbar-track) { background: transparent; }
    :global(::-webkit-scrollbar-thumb) { background: #222; border-radius: 3px; }
    :global(::-webkit-scrollbar-thumb:hover) { background: #333; }

    .app {
        display: flex;
        flex-direction: column;
        height: 100vh;
        overflow: hidden;
    }

    /* TitleBar is flex-shrink:0 inside itself, so it takes its 36px */

    .layout {
        display: flex;
        flex-direction: row;
        flex: 1;
        min-height: 0;   /* crucial — lets children shrink below content size */
        overflow: hidden;
    }

    .content {
        flex: 1;
        min-height: 0;
        min-width: 0;
        overflow-y: auto;
        padding: 0;
    }
</style>
