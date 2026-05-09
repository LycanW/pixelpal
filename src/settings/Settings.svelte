<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, emit } from '@tauri-apps/api/event';
  import HomeView from './HomeView.svelte';
  import PetDetailView from './PetDetailView.svelte';
  import DisplaySettings from './DisplaySettings.svelte';

  let view = $state<'home' | 'detail' | 'display'>('home');
  let selectedPet = $state('');
  let activePetId = $state('');
  let dirtyTabs = $state<Record<string, boolean>>({});
  let unlistenFn: (() => void) | null = null;

  async function setActivePet(id: string) {
    try {
      await invoke('set_active_pet', { id });
      activePetId = id;
      await emit('pet-changed', id);
    } catch (e) { console.error('setActivePet:', e); }
  }

  function openPetSettings(id: string) {
    selectedPet = id;
    view = 'detail';
  }

  function backToHome() {
    view = 'home';
    selectedPet = '';
  }

  async function refreshActive() {
    try { activePetId = await invoke<string>('get_active_pet'); } catch (e) { console.error('refreshActive:', e); }
  }

  async function handlePetChanged(e: { payload: unknown }) {
    const newPet = typeof e.payload === 'string' ? e.payload : '';
    if (!newPet) return;
    activePetId = newPet;
    // If we're viewing a pet detail and the pet changed externally
    if (view === 'detail' && selectedPet && selectedPet !== newPet) {
      if (Object.values(dirtyTabs).some(Boolean)) {
        if (window.confirm(`Pet changed to "${newPet}". Discard unsaved changes?`)) {
          selectedPet = newPet;
          dirtyTabs = {};
        }
      } else {
        selectedPet = newPet;
      }
    }
  }

  onMount(() => {
    refreshActive();
    const unlisten = listen('pet-changed', handlePetChanged);
    unlisten.then(fn => { unlistenFn = fn; }).catch(e => { console.error('listen pet-changed:', e); });
    return () => { unlistenFn?.(); };
  });
</script>

<div class="container">
  <header>
    <h1>Settings</h1>
    <div class="header-actions">
      {#if view === 'detail'}
        <!-- back handled inside PetDetailView -->
      {:else}
        <button class="gear-btn" onclick={() => { view = view === 'display' ? 'home' : 'display'; }}>
          {view === 'display' ? '← Pets' : '⚙ Display'}
        </button>
      {/if}
    </div>
  </header>

  <div style="display: {view === 'home' ? 'block' : 'none'}">
    <HomeView onActivatePet={setActivePet} onSettingsPet={openPetSettings} {activePetId} />
  </div>

  <div style="display: {view === 'detail' ? 'block' : 'none'}">
    {#if selectedPet}
      <PetDetailView petId={selectedPet} onBack={backToHome} onDirtyChange={(d) => { dirtyTabs = d; }} />
    {/if}
  </div>

  <div style="display: {view === 'display' ? 'block' : 'none'}">
    <DisplaySettings />
  </div>
</div>

<style>
  .container { max-width: 620px; margin: 0 auto; padding: 16px 20px; overflow-y: auto; height: 100vh; box-sizing: border-box; }
  header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  header h1 { font-size: 20px; margin: 0; }
  .gear-btn { background: none; border: 1px solid var(--border); color: var(--text-secondary); padding: 4px 12px; border-radius: var(--radius-sm); cursor: pointer; font-size: 13px; font-family: inherit; }
  .gear-btn:focus-visible { box-shadow: var(--focus-ring); }
</style>
