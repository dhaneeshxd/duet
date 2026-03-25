<script lang="ts">
    import { fly, fade, scale } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import {
        currentTrack, isPlaying, isPaused,
        albumArt, volume, showLyrics, showFullscreen,
        pauseTrack, resumeTrack,
        nextTrack, prevTrack,
        setVolume, position, seekTo,
        isShuffled, toggleShuffle,
        repeatMode, cycleRepeat,
    } from "$lib/stores/player";
    import { playlists, addToPlaylist } from "$lib/stores/library";
    import { listen } from "@tauri-apps/api/event";

    let audioInfo: AudioInfo | null = null
    let showPlaylistMenu = false

    interface AudioInfo {
        sample_rate: number
        bits_per_sample: number
        channels: number
        bitrate_kbps: number
        codec: string
        is_lossless: boolean
    }

    $: if ($currentTrack) {
        invoke<AudioInfo | null>('get_audio_info', { path: $currentTrack.path })
            .then(info => { audioInfo = info })
            .catch(() => { audioInfo = null })
    } else {
        audioInfo = null
    }

    function togglePlay() {
        if ($isPlaying) pauseTrack();
        else if ($isPaused) resumeTrack();
    }

    function handleVolume(e: Event) {
        setVolume(parseFloat((e.target as HTMLInputElement).value));
    }

    function fmt(s: number) {
        const m = Math.floor(s / 60);
        const sec = Math.floor(s % 60);
        return `${m}:${sec.toString().padStart(2, "0")}`;
    }

    $: duration = $currentTrack?.duration_secs ?? 0;
    $: progress = duration > 0 ? ($position / duration) * 100 : 0;

    function onProgressMousedown(e: MouseEvent) {
        e.preventDefault();
        const bar = e.currentTarget as HTMLElement;
        const rect = bar.getBoundingClientRect();

        function calcSecs(clientX: number): number {
            const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
            return Math.floor(pct * duration);
        }

        position.set(calcSecs(e.clientX));

        function onMove(ev: MouseEvent) {
            ev.preventDefault();
            position.set(calcSecs(ev.clientX));
        }

        function onUp(ev: MouseEvent) {
            const secs = calcSecs(ev.clientX);
            position.set(secs);
            seekTo(secs);
            window.removeEventListener("mousemove", onMove);
            window.removeEventListener("mouseup", onUp);
        }

        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onUp);
    }

    function closeMenu() { showPlaylistMenu = false; }

    async function handleAddToPlaylist(playlistId: number) {
        if ($currentTrack) {
            await addToPlaylist(playlistId, $currentTrack.id);
        }
        showPlaylistMenu = false;
    }

</script>

<svelte:window on:click={closeMenu} />

