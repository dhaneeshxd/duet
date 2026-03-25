import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface Track {
  id: number;
  path: string;
  title: string;
  artist: string;
  album: string;
  duration_secs: number;
}

export interface LyricLine {
  time: number;
  text: string;
}

export const currentTrack = writable<Track | null>(null);
export const isPlaying = writable(false);
export const isPaused = writable(false);
export const volume = writable(1.0);
export const albumArt = writable<string | null>(null);
export const lyrics = writable<string | null>(null);
export const showLyrics = writable(false);
export const showFullscreen = writable(false);
export const queue = writable<Track[]>([]);
export const position = writable(0);

export const currentDuration = derived(currentTrack, ($track) => {
  if (!$track) return "0:00";
  const m = Math.floor($track.duration_secs / 60);
  const s = $track.duration_secs % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
});

let ticker: ReturnType<typeof setInterval> | null = null;
let startTime = 0;
let startPosition = 0;

function startTicker() {
  stopTicker();
  startTime = Date.now();
  startPosition = get(position);

  ticker = setInterval(async () => {
    const elapsed = (Date.now() - startTime) / 1000;
    const track = get(currentTrack);
    if (!track) return;
    const currentPos = startPosition + elapsed;
    // Check if the track has finished
    if (currentPos >= track.duration_secs) {
      stopTicker();
      position.set(track.duration_secs);
      await nextTrack();
      return;
    }

    position.set(Math.floor(currentPos));
  }, 500);
}
export function toggleFullScreen() {
    showFullscreen.update(v => !v);
}

function stopTicker() {
  if (ticker) {
    clearInterval(ticker);
    ticker = null;
  }
}

export async function playTrack(track: Track, allTracks: Track[] = []) {
  try {
    const tracksToLoad = allTracks.length > 0 ? allTracks : [track];
    const ids = tracksToLoad.map((t) => t.id);
    const index = tracksToLoad.findIndex((t) => t.id === track.id);
    await invoke("load_queue", {
      trackIds: ids,
      startIndex: index >= 0 ? index : 0,
    });
    queue.set(tracksToLoad);

    await invoke("play_track", { trackId: track.id });
    currentTrack.set(track);
    isPlaying.set(true);
    isPaused.set(false);
    position.set(0);
    startTicker();

    const art = await invoke<string | null>("get_album_art", {
      path: track.path,
    });
    albumArt.set(art);
    const lyr = await invoke<string | null>("get_lyrics", { path: track.path });
    lyrics.set(lyr);
  } catch (e) {
    console.error("Failed to play track:", e);
  }
}

export async function nextTrack() {
  try {
    const track = await invoke<Track | null>("next_track");
    if (!track) return;

    currentTrack.set(track);
    isPlaying.set(true);
    isPaused.set(false);
    position.set(0);
    startTicker();

    const art = await invoke<string | null>("get_album_art", { path: track.path });
    albumArt.set(art);
    const lyr = await invoke<string | null>("get_lyrics", { path: track.path });
    lyrics.set(lyr);
  } catch (e) {
    console.error("nextTrack error:", e);
  }
}

export async function prevTrack() {
  try {
    const track = await invoke<Track | null>("prev_track");
    if (track) {
      currentTrack.set(track);
      isPlaying.set(true);
      isPaused.set(false);
      position.set(0);
      startTicker();
      const art = await invoke<string | null>("get_album_art", {
        path: track.path,
      });
      albumArt.set(art);
      const lyr = await invoke<string | null>("get_lyrics", {
        path: track.path,
      });
      lyrics.set(lyr);
    }
  } catch (e) {
    console.error(e);
  }
}

export async function pauseTrack() {
  try {
    await invoke("pause");
    isPlaying.set(false);
    isPaused.set(true);
    stopTicker();
  } catch (e) {
    console.error(e);
  }
}

export async function resumeTrack() {
  try {
    await invoke("resume");
    isPlaying.set(true);
    isPaused.set(false);
    startTime = Date.now();
    startPosition = get(position);
    startTicker();
  } catch (e) {
    console.error(e);
  }
}
export async function seekTo(secs: number) {
  try {
    await invoke("seek_to", { secs: secs });
    position.set(Math.floor(secs));
    // Update these so the ticker calculates elapsed time correctly from the new spot
    startTime = Date.now();
    startPosition = secs;
    isPlaying.set(true);
    isPaused.set(false);
    startTicker();
  } catch (e) {
    console.error(e);
  }
}
export async function setVolume(val: number) {
  try {
    await invoke("set_volume", { volume: val });
    volume.set(val);
  } catch (e) {
    console.error(e);
  }
}
export const isShuffled = writable(false);
export const showAddToPlaylist = writable(false);

export function toggleShuffle() {
  isShuffled.update((v) => {
    const newVal = !v;
    // Shuffle the queue in place
    const q = get(queue);
    if (newVal && q.length > 0) {
      const shuffled = [...q];
      for (let i = shuffled.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
      }
      queue.set(shuffled);
      // Re-sync queue with Rust
      const current = get(currentTrack);
      const ids = shuffled.map((t) => t.id);
      const startIndex = current
        ? shuffled.findIndex((t) => t.id === current.id)
        : 0;
      invoke("load_queue", {
        trackIds: ids,
        startIndex: Math.max(0, startIndex),
      });
    }
    return newVal;
  });
}
export type RepeatMode = "off" | "all" | "one";
export const repeatMode = writable<RepeatMode>("off");

export function cycleRepeat() {
  const current = get(repeatMode);
  let newMode: "off" | "all" | "one";
  if (current === "off") newMode = "all";
  else if (current === "all") newMode = "one";
  else newMode = "off";
  repeatMode.set(newMode);
  invoke("set_queue_repeat", { mode: newMode }).catch(console.error);
}
export function parseLyrics(raw: string): LyricLine[] {
  const result: LyricLine[] = [];
  const seenTexts = new Set<string>();

  for (const line of raw.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    // Match LRC timestamp [mm:ss.xx] or [mm:ss:xx]
    const tsMatch = trimmed.match(/^\[(\d+):(\d+)[.:](\d+)\](.*)/);
    if (tsMatch) {
      const time =
        parseInt(tsMatch[1]) * 60 +
        parseInt(tsMatch[2]) +
        parseInt(tsMatch[3]) / 100;
      const text = tsMatch[4].trim();
      if (!text) continue;

      // Only add if we haven't seen this exact text yet
      // OR if we have seen it but it's a chorus repeat (different timestamp far apart)
      const lastSeen = result.findLast((r) => r.text === text);
      if (!lastSeen || time - lastSeen.time > 30) {
        result.push({ time, text });
      }
    } else {
      // Plain line — only add if no timestamped version exists
      if (!seenTexts.has(trimmed)) {
        seenTexts.add(trimmed);
        result.push({ time: -1, text: trimmed });
      }
    }
  }

  return result;
}
