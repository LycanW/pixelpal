<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '../lib/i18n.svelte';
  import PetCard from './PetCard.svelte';
  import AiGenerationModal from './AiGenerationModal.svelte';

  let {
    onActivatePet,
    onSettingsPet,
    activePetId,
  }: {
    onActivatePet: (id: string) => void;
    onSettingsPet: (id: string) => void;
    activePetId: string;
  } = $props();

  let pets = $state<string[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showCreate = $state(false);
  let newName = $state('');
  let createTab = $state<'blank' | 'ai'>('blank');
  let showAiCreate = $state(false);

  async function loadPets() {
    loading = true;
    error = null;
    try { pets = await invoke<string[]>('list_pets'); }
    catch (e) { error = `Failed to load: ${e instanceof Error ? e.message : e}`; }
    finally { loading = false; }
  }

  async function doCreate() {
    const name = newName.trim();
    if (!name) return;
    try {
      await invoke('create_pet', { name, frameSize: 32 });
      showCreate = false;
      newName = '';
      loadPets();
    } catch (e) { error = `Create failed: ${e instanceof Error ? e.message : e}`; }
  }

  async function doImport() {
    try {
      await invoke('import_pet');
      loadPets();
    } catch (e) { error = `Import failed: ${e instanceof Error ? e.message : e}`; }
  }

  onMount(() => { loadPets(); });
</script>

<div class="home">
  <div class="toolbar">
    <h2>{t('home.title')}</h2>
    <div class="toolbar-actions">
      <button class="btn" onclick={() => { showCreate = true; }}>{t('home.new')}</button>
      <button class="btn subtle" onclick={doImport}>{t('home.import')}</button>
    </div>
  </div>

  {#if showCreate}
    <div class="modal-overlay" onclick={() => { showCreate = false; }} role="presentation">
      <div class="modal" onclick={(e: MouseEvent) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') showCreate = false; }}>
        <div class="tab-bar">
          <button class:active={createTab === 'blank'} onclick={() => createTab = 'blank'}>{t('home.newPet')}</button>
          <button class:active={createTab === 'ai'} onclick={() => createTab = 'ai'}>AI {t('home.new')}</button>
        </div>
        {#if createTab === 'blank'}
          <h3>{t('home.newPet')}</h3>
          <label>{t('home.name')} <input type="text" bind:value={newName} placeholder="my-pet" /></label>
          <div class="modal-actions">
            <button class="btn" onclick={doCreate} disabled={!newName.trim()}>{t('home.create')}</button>
            <button class="btn subtle" onclick={() => { showCreate = false; }}>{t('home.cancel')}</button>
          </div>
        {:else}
          <h3>AI {t('home.newPet')}</h3>
          <label>{t('ai.description')}
            <input type="text" bind:value={newName} placeholder="a cute orange cat" />
          </label>
          <div class="modal-actions">
            <button class="btn" onclick={() => { showCreate = false; showAiCreate = true; }} disabled={!newName.trim()}>
              {t('ai.generate')}
            </button>
            <button class="btn subtle" onclick={() => { showCreate = false; }}>{t('home.cancel')}</button>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if loading}
    <div class="grid">
      {#each [1,2,3] as _}
        <div class="card skeleton"><div class="skel-img"></div><div class="skel-line"></div></div>
      {/each}
    </div>
  {:else if error}
    <div class="error-box"><p>{error}</p><button class="btn" onclick={loadPets}>Retry</button></div>
  {:else if pets.length === 0}
    <p class="empty">{t('home.noPets')}</p>
  {:else}
    <div class="grid">
      {#each pets as id}
        <button class="card" class:active={id === activePetId} onclick={() => onActivatePet(id)}>
          <PetCard petId={id} />
          <span class="name-row">
            {id}
            <span class="gear-icon" role="button" tabindex="0" onclick={(e: MouseEvent) => { e.stopPropagation(); onSettingsPet(id); }} onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.stopPropagation(); onSettingsPet(id); } }} title="Settings">⚙</span>
          </span>
          {#if id === activePetId}<span class="badge">{t('home.active')}</span>{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

{#if showAiCreate}
  <AiGenerationModal
    petId={newName.trim()}
    animationName="idle"
    onClose={() => { showAiCreate = false; newName = ''; }}
    onSaved={() => {
      showAiCreate = false;
      newName = '';
      loadPets();
      onActivatePet(newName.trim());
    }}
  />
{/if}

<style>
  .toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  h2 { font-size: 15px; margin: 0; color: var(--text-primary); }
  .toolbar-actions { display: flex; gap: 6px; }
  .btn { padding: 5px 12px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .btn.subtle { background: transparent; color: var(--text-secondary); border-color: var(--border); }
  .grid { display: flex; flex-wrap: wrap; gap: 10px; }
  .card { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 10px; border: 2px solid var(--border); border-radius: var(--radius-md); background: var(--bg-secondary); cursor: pointer; width: 110px; color: var(--text-primary); font-family: inherit; position: relative; }
  .card:focus-visible { box-shadow: var(--focus-ring); }
  .card.active { border-color: var(--accent); background: #eef4fb; }
  .card span { font-size: 12px; text-align: center; }
  .name-row { display: flex; align-items: center; gap: 2px; }
  .gear-icon { background: none; border: none; cursor: pointer; font-size: 14px; padding: 2px; line-height: 1; color: var(--text-muted); border-radius: var(--radius-sm); }
  .gear-icon:hover { color: var(--text-primary); background: var(--border); }
  .gear-icon:focus-visible { box-shadow: var(--focus-ring); }
  .badge { font-size: 10px; color: var(--accent); font-weight: 600; }
  .empty { color: var(--text-muted); font-size: 13px; }
  .error-box { background: #fce4e4; border: 1px solid #c62828; border-radius: var(--radius-sm); padding: 10px; display: flex; flex-direction: column; gap: 8px; }
  .error-box p { margin: 0; font-size: 13px; color: #c62828; }
  .modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.35); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .modal { background: var(--bg-primary); border-radius: var(--radius-md); padding: 20px; min-width: 280px; display: flex; flex-direction: column; gap: 12px; box-shadow: 0 4px 16px rgba(0,0,0,0.15); }
  .modal h3 { margin: 0; font-size: 16px; color: var(--text-primary); }
  .modal label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--text-secondary); }
  .modal input { padding: 6px 8px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 13px; background: var(--bg-secondary); color: var(--text-primary); }
  .modal-actions { display: flex; gap: 6px; justify-content: flex-end; margin-top: 4px; }
  .skeleton { cursor: default; padding-top: 34px; }
  .skel-img { width: 64px; height: 64px; background: var(--border); border-radius: 4px; animation: pulse 1.5s infinite; }
  .skel-line { width: 50px; height: 10px; background: var(--border); border-radius: 3px; animation: pulse 1.5s infinite; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
  .tab-bar { display: flex; gap: 4px; margin-bottom: 12px; border-bottom: 1px solid var(--border); }
  .tab-bar button { padding: 6px 12px; border: none; background: none; cursor: pointer; font-size: 13px; color: var(--text-secondary); border-bottom: 2px solid transparent; margin-bottom: -1px; font-family: inherit; }
  .tab-bar button.active { color: var(--text-primary); border-bottom-color: var(--accent); font-weight: 600; }
</style>
