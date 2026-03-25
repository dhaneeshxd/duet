<script lang="ts">
    import { open } from "@tauri-apps/plugin-dialog";
    import { slide } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";
    import {
        playlists,
        selectedPlaylist,
        activeView,
        folders,
        isScanning,
        createPlaylist,
        deletePlaylist,
        loadPlaylistTracks,
        scanFolder,
        removeFolder,
        rescanAll,
        loadTracks,
        clearSearch,
    } from "$lib/stores/library";

    let newPlaylistName = "";
    let showInput = false;
    let showFolders = true;

    function handleLibrary() {
        activeView.set("library");
        selectedPlaylist.set(null);
    }

    async function handleScan() {
        const selected = await open({ directory: true, multiple: false });
        if (selected && typeof selected === "string") {
            await scanFolder(selected);
        }
    }

    async function handleCreatePlaylist() {
        if (newPlaylistName.trim()) {
            await createPlaylist(newPlaylistName.trim());
            newPlaylistName = "";
            showInput = false;
        }
    }

    async function handleSelectPlaylist(playlist: typeof $selectedPlaylist) {
        selectedPlaylist.set(playlist);
        activeView.set("playlist");
        if (playlist) await loadPlaylistTracks(playlist.id);
    }

    async function handleRescan(path: string) {
        await scanFolder(path);
    }

    async function handleRescanAll() {
        await rescanAll();
    }

    async function handleClearAll() {
        if (!confirm("Clear all tracks from library? Folders will remain."))
            return;
        try {
            await invoke("clear_library");
            await loadTracks();
        } catch (e) {
            console.error(e);
        }
    }
</script>

