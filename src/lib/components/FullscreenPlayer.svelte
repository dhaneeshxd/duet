<script lang="ts">
    import { onDestroy, tick } from "svelte";
    import { fly, fade } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";
    import { cubicOut } from "svelte/easing";
    import {
        currentTrack, isPlaying, isPaused,
        albumArt, lyrics, showFullscreen,
        pauseTrack, resumeTrack,
        nextTrack, prevTrack,
        volume, setVolume,
        position, seekTo,
        parseLyrics,
        isShuffled, toggleShuffle,
        repeatMode, cycleRepeat,
    } from "$lib/stores/player";
    import type { LyricLine } from "$lib/stores/player";
    interface AudioInfo {
        sample_rate: number
        bits_per_sample: number
        channels: number
        bitrate_kbps: number
        codec: string
        is_lossless: boolean
    }
    export let colors = ["#1a1a1a", "#333333", "#000000"];
    let lyricLines: LyricLine[] = [];
    let activeLine = -1;
    let lyricsEl: HTMLDivElement;
    let isUserScrolling = false;
    let scrollTimeout: ReturnType<typeof setTimeout>;
    let audioInfo: AudioInfo | null = null
    $: if ($currentTrack) {
        invoke<AudioInfo | null>('get_audio_info', { path: $currentTrack.path })
            .then(info => { audioInfo = info })
            .catch(() => { audioInfo = null })
    } else {
        audioInfo = null
    }
    $: lyricLines = $lyrics ? parseLyrics($lyrics) : [];

    $: {
        if (lyricLines.length > 0) {
            let found = -1;
            for (let i = lyricLines.length - 1; i >= 0; i--) {
                if (lyricLines[i].time >= 0 && $position >= lyricLines[i].time - 0.01) {
                    found = i;
                    break;
                }
            }
            if (found !== activeLine) {
                activeLine = found;
                if (!isUserScrolling) scrollToActive();
            }
        }
    }

    function onLyricsScroll() {
        isUserScrolling = true;
        clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => { isUserScrolling = false; }, 3000);
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
        const bar = e.currentTarget as HTMLElement;
        const rect = bar.getBoundingClientRect();

        function calcSecs(clientX: number): number {
            const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
            return Math.floor(pct * duration);
        }

        position.set(calcSecs(e.clientX));

        const onMove = (ev: MouseEvent) => position.set(calcSecs(ev.clientX));
        const onUp = (ev: MouseEvent) => {
            const secs = calcSecs(ev.clientX);
            position.set(secs);
            seekTo(secs);
            window.removeEventListener("mousemove", onMove);
            window.removeEventListener("mouseup", onUp);
        };

        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onUp);
    }

    async function clickLyric(line: LyricLine) {
        if (line.time >= 0) {
            position.set(Math.floor(line.time));
            await seekTo(line.time);
        }
    }
    async function scrollToActive() {
        await tick();
        if (!lyricsEl) return;
        const activeEl = lyricsEl.querySelector(".lyric-line.active") as HTMLElement;
        if (!activeEl) return;

        const containerHeight = lyricsEl.clientHeight;
        const itemTop = activeEl.offsetTop;
        const itemHeight = activeEl.clientHeight;

        // scroll so active line is in the middle
        lyricsEl.scrollTo({
            top: itemTop - containerHeight / 2 + itemHeight / 2,
            behavior: "smooth",
            block: 'center'
        });
    }

    const colorSets = [
        ["#f59e0b", "#7c3aed", "#06b6d4"],
        ["#ec4899", "#f59e0b", "#10b981"],
        ["#3b82f6", "#f59e0b", "#8b5cf6"],
        ["#06b6d4", "#ec4899", "#f59e0b"],
    ];
    let colorIdx = 0;
    $: colors = colorSets[colorIdx];

    const colorInterval = setInterval(() => {
        colorIdx = (colorIdx + 1) % colorSets.length;
    }, 4000);

    onDestroy(() => {
        clearInterval(colorInterval);
        clearTimeout(scrollTimeout);
    });
</script>

