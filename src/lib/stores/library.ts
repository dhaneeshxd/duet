import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Track } from "./player";

export interface Playlist {
  id: number;
  name: string;
  created_at: number;
}

export interface Album {
  name: string;
  artist: string;
  tracks: Track[];
}

export interface Folder {
  id: number;
  path: string;
}
export const selectedAlbum = writable<Album | null>(null)
export const albumArt = writable<string>("");
export const tracks = writable<Track[]>([]);
export const playlists = writable<Playlist[]>([]);
export const folders = writable<Folder[]>([]);
export const selectedPlaylist = writable<Playlist | null>(null);
export const playlistTracks = writable<Track[]>([]);
export const isScanning = writable(false);
export const scanResult = writable<string | null>(null);
export const activeView = writable<"library" | "albums" | "playlist">(
  "library",
);
export const sortBy = writable<"title" | "artist" | "album" | "duration_secs">(
  "title",
);
export const sortAsc = writable(true);
export const searchQuery = writable("");

export const albums = derived(tracks, ($tracks) => {
  const map = new Map<string, Album>()
  for (const track of $tracks) {
    // Group by album name only ignore per-track artist differences
    const key = (track.album || 'Unknown Album').toLowerCase().trim()
    if (!map.has(key)) {
      map.set(key, {
        name: track.album || 'Unknown Album',
        artist: track.artist || 'Unknown Artist',
        tracks: []
      })
    }
    map.get(key)!.tracks.push(track)
  }
  return Array.from(map.values())
    .sort((a, b) => a.name.localeCompare(b.name))
})

export const sortedTracks = derived(
  [tracks, sortBy, sortAsc],
  ([$tracks, $sortBy, $sortAsc]) => {
    return [...$tracks].sort((a, b) => {
      let va: string | number = a[$sortBy];
      let vb: string | number = b[$sortBy];
      if (typeof va === "string") va = va.toLowerCase();
      if (typeof vb === "string") vb = vb.toLowerCase();
      if (va < vb) return $sortAsc ? -1 : 1;
      if (va > vb) return $sortAsc ? 1 : -1;
      return 0;
    });
  },
);

export async function loadTracks() {
  try {
    tracks.set(await invoke<Track[]>("get_tracks"));
  } catch (e) {
    console.error(e);
  }
}

export async function loadPlaylists() {
  try {
    playlists.set(await invoke<Playlist[]>("get_playlists"));
  } catch (e) {
    console.error(e);
  }
}

export async function loadFolders() {
  try {
    folders.set(await invoke<Folder[]>("get_folders"));
  } catch (e) {
    console.error(e);
  }
}

export async function searchTracks(query: string) {
  try {
    tracks.set(await invoke<Track[]>("search_tracks", { query }));
  } catch (e) {
    console.error(e);
  }
}

export async function createPlaylist(name: string) {
  try {
    await invoke("create_playlist", { name });
    await loadPlaylists();
  } catch (e) {
    console.error(e);
  }
}

export async function deletePlaylist(id: number) {
  try {
    await invoke("delete_playlist", { id });
    await loadPlaylists();
    selectedPlaylist.set(null);
    activeView.set("library");
  } catch (e) {
    console.error(e);
  }
}

export async function loadPlaylistTracks(playlistId: number) {
  try {
    playlistTracks.set(
      await invoke<Track[]>("get_playlist_tracks", { playlistId }),
    );
  } catch (e) {
    console.error(e);
  }
}

export async function addToPlaylist(playlistId: number, trackId: number) {
  try {
    await invoke("add_track_to_playlist", { playlistId, trackId, position: 0 });
  } catch (e) {
    console.error(e);
  }
}

export async function scanFolder(folderPath: string) {
  isScanning.set(true);
  scanResult.set(null);
  try {
    const result = await invoke<{
      scanned: number;
      inserted: number;
      failed: number;
    }>("scan_folder", { folderPath });
    scanResult.set(`✓ ${result.inserted} new tracks added`);
    await loadTracks();
    await loadFolders();
  } catch (e) {
    scanResult.set("Scan failed: " + e);
  } finally {
    isScanning.set(false);
  }
}

export async function removeFolder(id: number) {
  try {
    await invoke("remove_folder", { id });
    await loadFolders();
    await loadTracks();
  } catch (e) {
    console.error(e);
  }
}

export async function clearSearch() {
  searchQuery.set("");
  try {
    const currentTracks = await invoke<Track[]>("get_tracks");
    tracks.set(currentTracks);
  } catch (e) {
    console.error(e);
  }
}

export async function rescanAll() {
  isScanning.set(true);
  scanResult.set(null);
  try {
    const result = await invoke<{
      scanned: number;
      inserted: number;
      failed: number;
    }>("rescan_all");
    scanResult.set(`✓ Rescan complete — ${result.inserted} new tracks`);
    await loadTracks();
  } catch (e) {
    scanResult.set("Rescan failed: " + e);
  } finally {
    isScanning.set(false);
  }
}
