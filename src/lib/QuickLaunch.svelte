<script lang="ts">
  import {
    searchTools,
    CATEGORIES,
    type ToolMeta,
    type CategoryId,
  } from "./tools";
  import { api, on } from "./api";

  const NCOLS = 3;
  const THEME_KEY = "devtoys.theme";

  let query = $state("");
  let sel = $state(0);
  let cat = $state<CategoryId | "all">("all");
  let inputEl = $state<HTMLInputElement | null>(null);
  let gridEl = $state<HTMLDivElement | null>(null);

  // Mirror the main window's theme (same localStorage origin).
  $effect(() => {
    try {
      const t = JSON.parse(localStorage.getItem(THEME_KEY) ?? '"light"');
      document.documentElement.setAttribute("data-theme", t);
    } catch {
      /* default light */
    }
  });

  const results = $derived.by(() => {
    let r = searchTools(query);
    if (cat !== "all") r = r.filter((t) => t.category === cat);
    return r;
  });

  $effect(() => {
    // keep selection valid as the result set changes
    void results;
    if (sel > results.length - 1) sel = Math.max(0, results.length - 1);
  });

  function catAccent(c: CategoryId): string {
    return `var(--c-${c})`;
  }
  function catBg(c: CategoryId): string {
    return `var(--c-${c}-bg)`;
  }

  function reset() {
    query = "";
    sel = 0;
    cat = "all";
    queueMicrotask(() => inputEl?.focus());
  }

  // Re-focus + reset whenever the popup is shown again.
  $effect(() => {
    const un = on<void>("quick-shown", () => reset());
    inputEl?.focus();
    return () => {
      un.then((fn) => fn()).catch(() => {});
    };
  });

  async function choose(t: ToolMeta | undefined) {
    if (!t) return;
    try {
      await api.quickOpen(t.id);
    } catch {
      /* not in Tauri */
    }
  }

  async function hide() {
    try {
      await api.quickHide();
    } catch {
      /* not in Tauri */
    }
  }

  function scrollToSel() {
    queueMicrotask(() => {
      gridEl?.querySelector<HTMLElement>(`[data-i="${sel}"]`)?.scrollIntoView({ block: "nearest" });
    });
  }

  function onKey(e: KeyboardEvent) {
    const n = results.length;
    if (e.key === "ArrowRight") {
      e.preventDefault();
      sel = Math.min(sel + 1, n - 1);
      scrollToSel();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      sel = Math.max(sel - 1, 0);
      scrollToSel();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      sel = Math.min(sel + NCOLS, n - 1);
      scrollToSel();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      sel = Math.max(sel - NCOLS, 0);
      scrollToSel();
    } else if (e.key === "Enter") {
      e.preventDefault();
      choose(results[sel]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      hide();
    }
  }
</script>

<div class="ql">
  <div class="ql-search">
    <span class="ql-ico">⌕</span>
    <!-- svelte-ignore a11y_autofocus -->
    <input
      class="ql-input"
      type="text"
      placeholder="Cerca uno strumento…"
      bind:value={query}
      bind:this={inputEl}
      onkeydown={onKey}
      oninput={() => (sel = 0)}
      autocomplete="off"
      spellcheck="false"
      autofocus
    />
    <kbd class="ql-esc">esc</kbd>
  </div>

  <div class="ql-chips">
    <button class="chip" class:on={cat === "all"} onclick={() => (cat = "all")}>Tutti</button>
    {#each CATEGORIES as c (c.id)}
      <button class="chip" class:on={cat === c.id} onclick={() => (cat = c.id)}>
        {c.emoji} {c.label}
      </button>
    {/each}
  </div>

  <div class="ql-grid" bind:this={gridEl}>
    {#if results.length === 0}
      <div class="ql-empty">Nessuno strumento trovato.</div>
    {:else}
      {#each results as t, i (t.id)}
        <!-- svelte-ignore a11y_mouse_events_have_key_events -->
        <button
          class="tile"
          class:sel={i === sel}
          data-i={i}
          style="--accent:{catAccent(t.category)}"
          onmousemove={() => (sel = i)}
          onclick={() => choose(t)}
        >
          <span class="tile-badge" style="background:{catBg(t.category)}">{t.emoji}</span>
          <span class="tile-name">{t.name}</span>
        </button>
      {/each}
    {/if}
  </div>

  <div class="ql-foot">
    <span class="ql-brand"><span class="ql-badge">&lt;/&gt;</span> DevToys</span>
    <span class="ql-hints">
      <kbd>↑</kbd><kbd>↓</kbd><kbd>←</kbd><kbd>→</kbd> naviga
      <kbd>↵</kbd> apri
      <kbd>esc</kbd> chiudi
    </span>
  </div>
</div>

<style>
  .ql {
    height: 100vh;
    padding: 9px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: var(--surface);
    border: 1.5px solid var(--border-2);
    border-radius: 16px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    animation: qlpop 0.14s ease;
  }
  @keyframes qlpop {
    from {
      opacity: 0;
      transform: scale(0.985) translateY(-6px);
    }
  }

  .ql-search {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 12px 14px 12px 6px;
    border-bottom: 1px solid var(--border);
  }
  .ql-ico {
    font-size: 20px;
    color: var(--ink-faint);
  }
  .ql-input {
    flex: 1;
    border: none;
    outline: none;
    background: none;
    font-size: 17px;
    font-weight: 500;
    color: var(--ink);
  }
  .ql-input::placeholder {
    color: var(--ink-faint);
  }
  .ql-esc {
    font-size: 11px;
    color: var(--ink-faint);
    background: var(--surface-3);
    padding: 3px 8px;
    border-radius: 6px;
  }

  .ql-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    padding: 5px 11px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 650;
    color: var(--ink-dim);
    background: var(--surface-3);
    white-space: nowrap;
    transition: background 0.13s ease, color 0.13s ease;
  }
  .chip:hover {
    background: var(--border-2);
    color: var(--ink);
  }
  .chip.on {
    background: linear-gradient(135deg, var(--primary), var(--primary-2));
    color: #fff;
  }

  .ql-grid {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 7px;
    padding: 2px;
    align-content: start;
  }
  .ql-empty {
    grid-column: 1 / -1;
    text-align: center;
    padding: 40px 0;
    color: var(--ink-dim);
    font-size: 14px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    text-align: center;
    padding: 14px 10px 13px;
    border-radius: 14px;
    border: 1.5px solid transparent;
    transition: transform 0.1s ease, background 0.13s ease, border-color 0.13s ease;
  }
  .tile:hover {
    background: var(--surface-3);
  }
  .tile.sel {
    background: var(--surface);
    border-color: var(--accent);
    box-shadow: 0 6px 18px color-mix(in srgb, var(--accent) 22%, transparent);
    transform: translateY(-2px);
  }
  .tile-badge {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 13px;
    font-size: 24px;
    flex-shrink: 0;
  }
  .tile-name {
    font-size: 12px;
    font-weight: 650;
    line-height: 1.25;
    color: var(--ink);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .ql-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 4px;
    border-top: 1px solid var(--border);
    color: var(--ink-faint);
    font-size: 11.5px;
  }
  .ql-brand {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-weight: 800;
    color: var(--ink-dim);
  }
  .ql-badge {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 7px;
    background: linear-gradient(135deg, var(--primary), var(--primary-2));
    color: #fff;
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 700;
  }
  .ql-hints {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  kbd {
    font-family: var(--mono);
    font-size: 10.5px;
    background: var(--surface-3);
    color: var(--ink-dim);
    padding: 1px 5px;
    border-radius: 5px;
  }
</style>
