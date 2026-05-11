<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { emit } from '@tauri-apps/api/event';
  import PetCanvas from './lib/pet/PetCanvas.svelte';

  let petId = $state('');
  let scale = $state(5);
  let unlistenScale: (() => void) | null = null;

  async function init() {
    try {
      petId = await invoke<string>('get_active_pet');
      const s = await invoke<number>('get_scale');
      if (typeof s === 'number' && s >= 1 && s <= 10) scale = s;
    } catch (e) { console.error('init:', e); }

    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = listen<number>('scale-changed', (e) => {
      if (typeof e.payload === 'number' && e.payload >= 1 && e.payload <= 10) {
        scale = e.payload;
      }
    });
    unlistenScale = await unlisten;
  }

  onMount(() => {
    init();
    return () => {
      unlistenScale?.();
    };
  });
</script>

<PetCanvas {petId} {scale} />
