<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { loadAnimation, loadGifAnimation, drawFrame, getFrameMetrics, isGifSprite } from '../lib/pet/SpriteLoader';
  import { AnimationController } from '../lib/pet/AnimationController';
  import type { GifFrameData } from '../lib/pet/types';

  let { petId }: { petId: string } = $props();

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let sprite: HTMLImageElement | GifFrameData | null = null;
  let controller: AnimationController | null = null;
  let animId = 0;
  let unlistenFn: (() => void) | null = null;
  let hover = $state(false);

  const PREVIEW_W = 64;
  const PREVIEW_H = 64;

  function drawPreviewFrame(frameIndex: number) {
    if (!ctx || !sprite) return;
    ctx.clearRect(0, 0, PREVIEW_W, PREVIEW_H);
    ctx.imageSmoothingEnabled = false;
    if (isGifSprite(sprite)) {
      const frameCount = defaultFrameCount || sprite.frames.length;
      if (frameIndex >= frameCount) return;
      const bitmap = sprite.frames[frameIndex];
      const fitScale = Math.min(PREVIEW_W / bitmap.width, PREVIEW_H / bitmap.height);
      const drawWidth = Math.round(bitmap.width * fitScale);
      const drawHeight = Math.round(bitmap.height * fitScale);
      const cx = Math.round((PREVIEW_W - drawWidth) / 2);
      const cy = Math.round((PREVIEW_H - drawHeight) / 2);
      ctx.drawImage(bitmap, cx, cy, drawWidth, drawHeight);
    } else {
      const { frameWidth, frameHeight } = getFrameMetrics(sprite, defaultFrameCount, defaultFramesPerRow);
      const fitScale = Math.min(PREVIEW_W / frameWidth, PREVIEW_H / frameHeight);
      const drawWidth = Math.round(frameWidth * fitScale);
      const drawHeight = Math.round(frameHeight * fitScale);
      const cx = Math.round((PREVIEW_W - drawWidth) / 2);
      const cy = Math.round((PREVIEW_H - drawHeight) / 2);
      drawFrame(ctx, sprite, frameIndex, frameWidth, frameHeight, cx, cy, drawWidth, drawHeight, defaultFramesPerRow);
    }
  }

  function renderStatic() {
    drawPreviewFrame(0);
  }

  function tick(now: number) {
    if (!ctx || !sprite || !controller) { animId = requestAnimationFrame(tick); return; }
    controller.update(16);
    drawPreviewFrame(controller.frameIndex);
    animId = requestAnimationFrame(tick);
  }

  let defaultAnim = $state('idle');
  let defaultSource = $state('idle.png');
  let defaultFrameTime = $state(600);
  let defaultFrameCount = $state(4);
  let defaultFramesPerRow = $state(2);

  async function init() {
    ctx = canvas.getContext('2d');
    if (!ctx) return;
    canvas.width = PREVIEW_W;
    canvas.height = PREVIEW_H;
    await loadPreview();
  }

  async function loadPreview() {
    if (!ctx) return;
    cancelAnimationFrame(animId);
    sprite = null;
    controller = null;
    ctx.clearRect(0, 0, PREVIEW_W, PREVIEW_H);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const cRaw = await invoke<string>('read_json', { id: petId, filename: 'config.json' });
      const cfg = JSON.parse(cRaw);
      const stateName = cfg.defaultState || 'idle';
      const animName = cfg.states?.[stateName]?.entry || stateName;
      const animDef = cfg.animations?.[animName];
      defaultAnim = animName;
      defaultSource = animDef?.source || `${animName}.png`;
      defaultFrameTime = animDef?.frameTime || 600;
      defaultFrameCount = animDef?.frameCount || 4;
      defaultFramesPerRow = animDef?.framesPerRow || 2;
    } catch (e) { console.error('PetCard config:', e); }
    const isGif = defaultSource.toLowerCase().endsWith('.gif');
    try {
      if (isGif) {
        const gifData = await loadGifAnimation(petId, defaultSource);
        sprite = gifData;
        defaultFrameCount = gifData.frames.length;
      } else {
        sprite = await loadAnimation(petId, defaultSource);
      }
      renderStatic();
    } catch (e) { console.error('PetCard sprite:', e); }
    controller = new AnimationController();
    controller.setAnimations({ [defaultAnim]: { source: defaultSource, frameTime: isGif ? 0 : defaultFrameTime, loop: true, frameCount: isGif ? undefined : defaultFrameCount, framesPerRow: defaultFramesPerRow } });
    if (isGif && (sprite as GifFrameData).delays) {
      controller.setFrameDelays(defaultAnim, (sprite as GifFrameData).delays);
    }
    controller.play(defaultAnim);
    if (hover) animId = requestAnimationFrame(tick);
  }

  function startAnim() {
    hover = true;
    animId = requestAnimationFrame(tick);
  }

  function stopAnim() {
    hover = false;
    cancelAnimationFrame(animId);
    if (controller) {
      controller.play(defaultAnim);
      controller.update(0);
    }
    renderStatic();
  }

  onMount(() => {
    init();
    const ul = listen('pet-changed', async (e) => {
      if (e.payload === petId) await loadPreview();
    });
    ul.then(fn => { unlistenFn = fn; }).catch(e => { console.error('PetCard listen:', e); });
    return () => {
      cancelAnimationFrame(animId);
      unlistenFn?.();
    };
  });
</script>

<canvas
  bind:this={canvas}
  onmouseenter={startAnim}
  onmouseleave={stopAnim}
  style="width: 64px; height: 64px; image-rendering: pixelated;"
></canvas>
