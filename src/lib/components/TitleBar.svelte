<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';

  let showInfoModal = false;
  let myName = "DHANEESKUMAR R";
  let githubUrl = "https://github.com/dhaneeshxd";
  let win: any;
  let isMaximized = false;
  let devices: string[] = [];
  let currentDevice = '';
  let showDevicePicker = false;
  let devicePollInterval: ReturnType<typeof setInterval>;
  let manualOverride = false;
  let _manualOverride = false;

   onMount(async () => {
      win = await getCurrentWindow();
      isMaximized = await win.isMaximized();
      win.listen('tauri://maximize', () => (isMaximized = true));
      win.listen('tauri://unmaximize', () => (isMaximized = false));

      devices = await invoke('list_audio_devices');
      currentDevice = await invoke('get_current_audio_device');
      devicePollInterval = setInterval(async () => {
          const [newDevices, newCurrent] = await Promise.all([
              invoke<string[]>('list_audio_devices'),
              invoke<string>('get_current_audio_device'),
          ]);

          // Any device count drop = unplug = clear manual override
          if (_manualOverride && newDevices.length < devices.length) {
              _manualOverride = false;
              manualOverride = false;
          }

          devices = newDevices.map(d => d.replace(/speakerss/gi, "").trim());
          if (!_manualOverride) {
              currentDevice = newCurrent;
          }
      }, 1000);
   });

   onDestroy(() => clearInterval(devicePollInterval));

  async function handleGithubClick(event: MouseEvent) {
      event.preventDefault();
      try {
        await openUrl(githubUrl);
      } catch (err) {
        console.error("Failed to open URL:", err);
      }
    }
    function shortName(name: string): string {
        if (!name) return 'Output';
        let clean = name.replace(/speakers/gi, "").trim();
        return clean;
      }

    async function selectDevice(device: string) {
        await invoke('set_audio_device', { deviceName: device });
        _manualOverride = true;
        manualOverride = true;
        currentDevice = device;
        showDevicePicker = false;
    }

    async function clearOverride() {
        await invoke('clear_audio_device_override');
        _manualOverride = false;
        manualOverride = false;
        currentDevice = await invoke('get_current_audio_device');
        showDevicePicker = false;
    }

    const minimize = () => win?.minimize();
    const maximizeOrRestore = () =>
      win?.isMaximized().then((max: boolean) => (max ? win.unmaximize() : win.maximize()));
    const close = () => win?.close();
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="app-info" data-tauri-drag-region>
    <button class="name-trigger" on:click={() => (showInfoModal = true)}>
      <span class="name">Duet!</span>
    </button>
  </div>
  {#if showInfoModal}
    <div class="modal-overlay" on:click={() => (showInfoModal = false)}>
      <div class="info-card" on:click|stopPropagation>
        <div class="glow-bg"></div>
        <button class="close-modal" on:click={() => (showInfoModal = false)}>✕</button>

        <div class="card-content">
          <div class="avatar">DK!</div>
          <h2>{myName}</h2>
          <p class="role">Developer • DUET Music</p>
          <div class="links">
            <a
              href={githubUrl}
              class="github-link"
              on:click={handleGithubClick}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.041-1.416-4.041-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.744.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
              GitHub Profile
            </a>
          </div>
        </div>
      </div>
    </div>
  {/if}
  <div class="controls">
    <div class="device-switcher" class:open={showDevicePicker}>
        <button class="device-btn" on:click={() => (showDevicePicker = !showDevicePicker)}>
          <svg width="11" height="11" viewBox="0 0 12 12" fill="none" style="flex-shrink: 0;">
            <path d="M2 4.5h2l2.5-2v7L4 7.5H2z" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/>
            <path d="M8 3.5c.8.6 1.2 1.5 1.2 2.5S8.8 7.9 8 8.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/>
          </svg>

          <span class="device-name" style="flex: 1; text-align: left; padding: 0 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            {shortName(currentDevice)}
          </span>

          <svg width="7" height="7" viewBox="0 0 8 8" style="flex-shrink: 0; opacity: 0.5;">
            <path d="M2 3l2 2 2-2" stroke="currentColor" fill="none" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
        </button>

      {#if showDevicePicker}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <div class="backdrop" on:click={() => (showDevicePicker = false)}></div>
        <div class="device-dropdown">
          <div class="dropdown-label">Audio Output</div>
          {#each devices as device}
            <button
              class="device-option"
              class:active={device === currentDevice}
              on:click={() => selectDevice(device)}
            >
              <span class="check">{device === currentDevice ? '✓' : ''}</span>
              <span class="dname">{device.replace(/Speakers/, "").trim()}</span>
            </button>
          {/each}
          <div class="dropdown-divider"></div>
          <button class="device-option auto-option" on:click={clearOverride}>
            <span class="check">{!manualOverride ? '✓' : ''}</span>
            <span class="dname">Auto (follow OS)</span>
          </button>
        </div>
      {/if}
    </div>

    <button class="btn" on:click={minimize}>
      <svg width="12" height="12" viewBox="0 0 12 12">
        <rect x="3" y="6" width="6" height="1" fill="currentColor"/>
      </svg>
    </button>
    <button class="btn" on:click={maximizeOrRestore}>
      {#if isMaximized}
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path d="M3 4.5h4.5V9H3zM4.5 3h4.5v4.5H8" fill="none" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      {:else}
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="3.5" y="3.5" width="5" height="5" stroke="currentColor" fill="none" stroke-width="1.2"/>
        </svg>
      {/if}
    </button>
    <button class="btn btn-close" on:click={close}>
      <svg width="12" height="12" viewBox="0 0 12 12">
        <path d="M3.5 3.5l5 5m0-5l-5 5" stroke="currentColor" fill="none" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .titlebar {
    height: 35px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px;
    background: transparent;
    user-select: none;
    position: relative;
  }
  .app-info {
      display: flex;
      align-items: center;
      gap: 12px;
      -webkit-app-region: drag;
    }

    .name {
      font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
      font-size: 23px;
      font-weight: 900;
      letter-spacing: -0.5px;
      text-transform: uppercase;
      background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      filter: drop-shadow(0 0 12px rgba(245, 158, 11, 0.4));
      line-height: 1;
      display: inline-block;
    }

    .name-trigger {
      background: none;
      border: none;
      padding: 0;
      margin: 0;
      cursor: pointer;
      -webkit-app-region: no-drag;
      transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    .name-trigger:hover {
      transform: scale(1.05);
    }

    /* --- Super UI Modal --- */
    .modal-overlay {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.8);
      backdrop-filter: blur(12px);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 9999;
      animation: fadeIn 0.2s ease;
    }

    .info-card {
      position: relative;
      width: 300px;
      background: #111;
      border: 1px solid rgba(245, 158, 11, 0.3);
      border-radius: 24px;
      padding: 40px 20px;
      text-align: center;
      box-shadow: 0 20px 40px rgba(0,0,0,0.6);
      animation: popIn 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }

    .glow-bg {
      position: absolute;
      inset: 0;
      background: radial-gradient(circle at center, rgba(245, 158, 11, 0.15) 0%, transparent 70%);
      pointer-events: none;
    }

    .avatar {
      width: 70px;
      height: 70px;
      background: linear-gradient(135deg, #f59e0b, #d97706);
      border-radius: 18px;
      margin: 0 auto 15px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 28px;
      font-weight: 900;
      color: #000;
    }

    .info-card h2 { color: #fff; margin: 0; font-size: 22px; }

    .role {
      color: #f59e0b;
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 2px;
      margin: 10px 0 25px;
    }

    .github-link {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      background: #fff;
      color: #000;
      text-decoration: none;
      padding: 10px 20px;
      border-radius: 12px;
      font-weight: 700;
      font-size: 13px;
      transition: 0.2s;
    }

    .github-link:hover {
      background: #f59e0b;
      transform: translateY(-2px);
    }

    .close-modal {
      position: absolute;
      top: 15px;
      right: 15px;
      background: none;
      border: none;
      color: #444;
      cursor: pointer;
      font-size: 16px;
    }

    .close-modal:hover { color: #fff; }

    @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
    @keyframes popIn {
      from { opacity: 0; transform: scale(0.9) translateY(10px); }
      to { opacity: 1; transform: scale(1) translateY(0); }
    }

  .controls {
    display: flex;
    align-items: center;
    height: 100%;
    -webkit-app-region: no-drag;
  }

  /* ── Device switcher ── */
  .device-switcher {
    position: relative;
    display: flex;
    align-items: center;
    margin-right: 4px;
  }

  .device-btn {
      display: flex;
      align-items: center;
      justify-content: flex-start;
      height: 24px;
      padding: 0 10px;
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 12px;
      background: rgba(255, 255, 255, 0.05);
      color: #999;
      font-size: 11px;
      cursor: pointer;
      width: auto;
      transition: all 0.2s ease;
    }

    .device-name {
      flex: 2;
      text-align: left;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      font-weight: 500;
    }

  .device-btn:hover,
  .device-switcher.open .device-btn {
    background: rgba(255, 255, 255, 0.09);
    color: #bbb;
    border-color: rgba(255, 255, 255, 0.13);
  }

  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .device-dropdown {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    left: auto;
    transform: none;
    background: #181818;
    border: 1px solid rgba(255, 255, 255, 0.09);
    border-radius: 8px;
    padding: 4px;
    min-width: 220px;
    z-index: 100;
    box-shadow: 0 8px 28px rgba(0, 0, 0, 0.5);
  }

  .dropdown-label {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: #444;
    padding: 4px 10px 6px;
  }

  .device-option {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 8px;
    border: none;
    background: transparent;
    color: #888;
    font-size: 11px;
    text-align: left;
    border-radius: 5px;
    cursor: pointer;
  }

  .device-option:hover {
    background: rgba(255, 255, 255, 0.06);
    color: #ddd;
  }

  .device-option.active {
    color: #fff;
  }

  .check {
    width: 12px;
    font-size: 10px;
    color: #f90;
    flex-shrink: 0;
  }

  .dname {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── Window buttons ── */
  .btn {
    width: 36px;
    height: 32px;
    border: none;
    background: transparent;
    color: #888;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.1s;
    cursor: pointer;
  }

  .btn:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #fff;
  }

  .btn-close:hover {
    background: #e81123 !important;
    color: white !important;
  }

  .dropdown-divider {
    height: 1px;
    background: rgba(255,255,255,0.06);
    margin: 4px 0;
  }
  .auto-option { color: #555; }
  .auto-option:hover { color: #aaa; }
</style>
