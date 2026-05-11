<script lang="ts">
  import StateEditor from './StateEditor.svelte';
  import PetConfigEditor from './PetConfigEditor.svelte';
  import AnimationEditor from './AnimationEditor.svelte';

  let { petId, onBack, onDirtyChange }: { petId: string; onBack: () => void; onDirtyChange?: (d: Record<string, boolean>) => void } = $props();

  import { t } from '../lib/i18n.svelte';

  let tab = $state('interactions');
  let dirtyTabs = $state<Record<string, boolean>>({});

  const tabs = [
    { id: 'interactions', label: t('detail.interactions') },
    { id: 'animations', label: t('detail.animations') },
    { id: 'config', label: t('detail.config') },
  ];

  function setDirty(t: string, d: boolean) {
    const next = { ...dirtyTabs, [t]: d };
    dirtyTabs = next;
    onDirtyChange?.(next);
  }

  function switchTab(newTab: string) {
    if (newTab === tab) return;
    if (Object.values(dirtyTabs).some(Boolean)) {
      if (!window.confirm('You have unsaved changes. Switch tabs?')) return;
      dirtyTabs = {};
    }
    tab = newTab;
  }

  function handleBack() {
    if (Object.values(dirtyTabs).some(Boolean)) {
      if (!window.confirm('You have unsaved changes. Go back?')) return;
    }
    onBack();
  }
</script>

<div class="detail">
  <div class="top-bar">
    <button class="back-btn" onclick={handleBack}>{t('detail.back')}</button>
    <span class="pet-name">{petId}</span>
  </div>

  <div class="tab-bar" role="tablist">
    {#each tabs as t}
      <button role="tab" aria-selected={tab === t.id} class:dirty={dirtyTabs[t.id]} class:active={tab === t.id} onclick={() => switchTab(t.id)}>
        {t.label}{#if dirtyTabs[t.id]}<span class="dot">●</span>{/if}
      </button>
    {/each}
  </div>

  <div class="tab-content">
    {#if tab === 'interactions'}
      <StateEditor {petId} onDirtyChange={(d) => setDirty('interactions', d)} />
    {/if}
    {#if tab === 'animations'}
      <AnimationEditor {petId} onDirtyChange={(d) => setDirty('animations', d)} />
    {/if}
    {#if tab === 'config'}
      <PetConfigEditor {petId} onDirtyChange={(d) => setDirty('config', d)} />
    {/if}
  </div>
</div>

<style>
  .detail { }
  .top-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
  .back-btn { background: none; border: none; color: var(--accent); cursor: pointer; font-size: 14px; padding: 0; font-family: inherit; }
  .back-btn:hover { text-decoration: underline; }
  .pet-name { font-size: 16px; font-weight: 600; color: var(--text-primary); }
  .tab-bar { display: flex; gap: 4px; border-bottom: 2px solid var(--border); margin-bottom: 16px; }
  .tab-bar button { padding: 8px 16px; border: none; background: none; cursor: pointer; font-size: 14px; color: var(--text-secondary); border-bottom: 2px solid transparent; margin-bottom: -2px; position: relative; font-family: inherit; }
  .tab-bar button:focus-visible { box-shadow: var(--focus-ring); border-radius: var(--radius-sm); }
  .tab-bar button.active { color: var(--text-primary); border-bottom-color: var(--accent); font-weight: 600; }
  .dot { color: var(--accent); margin-left: 4px; font-size: 8px; vertical-align: super; }
  .tab-content { min-height: 300px; }
</style>