<footer class="player-bar">
    <!-- Left — track info -->
    <div class="track-info">
        <div
            class="album-art"
            role="button"
            tabindex="0"
            on:click={() => showFullscreen.set(true)}
            on:keydown={(e) => e.key === "Enter" && showFullscreen.set(true)}
        >
            {#key $currentTrack?.id}
                {#if $albumArt}
                    <img src="data:image/jpeg;base64,{$albumArt}" alt="Album art"
                        in:fade={{ duration: 300 }} />
                {:else}
                    <div class="art-placeholder" in:fade={{ duration: 300 }}>♪</div>
                {/if}
            {/key}
        </div>

        <div class="meta">
            {#if $currentTrack}
                {#key $currentTrack.id}
                    <span class="title" in:fly={{ y: 6, duration: 200, easing: cubicOut }}>
                        {$currentTrack.title}
                    </span>
                    <span class="artist" in:fly={{ y: 6, duration: 200, delay: 30, easing: cubicOut }}>
                        {$currentTrack.artist}
                    </span>
                    {#if audioInfo}
                        <div class="audio-badges" in:fly={{ y: 4, duration: 200, delay: 60 }}>
                            {#if audioInfo.is_lossless}
                                <span class="badge lossless">LOSSLESS</span>
                            {/if}
                            <span class="badge codec">{audioInfo.codec}</span>
                            {#if audioInfo.bits_per_sample >= 24}
                                <span class="badge hires">{audioInfo.bits_per_sample}bit</span>
                            {/if}
                            {#if audioInfo.sample_rate > 0}
                                <span class="badge">{(audioInfo.sample_rate / 1000).toFixed(1)}kHz</span>
                            {/if}
                            {#if audioInfo.bitrate_kbps > 0}
                                <span class="badge">{audioInfo.bitrate_kbps}kbps</span>
                            {/if}
                            <span class="badge">
                                {audioInfo.channels === 2 ? 'Stereo' : audioInfo.channels === 1 ? 'Mono' : audioInfo.channels + 'ch'}
                            </span>
                        </div>
                    {/if}
                {/key}
            {:else}
                <span class="title muted">No track selected</span>
            {/if}
        </div>

        <!-- Add to playlist -->
        <div class="playlist-wrap">
            <button
                class="icon-btn"
                class:active={showPlaylistMenu}
                disabled={!$currentTrack}
                title="Add to playlist"
                on:click|stopPropagation={() => showPlaylistMenu = !showPlaylistMenu}
            >+</button>

            {#if showPlaylistMenu}
                <div
                    class="playlist-dropdown"
                    transition:fly={{ y: 8, duration: 150, easing: cubicOut }}
                    on:click|stopPropagation
                >
                    <div class="pd-header">Add to playlist</div>
                    {#if $playlists.length === 0}
                        <div class="pd-empty">No playlists yet</div>
                    {:else}
                        {#each $playlists as pl}
                            <button class="pd-item" on:click={() => handleAddToPlaylist(pl.id)}>
                                ♬ {pl.name}
                            </button>
                        {/each}
                    {/if}
                </div>
            {/if}
        </div>
    </div>

    <!-- Center — controls + progress -->
    <div class="center">
        <div class="controls">
            <button
                class="shuffle-repeat-btn"
                disabled={!$currentTrack}
                class:active={$isShuffled}
                on:click={toggleShuffle}
                title="Shuffle"
            >⇄</button>

            <button class="ctrl-btn" on:click={prevTrack} disabled={!$currentTrack}>⏮</button>

            <button
              class="play-btn {$isPlaying ? 'pause-btn' : 'play-icon'}"
              on:click={togglePlay}
              disabled={!$currentTrack}
            >
                {#key $isPlaying}
                    <span in:scale={{ duration: 150, start: 0.7 }}>
                        {$isPlaying ? "❚❚" : "▶"}
                    </span>
                {/key}
            </button>

            <button class="ctrl-btn" on:click={nextTrack} disabled={!$currentTrack}>⏭</button>

            <button
                class="shuffle-repeat-btn"
                disabled={!$currentTrack}
                class:active={$repeatMode !== 'off'}
                on:click={cycleRepeat}
                title="Repeat: {$repeatMode}"
            >{$repeatMode === 'one' ? '↺¹' : '↺'}</button>
        </div>

        <div class="progress-row">
            <span class="time">{fmt($position)}</span>
            <div
                class="progress-track"
                class:disabled={!$currentTrack}
                on:mousedown={onProgressMousedown}
                role="slider"
                tabindex={$currentTrack ? 0 : -1}
                aria-valuenow={$position}
                aria-valuemin={0}
                aria-valuemax={duration}
                aria-disabled={!$currentTrack}
                on:keydown={(e) => {
                    if (!$currentTrack) return;
                    if (e.key === "ArrowRight") seekTo(Math.min($position + 5, duration));
                    if (e.key === "ArrowLeft") seekTo(Math.max($position - 5, 0));
                }}
            >
                <div class="progress-bg"></div>
                <div class="progress-filled" style="width: {progress}%">
                    <div class="progress-dot"></div>
                </div>
            </div>
            <span class="time">{fmt(duration)}</span>
        </div>
    </div>

    <!-- Right — volume + toggles -->
    <div class="right-controls">
        <button
            class="lyrics-btn"
            class:active={$showLyrics}
            on:click={() => showLyrics.update(v => !v)}
            title="Lyrics"
        >♫</button>

        <button
            class="icon-btn"
            on:click={() => showFullscreen.update(v => !v)}
            title="Fullscreen"
        >⛶</button>

        <div class="vol-wrap">
            <span class="vol-icon">🔊</span>
            <input
                type="range" min="0" max="1" step="0.01"
                value={$volume}
                on:input={handleVolume}
                class="vol-slider"
            />
        </div>
    </div>
</footer>

<style>
    .player-bar {
        width: 100%;
        height: 88px;
        min-height: 88px;
        background: rgba(8, 8, 8, 0.98);
        backdrop-filter: blur(20px);
        border-top: 1px solid #1a1a1a;
        display: grid;
        grid-template-columns: 1fr 2fr 1fr;
        align-items: center;
        padding: 0 20px;
        gap: 16px;
        flex-shrink: 0;
        z-index: 100;
    }

    /* Left */
    .track-info {
        display: flex;
        align-items: center;
        gap: 10px;
        min-width: 0;
    }

    .album-art {
        width: 52px;
        height: 52px;
        border-radius: 8px;
        overflow: hidden;
        flex-shrink: 0;
        background: #1a1a1a;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: transform 0.2s, box-shadow 0.2s;
    }

    .album-art:hover {
        transform: scale(1.06);
        box-shadow: 0 4px 20px rgba(245, 158, 11, 0.3);
    }

    .album-art img { width: 100%; height: 100%; object-fit: cover; }
    .art-placeholder { color: #2a2a2a; font-size: 22px; }

    .meta {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
        flex: 1;
        overflow: hidden;
    }

    .title {
        display: block;
        font-size: 13px;
        font-weight: 600;
        color: #e8e8e8;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .title.muted { color: #333; }

    .artist {
        display: block;
        font-size: 11px;
        color: #666;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .audio-badges {
        display: flex;
        gap: 3px;
        flex-wrap: nowrap;
        overflow: hidden;
        margin-top: 2px;
    }

    .badge {
        font-size: 8px;
        font-weight: 700;
        padding: 2px 4px;
        border-radius: 3px;
        background: #1a1a1a;
        color: #444;
        letter-spacing: 0.3px;
        white-space: nowrap;
        flex-shrink: 0;
        border: 1px solid #222;
    }

    .badge.lossless {
        background: rgba(245, 158, 11, 0.12);
        color: #f59e0b;
        border-color: rgba(245, 158, 11, 0.25);
    }

    .badge.codec {
        background: rgba(99, 102, 241, 0.1);
        color: #818cf8;
        border-color: rgba(99, 102, 241, 0.2);
    }

    .badge.hires {
        background: rgba(16, 185, 129, 0.1);
        color: #34d399;
        border-color: rgba(16, 185, 129, 0.2);
    }

    /* Playlist dropdown */
    .playlist-wrap {
        position: relative;
        flex-shrink: 0;
    }

    .playlist-dropdown {
        position: absolute;
        bottom: calc(100% + 10px);
        left: 50%;
        transform: translateX(-50%);
        background: #1c1c1c;
        border: 1px solid #2a2a2a;
        border-radius: 10px;
        padding: 6px;
        min-width: 180px;
        z-index: 200;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    }

    .pd-header {
        font-size: 10px;
        color: #444;
        text-transform: uppercase;
        letter-spacing: 1px;
        padding: 6px 10px 8px;
        border-bottom: 1px solid #222;
        margin-bottom: 4px;
    }

    .pd-empty { font-size: 13px; color: #333; padding: 8px 10px; }

    .pd-item {
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

    .pd-item:hover { background: #252525; color: #f59e0b; }

    /* Center */
    .center {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        width: 100%;
    }

    .controls {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .ctrl-btn {
        background: none;
        border: none;
        color: #555;
        font-size: 15px;
        cursor: pointer;
        width: 30px;
        height: 30px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s;
    }
    .shuffle-repeat-btn {
        background: none;
        border: none;
        color: #555;
        font-size: 15px;
        cursor: pointer;
        width: 30px;
        height: 30px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s;
        transform: translateY(2px);
    }

    .shuffle-repeat-btn:hover {
        background-color: rgba(255, 255, 255, 0.1);
        color: #bbb;
    }

    .shuffle-repeat-btn.active {
        color:  #f59e0b;
        text-shadow: 0 0 8px rgba(29, 185, 84, 0.5);
    }
    .shuffle-repeat-btn.disabled,
    .shuffle-repeat-btn:disabled  {
        background: none !important;
        color: #555 !important;
        transform: none !important;
        pointer-events: none !important;
        cursor: not-allowed !important;
    }
    .shuffle-repeat-btn.disabled:hover,
    .shuffle-repeat-btn:disabled:hover {
        transform: none !important;
        background: transparent !important;
    }

    /* Indicator for Repeat One */
    .mode-indicator {
        position: absolute;
        top: 2px;
        right: 2px;
        font-size: 8px;
        font-weight: bold;
    }
    .ctrl-btn:hover:not(:disabled) { color: #fff; transform: scale(1.12); }
    .ctrl-btn:active:not(:disabled) { transform: scale(0.95); }
    .ctrl-btn:disabled { opacity: 0.2; cursor: not-allowed; }
    .ctrl-btn.active { color: #f59e0b; }

    .play-btn {
        width: 42px;
        height: 42px;
        border-radius: 50%;
        border: none;
        background: #f59e0b;
        color: #000;
        font-size: 17px;
        font-weight: 700;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
        text-indent: 2px;
    }
    .pause-btn {
        width: 42px;
        height: 42px;
        border-radius: 50%;
        border: none;
        background: #f59e0b;
        color: #000;
        font-size: 17px;
        font-weight: 700;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
        text-indent: -1px;
    }

    .play-btn:hover:not(:disabled) {
        transform: scale(1.1);
        background: #fbbf24;
        box-shadow: 0 0 24px rgba(245, 158, 11, 0.5);
    }

    .play-btn:active:not(:disabled) { transform: scale(0.95); }
    .play-btn:disabled { opacity: 0.3; cursor: not-allowed; }

    /* Progress */
    .progress-row {
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
    }

    .time {
        font-size: 11px;
        color: #555;
        min-width: 36px;
        text-align: center;
        font-variant-numeric: tabular-nums;
        flex-shrink: 0;
    }

    .progress-track {
        flex: 1;
        height: 20px;
        display: flex;
        align-items: center;
        position: relative;
        cursor: pointer;
    }
    .progress-track.disabled {
        opacity: 0.3;
        pointer-events: none;
        cursor: not-allowed;
    }
    progress-track.disabled .progress-dot {
        display: none;
    }
    .progress-bg {
        position: absolute;
        left: 0; right: 0;
        height: 3px;
        background: #222;
        border-radius: 2px;
        transition: background 0.15s;
    }

    .progress-track:hover .progress-bg { background: #333; }

    .progress-filled {
        position: absolute;
        left: 0;
        height: 3px;
        background: #f59e0b;
        border-radius: 2px;
        pointer-events: none;
        display: flex;
        align-items: center;
        transition: width 0.3s linear;
    }

    .progress-track:hover .progress-filled { background: #fbbf24; }

    .progress-dot {
        position: absolute;
        right: -5px;
        top: 50%;
        transform: translateY(-50%);
        width: 11px; height: 11px;
        border-radius: 50%;
        background: #fff;
        opacity: 0;
        transition: opacity 0.15s;
        pointer-events: none;
    }

    .progress-track:hover .progress-dot { opacity: 1; }

    /* Right */
    .right-controls {
        display: flex;
        align-items: center;
        gap: 10px;
        justify-content: flex-end;
    }

    .icon-btn {
        background: none;
        border: none;
        color: #555;
        font-size: 16px;
        cursor: pointer;
        padding: 6px;
        border-radius: 6px;
        transition: all 0.15s;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
    }
    .lyrics-btn {
        background: none;
        border: none;
        color: #555;
        font-size: 16px;
        cursor: pointer;
        border-radius: 6px;
        transition: all 0.15s;
        display: flex;
        align-items: center;
        justify-content: right;
        width: 30px;
        height: 30px;
        margin-top: 4px;
    }

    .icon-btn:hover { color: #fff; background: #1a1a1a; }
    .icon-btn:active { transform: scale(0.9); }
    .icon-btn.active { color: #f59e0b; }
    .icon-btn:disabled { opacity: 0.3; cursor: not-allowed; }

    .vol-wrap {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .vol-icon { font-size: 13px; color: #555; }

    .vol-slider {
        width: 110px;
        accent-color: #f59e0b;
        cursor: pointer;
        height: 3px;
        margin-top: 2px;
    }
</style>
