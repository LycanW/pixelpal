<script lang="ts">
  interface Toast { id: number; message: string; type: 'success' | 'error' | 'info'; }

  let toasts = $state<Toast[]>([]);
  let nextId = 0;

  export function addToast(message: string, type: 'success' | 'error' | 'info') {
    const id = nextId++;
    toasts = [...toasts, { id, message, type }];
    setTimeout(() => { toasts = toasts.filter(t => t.id !== id); }, 3500);
  }
</script>

{#if toasts.length > 0}
  <div class="toast-area" role="status" aria-live="polite">
    {#each toasts as t}
      <div class="toast {t.type}">{t.message}</div>
    {/each}
  </div>
{/if}

<style>
  .toast-area {
    position: fixed; top: 12px; right: 12px; z-index: 999;
    display: flex; flex-direction: column; gap: 8px; pointer-events: none;
  }
  .toast {
    padding: 10px 16px; border-radius: 6px; font-size: 13px; font-weight: 500;
    color: #fff; pointer-events: auto;
    animation: slideIn 0.25s ease;
    box-shadow: 0 2px 8px rgba(0,0,0,0.15); max-width: 360px;
  }
  .toast.success { background: #2e7d32; }
  .toast.error   { background: #c62828; }
  .toast.info    { background: #1565c0; }
  @keyframes slideIn { from { opacity: 0; transform: translateX(20px); } to { opacity: 1; transform: translateX(0); } }
</style>
