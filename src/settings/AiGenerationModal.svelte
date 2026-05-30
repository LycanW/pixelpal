<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '../lib/i18n.svelte';

  let {
    petId,
    animationName = 'idle',
    onClose,
    onSaved,
  }: {
    petId: string;
    animationName?: string;
    onClose: () => void;
    onSaved: () => void;
  } = $props();

  let step = $state<1 | 2 | 3>(1);
  let description = $state('');
  let baseDescription = $state('');
  let baseImage = $state('');
  let frameCount = $state(4);
  let framesPerRow = $state(2);
  let frames: string[] = $state([]);
  let spritesheetImage = $state('');
  let generating = $state(false);
  let currentFrame = $state(0);
  let error = $state<string | null>(null);

  async function generateBase() {
    error = null;
    generating = true;
    try {
      const result = await invoke<string>('generate_base', { description });
      baseImage = result;
      baseDescription = description;
      step = 2;
    } catch (e) {
      error = String(e);
    } finally {
      generating = false;
    }
  }

  async function generateAnimation() {
    error = null;
    generating = true;
    frames = [];
    currentFrame = 0;

    try {
      const poses = [
        'standing neutral, eyes open',
        'standing neutral, eyes half closed',
        'standing neutral, eyes fully closed',
        'standing neutral, eyes half closed again',
      ];

      for (let i = 0; i < frameCount; i++) {
        currentFrame = i + 1;
        const pose = poses[i % poses.length];
        const frame = await invoke<string>('generate_frame', {
          baseDescription,
          animationName,
          frameIndex: i,
          totalFrames: frameCount,
          poseDescription: pose,
        });
        frames = [...frames, frame];
        if (i < frameCount - 1) {
          await new Promise(r => setTimeout(r, 200));
        }
      }

      await composePreview();
      step = 3;
    } catch (e) {
      error = String(e);
    } finally {
      generating = false;
    }
  }

  async function composePreview() {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d')!;
    const imgs = await Promise.all(frames.map(src => loadImage(src)));
    const fw = Math.max(...imgs.map(i => i.naturalWidth));
    const fh = Math.max(...imgs.map(i => i.naturalHeight));
    const rows = Math.ceil(frameCount / framesPerRow);
    canvas.width = fw * framesPerRow;
    canvas.height = fh * rows;
    ctx.imageSmoothingEnabled = false;

    imgs.forEach((img, idx) => {
      const col = idx % framesPerRow;
      const row = Math.floor(idx / framesPerRow);
      ctx.drawImage(img, col * fw, row * fh, fw, fh);
    });

    spritesheetImage = canvas.toDataURL('image/png');
  }

  function loadImage(src: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = reject;
      img.src = src;
    });
  }

  async function save() {
    try {
      await invoke('save_ai_sprite', {
        petId,
        filename: `${animationName}.png`,
        frames,
        framesPerRow,
      });
      onSaved();
      onClose();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="modal-overlay" onclick={onClose} role="presentation">
  <div class="modal" onclick={(e: MouseEvent) => e.stopPropagation()} onkeydown={(e: KeyboardEvent) => e.stopPropagation()} role="dialog" tabindex="-1">
    {#if step === 1}
      <h3>{t('ai.step1')} — {animationName}</h3>
      <label>{t('ai.description')}
        <input type="text" bind:value={description} placeholder="a cute orange cat" />
      </label>
      {#if error}<div class="error-box"><p>{error}</p></div>{/if}
      <button class="btn" onclick={generateBase} disabled={generating || !description.trim()}>
        {generating ? t('ai.generating') : t('ai.generateBase')}
      </button>
    {/if}

    {#if step === 2}
      <h3>{t('ai.step2')} — {animationName}</h3>
      <div class="base-preview">
        <img src={baseImage} alt="base" />
        <p>{t('ai.baseConfirm')}</p>
      </div>
      <div class="params">
        <label>{t('ai.frameCount')} <input type="number" min={1} max={16} bind:value={frameCount} /></label>
        <label>{t('ai.framesPerRow')} <input type="number" min={1} max={8} bind:value={framesPerRow} /></label>
      </div>
      {#if error}<div class="error-box"><p>{error}</p></div>{/if}
      <div class="actions">
        <button class="btn subtle" onclick={() => { step = 1; }}>{t('ai.regenerateBase')}</button>
        <button class="btn" onclick={generateAnimation} disabled={generating}>
          {generating ? `${t('ai.generating')} ${currentFrame}/${frameCount}` : t('ai.generateSpritesheet')}
        </button>
      </div>
    {/if}

    {#if step === 3}
      <h3>Preview — {animationName}</h3>
      <div class="preview">
        <img src={spritesheetImage} alt="spritesheet" />
      </div>
      <div class="frame-previews">
        {#each frames as frame, i}
          <div class="frame-thumb">
            <img src={frame} alt={`frame ${i}`} />
            <span>{i}</span>
          </div>
        {/each}
      </div>
      {#if error}<div class="error-box"><p>{error}</p></div>{/if}
      <div class="actions">
        <button class="btn subtle" onclick={() => { step = 2; frames = []; }}>{t('ai.retryAll')}</button>
        <button class="btn" onclick={save}>{t('ai.save')}</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.35); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .modal { background: var(--bg-primary); border-radius: var(--radius-md); padding: 20px; min-width: 400px; max-width: 600px; max-height: 80vh; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; }
  h3 { margin: 0; font-size: 16px; color: var(--text-primary); }
  label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--text-secondary); }
  input { padding: 6px 8px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 13px; background: var(--bg-secondary); color: var(--text-primary); }
  .base-preview { display: flex; flex-direction: column; align-items: center; gap: 8px; }
  .base-preview img { max-width: 200px; max-height: 200px; image-rendering: pixelated; }
  .params { display: flex; gap: 12px; }
  .params label { flex: 1; }
  .params input { width: 60px; }
  .preview img { max-width: 100%; image-rendering: pixelated; }
  .frame-previews { display: flex; gap: 6px; flex-wrap: wrap; }
  .frame-thumb { display: flex; flex-direction: column; align-items: center; gap: 2px; }
  .frame-thumb img { width: 64px; height: 64px; image-rendering: pixelated; border: 1px solid var(--border); }
  .frame-thumb span { font-size: 11px; color: var(--text-muted); }
  .actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 8px; }
  .btn { padding: 6px 14px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
  .btn:disabled { opacity: 0.5; cursor: default; }
  .btn.subtle { background: transparent; color: var(--text-secondary); border-color: var(--border); }
  .error-box { background: #fce4e4; border: 1px solid #c62828; border-radius: var(--radius-sm); padding: 8px; }
  .error-box p { margin: 0; font-size: 13px; color: #c62828; }
</style>
