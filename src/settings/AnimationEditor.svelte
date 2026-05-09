<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { emit } from '@tauri-apps/api/event';
  import type { AnimationDef } from '../lib/pet/types';

  let { petId, onDirtyChange }: { petId: string; onDirtyChange?: (dirty: boolean) => void } = $props();

  let animations = $state<Record<string, AnimationDef>>({});
  let loading = $state(true);
  let error = $state<string | null>(null);
  let dirty = $state(false);
  let addForm = $state(false);
  let newName = $state('');
  let imageFiles = $state<string[]>([]);
  let renames = $state<Record<string, string>>({});

  function markDirty() { dirty = true; onDirtyChange?.(true); }
  function clearDirty() { dirty = false; onDirtyChange?.(false); }

  async function loadImages() {
    try { imageFiles = await invoke<string[]>('list_pet_images', { id: petId }); }
    catch (e) { console.error('loadImages:', e); imageFiles = []; }
  }

  async function load() {
    loading = true;
    error = null;
    try {
      const raw = await invoke<string>('read_json', { id: petId, filename: 'config.json' });
      const cfg = JSON.parse(raw);
      animations = cfg.animations || {};
      renames = {};
      clearDirty();
    } catch (e) {
      error = `Failed to load: ${e instanceof Error ? e.message : e}`;
    } finally {
      loading = false;
    }
  }

  function usedBy(source: string, defs: Record<string, AnimationDef> = animations) {
    return Object.entries(defs).filter(([, anim]) => anim.source === source).map(([name]) => name);
  }

  function cleanupAnimationReferences(cfg: any, removedName: string, fallbackName: string | undefined) {
    for (const state of Object.values(cfg.states || {}) as any[]) {
      if (state.entry === removedName && fallbackName) state.entry = fallbackName;
      for (const transition of Object.values(state.transitions || {}) as any[]) {
        if (transition.animation === removedName) delete transition.animation;
      }
    }
  }

  async function deleteImageFile(filename: string) {
    error = null;
    try {
      await invoke('delete_pet_image', { id: petId, filename });
      await loadImages();
      await emit('pet-changed', petId);
    } catch (e) {
      error = `Delete image failed: ${e instanceof Error ? e.message : e}`;
    }
  }

  async function deleteUnusedImage(filename: string) {
    if (usedBy(filename).length > 0) return;
    if (!window.confirm(`Delete image file "${filename}"?`)) return;
    await deleteImageFile(filename);
  }

  function renameReferences(cfg: any, oldName: string, newName: string) {
    for (const state of Object.values(cfg.states || {}) as any[]) {
      if (state.entry === oldName) state.entry = newName;
      for (const transition of Object.values(state.transitions || {}) as any[]) {
        if (transition.animation === oldName) transition.animation = newName;
      }
    }
  }

  function rename(oldName: string, rawName: string) {
    const newName = rawName.trim();
    if (!newName || newName === oldName || animations[newName]) return;
    const next: Record<string, AnimationDef> = {};
    for (const [name, anim] of Object.entries(animations)) {
      next[name === oldName ? newName : name] = anim;
    }
    animations = next;
    const originalName = Object.entries(renames).find(([, currentName]) => currentName === oldName)?.[0] ?? oldName;
    renames = { ...renames, [originalName]: newName };
    markDirty();
  }

  async function save() {
    error = null;
    try {
      const raw = await invoke<string>('read_json', { id: petId, filename: 'config.json' });
      const cfg = JSON.parse(raw);
      for (const removedName of Object.keys(cfg.animations || {})) {
        if (!animations[removedName]) cleanupAnimationReferences(cfg, removedName, Object.keys(animations)[0]);
      }
      for (const [oldName, newName] of Object.entries(renames)) {
        renameReferences(cfg, oldName, newName);
      }
      for (const [name, anim] of Object.entries(animations)) {
        const isGif = anim.source.toLowerCase().endsWith('.gif');
        if (isGif) continue;
        const fc = anim.frameCount ?? 4;
        const fpr = anim.framesPerRow ?? 2;
        if (!fc || fc <= 0) {
          error = `"${name}": Frames is empty or invalid`;
          return;
        }
        if (fc % fpr !== 0) {
          error = `"${name}": Frames (${fc}) must be divisible by Per Row (${fpr})`;
          return;
        }
      }
      cfg.animations = animations;
      if (cfg.defaultState && cfg.states?.[cfg.defaultState] && !animations[cfg.states[cfg.defaultState].entry]) {
        cfg.states[cfg.defaultState].entry = Object.keys(animations)[0] || '';
      }
      renames = {};
      await invoke('write_json', { id: petId, filename: 'config.json', content: JSON.stringify(cfg, null, 2) });
      clearDirty();
      await emit('pet-changed', petId);
    } catch (e) {
      error = `Save failed: ${e instanceof Error ? e.message : e}`;
    }
  }

  function gifDefaults(filename: string) {
    const isGif = filename.toLowerCase().endsWith('.gif');
    return {
      frameTime: isGif ? 0 : 100,
      frameCount: isGif ? undefined : 4,
      framesPerRow: isGif ? 1 : 2,
    };
  }

  function add() {
    const name = newName.trim();
    if (!name || animations[name]) return;
    const src = imageFiles[0] || `${name}.png`;
    animations = { ...animations, [name]: { source: src, ...gifDefaults(src), loop: true } };
    newName = '';
    addForm = false;
    markDirty();
  }

  async function doImportImage() {
    try {
      const fname = await invoke<string>('import_pet_image', { id: petId });
      await loadImages();
      // Pre-fill new animation name from filename if add form is open
      if (addForm && !newName) {
        newName = fname.replace(/\.(png|webp|jpg|jpeg|gif)$/i, '');
      }
    } catch (e) {
      if (typeof e === 'string' && e !== 'No file selected') {
        error = `Import failed: ${e}`;
      }
    }
  }

  async function remove(name: string) {
    if (!window.confirm(`Delete animation "${name}"?`)) return;
    const source = animations[name]?.source;
    const next = { ...animations };
    delete next[name];
    animations = next;
    markDirty();

    if (source && usedBy(source, next).length === 0 && window.confirm(`Also delete unused image file "${source}"?`)) {
      await deleteImageFile(source);
    }
  }

  onMount(() => { load(); loadImages(); });
