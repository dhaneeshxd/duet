<script lang="ts">
    import { onMount } from 'svelte';
    import { get } from 'svelte/store';
    import { fade, fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { invoke } from "@tauri-apps/api/core";
    import SearchBar from "$lib/components/SearchBar.svelte";
    import TrackList from "$lib/components/TrackList.svelte";
    import LyricsPanel from "$lib/components/LyricsPanel.svelte";
    import {
        sortedTracks, playlistTracks,
        activeView, selectedPlaylist,
        albums, scanResult, rescanAll,
        selectedAlbum,
    } from "$lib/stores/library";
    import { isPlaying,
        position,
        seekTo,
        playTrack } from "$lib/stores/player";
    import type { Album } from "$lib/stores/library";
    onMount(() => {
        const handleDeviceChange = async () => {
            const currentlyPlaying = get(isPlaying);

            if (currentlyPlaying) {
                console.log("Audio device change detected. Re-syncing stream...");
                const currentPos = get(position);
                await seekTo(currentPos);
            }
        };

        navigator.mediaDevices.addEventListener('devicechange', handleDeviceChange);
        return () => {
            navigator.mediaDevices.removeEventListener('devicechange', handleDeviceChange);
        };
    });

    let showMenu = false;
    let albumArts: Record<string, string | null> = {};
    let fetchingArts = false;

    function closeMenu() { showMenu = false; }

    $: if ($activeView === "albums" && $albums.length > 0 && !fetchingArts) {
        fetchAllAlbumArts();
    }

    function openAlbum(album: Album) {
        selectedAlbum.set(album)
        activeView.set('album-detail' as any)
    }

    function backToAlbums() {
        activeView.set('albums')
        selectedAlbum.set(null)
    }

    async function fetchAllAlbumArts() {
        fetchingArts = true;
        for (const album of $albums) {
            const key = (album.name || "Unknown Album").toLowerCase().trim();
            if (albumArts[key] !== undefined) continue;
            if (album.tracks.length === 0) { albumArts[key] = null; continue; }
            try {
                const art = await invoke<string | null>("get_album_art", {
                    path: album.tracks[0].path,
                });
                albumArts[key] = art;
                albumArts = { ...albumArts };
            } catch {
                albumArts[key] = null;
            }
        }
        fetchingArts = false;
    }

    $: albumDetailArt = $selectedAlbum
        ? albumArts[($selectedAlbum.name || 'Unknown Album').toLowerCase().trim()]
        : null
</script>

<svelte:window on:click={closeMenu} />

<div class="page">
    <div class="main">
        <!-- Top bar -->
        <div class="top-bar">
            <div class="breadcrumb">
                <button class="crumb" on:click={() => {
                    activeView.set("library");
                    selectedPlaylist.set(null);
                    selectedAlbum.set(null);
                }}>Library</button>

                {#if $activeView === "albums"}
                    <span class="crumb-sep">/</span>
                    <span class="crumb active">Albums</span>
                {:else if $activeView === ("album-detail" as any) && $selectedAlbum}
                    <span class="crumb-sep">/</span>
                    <button class="crumb" on:click={backToAlbums}>Albums</button>
                    <span class="crumb-sep">/</span>
                    <span class="crumb active">{$selectedAlbum.name}</span>
                {:else if $activeView === "playlist" && $selectedPlaylist}
                    <span class="crumb-sep">/</span>
                    <span class="crumb active">{$selectedPlaylist.name}</span>
                {:else}
                    <span class="crumb-sep">/</span>
                    <span class="crumb active">All Tracks</span>
                {/if}
            </div>

            <div class="top-actions">
                {#if $activeView === "library" || $activeView === "playlist"}
                    <SearchBar />
                {/if}

                <div class="menu-wrap">
                    <button class="menu-btn"
                        on:click|stopPropagation={() => (showMenu = !showMenu)}
                        title="Options">⋮</button>

                    {#if showMenu}
                        <div class="dropdown" on:click|stopPropagation
                            transition:fly={{ y: -8, duration: 150 }}>
                            <div class="dropdown-section">FILE</div>
                            <button class="dropdown-item" on:click={() => { showMenu = false; rescanAll(); }}>
                                <span>↺</span> Rescan Library
                            </button>
                            <div class="dropdown-divider"></div>
                            <div class="dropdown-section">VIEW</div>
                            <button class="dropdown-item" on:click={() => { showMenu = false; window.location.reload(); }}>
                                <span>↺</span> Refresh Page
                            </button>
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        {#if $scanResult}
            <div class="scan-banner" in:fly={{ y: -8, duration: 200 }} out:fade>
                {$scanResult}
            </div>
        {/if}

        <!-- Library view -->
        {#if $activeView === "library"}
            <div class="list-wrap" in:fade={{ duration: 150 }}>
                <TrackList tracks={$sortedTracks} showSortControls={true} />
            </div>

        <!-- Playlist view -->
        {:else if $activeView === "playlist"}
            <div class="list-wrap" in:fade={{ duration: 150 }}>
                <TrackList tracks={$playlistTracks} showSortControls={true} />
            </div>

        <!-- Albums grid -->
        {:else if $activeView === "albums"}
            <div class="albums-grid" in:fade={{ duration: 180 }}>
                {#each $albums as album, i}
                    {@const key = (album.name || "Unknown Album").toLowerCase().trim()}
                    {@const art = albumArts[key]}
                    <div
                        class="album-card"
                        role="button"
                        tabindex="0"
                        in:fly={{ y: 16, duration: 200, delay: Math.min(i * 15, 300), easing: cubicOut }}
                        on:click={() => openAlbum(album)}
                        on:keydown={(e) => e.key === "Enter" && openAlbum(album)}
                    >
                        <div class="album-art-wrap">
                            {#if art}
                                <img class="album-art-img"
                                    src="data:image/jpeg;base64,{art}"
                                    alt={album.name}
                                    in:fade={{ duration: 300 }}
                                />
                            {:else if art === null}
                                <span class="art-placeholder">♪</span>
                            {:else}
                                <span class="art-loading">·</span>
                            {/if}
                        </div>
                        <p class="album-name">{album.name}</p>
                        <p class="album-artist">{album.artist}</p>
                        <p class="album-count">{album.tracks.length} tracks</p>
                    </div>
                {/each}
            </div>

        <!-- Album detail view -->
        {:else if $activeView === ("album-detail" as any) && $selectedAlbum}
            <div class="album-detail" in:fade={{ duration: 150 }}>
                <!-- Album header -->
                <div class="album-header">
                    <div class="album-header-art">
                        {#if albumDetailArt}
                            <img src="data:image/jpeg;base64,{albumDetailArt}" alt={$selectedAlbum.name} />
                        {:else}
                            <span class="art-placeholder">♪</span>
                        {/if}
                    </div>
                    <div class="album-header-info">
                        <p class="album-header-type">Album</p>
                        <h1 class="album-header-title">{$selectedAlbum.name}</h1>
                        <p class="album-header-artist">{$selectedAlbum.artist}</p>
                        <p class="album-header-count">{$selectedAlbum.tracks.length} tracks</p>
                        <button
                            class="play-all-btn"
                            on:click={() => $selectedAlbum && playTrack($selectedAlbum.tracks[0], $selectedAlbum.tracks)}
                        >
                            ▶ Play All
                        </button>
                    </div>
                </div>

                <!-- Track list -->
                <div class="album-tracks">
                    <TrackList
                        tracks={$selectedAlbum.tracks}
                        showSortControls={false}
                    />
                </div>
            </div>
        {/if}
    </div>

    <LyricsPanel />
</div>

<style>
    .page {
        display: flex;
        height: 100%;
        overflow: hidden;
    }

    .main {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    /* Top bar */
    .top-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px 24px 12px;
        border-bottom: 1px solid #141414;
        flex-shrink: 0;
        gap: 16px;
    }

    .breadcrumb {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
    }

    .crumb {
        background: none;
        border: none;
        color: #555;
        cursor: pointer;
        font-size: 13px;
        padding: 0;
        transition: color 0.15s;
    }

    .crumb:hover { color: #aaa; }
    .crumb.active { color: #e8e8e8; font-weight: 600; cursor: default; }
    .crumb-sep { color: #333; }

    .top-actions {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .menu-wrap { position: relative; }

    .menu-btn {
        background: none;
        border: 1px solid #222;
        color: #666;
        width: 32px; height: 32px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 18px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s;
        line-height: 1;
    }

    .menu-btn:hover { border-color: #444; color: #ddd; background: #1a1a1a; }

    .dropdown {
        position: absolute;
        top: calc(100% + 6px);
        right: 0;
        background: #1a1a1a;
        border: 1px solid #2a2a2a;
        border-radius: 10px;
        padding: 6px;
        min-width: 200px;
        z-index: 100;
        box-shadow: 0 8px 32px rgba(0,0,0,0.5);
    }

    .dropdown-section {
        font-size: 10px;
        font-weight: 700;
        color: #444;
        text-transform: uppercase;
        letter-spacing: 1.5px;
        padding: 6px 10px 4px;
    }

    .dropdown-divider { height: 1px; background: #222; margin: 4px 0; }

    .dropdown-item {
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
        text-align: left;
        background: none;
        border: none;
        color: #bbb;
        padding: 8px 10px;
        font-size: 13px;
        cursor: pointer;
        border-radius: 6px;
        transition: all 0.12s;
    }

    .dropdown-item:hover { background: #252525; color: #fff; }

    .scan-banner {
        background: #1a1500;
        border-bottom: 1px solid #2a2000;
        color: #f59e0b;
        font-size: 12px;
        padding: 8px 24px;
        flex-shrink: 0;
    }

    /* List wrap */
    .list-wrap {
        flex: 1;
        overflow-y: auto;
        min-height: 0;
    }

    .list-wrap::-webkit-scrollbar { width: 5px; }
    .list-wrap::-webkit-scrollbar-thumb { background: #1e1e1e; border-radius: 3px; }

    /* Albums grid */
    .albums-grid {
        flex: 1;
        overflow-y: auto;
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
        gap: 16px;
        padding: 16px;
        align-content: start;
    }

    .albums-grid::-webkit-scrollbar { width: 5px; }
    .albums-grid::-webkit-scrollbar-thumb { background: #1e1e1e; border-radius: 3px; }

    .album-card {
        background: #111;
        border-radius: 10px;
        padding: 12px;
        cursor: pointer;
        transition: background 0.2s, transform 0.2s, box-shadow 0.2s;
    }

    .album-card:hover {
        background: #1a1a1a;
        transform: translateY(-4px);
        box-shadow: 0 12px 32px rgba(0,0,0,0.5);
    }

    .album-card:active { transform: translateY(-2px); }

    .album-art-wrap {
        width: 100%;
        aspect-ratio: 1;
        background: #1a1a1a;
        border-radius: 8px;
        overflow: hidden;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 10px;
        transition: background 0.2s;
    }

    .album-card:hover .album-art-wrap { background: #222; }

    .album-art-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
    }

    .art-placeholder { font-size: 36px; color: #2a2a2a; }

    .art-loading {
        font-size: 24px;
        color: #333;
        animation: pulse 0.8s ease-in-out infinite;
    }

    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.2; }
    }

    .album-name {
        font-size: 13px;
        font-weight: 600;
        color: #ddd;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .album-artist {
        font-size: 12px;
        color: #555;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        margin-top: 2px;
    }

    .album-count { font-size: 11px; color: #333; margin-top: 4px; }

    /* Album detail */
    .album-detail {
        flex: 1;
        overflow-y: auto;
        min-height: 0;
        display: flex;
        flex-direction: column;
    }

    .album-detail::-webkit-scrollbar { width: 5px; }
    .album-detail::-webkit-scrollbar-thumb { background: #1e1e1e; border-radius: 3px; }

    .album-header {
        display: flex;
        align-items: flex-end;
        gap: 28px;
        padding: 28px 24px 24px;
        background: linear-gradient(to bottom, #1a1a1a, #0a0a0a);
        flex-shrink: 0;
    }

    .album-header-art {
        width: 160px;
        height: 160px;
        border-radius: 12px;
        overflow: hidden;
        flex-shrink: 0;
        background: #1a1a1a;
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 16px 48px rgba(0,0,0,0.5);
    }

    .album-header-art img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .album-header-info {
        display: flex;
        flex-direction: column;
        gap: 6px;
        min-width: 0;
    }

    .album-header-type {
        font-size: 11px;
        font-weight: 700;
        color: #555;
        text-transform: uppercase;
        letter-spacing: 1.5px;
    }

    .album-header-title {
        font-size: 32px;
        font-weight: 800;
        color: #fff;
        margin: 0;
        line-height: 1.1;
    }

    .album-header-artist {
        font-size: 14px;
        color: #888;
    }

    .album-header-count {
        font-size: 12px;
        color: #444;
    }

    .play-all-btn {
        margin-top: 8px;
        background: #f59e0b;
        border: none;
        color: #000;
        padding: 10px 24px;
        border-radius: 24px;
        font-size: 14px;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.2s;
        align-self: flex-start;
    }

    .play-all-btn:hover {
        background: #fbbf24;
        transform: scale(1.04);
        box-shadow: 0 4px 16px rgba(245,158,11,0.4);
    }

    .album-tracks {
        flex: 1;
        padding: 8px 0;
    }
</style>
