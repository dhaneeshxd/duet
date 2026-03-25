<script lang="ts">
    import { fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { playTrack, currentTrack, isPlaying } from "$lib/stores/player";
    import {
        sortBy,
        sortAsc,
        playlists,
        addToPlaylist,
    } from "$lib/stores/library";
    import { formatDuration } from "$lib/utils";
    import type { Track } from "$lib/stores/player";

    export let tracks: Track[] = [];
    export let showSortControls = true;

    let contextMenu: { x: number; y: number; track: Track } | null = null;

    function handleSort(col: typeof $sortBy) {
        if ($sortBy === col) sortAsc.update((v) => !v);
        else {
            sortBy.set(col);
            sortAsc.set(true);
        }
    }

    function showContext(e: MouseEvent, track: Track) {
        e.preventDefault();
        contextMenu = { x: e.clientX, y: e.clientY, track };
    }

    function closeContext() {
        contextMenu = null;
    }

    function sortIcon(col: string) {
        if ($sortBy !== col) return "";
        return $sortAsc ? " ↑" : " ↓";
    }

    async function handleAddToPlaylist(playlistId: number) {
        if (contextMenu) {
            await addToPlaylist(playlistId, contextMenu.track.id);
            closeContext();
        }
    }

</script>

<svelte:window on:click={closeContext} />

<div class="track-list">
    {#if tracks.length === 0}
        <div class="empty">
            <p>No tracks found.</p>
            <p class="hint">Add a music folder from the sidebar.</p>
        </div>
    {:else}
        {#if showSortControls}
            <div class="header">
                <span class="col-num">#</span>
                <button
                    class="sort-btn col-title"
                    on:click={() => handleSort("title")}
                >
                    Title{sortIcon("title")}
                </button>
                <button
                    class="sort-btn col-album"
                    on:click={() => handleSort("album")}
                >
                    Album{sortIcon("album")}
                </button>
                <button
                    class="sort-btn col-artist"
                    on:click={() => handleSort("artist")}
                >
                    Artist{sortIcon("artist")}
                </button>
                <button
                    class="sort-btn col-dur"
                    on:click={() => handleSort("duration_secs")}
                >
                    Duration{sortIcon("duration_secs")}
                </button>
            </div>
        {/if}

        {#each tracks as track, i (track.id)}
            {@const isActive = $currentTrack?.id === track.id}
            <div
                class="track-row"
                class:active={isActive}
                in:fly={{
                    x: -6,
                    duration: 140,
                    delay: Math.min(i * 6, 300),
                    easing: cubicOut,
                }}
                on:click={() => playTrack(track, tracks)}
                on:contextmenu={(e) => showContext(e, track)}
                role="row"
                tabindex="0"
                on:keydown={(e) =>
                    e.key === "Enter" && playTrack(track, tracks)}
            >
                <span class="col-num">
                    {#if isActive && $isPlaying}
                        <span class="eq-icon">▶</span>
                    {:else}
                        <span class="row-num">{i + 1}</span>
                    {/if}
                </span>
                <span class="col-title col-text" class:amber={isActive}>
                    {track.title || "Unknown Title"}
                </span>
                <span class="col-album col-text">{track.album || "—"}</span>
                <span class="col-artist col-text">{track.artist || "—"}</span>
                <span class="col-dur col-text"
                    >{formatDuration(track.duration_secs)}</span
                >
            </div>
        {/each}
    {/if}
</div>

{#if contextMenu}
    {@const menuX =
        contextMenu.x + 190 > window.innerWidth
            ? contextMenu.x - 190
            : contextMenu.x}
    {@const menuY =
        contextMenu.y + 200 > window.innerHeight
            ? contextMenu.y - 200
            : contextMenu.y}
    <div
        class="context-menu"
        style="left:{menuX}px; top:{menuY}px"
        on:click|stopPropagation
        role="menu"
    >
        <div class="ctx-header">Add to playlist</div>
        {#if $playlists.length === 0}
            <div class="ctx-empty">Create a playlist first</div>
        {:else}
            {#each $playlists as pl}
                <button
                    class="ctx-item"
                    on:click={() => handleAddToPlaylist(pl.id)}
                >
                    ♬ {pl.name}
                </button>
            {/each}
        {/if}
    </div>
{/if}

<style>
    .track-list {
        display: flex;
        flex-direction: column;
        width: 100%;
        min-width: 0;
    }

    .empty {
        text-align: center;
        padding: 80px 0;
        color: #444;
    }

    .empty .hint {
        font-size: 13px;
        margin-top: 8px;
        color: #2a2a2a;
    }

    /* Header */
    .header {
        display: grid;
        grid-template-columns: 48px 1fr minmax(0, 0.7fr) minmax(0, 0.6fr) 80px;
        padding: 0 16px;
        border-bottom: 1px solid #141414;
        margin-bottom: 4px;
        position: sticky;
        top: 0;
        background: #0a0a0a;
        z-index: 10;
        height: 36px;
        align-items: center;
    }

    .sort-btn {
        background: none;
        border: none;
        color: #3a3a3a;
        font-size: 11px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 1px;
        cursor: pointer;
        text-align: left;
        padding: 0;
        transition: color 0.15s;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .sort-btn:hover {
        color: #f59e0b;
    }

    /* Track rows */
    .track-row {
        display: grid;
        grid-template-columns: 48px 1fr minmax(0, 0.7fr) minmax(0, 0.6fr) 80px;
        padding: 6px 16px;
        border-radius: 4px;
        cursor: pointer;
        align-items: center;
        transition: background 0.1s;
        user-select: none;
    }

    .track-row:hover {
        background: #141414;
    }
    .track-row:active {
        background: #1a1a1a;
    }
    .track-row.active {
        background: #1a1400;
    }

    .col-num {
        font-size: 13px;
        color: #3a3a3a;
        text-align: center;
        width: 48px;
    }

    .row-num {
        color: #3a3a3a;
    }
    .track-row:hover .row-num {
        color: #888;
    }

    .eq-icon {
        color: #f59e0b;
        font-size: 10px;
        animation: pulse 1.5s ease-in-out infinite;
    }

    @keyframes pulse {
        0%,
        100% {
            opacity: 1;
        }
        50% {
            opacity: 0.4;
        }
    }

    .col-text {
        font-size: 13px;
        color: #666;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        padding-right: 16px;
        transition: color 0.1s;
    }

    .col-title {
        font-size: 14px;
        color: #d8d8d8;
        font-weight: 500;
    }

    .col-title.amber {
        color: #f59e0b;
    }
    .track-row:hover .col-title {
        color: #fff;
    }
    .track-row:hover .col-text {
        color: #888;
    }

    .col-dur {
        text-align: right;
        padding-right: 0;
        color: #3a3a3a;
    }

    /* Context menu */
    .context-menu {
        position: fixed;
        background: #1c1c1c;
        border: 1px solid #2a2a2a;
        border-radius: 10px;
        padding: 6px;
        min-width: 180px;
        z-index: 1000;
        box-shadow: 0 12px 40px rgba(0, 0, 0, 0.7);
        animation: ctxIn 0.12s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    @keyframes ctxIn {
        from {
            opacity: 0;
            transform: scale(0.92);
        }
        to {
            opacity: 1;
            transform: scale(1);
        }
    }

    .ctx-header {
        font-size: 10px;
        color: #444;
        text-transform: uppercase;
        letter-spacing: 1px;
        padding: 6px 10px 8px;
        border-bottom: 1px solid #222;
        margin-bottom: 4px;
    }

    .ctx-empty {
        font-size: 13px;
        color: #333;
        padding: 8px 10px;
    }

    .ctx-item {
        display: block;
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

    .ctx-item:hover {
        background: #252525;
        color: #f59e0b;
    }
</style>
