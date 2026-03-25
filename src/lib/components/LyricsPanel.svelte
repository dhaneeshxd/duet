<script lang="ts">
    import { fly, fade } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import {
        lyrics,
        currentTrack,
        showLyrics,
        parseLyrics,
    } from "$lib/stores/player";

    $: lines = $lyrics ? parseLyrics($lyrics) : [];
</script>

{#if $showLyrics}
    <div
        class="lyrics-panel"
        transition:fly={{ x: 280, duration: 280, easing: cubicOut }}
    >
        <div class="lyrics-header">
            <span>Lyrics</span>
            <button on:click={() => showLyrics.set(false)}>✕</button>
        </div>

        <div class="lyrics-content">
            {#if !$currentTrack}
                <p class="empty" in:fade={{ duration: 200 }}>
                    Play a track to see lyrics
                </p>
            {:else if lines.length === 0}
                <p class="empty" in:fade={{ duration: 200 }}>
                    No lyrics found for this track
                </p>
            {:else}
                <div class="track-info" in:fly={{ y: -8, duration: 200 }}>
                    <p class="title">{$currentTrack.title}</p>
                    <p class="artist">{$currentTrack.artist}</p>
                </div>
                <div class="lyrics-text">
                    {#each lines as line, i (i)}
                        <p
                            class="lyric-line"
                            in:fly={{
                                x: 12,
                                duration: 160,
                                delay: Math.min(i * 6, 300),
                                easing: cubicOut,
                            }}
                        >
                            {line.text}
                        </p>
                    {/each}
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    .lyrics-panel {
        width: 280px;
        min-width: 280px;
        background: #0f0f0f;
        border-left: 1px solid #1a1a1a;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .lyrics-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px 20px;
        border-bottom: 1px solid #1a1a1a;
        font-size: 11px;
        font-weight: 700;
        color: #555;
        text-transform: uppercase;
        letter-spacing: 1.5px;
        flex-shrink: 0;
    }

    .lyrics-header button {
        background: none;
        border: none;
        color: #444;
        cursor: pointer;
        font-size: 14px;
        padding: 4px;
        border-radius: 4px;
        transition: all 0.15s;
    }

    .lyrics-header button:hover {
        color: #fff;
        background: #1a1a1a;
    }

    .lyrics-content {
        flex: 1;
        overflow-y: auto;
        padding: 20px 18px;
    }

    .lyrics-content::-webkit-scrollbar {
        width: 4px;
    }
    .lyrics-content::-webkit-scrollbar-thumb {
        background: #1e1e1e;
        border-radius: 2px;
    }

    .empty {
        color: #333;
        font-size: 14px;
        text-align: center;
        margin-top: 40px;
    }

    .track-info {
        margin-bottom: 20px;
        padding-bottom: 16px;
        border-bottom: 1px solid #1a1a1a;
    }

    .title {
        font-size: 15px;
        font-weight: 600;
        color: #e8e8e8;
        margin-bottom: 4px;
    }

    .artist {
        font-size: 12px;
        color: #555;
    }

    .lyrics-text {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .lyric-line {
        font-size: 14px;
        line-height: 1.8;
        color: #888;
        margin: 0;
        transition: color 0.2s;
    }

    .lyric-line:hover {
        color: #bbb;
    }
</style>
