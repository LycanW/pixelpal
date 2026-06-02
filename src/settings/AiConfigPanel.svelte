<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '../lib/i18n.svelte';

  let baseUrl = $state('');
  let apiKey = $state('');
  let model = $state('gpt-image-1');
  let loading = $state(true);
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let saveSuccess = $state(false);

  async function load() {
    loading = true;
    try {
      const cfg = await invoke<{ base_url: string; has_key: boolean; model: string }>('get_ai_config');
      baseUrl = cfg.base_url;
      model = cfg.model || 'gpt-image-1';
    } catch (e) {
      console.error('load AI config:', e);
    } finally {
      loading = false;
    }
  }

  async function save() {
    saveError = null;
    saveSuccess = false;
    saving = true;
    try {
      await invoke('set_ai_config', { baseUrl, apiKey, model });
      apiKey = '';
      saveSuccess = true;
      setTimeout(() => { saveSuccess = false; }, 3000);
    } catch (e) {
      saveError = String(e);
      console.error('save AI config:', e);
    } finally {
      saving = false;
    }
  }

  load();
</script>

<div class="ai-config">
  <h3>{t('ai.title')}</h3>
  {#if loading}
    <p class="status">Loading...</p>
  {:else}
    <label>{t('ai.baseUrl')}
      <input type="text" bind:value={baseUrl} placeholder="https://api.openai.com/v1" />
    </label>
    <label>{t('ai.apiKey')}
      <input type="password" bind:value={apiKey} placeholder="sk-..." />
    </label>
    <label>{t('ai.model')}
      <input type="text" bind:value={model} placeholder="gpt-image-1" />
    </label>
    {#if saveSuccess}<p class="success">{t('ai.saveSuccess')}</p>{/if}
    {#if saveError}<p class="error">{t('ai.saveError')}: {saveError}</p>{/if}
    <button class="btn" onclick={save} disabled={saving}>{t('ai.save')}</button>
  {/if}
</div>

<style>
  .ai-config { margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border); }
  h3 { font-size: 15px; margin: 0 0 12px; color: var(--text-primary); }
  label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--text-secondary); margin-bottom: 10px; }
  input { padding: 6px 8px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 13px; background: var(--bg-secondary); color: var(--text-primary); }
  .btn { padding: 5px 14px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .status { color: var(--text-muted); font-size: 13px; }
  .success { color: #2e7d32; font-size: 12px; margin: 0 0 8px; }
  .error { color: #c62828; font-size: 12px; margin: 0 0 8px; }
</style>
