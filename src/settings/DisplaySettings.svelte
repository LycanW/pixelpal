<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { emit } from '@tauri-apps/api/event';

  let petsDir = $state('');
  let scale = $state(5);
  let alwaysOnTop = $state(true);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function load() {
    loading = true;
    error = null;
    try {
      petsDir = await invoke<string>('get_pets_dir');
      scale = await invoke<number>('get_scale');
      alwaysOnTop = await invoke<boolean>('get_always_on_top');
    } catch (e) {
      error = `Failed to load settings: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  async function setScale(s: number) {
    scale = s;
    try {
      await invoke('set_scale', { scale: s });
      await emit('scale-changed', s);
    } catch (e) {
      console.error('Scale update failed:', e);
    }
  }

  async function toggleAot() {
    alwaysOnTop = !alwaysOnTop;
    try {
      await invoke('set_always_on_top', { on: alwaysOnTop });
    } catch (e) {
      alwaysOnTop = !alwaysOnTop;
      console.error('Always-on-top toggle failed:', e);
    }
  }

  load();
</script>

<div class="editor-panel">
  <h2>Display</h2>

  {#if loading}
    <p class="status-message">Loading settings…</p>
  {:else if error}
    <div class="error-box">
      <p>{error}</p>
      <button class="btn" onclick={() => load()}>Retry</button>
    </div>
  {:else}
    <div class="field">
      <span class="label">Pets Directory</span>
      <div class="path-row"><input readonly value={petsDir} /><span class="path-hint">app-settings.json</span></div>
    </div>

    <div class="field">
      <label class="label" for="scale-range">Pet Scale: {scale}&times;</label>
      <div class="scale-row">
        <input id="scale-range" type="range" min="1" max="10" value={scale} oninput={(e) => { const v = parseInt((e.target as HTMLInputElement).value); if (!isNaN(v)) setScale(v); }} />
        <span class="scale-val">{scale}&times;</span>
      </div>
    </div>

    <div class="field">
      <label class="label" for="aot-check">Always on Top</label>
      <label class="toggle" for="aot-check">
        <input id="aot-check" type="checkbox" checked={alwaysOnTop} onchange={toggleAot} />
        {alwaysOnTop ? 'On' : 'Off'}
      </label>
    </div>
  {/if}
</div>

<style>
  .editor-panel { }
  h2 { font-size: 15px; margin: 0 0 14px; color: var(--text-primary); }
  .field { margin-bottom: 18px; }
  .field .label { display: block; font-size: 13px; font-weight: 500; margin-bottom: 4px; color: var(--text-primary); }
  .status-message { color: var(--text-muted); font-size: 13px; }
  .error-box { background: #fce4e4; border: 1px solid #c62828; border-radius: var(--radius-sm); padding: 10px; display: flex; flex-direction: column; gap: 8px; }
  .error-box p { margin: 0; font-size: 13px; color: #c62828; }
  .btn { padding: 5px 12px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
  .path-row { display: flex; align-items: center; gap: 10px; }
  .path-row input { flex: 1; padding: 6px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 12px; background: var(--bg-secondary); color: var(--text-secondary); }
  .path-hint { font-size: 11px; color: var(--text-muted); white-space: nowrap; }
  .scale-row { display: flex; align-items: center; gap: 10px; }
  .scale-row input[type=range] { flex: 1; accent-color: var(--accent); }
  .scale-val { font-size: 16px; font-weight: 600; color: var(--text-primary); min-width: 40px; }
  .toggle { display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 13px; color: var(--text-primary); }
  .toggle input { width: 18px; height: 18px; }
</style>
