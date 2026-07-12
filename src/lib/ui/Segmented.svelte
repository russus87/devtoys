<script lang="ts">
  interface Opt {
    value: string;
    label: string;
  }
  interface Props {
    options: Opt[];
    value?: string;
    onChange?: (v: string) => void;
  }
  let { options, value = $bindable(""), onChange }: Props = $props();

  function pick(v: string) {
    value = v;
    onChange?.(v);
  }
</script>

<div class="seg" role="tablist">
  {#each options as o (o.value)}
    <button
      role="tab"
      aria-selected={value === o.value}
      class:active={value === o.value}
      onclick={() => pick(o.value)}
    >
      {o.label}
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    padding: 3px;
    gap: 2px;
    background: var(--surface-3);
    border-radius: 11px;
  }
  .seg button {
    padding: 6px 14px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--ink-dim);
    transition: all 0.14s ease;
    white-space: nowrap;
  }
  .seg button:hover {
    color: var(--ink);
  }
  .seg button.active {
    background: var(--surface);
    color: var(--primary);
    box-shadow: var(--shadow-sm);
  }
</style>
