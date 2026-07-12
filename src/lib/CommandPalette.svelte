<script lang="ts">
  import { searchTools, CATEGORIES, type ToolMeta, type CategoryId } from "./tools";

  interface Props {
    onSelect: (id: string) => void;
    onClose: () => void;
  }
  let { onSelect, onClose }: Props = $props();

  let query = $state("");
  let sel = $state(0);
  let listEl = $state<HTMLDivElement | null>(null);

  const results = $derived(searchTools(query));

  // keep the selection in range whenever the result set changes
  $effect(() => {
    if (sel > results.length - 1) sel = Math.max(0, results.length - 1);
  });

  function catAccent(cat: CategoryId): string {
    return `var(--c-${cat})`;
  }
  function catBg(cat: CategoryId): string {
    return `var(--c-${cat}-bg)`;
  }
  function catLabel(cat: CategoryId): string {
    return CATEGORIES.find((c) => c.id === cat)?.label ?? "";
  }

  function choose(t: ToolMeta | undefined) {
    if (t) onSelect(t.id);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      sel = Math.min(sel + 1, results.length - 1);
      scrollToSel();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      sel = Math.max(sel - 1, 0);
      scrollToSel();
    } else if (e.key === "Enter") {
      e.preventDefault();
      choose(results[sel]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  function scrollToSel() {
    queueMicrotask(() => {
      listEl?.querySelector<HTMLElement>(`[data-i="${sel}"]`)?.scrollIntoView({ block: "nearest" });
    });
  }

  function focus(node: HTMLInputElement) {
    node.focus();
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div class="palette" onclick={(e) => e.stopPropagation()}>
    <div class="p-search">
      <span class="p-ico">⌕</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="p-input"
        type="text"
        placeholder="Cerca uno strumento…"
        bind:value={query}
        onkeydown={onKey}
        use:focus
        autocomplete="off"
        spellcheck="false"
      />
      <kbd class="p-esc">esc</kbd>
    </div>

    <div class="p-list" bind:this={listEl}>
      {#if results.length === 0}
        <div class="p-empty">Nessuno strumento trovato.</div>
      {:else}
        {#each results as t, i (t.id)}
          <!-- svelte-ignore a11y_mouse_events_have_key_events -->
          <div
            class="p-row"
            class:sel={i === sel}
            data-i={i}
            role="option"
            aria-selected={i === sel}
            tabindex="-1"
            onmousemove={() => (sel = i)}
            onclick={() => choose(t)}
          >
            <span class="p-badge" style="background:{catBg(t.category)}">{t.emoji}</span>
            <span class="p-text">
              <span class="p-name">{t.name}</span>
              <span class="p-desc">{t.desc}</span>
            </span>
            <span class="p-tag" style="color:{catAccent(t.category)}">{catLabel(t.category)}</span>
          </div>
        {/each}
      {/if}
    </div>

    <div class="p-foot">
      <span><kbd>↑</kbd><kbd>↓</kbd> naviga</span>
      <span><kbd>↵</kbd> apri</span>
      <span><kbd>esc</kbd> chiudi</span>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    background: rgba(30, 24, 60, 0.34);
    backdrop-filter: blur(3px);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12vh;
    animation: ov 0.12s ease;
  }
  @keyframes ov {
    from {
      opacity: 0;
    }
  }
  .palette {
    width: min(600px, 92vw);
    max-height: 66vh;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1.5px solid var(--border);
    border-radius: 18px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    animation: pop 0.14s ease;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-8px) scale(0.98);
    }
  }
  .p-search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }
  .p-ico {
    font-size: 18px;
    color: var(--ink-faint);
  }
  .p-input {
    flex: 1;
    border: none;
    outline: none;
    background: none;
    font-size: 16px;
    color: var(--ink);
  }
  .p-input::placeholder {
    color: var(--ink-faint);
  }
  .p-esc {
    font-size: 11px;
    color: var(--ink-faint);
    background: var(--surface-3);
    padding: 3px 7px;
    border-radius: 6px;
  }
  .p-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
  .p-empty {
    padding: 26px;
    text-align: center;
    color: var(--ink-dim);
    font-size: 14px;
  }
  .p-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 11px;
    border-radius: 12px;
    cursor: pointer;
  }
  .p-row.sel {
    background: var(--primary-soft);
  }
  .p-badge {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    font-size: 19px;
    flex-shrink: 0;
  }
  .p-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .p-name {
    font-weight: 700;
    font-size: 14px;
    color: var(--ink);
  }
  .p-desc {
    font-size: 12px;
    color: var(--ink-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .p-tag {
    font-size: 10.5px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    flex-shrink: 0;
  }
  .p-foot {
    display: flex;
    gap: 16px;
    padding: 10px 16px;
    border-top: 1px solid var(--border);
    color: var(--ink-faint);
    font-size: 12px;
  }
  kbd {
    font-family: var(--mono);
    font-size: 11px;
    background: var(--surface-3);
    color: var(--ink-dim);
    padding: 1px 6px;
    border-radius: 5px;
    margin: 0 1px;
  }
</style>