{#if $showFullscreen}
    <div class="fullscreen" in:fade={{ duration: 200 }}>

        <!-- Single unified ambient background — covers entire screen -->
        <div class="ambient-container">
            <div class="ambient-filter">
                <div class="orb orb1" style="background: {colors[0]};"></div>
                <div class="orb orb2" style="background: {colors[1]};"></div>
                <div class="orb orb3" style="background: {colors[2]};"></div>
            </div>
            <div class="glass-overlay"></div>
        </div>

        <!-- Close button -->
        <button class="close-btn" on:click={() => showFullscreen.set(false)}>✕</button>

        <div class="content">
            <!-- Left — player -->
            <div class="left">

                <!-- Album art -->
                <div class="art-wrap">
                    {#key $currentTrack?.id}
                        {#if $albumArt}
                            <img class="art"
                                src="data:image/jpeg;base64,{$albumArt}"
                                alt="Album art"
                                in:fade={{ duration: 400 }}
                            />
                        {:else}
                            <div class="art-empty">♪</div>
                        {/if}
                    {/key}
                </div>

                <!-- Track info -->
                {#key $currentTrack?.id}
                    <div class="track-meta" in:fly={{ y: 10, duration: 300, easing: cubicOut }}>
                        <h2 class="t-title">{$currentTrack?.title ?? "No track"}</h2>
                        <p class="t-artist">{$currentTrack?.artist ?? ""}</p>
                        <p class="t-album">{$currentTrack?.album ?? ""}</p>
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
                    </div>
                {/key}

                <!-- Progress bar -->
                <div class="progress-section">
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
                            if (e.key === "ArrowRight") seekTo(Math.min($position + 5, duration));
                            if (e.key === "ArrowLeft") seekTo(Math.max($position - 5, 0));
                        }}
                    >
                        <div class="progress-bg"></div>
                        <div class="progress-filled" style="width:{progress}%">
                            <div class="progress-dot"></div>
                        </div>
                    </div>
                    <div class="time-row">
                        <span>{fmt($position)}</span>
                        <span>{fmt(duration)}</span>
                    </div>
                </div>

                <!-- Controls -->
                <div class="controls">
                    <button class="ctrl" on:click={prevTrack} disabled={!$currentTrack}>Prev</button>

                    <button class="play" on:click={togglePlay} disabled={!$currentTrack}>
                        {$isPlaying ? "Pause" : "Play"}
                    </button>

                    <button class="ctrl" on:click={nextTrack} disabled={!$currentTrack}>Next</button>
                </div>

                <!-- Volume -->
                <div class="vol-row">
                    <span class="vol-icon">🔇</span>
                    <input
                        type="range" min="0" max="1" step="0.01"
                        value={$volume}
                        on:input={handleVolume}
                        class="vol-slider"
                    />
                    <span class="vol-icon">🔊</span>
                </div>

            </div>

            <!-- Right — lyrics -->
            <div class="right">
                {#if lyricLines.length === 0}
                    <div class="no-lyrics">
                        {$currentTrack ? "No lyrics embedded in this file" : "Play a track"}
                    </div>
                {:else}
                    <div class="lyrics-container">
                        <div class="blur-top"></div>
                        <div
                            class="lyrics-scroll"
                            bind:this={lyricsEl}
                            on:scroll={onLyricsScroll}
                        >
                            <div class="lyrics-pad-top"></div>
                            {#each lyricLines as line, i}
                                <button
                                    class="lyric-line"
                                    class:active={i === activeLine}
                                    class:near={Math.abs(i - activeLine) === 1}
                                    class:far2={Math.abs(i - activeLine) === 2}
                                    class:far3={Math.abs(i - activeLine) >= 3}
                                    class:past={i < activeLine}
                                    class:has-time={line.time >= 0}
                                    on:click={() => clickLyric(line)}
                                >
                                    {line.text}
                                </button>
                            {/each}
                            <div class="lyrics-pad-bottom"></div>
                        </div>
                        <div class="blur-bottom"></div>
                    </div>
                {/if}
            </div>
        </div>
    </div>
{/if}

<style>
    .fullscreen {
        position: fixed;
        inset: 0;
        z-index: 500;
        overflow: hidden;
        background: #060606;
    }

    .ambient-wrapper {
        position: fixed;
        inset: 0;
        z-index: -1;
        background: #0a0a0a;
        overflow: hidden;
    }

    .ambient-filter {
        position: absolute;
        inset: 0;
        filter: saturate(1.2) brightness(1.1);
    }

    .orb {
        position: absolute;
        border-radius: 50%;
        filter: blur(140px); /* Increased blur for even LESS intensity/edges */
        mix-blend-mode: screen;
        opacity: 0.35;

        transition: background 2.5s cubic-bezier(0.2, 0, 0.3, 1);
        will-change: transform, background;
    }

    .ambient-overlay {
        position: absolute;
        inset: 0;
        backdrop-filter: blur(20px);
        background: radial-gradient(
            circle at center,
            transparent 20%,
            rgba(0,0,0,0.2) 100%
        );
        pointer-events: none;
    }

    /* Animations stay the same for that fluid movement */
    .orb1 { width: 800px; height: 800px; top: -200px; left: -150px; animation: float1 18s ease-in-out infinite; }
    .orb2 { width: 700px; height: 700px; bottom: -250px; right: -200px; animation: float2 22s ease-in-out infinite; }
    .orb3 { width: 600px; height: 600px; top: 40%; left: 40%; animation: float3 15s ease-in-out infinite; }

    @keyframes float1 { 0%, 100% { transform: translate(0,0) scale(1); } 33% { transform: translate(120px, 90px) scale(1.1); } 66% { transform: translate(-60px, 110px) scale(0.9); } }
    @keyframes float2 { 0%, 100% { transform: translate(0,0) scale(1); } 33% { transform: translate(-100px, -120px) scale(1.15); } 66% { transform: translate(100px, -60px) scale(0.95); } }
    @keyframes float3 { 0%, 100% { transform: translate(-50%, -50%) scale(1); } 50% { transform: translate(-50%, -50%) scale(1.35); } }
    .close-btn {
        position: absolute;
        top: 20px; right: 20px;
        z-index: 10;
        background: rgba(255,255,255,0.07);
        border: 1px solid rgba(255,255,255,0.1);
        color: #888;
        width: 36px; height: 36px;
        border-radius: 50%;
        font-size: 14px;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
        backdrop-filter: blur(10px);
    }

    .close-btn:hover { background: rgba(255,255,255,0.14); color: #fff; border-color: rgba(255,255,255,0.2); }

    /* Content grid */
    .content {
        position: relative;
        z-index: 2;
        display: grid;
        grid-template-columns: 0.6fr 1fr;
        height: 100%;
    }

    /* Left panel */
    .left {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 40px 48px;
    }

    /* Album art */
    .art-wrap {
        width: min(260px, 36vw);
        height: min(260px, 36vw);
        border-radius: 16px;
        overflow: hidden;
        flex-shrink: 0;
        box-shadow:
            0 32px 80px rgba(0,0,0,0.6),
            0 0 0 1px rgba(255,255,255,0.05);
        transition: transform 0.3s ease, box-shadow 0.3s ease;
    }

    .art-wrap:hover {
        transform: scale(1.02);
        box-shadow: 0 40px 100px rgba(0,0,0,0.7), 0 0 0 1px rgba(255,255,255,0.08);
    }

    .art { width: 100%; height: 100%; object-fit: cover; }

    .art-empty {
        width: 100%; height: 100%;
        background: rgba(255,255,255,0.03);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 72px;
        color: rgba(255,255,255,0.1);
    }

    /* Track info */
    .track-meta { text-align: center; width: 100%; }

    .t-title {
        margin-top: 30px;
        font-size: 20px;
        font-weight: 700;
        color: #ffffff;
        margin-bottom: 5px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .t-artist {
        font-size: 14px;
        color: #f59e0b;
        margin-bottom: 3px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .t-album {
        font-size: 12px;
        color: rgba(255,255,255,0.3);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    /* Progress */
    .progress-section {
        width: 100%;
        max-width: 340px;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .progress-track {
        width: 100%;
        height: 30px;
        display: flex;
        align-items: center;
        position: relative;
        cursor: pointer;
    }

    .progress-bg {
        position: absolute;
        left: 0; right: 0;
        height: 3px;
        background: rgba(255,255,255,0.1);
        border-radius: 2px;
        transition: background 0.15s;
    }

    .progress-track:hover .progress-bg { background: rgba(255,255,255,0.18); }

    .progress-filled {
        position: absolute;
        left: 0;
        height: 3px;
        background: #f59e0b;
        border-radius: 2px;
        pointer-events: none;
        transition: width 0.4s linear;
        display: flex;
        align-items: center;
    }

    .progress-track:hover .progress-filled { background: #fbbf24; }

    .progress-track.disabled {
        opacity: 0.3;
        pointer-events: none;
        cursor: not-allowed;
    }
    progress-track.disabled .progress-dot {
        display: none;
    }

    .progress-dot {
        position: absolute;
        right: -5px;
        top: 50%;
        transform: translateY(-50%);
        width: 12px; height: 12px;
        border-radius: 50%;
        background: #fff;
        opacity: 0;
        transition: opacity 0.15s;
        pointer-events: none;
        box-shadow: 0 0 8px rgba(255,255,255,0.5);
    }

    .progress-track:hover .progress-dot { opacity: 1; }

    .time-row {
        display: flex;
        justify-content: space-between;
        font-size: 11px;
        color: rgba(255,255,255,0.35);
        font-variant-numeric: tabular-nums;
    }

    /* Controls container */
    .controls {
        display: flex;
        align-items: center;
        gap: 20px;
    }

    /* Container to keep everything centered and spaced */
    .controls-container {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 30px;
        margin-top: 20px;
    }

    /* Hide the shuffle/repeat icons completely */
    .ctrl-sm {
        display: none !important;
    }

    /* Base style for all text buttons */
    .ctrl, .play {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius:90px;
        padding: 6px 14px;
        cursor: pointer;
        font-family: 'Inter', sans-serif;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        font-size: 11px;
        font-weight: 700;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .ctrl {
        color: rgba(255, 255, 255, 0.5);
        margin-top: 2px;
    }

    /* PAUSE/PLAY */
    .play {
        color: #f59e0b;
        border: 1px solid rgba(245, 158, 11, 0.4);
        background: rgba(245, 158, 11, 0.05);
        min-width: 80px;
        margin-top: 2px;
    }

    /* Hover Effects */
    .ctrl:hover:not(:disabled) {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(255, 255, 255, 0.3);
        color: #fff;
    }

    .play:hover:not(:disabled) {
        background: rgba(245, 158, 11, 0.15);
        border-color: #f59e0b;
        box-shadow: 0 0 10px rgba(245, 158, 11, 0.2);
    }

    /* Pressed State */
    .ctrl:active, .play:active {
        transform: scale(0.96);
    }

    /* Disabled */
    button:disabled {
        opacity: 0.2;
        cursor: not-allowed;
        border-color: rgba(255,255,255,0.05);
    }

    /* Volume row moved down */
    .vol-row {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-top: 20px;
    }

    .vol-icon { font-size: 13px; opacity: 0.4; }

    .vol-slider {
        width: 160px;
        height: 2px;
        accent-color: #f59e0b;
        cursor: pointer;
    }

    /* Right — lyrics */
    .right {
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .no-lyrics {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: rgba(255,255,255,0.2);
        font-size: 15px;
    }

    .lyrics-container {
        position: relative;
        flex: 1;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    .blur-top {
        position: absolute;
        top: 0; left: 0; right: 0;
        height: 0%;
        background: linear-gradient(
            to bottom,
            #060606 0%,
            rgba(6,6,6,0.85) 50%,
            transparent 100%
        );
        z-index: 2;
        pointer-events: none;
    }

    .blur-bottom {
        position: absolute;
        bottom: 0; left: 0; right: 0;
        height: 0%;
        background: linear-gradient(
            to top,
            #060606 0%,
            rgba(6,6,6,0.85) 50%,
            transparent 100%
        );
        z-index: 2;
        pointer-events: none;
    }

    .lyrics-scroll {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        scrollbar-width: none;
        padding: 0 24px;
    }

    .lyrics-scroll::-webkit-scrollbar { display: none; }

    .lyrics-pad-top { flex-shrink: 0; height: 50vh; }
    .lyrics-pad-bottom { flex-shrink: 0; height: 50vh; }
    .lyrics-scroll {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        display: flex;
        flex-direction: column;
        align-items: center;
        scrollbar-width: none;
        padding: 0 52px;
        scroll-behavior: smooth;
    }

    .lyrics-scroll::-webkit-scrollbar {
        display: none;
    }
    /* Lyrics container scroll */
    .lyrics-scroll {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        display: flex;
        flex-direction: column;
        align-items: center;
        scrollbar-width: none;
        padding: 0 52px;
        scroll-behavior: smooth;
        scroll-padding: 50% 0;
    }

    .lyrics-scroll::-webkit-scrollbar {
        display: none;
    }

    /* All lyric lines have the same height to avoid jumpiness */
    .lyric-line {
        background: none;
        border: none;
        text-align: center;
        font-family: inherit;
        width: 100%;
        height: 42px;
        line-height: 42px;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        cursor: default;
        font-size: auto;
        font-weight: 400;
        color: rgba(255,255,255,0.3);
        opacity: 0.5;
        filter: blur(3px);

        transition:
            color 0.5s ease-in-out,
            transform 0.5s ease-in-out,
            opacity 0.5s ease-in-out,
            filter 0.5s ease-in-out;
    }

    /* Active line: scaled and sharp */
    .lyric-line.active {
        transform: scale(2) translateX(0);
        font-weight: 800;
        color: #ffffff;
        opacity: 1;
        filter: blur(0);
        text-shadow: 0 0 40px rgba(245,158,11,0.45);
        cursor: default;
    }

    /* Optional: slightly bigger scale for near lines */
    .lyric-line.near {
        transform: scale(1.4) translateX(4px);
        color: rgba(255,255,255,0.5);
        opacity: 0.5;
        filter: blur(2px);
    }

    /* Other lines remain blurred */
    .lyric-line.far2,
    .lyric-line.far3 {
        transform: scale(1);
        color: rgba(255,255,255,0.25);
        opacity: 0.2;
        filter: blur(7px);
    }

    /* Hover effect on non-active lines with time */
    .lyric-line.has-time:not(.active):hover {
        color: rgba(255,255,255,0.6);
        filter: blur(1px);
        cursor: pointer;
    }
    .audio-badges {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 5px;
        flex-wrap: nowrap;
        overflow: visible;
        margin-top: 4px;
        margin-bottom: 4px;
        min-height: 1.5rem;
        padding: 2px 0;
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
</style>
