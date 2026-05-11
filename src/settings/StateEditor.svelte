<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { emit, listen } from '@tauri-apps/api/event';
  import { t } from '../lib/i18n.svelte';
  import type { PetEvent, StateConfig } from '../lib/pet/types';

  let { petId, onDirtyChange }: { petId: string; onDirtyChange?: (dirty: boolean) => void } = $props();

  const EVENTS: PetEvent[] = ['press', 'click', 'dblclick', 'right_click', 'drag_start', 'drag_end', 'animation_end'];

  let states = $state<Record<string, StateConfig>>({});
  let animationNames = $state<string[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let dirty = $state(false);
  let dirtyStates = $state<Record<string, boolean>>({});

  let stateNames = $derived(Object.keys(states));
  let unlistenFn: (() => void) | null = null;

  function markDirty(name?: string) {
    dirty = true;
    if (name) dirtyStates = { ...dirtyStates, [name]: true };
    onDirtyChange?.(true);
  }
  function clearDirty() {
    dirty = false;
    dirtyStates = {};
    onDirtyChange?.(false);
  }

  function fallbackAnimation() {
    return animationNames[0] || 'idle';
  }

  function uniqueStateName() {
    let i = 1;
    let name = 'state';
    while (states[name]) {
      i += 1;
      name = `state_${i}`;
    }
    return name;
  }

  function firstAvailableEvent(state: StateConfig) {
    return EVENTS.find((event) => !state.transitions[event]) ?? null;
  }

  async function refreshAnimationNames() {
    try {
      const raw = await invoke<string>('read_json', { id: petId, filename: 'config.json' });
      animationNames = Object.keys(JSON.parse(raw).animations || {});
    } catch (e) {
      console.error('refreshAnimationNames:', e);
    }
  }

  async function load() {
    loading = true;
    error = null;
    try {
      const raw = await invoke<string>('read_json', { id: petId, filename: 'config.json' });
      const cfg = JSON.parse(raw);
      animationNames = Object.keys(cfg.animations || {});
      states = cfg.states || {};
      if (Object.keys(states).length === 0) {
        states = { idle: { entry: fallbackAnimation(), transitions: {} } };
        markDirty('idle');
      } else {
        clearDirty();
      }
    } catch (e) {
      error = `Failed to load: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  async function saveState(name: string) {
    error = null;
    try {
      const raw = await invoke<string>('read_json', { id: petId, filename: 'config.json' });
      const cfg = JSON.parse(raw);
      cfg.states = states;
      if (!cfg.defaultState || !states[cfg.defaultState]) {
        cfg.defaultState = Object.keys(states)[0] || 'idle';
      }
      await invoke('write_json', { id: petId, filename: 'config.json', content: JSON.stringify(cfg, null, 2) });
      clearDirty();
      await emit('pet-changed', petId);
    } catch (e) {
      error = `Save failed: ${e instanceof Error ? e.message : e}`;
    }
  }

  function addState() {
    const name = uniqueStateName();
    states = { ...states, [name]: { entry: fallbackAnimation(), transitions: {} } };
    markDirty(name);
  }

  function removeState(name: string) {
    if (stateNames.length <= 1) return;
    if (!window.confirm(`Delete state "${name}"?`)) return;

    const affected = new Set<string>();
    const next: Record<string, StateConfig> = {};
    for (const [stateName, state] of Object.entries(states)) {
      if (stateName === name) continue;
      const transitions = { ...state.transitions };
      for (const [event, transition] of Object.entries(transitions)) {
        if (transition?.target === name) {
          delete transitions[event as PetEvent];
          affected.add(stateName);
        }
      }
      next[stateName] = { ...state, transitions };
    }
    states = next;
    dirtyStates = { ...dirtyStates, ...Object.fromEntries([...affected].map((stateName) => [stateName, true])) };
    markDirty();
  }

  function updateEntry(name: string, entry: string) {
    states = { ...states, [name]: { ...states[name], entry } };
    markDirty(name);
  }

  function addTransition(name: string) {
    const state = states[name];
    const event = firstAvailableEvent(state);
    if (!event) return;

    states = {
      ...states,
      [name]: {
        ...state,
        transitions: {
          ...state.transitions,
          [event]: { target: name },
        },
      },
    };
    markDirty(name);
  }

  function updateTransitionEvent(name: string, oldEvent: PetEvent, newEvent: PetEvent) {
    if (oldEvent === newEvent || states[name].transitions[newEvent]) return;
    const transitions = { ...states[name].transitions };
    const transition = transitions[oldEvent];
    delete transitions[oldEvent];
    if (transition) transitions[newEvent] = transition;
    states = { ...states, [name]: { ...states[name], transitions } };
    markDirty(name);
  }

  function updateTransitionTarget(name: string, event: PetEvent, target: string) {
    const transition = states[name].transitions[event];
    if (!transition) return;
    states = {
      ...states,
      [name]: {
        ...states[name],
        transitions: {
          ...states[name].transitions,
          [event]: { ...transition, target },
        },
      },
    };
    markDirty(name);
  }

  function updateTransitionAnimation(name: string, event: PetEvent, animation: string) {
    const transition = states[name].transitions[event];
    if (!transition) return;
    const next = { ...transition, animation: animation || undefined };
    states = {
      ...states,
      [name]: {
        ...states[name],
        transitions: {
          ...states[name].transitions,
          [event]: next,
        },
      },
    };
    markDirty(name);
  }

  function removeTransition(name: string, event: PetEvent) {
    const transitions = { ...states[name].transitions };
    delete transitions[event];
    states = { ...states, [name]: { ...states[name], transitions } };
    markDirty(name);
  }

  onMount(() => {
    load();
    const ul = listen('pet-changed', async (e) => {
      if (typeof e.payload === 'string' && e.payload === petId && !dirty) await load();
      if (typeof e.payload === 'string' && e.payload === petId && dirty) await refreshAnimationNames();
    });
    ul.then(fn => { unlistenFn = fn; }).catch(e => { console.error('listen pet-changed:', e); });
    return () => { unlistenFn?.(); };
  });
</script>

<div class="editor-panel">
  <div class="header">
    <h2>{t('state.title')}</h2>
    <button class="btn" onclick={addState}>{t('state.add')}</button>
  </div>

  {#if loading}
    <p class="status">Loading…</p>
  {:else if error && !dirty}
    <div class="error-box"><p>{error}</p><button class="btn" onclick={load}>Retry</button></div>
  {:else}
    <div class="states">
      {#each Object.entries(states) as [name, state]}
        <section class="state-card">
          <div class="state-header">
            <h3>{name}</h3>
            <div class="state-actions">
              {#if dirtyStates[name]}
                <button class="btn" onclick={() => saveState(name)}>Save</button>
              {/if}
              <button class="del-btn" onclick={() => removeState(name)} disabled={stateNames.length <= 1} title="Remove state">✕</button>
            </div>
          </div>

          <label class="entry">{t('state.entry')}
            <select value={state.entry} onchange={(e) => updateEntry(name, (e.target as HTMLSelectElement).value)}>
              {#each animationNames as anim}<option value={anim}>{anim}</option>{/each}
              {#if !animationNames.includes(state.entry)}<option value={state.entry}>{state.entry}</option>{/if}
            </select>
          </label>

          <div class="transitions-head">
            <span>{t('state.transitions')}</span>
            <button class="btn subtle" onclick={() => addTransition(name)} disabled={!firstAvailableEvent(state)}>{t('state.addTransition')}</button>
          </div>

          {#if Object.keys(state.transitions).length === 0}
            <p class="empty">{t('state.noTransitions')}</p>
          {:else}
            <div class="table">
              <div class="row hdr">
                <span>{t('state.event')}</span>
                <span>{t('state.target')}</span>
                <span>{t('state.override')}</span>
                <span></span>
              </div>
              {#each Object.entries(state.transitions) as [event, transition]}
                <div class="row">
                  <select value={event} onchange={(e) => updateTransitionEvent(name, event as PetEvent, (e.target as HTMLSelectElement).value as PetEvent)}>
                    {#each EVENTS as ev}
                      <option value={ev} disabled={ev !== event && !!state.transitions[ev]}>{ev}</option>
                    {/each}
                  </select>
                  <select value={transition.target} onchange={(e) => updateTransitionTarget(name, event as PetEvent, (e.target as HTMLSelectElement).value)}>
                    {#each stateNames as stateName}<option value={stateName}>{stateName}</option>{/each}
                  </select>
                  <select value={transition.animation ?? ''} onchange={(e) => updateTransitionAnimation(name, event as PetEvent, (e.target as HTMLSelectElement).value)}>
                    <option value="">{t('state.useTarget')}</option>
                    {#each animationNames as anim}<option value={anim}>{anim}</option>{/each}
                    {#if transition.animation && !animationNames.includes(transition.animation)}<option value={transition.animation}>{transition.animation}</option>{/if}
                  </select>
                  <button class="del-btn" onclick={() => removeTransition(name, event as PetEvent)} title="Remove transition">✕</button>
                </div>
              {/each}
            </div>
          {/if}
        </section>
      {/each}
    </div>
  {/if}

  {#if error && dirty}
    <div class="error-box"><p>{error}</p></div>
  {/if}
</div>

<style>
  .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; flex-wrap: wrap; gap: 8px; }
  h2 { font-size: 15px; margin: 0; color: var(--text-primary); }
  h3 { font-size: 14px; margin: 0; color: var(--text-primary); }
  .btn { padding: 5px 12px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .btn.subtle { background: transparent; color: var(--text-secondary); border-color: var(--border); }
  .status, .empty { color: var(--text-muted); font-size: 13px; }
  .states { display: flex; flex-direction: column; gap: 10px; }
  .state-card { border: 1px solid var(--border); border-radius: var(--radius-md); padding: 10px; background: var(--bg-primary); display: flex; flex-direction: column; gap: 10px; }
  .state-header, .transitions-head { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
  .state-actions { display: flex; align-items: center; gap: 6px; }
  .entry { display: grid; grid-template-columns: 120px minmax(140px, 1fr); gap: 8px; align-items: center; font-size: 13px; color: var(--text-secondary); }
  .entry select, .row select { padding: 5px 6px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 12px; background: var(--bg-secondary); color: var(--text-primary); width: 100%; box-sizing: border-box; }
  .entry select:focus, .row select:focus { outline: none; box-shadow: var(--focus-ring); }
  .transitions-head span { font-size: 12px; font-weight: 600; color: var(--text-muted); }
  .table { display: flex; flex-direction: column; gap: 4px; }
  .row { display: grid; grid-template-columns: minmax(110px, 0.8fr) minmax(110px, 1fr) minmax(130px, 1fr) 28px; gap: 4px; align-items: center; }
  .row.hdr { font-size: 11px; color: var(--text-muted); font-weight: 600; }
  .row.hdr span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .del-btn { background: none; border: none; cursor: pointer; color: var(--danger); font-size: 14px; width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; }
  .del-btn:disabled { opacity: 0.35; cursor: default; }
  .error-box { background: #fce4e4; border: 1px solid #c62828; border-radius: var(--radius-sm); padding: 10px; display: flex; flex-direction: column; gap: 8px; margin-bottom: 8px; }
  .error-box p { margin: 0; font-size: 13px; color: #c62828; }
</style>
