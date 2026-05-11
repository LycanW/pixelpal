<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { emit } from '@tauri-apps/api/event';
  import { t } from '../lib/i18n.svelte';

  let { petId, onDirtyChange }: { petId: string; onDirtyChange?: (dirty: boolean) => void } = $props();

  let name = $state('');
  let defaultState = $state('idle');
  let stateNames = $state<string[]>(['idle']);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let dirty = $state(false);

  function markDirty() { dirty = true; onDirtyChange?.(true); }
  function clearDirty() { dirty = false; onDirtyChange?.(false); }

  async function load() {
    loading = true;
    error = null;
    try {
      const [mRaw, cRaw] = await Promise.all([
        invoke<string>('read_json', { id: petId, filename: 'manifest.json' }),
        invoke<string>('read_json', { id: petId, filename: 'config.json' }),
      ]);
      const m = JSON.parse(mRaw);
      const c = JSON.parse(cRaw);
      name = m.name || petId;
      defaultState = c.defaultState || 'idle';
      stateNames = Object.keys(c.states || {});
      clearDirty();
    } catch (e) {
      error = `Failed to load: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  async function save() {
    error = null;
    try {
      const manifest = { name };
      await invoke('write_json', { id: petId, filename: 'manifest.json', content: JSON.stringify(manifest, null, 2) });
      const cRaw = await invoke<string>('read_json', { id: petId, filename: 'config.json' });
      const cfg = JSON.parse(cRaw);
      cfg.defaultState = defaultState;
      await invoke('write_json', { id: petId, filename: 'config.json', content: JSON.stringify(cfg, null, 2) });
      clearDirty();
      await emit('pet-changed', petId);
    } catch (e) {
      error = `Save failed: ${e instanceof Error ? e.message : e}`;
    }
  }

  onMount(() => { load(); });
</script>

<div class="editor-panel">
  <h2>{t('config.title')} — {petId}</h2>

  {#if loading}
    <p class="status">Loading…</p>
  {:else if error && !dirty}
    <div class="error-box"><p>{error}</p><button class="btn" onclick={load}>Retry</button></div>
  {:else}
    <div class="fields">
      <label>{t('config.name')} <input type="text" bind:value={name} oninput={markDirty} /></label>
      <label>{t('config.defaultState')}
        <select bind:value={defaultState} onchange={markDirty}>
          {#each stateNames as state}<option value={state}>{state}</option>{/each}
        </select>
      </label>
    </div>

    {#if error}
      <div class="error-box"><p>{error}</p></div>
    {/if}

    {#if dirty}
      <button class="btn" onclick={save}>{t('config.save')}</button>
    {/if}
  {/if}
</div>

<style>
  .editor-panel { }
  h2 { font-size: 15px; margin: 0 0 12px; color: var(--text-primary); }
  .status { color: var(--text-muted); font-size: 13px; }
  .fields { display: flex; flex-direction: column; gap: 10px; max-width: 320px; }
  .fields label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--text-secondary); }
  .fields input, .fields select { padding: 6px 8px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 13px; background: var(--bg-secondary); color: var(--text-primary); }
  .fields input:focus, .fields select:focus { outline: none; box-shadow: var(--focus-ring); }
  .btn { margin-top: 10px; padding: 5px 14px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
  .error-box { background: #fce4e4; border: 1px solid #c62828; border-radius: var(--radius-sm); padding: 10px; display: flex; flex-direction: column; gap: 8px; margin-top: 8px; }
  .error-box p { margin: 0; font-size: 13px; color: #c62828; }
</style>