</script>

<div class="editor-panel">
  <div class="header">
    <h2>Animations</h2>
    {#if addForm}
      <div class="inline-add">
        <input type="text" bind:value={newName} placeholder="animation name" />
        <button class="btn" onclick={add} disabled={!newName.trim()}>+</button>
        <button class="btn subtle" onclick={doImportImage}>Import Image</button>
        <button class="btn subtle" onclick={() => { addForm = false; }}>Cancel</button>
      </div>
    {:else}
      <button class="btn" onclick={() => { addForm = true; }}>+ Add</button>
      <button class="btn subtle" onclick={doImportImage}>Import Image</button>
    {/if}
  </div>

  {#if loading}
    <p class="status">Loading…</p>
  {:else if error && !dirty}
    <div class="error-box"><p>{error}</p><button class="btn" onclick={load}>Retry</button></div>
  {:else if Object.keys(animations).length === 0}
    <p class="empty">No animations. Click "+ Add" to create one.</p>
  {:else}
    <div class="table-wrap">
      <div class="table">
        <div class="row hdr">
          <span class="nm">Name</span>
          <span class="src">Source</span>
          <span class="ft">Frame (ms)</span>
          <span class="fc">Frames</span>
          <span class="fpr">Per Row</span>
          <span class="lp">Loop</span>
          <span class="du">Duration</span>
          <span></span>
        </div>
        {#each Object.entries(animations) as [name, anim]}
          <div class="row">
            <input class="nm" value={name} onblur={(e) => rename(name, (e.target as HTMLInputElement).value)} onkeydown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} />
            <select class="src" value={anim.source} onchange={(e) => { anim.source = (e.target as HTMLSelectElement).value; Object.assign(anim, gifDefaults(anim.source)); markDirty(); }}>
              {#each imageFiles as f}<option value={f}>{f}</option>{/each}
              {#if !imageFiles.includes(anim.source)}<option value={anim.source}>{anim.source}</option>{/if}
            </select>
            <input class="ft" type="number" min={0} value={anim.frameTime} onblur={(e) => { const v = parseInt((e.target as HTMLInputElement).value); anim.frameTime = isNaN(v) ? 0 : Math.max(0, v); markDirty(); }} onkeydown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} placeholder={anim.source.toLowerCase().endsWith('.gif') ? 'auto' : '100'} />
            <input class="fc" type="number" min={0} value={anim.frameCount ?? ''} onblur={(e) => { const v = (e.target as HTMLInputElement).value; anim.frameCount = v ? Math.max(0, parseInt(v)) : undefined; markDirty(); }} onkeydown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} placeholder={anim.source.toLowerCase().endsWith('.gif') ? 'all' : '4'} />
            <input class="fpr" type="number" min={1} value={anim.source.toLowerCase().endsWith('.gif') ? '' : (anim.framesPerRow ?? '')} disabled={anim.source.toLowerCase().endsWith('.gif')} onblur={(e) => { const v = (e.target as HTMLInputElement).value; anim.framesPerRow = v ? Math.max(1, parseInt(v)) : undefined; markDirty(); }} onkeydown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} placeholder={anim.source.toLowerCase().endsWith('.gif') ? '—' : '2'} />
            <label class="lp"><input type="checkbox" checked={anim.loop} onchange={(e) => { anim.loop = (e.target as HTMLInputElement).checked; markDirty(); }} /></label>
            <input class="du" type="number" min={0} value={anim.duration ?? ''} onblur={(e) => { const v = (e.target as HTMLInputElement).value; anim.duration = v ? Math.max(0, parseInt(v)) : undefined; markDirty(); }} onkeydown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} placeholder="—" />
            <button class="del-btn" onclick={() => remove(name)} title="Remove">✕</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if error && dirty}
    <div class="error-box"><p>{error}</p></div>
  {/if}

  {#if dirty && !loading}
    <button class="btn" onclick={save}>Save Changes</button>
  {/if}

  <section class="assets">
    <div class="asset-header">
      <h2>Image Assets</h2>
      <button class="btn subtle" onclick={doImportImage}>Import Image</button>
    </div>
    {#if imageFiles.length === 0}
      <p class="empty">No image files.</p>
    {:else}
      <div class="asset-table">
        <div class="asset-row hdr">
          <span>File</span>
          <span>Usage</span>
          <span></span>
        </div>
        {#each imageFiles as file}
          {@const usage = usedBy(file)}
          <div class="asset-row">
            <span class="file-name">{file}</span>
            <span class:unused={usage.length === 0}>{usage.length ? `Used by: ${usage.join(', ')}` : 'Unused'}</span>
            <button class="btn subtle" onclick={() => deleteUnusedImage(file)} disabled={usage.length > 0}>Delete file</button>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .editor-panel { }
  .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; flex-wrap: wrap; gap: 8px; }
  h2 { font-size: 15px; margin: 0; color: var(--text-primary); }
  .btn { padding: 5px 12px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .btn.subtle { background: transparent; color: var(--text-secondary); border-color: var(--border); }
  .status { color: var(--text-muted); font-size: 13px; }
  .empty { color: var(--text-muted); font-size: 13px; }
  .error-box { background: #fce4e4; border: 1px solid #c62828; border-radius: var(--radius-sm); padding: 10px; display: flex; flex-direction: column; gap: 8px; margin-bottom: 8px; }
  .error-box p { margin: 0; font-size: 13px; color: #c62828; }
  .table-wrap { overflow-x: auto; }
  .table { display: flex; flex-direction: column; gap: 4px; }
  .row { display: grid; grid-template-columns: minmax(72px,1fr) minmax(60px,1fr) minmax(52px,0.7fr) 46px 50px 36px 56px 28px; gap: 4px; align-items: center; padding: 3px 0; }
  .row.hdr { font-size: 11px; color: var(--text-muted); font-weight: 600; }
  .row.hdr span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .row input, .row select { padding: 4px 4px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 12px; background: var(--bg-secondary); color: var(--text-primary); width: 100%; box-sizing: border-box; }
  .row input:focus, .row select:focus { outline: none; box-shadow: var(--focus-ring); }
  .nm { font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .lp { text-align: center; }
  .lp input { width: auto; }
  .del-btn { background: none; border: none; cursor: pointer; color: var(--danger); font-size: 14px; width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
  .inline-add { display: flex; gap: 6px; align-items: center; }
  .inline-add input { padding: 4px 8px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 12px; background: var(--bg-secondary); color: var(--text-primary); }
  .assets { margin-top: 18px; border-top: 1px solid var(--border); padding-top: 12px; }
  .asset-header { display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 8px; }
  .asset-table { display: flex; flex-direction: column; gap: 4px; }
  .asset-row { display: grid; grid-template-columns: minmax(120px, 1fr) minmax(150px, 1.4fr) 86px; gap: 6px; align-items: center; font-size: 12px; color: var(--text-secondary); }
  .asset-row.hdr { font-size: 11px; color: var(--text-muted); font-weight: 600; }
  .file-name { color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .unused { color: var(--text-muted); }
</style>
