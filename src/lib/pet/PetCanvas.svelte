<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { AnimationController } from './AnimationController';
  import { StateMachine } from './StateMachine';
  import { render } from './SpriteRenderer';
  import { loadAnimation, loadGifAnimation } from './SpriteLoader';
  import { FRAME_SIZE } from './config';
  import type { PetConfig, PetEvent, GifFrameData } from './types';

  let { petId = '', scale = 5 }: { petId?: string; scale?: number } = $props();


  let mainWindow: ReturnType<typeof getCurrentWindow> | null = null;

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let controller = new AnimationController();
  let stateMachine: StateMachine | null = null;
  let sprites = new Map<string, HTMLImageElement | GifFrameData>();
  let config: PetConfig = { animations: {}, defaultState: 'idle', states: {} };
  let animationId: number;
  let lastTime = 0;

  let downX = 0;
  let downY = 0;
  let grabbing = false;
  let grabTime = 0;
  let lastMoveTime = 0;
  let unlistenPetChanged: (() => void) | null = null;
  let dragOffX = 0;
  let dragOffY = 0;
  let targetX = 0;
  let targetY = 0;
  let skipNextClick = false;
  let ready = $state(false);
  let isWayland = false;
  let resizeObserver: ResizeObserver | null = null;
  let loadVersion = 0;

  let canvasStyle = $derived(
    isWayland
      ? `width: ${FRAME_SIZE * scale}px; height: ${FRAME_SIZE * scale}px; cursor: grab;`
      : 'width: 100%; height: 100%; cursor: grab;'
  );

  $effect(() => {
    if (ready && petId) {
      const version = ++loadVersion;
      loadPet(petId, version).then((ok) => { if (ok) applyConfig(); });
    }
  });

  function dispatch(event: PetEvent) {
    stateMachine?.dispatch(event);
  }

  function applyConfig() {
    controller.setAnimations(config.animations);
    stateMachine = new StateMachine(config.states, controller, config.defaultState);
    stateMachine.start();
    lastAction = '';
  }

  async function loadPet(id: string, version: number): Promise<boolean> {
    sprites.clear();
    try {
      const raw = await invoke<string>('read_json', { id, filename: 'config.json' });
      const cfg: PetConfig = JSON.parse(raw);
      for (const [name, def] of Object.entries(cfg.animations)) {
        try {
          if (def.source.toLowerCase().endsWith('.gif')) {
            const gifData = await loadGifAnimation(id, def.source);
            sprites.set(name, gifData);
            controller.setFrameDelays(name, gifData.delays);
          } else {
            const img = await loadAnimation(id, def.source);
            sprites.set(name, img);
          }
        } catch (e) {
          console.warn(`failed to load sprite for animation "${name}":`, e);
        }
      }
      if (version !== loadVersion) return false;
      config = cfg;
      return true;
    } catch (e) {
      console.error('loadPet:', e);
      if (version !== loadVersion) return false;
      config = { animations: {}, defaultState: 'idle', states: {} };
      return true;
    }
  }

  async function init() {
    try {
      const info = await invoke<{ is_wayland: boolean }>('get_platform_info');
      isWayland = info.is_wayland;
    } catch (e) { console.error('get_platform_info:', e); }

    const { listen } = await import('@tauri-apps/api/event');
    unlistenPetChanged = await listen('pet-changed', async (e) => {
      if (typeof e.payload === 'string') {
        const version = ++loadVersion;
        const ok = await loadPet(e.payload, version);
        if (ok) applyConfig();
      }
    });

    controller.onEnd(() => {
      stateMachine?.dispatch('animation_end');
    });

    lastTime = performance.now();
    animationId = requestAnimationFrame(tick);
  }

  function computeSize() {
    return { w: FRAME_SIZE * scale, h: FRAME_SIZE * scale };
  }

  async function applyResize(w: number, h: number) {
    if (!mainWindow) return;
    try {
      await mainWindow.setSize(new LogicalSize(w, h));
      if (!isWayland) {
        const factor = await mainWindow.scaleFactor();
        const pos = await mainWindow.outerPosition();
        const oldW = canvas.width;
        const oldH = canvas.height;
        const physCx = pos.x + (oldW * factor) / 2;
        const physCy = pos.y + (oldH * factor) / 2;
        const newPhysW = w * factor;
        const newPhysH = h * factor;
        await mainWindow.setPosition(
          new LogicalPosition(
            Math.round((physCx - newPhysW / 2) / factor),
            Math.round((physCy - newPhysH / 2) / factor),
          ),
        );
      }
    } catch (e) {
      console.error('resize window:', e);
    }
  }

  $effect(() => {
    if (ready && ctx) {
      const _scale = scale;
      const { w, h } = computeSize();
      if (canvas.width !== w || canvas.height !== h) {
        applyResize(w, h);
      }
    }
  });

  onMount(() => {
    ctx = canvas.getContext('2d');
    if (!ctx) return;
    mainWindow = getCurrentWindow();

    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        const w = Math.max(1, Math.round(width));
        const h = Math.max(1, Math.round(height));
        if (canvas.width !== w || canvas.height !== h) {
          canvas.width = w;
          canvas.height = h;
        }
      }
    });
    resizeObserver.observe(canvas);

    const initialSize = FRAME_SIZE * scale;
    mainWindow.setSize(new LogicalSize(initialSize, initialSize)).catch(() => {});

    init();

    ready = true;

    return () => {
      cancelAnimationFrame(animationId);
      unlistenPetChanged?.();
      resizeObserver?.disconnect();
    };
  });

  let lastAction = '';

  function tick(now: number) {
    const delta = Math.min(now - lastTime, 100);
    lastTime = now;

    if (grabbing && (now - lastMoveTime > 1000 || Date.now() - grabTime > 5000)) {
      stopWalk();
    }

    controller.update(delta);

    if (ctx) {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      if (!petId) {
        ctx.fillStyle = '#888';
        ctx.font = `${10 * scale}px sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('未找到宠物', canvas.width / 2, canvas.height / 2 - 6 * scale);
        ctx.font = `${8 * scale}px sans-serif`;
        ctx.fillText('请在设置中添加', canvas.width / 2, canvas.height / 2 + 8 * scale);
      } else {
        const sprite = sprites.get(controller.currentAction);
        const animDef = config.animations[controller.currentAction];
        if (sprite) render(ctx, sprite, controller.frameIndex, animDef);
      }
    }

    if (grabbing && mainWindow && !isWayland) {
      mainWindow.setPosition(new LogicalPosition(targetX, targetY)).catch(() => {});
    }

    animationId = requestAnimationFrame(tick);
  }

  function stopWalk() {
    dispatch('drag_end');
    grabbing = false;
  }

  function handleMouseDown(e: MouseEvent) {
    skipNextClick = false;
    if (e.button !== 0) return;
    if (controller.currentAction === 'sleep') dispatch('dblclick');
    downX = e.clientX;
    downY = e.clientY;
    grabbing = false;
    dispatch('press');
  }

  function handleMouseMove(e: MouseEvent) {
    lastMoveTime = performance.now();
    if (e.buttons === 0) {
      if (grabbing) stopWalk();
      return;
    }
    if (grabbing) {
      if (!isWayland) {
        targetX = e.screenX - dragOffX;
        targetY = e.screenY - dragOffY;
      }
      return;
    }
    // Only left button (bit 0) starts a drag; middle/right just click.
    if (!(e.buttons & 1)) return;
    const dx = Math.abs(e.clientX - downX);
    const dy = Math.abs(e.clientY - downY);
    if (dx > 4 || dy > 4) {
      grabbing = true;
      grabTime = Date.now();
      dragOffX = e.clientX;
      dragOffY = e.clientY;
      targetX = e.screenX - dragOffX;
      targetY = e.screenY - dragOffY;
      dispatch('press');
      dispatch('drag_start');
      if (isWayland && mainWindow) {
        mainWindow.startDragging().catch(() => {});
      }
    }
  }

  function handleMouseUp() {
    if (grabbing) stopWalk();
  }

  function handleMouseLeave() {
    if (!grabbing) return;
    stopWalk();
  }

  function handleClick(e: MouseEvent) {
    if (e.button !== 0 || skipNextClick) {
      skipNextClick = false;
      return;
    }
    dispatch('click');
    // Immediately render so press→walk never visibly flashes before the reaction
    if (ctx) {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const sprite = sprites.get(controller.currentAction);
      const animDef = config.animations[controller.currentAction];
      if (sprite) render(ctx, sprite, controller.frameIndex, animDef);
    }
  }
  function handleDblClick(e: MouseEvent) {
    if (e.button !== 0) return;
    dispatch('dblclick');
  }
  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    skipNextClick = true;
    dispatch('right_click');
  }
</script>

<canvas
  bind:this={canvas}
  onmousedown={handleMouseDown}
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
  onmouseleave={handleMouseLeave}
  onclick={handleClick}
  ondblclick={handleDblClick}
  oncontextmenu={handleContextMenu}
  style={canvasStyle}
></canvas>