<aside class="sidebar">


    <nav>
        <button
            class="nav-item"
            class:active={$activeView === "library"}
            on:click={handleLibrary}
        >
            <span class="nav-icon">♫</span>
            Library
        </button>
        <button
            class="nav-item"
            class:active={$activeView === "albums"}
            on:click={() => {
              clearSearch();
              activeView.set("albums");
            }}
        >
            <span class="nav-icon">◉</span>
            Albums
        </button>
    </nav>

    <div class="divider"></div>

    <!-- Music Folders -->
    <div
        class="section-header clickable"
        on:click={() => (showFolders = !showFolders)}
        role="button"
        tabindex="0"
        on:keydown={(e) => e.key === "Enter" && (showFolders = !showFolders)}
    >
        <span>Music Folders</span>
        <span class="chevron" class:open={showFolders}>›</span>
    </div>

    {#if showFolders}
        <div class="folder-section" transition:slide={{ duration: 200 }}>
            <button
                class="add-folder-btn"
                on:click={handleScan}
                disabled={$isScanning}
            >
                {$isScanning ? "⟳ Scanning..." : "+ Add Folder"}
            </button>

            {#each $folders as folder (folder.id)}
                <div class="folder-item">
                    <span class="folder-path" title={folder.path}>
                        {folder.path.split("\\").pop() ||
                            folder.path.split("/").pop() ||
                            folder.path}
                    </span>
                    <div class="folder-actions">
                        <button
                            class="folder-action-btn"
                            title="Rescan this folder"
                            on:click={() => handleRescan(folder.path)}
                            disabled={$isScanning}>↺</button
                        >
                        <button
                            class="folder-action-btn remove"
                            title="Remove folder"
                            on:click={() => removeFolder(folder.id)}>✕</button
                        >
                    </div>
                </div>
            {/each}

            {#if $folders.length === 0}
                <p class="folder-empty">No folders added yet</p>
            {/if}

            {#if $folders.length > 0}
                <div class="folder-footer">
                    <button
                        class="footer-btn"
                        on:click={handleRescanAll}
                        disabled={$isScanning}
                        title="Rescan all folders for new files"
                    >
                        ↺ Refresh
                    </button>
                    <button
                        class="footer-btn danger"
                        on:click={handleClearAll}
                        title="Remove all tracks from library"
                    >
                        ✕ Clear
                    </button>
                </div>
            {/if}
        </div>
    {/if}

    <div class="divider"></div>

    <!-- Playlists -->
    <div class="section-header">
        <span>Playlists</span>
        <button class="add-btn" on:click={() => (showInput = !showInput)}>
            {showInput ? "✕" : "+"}
        </button>
    </div>

    {#if showInput}
        <div class="playlist-input" transition:slide={{ duration: 180 }}>
            <input
                bind:value={newPlaylistName}
                placeholder="Playlist name..."
                autofocus
                on:keydown={(e) => e.key === "Enter" && handleCreatePlaylist()}
            />
            <button on:click={handleCreatePlaylist}>✓</button>
        </div>
    {/if}

    <div class="playlist-list">
        {#each $playlists as playlist (playlist.id)}
            <div
                class="playlist-item"
                class:active={$selectedPlaylist?.id === playlist.id}
            >
                <button
                    class="playlist-name"
                    on:click={() => handleSelectPlaylist(playlist)}
                >
                    ♬ {playlist.name}
                </button>
                <button
                    class="delete-btn"
                    on:click={() => deletePlaylist(playlist.id)}
                    title="Delete playlist">✕</button
                >
            </div>
        {/each}

        {#if $playlists.length === 0}
            <p class="playlist-empty">No playlists yet</p>
        {/if}
    </div>
</aside>

<style>
    .sidebar {
        width: 220px;
        min-width: 220px;
        background: #0f0f0f;
        display: flex;
        flex-direction: column;
        border-right: 1px solid #1a1a1a;
        overflow-y: auto;
    }

    .sidebar::-webkit-scrollbar {
        width: 4px;
    }
    .sidebar::-webkit-scrollbar-thumb {
        background: #1e1e1e;
        border-radius: 2px;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 20px 16px 16px;
        border-bottom: 1px solid #1a1a1a;
        flex-shrink: 0;
    }

    .logo-icon {
        font-size: 22px;
        color: #f59e0b;
    }
    .logo-text {
        font-size: 18px;
        font-weight: 700;
        color: #fff;
        letter-spacing: 0.5px;
    }

    nav {
        display: flex;
        flex-direction: column;
        padding: 8px;
        gap: 2px;
    }

    .nav-item {
        width: 100%;
        text-align: left;
        background: none;
        border: none;
        color: #555;
        padding: 8px 10px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 13px;
        display: flex;
        align-items: center;
        gap: 8px;
        transition: all 0.15s;
    }

    .nav-item:hover {
        background: #161616;
        color: #ccc;
    }
    .nav-item.active {
        background: #1e1e1e;
        color: #f59e0b;
        font-weight: 600;
    }

    .nav-icon {
        font-size: 14px;
        width: 16px;
        text-align: center;
    }

    .divider {
        height: 1px;
        background: #161616;
        margin: 4px 0;
        flex-shrink: 0;
    }

    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 16px 4px;
        font-size: 10px;
        font-weight: 700;
        color: #3a3a3a;
        text-transform: uppercase;
        letter-spacing: 1.5px;
        flex-shrink: 0;
    }

    .section-header.clickable {
        cursor: pointer;
        user-select: none;
        transition: color 0.15s;
    }

    .section-header.clickable:hover {
        color: #666;
    }

    .chevron {
        font-size: 16px;
        transition: transform 0.2s;
        display: inline-block;
        color: #3a3a3a;
    }

    .chevron.open {
        transform: rotate(90deg);
    }

    /* Folder section */
    .folder-section {
        padding: 4px 8px 8px;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .add-folder-btn {
        width: 100%;
        background: transparent;
        border: 1px dashed #242424;
        color: #555;
        padding: 7px 10px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 12px;
        text-align: left;
        transition: all 0.15s;
    }

    .add-folder-btn:hover:not(:disabled) {
        border-color: #f59e0b;
        color: #f59e0b;
        background: rgba(245, 158, 11, 0.05);
    }

    .add-folder-btn:disabled {
        opacity: 0.4;
        cursor: not-allowed;
    }

    .folder-item {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 5px 8px;
        border-radius: 6px;
        background: #141414;
        transition: background 0.15s;
    }

    .folder-item:hover {
        background: #1a1a1a;
    }

    .folder-path {
        flex: 1;
        font-size: 11px;
        color: #555;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .folder-actions {
        display: flex;
        gap: 2px;
        opacity: 0;
        transition: opacity 0.15s;
        flex-shrink: 0;
    }

    .folder-item:hover .folder-actions {
        opacity: 1;
    }

    .folder-action-btn {
        background: none;
        border: none;
        color: #555;
        cursor: pointer;
        padding: 3px 5px;
        border-radius: 4px;
        font-size: 12px;
        transition: all 0.15s;
        line-height: 1;
    }

    .folder-action-btn:hover:not(:disabled) {
        color: #fff;
        background: #2a2a2a;
    }
    .folder-action-btn.remove:hover {
        color: #ff5555 !important;
        background: rgba(255, 68, 68, 0.1) !important;
    }
    .folder-action-btn:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }

    .folder-empty {
        font-size: 11px;
        color: #2a2a2a;
        padding: 4px 8px;
    }

    .folder-footer {
        display: flex;
        gap: 6px;
        margin-top: 2px;
    }

    .footer-btn {
        flex: 1;
        background: #141414;
        border: 1px solid #1e1e1e;
        color: #555;
        padding: 6px 8px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 11px;
        transition: all 0.15s;
        text-align: center;
        white-space: nowrap;
    }

    .footer-btn:hover:not(:disabled) {
        border-color: #f59e0b;
        color: #f59e0b;
        background: rgba(245, 158, 11, 0.05);
    }

    .footer-btn.danger:hover:not(:disabled) {
        border-color: #ff5555;
        color: #ff5555;
        background: rgba(255, 68, 68, 0.05);
    }

    .footer-btn:disabled {
        opacity: 0.4;
        cursor: not-allowed;
    }

    /* Playlist section */
    .add-btn {
        background: none;
        border: none;
        color: #3a3a3a;
        font-size: 18px;
        cursor: pointer;
        padding: 0 4px;
        border-radius: 4px;
        transition: all 0.15s;
        line-height: 1;
    }

    .add-btn:hover {
        color: #f59e0b;
    }

    .playlist-input {
        display: flex;
        gap: 6px;
        padding: 4px 8px 8px;
    }

    .playlist-input input {
        flex: 1;
        background: #141414;
        border: 1px solid #242424;
        border-radius: 6px;
        color: #e8e8e8;
        padding: 7px 10px;
        font-size: 12px;
        outline: none;
        transition: border-color 0.15s;
    }

    .playlist-input input:focus {
        border-color: #f59e0b;
    }

    .playlist-input button {
        background: #f59e0b;
        border: none;
        border-radius: 6px;
        color: #000;
        padding: 7px 10px;
        cursor: pointer;
        font-weight: 700;
        font-size: 13px;
        transition: background 0.15s;
    }

    .playlist-input button:hover {
        background: #fbbf24;
    }

    .playlist-list {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 0 8px 8px;
        overflow-y: auto;
        flex: 1;
    }

    .playlist-empty {
        font-size: 11px;
        color: #2a2a2a;
        padding: 4px 8px;
    }

    .playlist-item {
        display: flex;
        align-items: center;
        border-radius: 6px;
        transition: background 0.12s;
    }

    .playlist-item:hover {
        background: #161616;
    }
    .playlist-item.active {
        background: #1e1e1e;
    }

    .playlist-name {
        flex: 1;
        text-align: left;
        background: none;
        border: none;
        color: #555;
        padding: 7px 10px;
        font-size: 13px;
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        transition: color 0.12s;
    }

    .playlist-item:hover .playlist-name {
        color: #bbb;
    }
    .playlist-item.active .playlist-name {
        color: #f59e0b;
    }

    .delete-btn {
        background: none;
        border: none;
        color: transparent;
        font-size: 10px;
        padding: 7px 8px;
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.12s;
        flex-shrink: 0;
    }

    .playlist-item:hover .delete-btn {
        color: #3a3a3a;
    }
    .delete-btn:hover {
        color: #ff5555 !important;
        background: rgba(255, 68, 68, 0.1);
    }
</style>
