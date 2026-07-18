<script lang="ts">
  import {
    TOOLS,
    CATEGORIES,
    searchTools,
    toolsByCategory,
    type ToolMeta,
    type CategoryId,
  } from "./lib/tools";
  import CommandPalette from "./lib/CommandPalette.svelte";
  import {
    api,
    on,
    inTauri,
    type UpdateInfo,
    type UpdateProgress,
  } from "./lib/api";
  import { openUrl } from "@tauri-apps/plugin-opener";

  // ---- persisted UI state ------------------------------------------------
  const THEME_KEY = "devtoys.theme";
  const FAV_KEY = "devtoys.favs";
  const RECENT_KEY = "devtoys.recent";

  function load<T>(key: string, fallback: T): T {
    try {
      const v = localStorage.getItem(key);
      return v ? (JSON.parse(v) as T) : fallback;
    } catch {
      return fallback;
    }
  }

  let theme = $state<"light" | "dark">(load(THEME_KEY, "light"));
  let favs = $state<string[]>(load(FAV_KEY, []));
  let recent = $state<string[]>(load(RECENT_KEY, []));

  let query = $state("");
  let activeId = $state<string | null>(null);
  let paletteOpen = $state(false);

  // ---- self-update + tray -----------------------------------------------
  const RELEASES_URL = "https://github.com/russus87/devtoys/releases";
  let update = $state<UpdateInfo | null>(null);
  let progress = $state<UpdateProgress | null>(null);
  let updateBusy = $state(false);
  let updateDismissed = $state(false);
  let upToDateMsg = $state(false);

  async function checkUpdate(manual = false) {
    try {
      const info = await api.updateCheck();
      update = info;
      if (info.available) updateDismissed = false;
      if (manual && !info.available) {
        upToDateMsg = true;
        setTimeout(() => (upToDateMsg = false), 3500);
      }
    } catch (e) {
      if (manual) {
        progress = { phase: "error", message: String(e), percent: -1 };
      }
    }
  }

  async function installUpdate() {
    if (!update) return;
    if (update.installable) {
      updateBusy = true;
      progress = { phase: "download", message: "Avvio…", percent: -1 };
      try {
        await api.updateInstall();
      } catch {
        /* error surfaced via the update-progress event */
      } finally {
        updateBusy = false;
      }
    } else {
      openRelease();
    }
  }

  function openRelease() {
    openUrl(RELEASES_URL).catch(() => {});
  }

  $effect(() => {
    if (!inTauri()) return;

    // Feed the tray the tool list (used by the native Linux menu).
    api
      .setTrayMenu(
        CATEGORIES.map((c) => ({
          label: c.label,
          emoji: c.emoji,
          tools: toolsByCategory(c.id).map((t) => ({ id: t.id, name: t.name })),
        })),
      )
      .catch(() => {});

    // Silent check on startup — the banner only appears if there's an update.
    checkUpdate();

    const unlisteners: Promise<() => void>[] = [
      on<string>("open-tool", (id) => open(id)),
      on<void>("menu-check-update", () => checkUpdate(true)),
      on<UpdateInfo>("update-available", (info) => {
        update = info;
        updateDismissed = false;
      }),
      on<UpdateProgress>("update-progress", (p) => (progress = p)),
    ];
    return () => {
      unlisteners.forEach((u) => u.then((fn) => fn()).catch(() => {}));
    };
  });

  // global Ctrl/⌘+K toggles the command palette
  $effect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        paletteOpen = !paletteOpen;
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  $effect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem(THEME_KEY, JSON.stringify(theme));
  });
  $effect(() => localStorage.setItem(FAV_KEY, JSON.stringify(favs)));
  $effect(() => localStorage.setItem(RECENT_KEY, JSON.stringify(recent)));

  const active = $derived(TOOLS.find((t) => t.id === activeId) ?? null);
  const ActiveComp = $derived(active?.component ?? null);
  const results = $derived(searchTools(query));
  const favTools = $derived(
    favs.map((id) => TOOLS.find((t) => t.id === id)).filter((t): t is ToolMeta => !!t),
  );
  const recentTools = $derived(
    recent.map((id) => TOOLS.find((t) => t.id === id)).filter((t): t is ToolMeta => !!t),
  );

  function open(id: string) {
    activeId = id;
    recent = [id, ...recent.filter((r) => r !== id)].slice(0, 6);
    query = "";
  }
  function home() {
    activeId = null;
  }
  function toggleFav(id: string, ev?: Event) {
    ev?.stopPropagation();
    favs = favs.includes(id) ? favs.filter((f) => f !== id) : [...favs, id];
  }
  function toggleTheme() {
    theme = theme === "light" ? "dark" : "light";
  }

  function catAccent(cat: CategoryId): string {
    return `var(--c-${cat})`;
  }
  function catBg(cat: CategoryId): string {
    return `var(--c-${cat}-bg)`;
  }
  function catLabel(cat: CategoryId): string {
    return CATEGORIES.find((c) => c.id === cat)?.label ?? "";
  }
</script>

<div class="app">
  <!-- self-update banner -->
  {#if progress && (progress.phase === "download" || progress.phase === "install")}
    <div class="updatebar busy">
      <span class="ub-ico">⬇️</span>
      <span class="ub-text">{progress.message}</span>
      <div class="ub-track">
        <div
          class="ub-fill"
          class:indeterminate={progress.percent < 0}
          style={progress.percent >= 0 ? `width:${progress.percent}%` : ""}
        ></div>
      </div>
    </div>
  {:else if progress && progress.phase === "done"}
    <div class="updatebar ok">
      <span class="ub-ico">✅</span>
      <span class="ub-text">{progress.message}</span>
      <button class="ub-btn" onclick={() => (progress = null)}>Chiudi</button>
    </div>
  {:else if progress && progress.phase === "error"}
    <div class="updatebar bad">
      <span class="ub-ico">⚠️</span>
      <span class="ub-text">Aggiornamento non riuscito: {progress.message}</span>
      <button class="ub-btn" onclick={openRelease}>Apri release</button>
      <button class="ub-x" onclick={() => (progress = null)} aria-label="Chiudi">✕</button>
    </div>
  {:else if update?.available && !updateDismissed}
    <div class="updatebar">
      <span class="ub-ico">✨</span>
      <span class="ub-text">
        Nuova versione <strong>v{update.latest}</strong> disponibile
        <span class="ub-dim">(hai la v{update.current})</span>
      </span>
      {#if update.installable}
        <button class="ub-btn primary" onclick={installUpdate} disabled={updateBusy}>
          Scarica e installa
        </button>
      {/if}
      <button class="ub-btn" onclick={openRelease}>Apri release</button>
      <button class="ub-x" onclick={() => (updateDismissed = true)} aria-label="Ignora">✕</button>
    </div>
  {:else if upToDateMsg}
    <div class="updatebar ok">
      <span class="ub-ico">✅</span>
      <span class="ub-text">DevToys è aggiornato all'ultima versione.</span>
    </div>
  {/if}

  <!-- top bar -->
  <header class="topbar">
    <button class="brand" onclick={home} aria-label="Home">
      <span class="brand-badge">&lt;/&gt;</span>
      <span class="brand-name">Dev<span class="brand-accent">Toys</span></span>
    </button>

    <div class="search">
      <span class="search-ico">⌕</span>
      <input
        class="search-input"
        type="text"
        placeholder="Cerca uno strumento…"
        bind:value={query}
        oninput={() => (activeId = null)}
      />
      {#if query}
        <button class="search-clear" onclick={() => (query = "")} aria-label="Pulisci">✕</button>
      {/if}
    </div>

    <div class="top-actions">
      <button class="kbd-btn" onclick={() => (paletteOpen = true)} title="Palette comandi (Ctrl+K)">
        <span class="kbd-ico">⌘</span>K
      </button>
      {#if inTauri()}
        <button
          class="icon-btn"
          onclick={() => checkUpdate(true)}
          title="Controlla aggiornamenti"
          aria-label="Controlla aggiornamenti"
        >
          ⟳
        </button>
      {/if}
      <button class="icon-btn" onclick={toggleTheme} title="Tema" aria-label="Cambia tema">
        {theme === "light" ? "🌙" : "☀️"}
      </button>
    </div>
  </header>

  <!-- content -->
  <main class="content">
    {#if active}
      <!-- tool view -->
      <div class="tool-view">
        <div class="tool-head">
          <button class="back" onclick={home} aria-label="Indietro">←</button>
          <div class="tool-badge" style="background:{catBg(active.category)};color:{catAccent(active.category)}">
            {active.emoji}
          </div>
          <div class="tool-title">
            <h1>{active.name}</h1>
            <p>{active.desc}</p>
          </div>
          <button
            class="star"
            class:on={favs.includes(active.id)}
            onclick={() => toggleFav(active.id)}
            title="Preferito"
            aria-label="Preferito"
          >
            {favs.includes(active.id) ? "★" : "☆"}
          </button>
        </div>
        <div class="tool-stage">
          {#if ActiveComp}
            <ActiveComp />
          {/if}
        </div>
      </div>
    {:else if query}
      <!-- search results -->
      <section class="section">
        <div class="section-head">
          <h2>Risultati <span class="cnt">({results.length})</span></h2>
        </div>
        {#if results.length}
          <div class="grid">
            {#each results as t (t.id)}
              {@render card(t)}
            {/each}
          </div>
        {:else}
          <p class="empty">Nessuno strumento trovato per «{query}».</p>
        {/if}
      </section>
    {:else}
      <!-- home dashboard -->
      <div class="hero">
        <h1>Strumenti per sviluppatori</h1>
        <p>
          Ogni piccola utility che continui a cercare su Google, in un'unica app offline.
          I tuoi dati non lasciano mai il computer.
        </p>
      </div>

      {#if favTools.length}
        <section class="section">
          <div class="section-head">
            <h2>⭐ Preferiti <span class="cnt">({favTools.length})</span></h2>
          </div>
          <div class="grid">
            {#each favTools as t (t.id)}
              {@render card(t)}
            {/each}
          </div>
        </section>
      {/if}

      {#if recentTools.length}
        <section class="section">
          <div class="section-head">
            <h2>🕘 Recenti</h2>
          </div>
          <div class="grid">
            {#each recentTools as t (t.id)}
              {@render card(t)}
            {/each}
          </div>
        </section>
      {/if}

      {#each CATEGORIES as c (c.id)}
        <section class="section">
          <div class="section-head">
            <h2>{c.emoji} {c.label} <span class="cnt">({toolsByCategory(c.id).length})</span></h2>
          </div>
          <div class="grid">
            {#each toolsByCategory(c.id) as t (t.id)}
              {@render card(t)}
            {/each}
          </div>
        </section>
      {/each}

      <footer class="foot">
        DevToys — v{__APP_VERSION__}
      </footer>
    {/if}
  </main>

  {#if paletteOpen}
    <CommandPalette
      onSelect={(id) => {
        open(id);
        paletteOpen = false;
      }}
      onClose={() => (paletteOpen = false)}
    />
  {/if}
</div>

<!-- tool card snippet -->
{#snippet card(t: ToolMeta)}
  <button class="tcard" onclick={() => open(t.id)}>
    <div class="tcard-band" style="background:{catBg(t.category)}">
      <span class="tcard-emoji">{t.emoji}</span>
      <span
        class="tcard-star"
        class:on={favs.includes(t.id)}
        role="button"
        tabindex="-1"
        onclick={(e) => toggleFav(t.id, e)}
        onkeydown={() => {}}
      >
        {favs.includes(t.id) ? "★" : "☆"}
      </span>
    </div>
    <div class="tcard-body">
      <div class="tcard-name">{t.name}</div>
      <div class="tcard-desc">{t.desc}</div>
      <div class="tcard-foot">
        <span class="tcard-tag" style="color:{catAccent(t.category)};background:{catBg(t.category)}">
          {catLabel(t.category)}
        </span>
        <span class="tcard-open" style="color:{catAccent(t.category)}">Apri →</span>
      </div>
    </div>
  </button>
{/snippet}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* self-update banner ---------------------------------------------------- */
  .updatebar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 20px;
    background: linear-gradient(135deg, var(--primary-soft), var(--surface-3));
    border-bottom: 1px solid var(--border);
    font-size: 13.5px;
    flex-shrink: 0;
    z-index: 6;
  }
  .updatebar.ok {
    background: color-mix(in srgb, var(--green) 12%, var(--surface));
  }
  .updatebar.bad {
    background: color-mix(in srgb, var(--red) 12%, var(--surface));
  }
  .ub-ico {
    font-size: 16px;
    flex-shrink: 0;
  }
  .ub-text {
    color: var(--ink);
    min-width: 0;
  }
  .ub-dim {
    color: var(--ink-faint);
  }
  .ub-track {
    flex: 1;
    height: 6px;
    border-radius: 999px;
    background: var(--border-2);
    overflow: hidden;
    min-width: 80px;
    max-width: 320px;
  }
  .ub-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--primary), var(--primary-2));
    transition: width 0.2s ease;
  }
  .ub-fill.indeterminate {
    width: 35%;
    animation: ubslide 1.1s ease-in-out infinite;
  }
  @keyframes ubslide {
    0% { margin-left: -35%; }
    100% { margin-left: 100%; }
  }
  .ub-btn {
    margin-left: auto;
    padding: 6px 13px;
    border-radius: 9px;
    font-size: 12.5px;
    font-weight: 700;
    background: var(--surface-3);
    color: var(--ink);
    white-space: nowrap;
  }
  .ub-btn + .ub-btn {
    margin-left: 0;
  }
  .ub-btn:hover {
    background: var(--border-2);
  }
  .ub-btn.primary {
    background: linear-gradient(135deg, var(--primary), var(--primary-2));
    color: #fff;
  }
  .ub-btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .ub-x {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    color: var(--ink-dim);
    flex-shrink: 0;
  }
  .ub-x:hover {
    background: var(--surface);
    color: var(--ink);
  }

  /* top bar --------------------------------------------------------------- */
  .topbar {
    display: flex;
    align-items: center;
    gap: 18px;
    padding: 14px 24px;
    background: color-mix(in srgb, var(--surface) 72%, transparent);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    z-index: 5;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
  }
  .brand-badge {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 13px;
    background: linear-gradient(135deg, var(--primary), var(--primary-2));
    color: #fff;
    font-family: var(--mono);
    font-weight: 700;
    font-size: 15px;
    box-shadow: 0 8px 18px rgba(108, 92, 240, 0.35);
  }
  .brand-name {
    font-size: 19px;
    font-weight: 800;
    letter-spacing: -0.01em;
  }
  .brand-accent {
    color: var(--primary);
  }
  .search {
    position: relative;
    flex: 1;
    max-width: 560px;
    margin: 0 auto;
    display: flex;
    align-items: center;
  }
  .search-ico {
    position: absolute;
    left: 15px;
    font-size: 17px;
    color: var(--ink-faint);
    pointer-events: none;
  }
  .search-input {
    width: 100%;
    padding: 11px 40px 11px 42px;
    border-radius: 14px;
    border: 1.5px solid var(--border-2);
    background: var(--surface);
    font-size: 14.5px;
    outline: none;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }
  .search-input:focus {
    border-color: var(--primary);
    box-shadow: 0 0 0 4px var(--primary-soft);
  }
  .search-clear {
    position: absolute;
    right: 12px;
    color: var(--ink-faint);
    font-size: 13px;
    width: 22px;
    height: 22px;
    border-radius: 7px;
  }
  .search-clear:hover {
    background: var(--surface-3);
    color: var(--ink);
  }
  .top-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .kbd-btn {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 40px;
    padding: 0 13px;
    border-radius: 12px;
    background: var(--surface-3);
    color: var(--ink-dim);
    font-family: var(--mono);
    font-size: 13px;
    font-weight: 600;
    transition: background 0.15s ease, color 0.15s ease;
  }
  .kbd-btn:hover {
    background: var(--border-2);
    color: var(--ink);
  }
  .kbd-ico {
    font-size: 14px;
  }
  .icon-btn {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    font-size: 17px;
    background: var(--surface-3);
    display: grid;
    place-items: center;
    transition: background 0.15s ease;
  }
  .icon-btn:hover {
    background: var(--border-2);
  }

  /* content --------------------------------------------------------------- */
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 30px 34px 40px;
  }
  .hero {
    margin-bottom: 26px;
  }
  .hero h1 {
    margin: 0 0 8px;
    font-size: 30px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }
  .hero p {
    margin: 0;
    max-width: 620px;
    color: var(--ink-dim);
    font-size: 14.5px;
    line-height: 1.55;
  }

  .section {
    margin-bottom: 30px;
  }
  .section-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 15px;
  }
  .section-head h2 {
    margin: 0;
    font-size: 16.5px;
    font-weight: 800;
    letter-spacing: -0.01em;
  }
  .cnt {
    color: var(--ink-faint);
    font-weight: 700;
    font-size: 14px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(258px, 1fr));
    gap: 16px;
  }

  /* tool card ------------------------------------------------------------- */
  .tcard {
    text-align: left;
    background: var(--surface);
    border: 1.5px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: transform 0.14s ease, box-shadow 0.16s ease, border-color 0.16s ease;
    display: flex;
    flex-direction: column;
  }
  .tcard:hover {
    transform: translateY(-4px);
    box-shadow: var(--shadow);
    border-color: transparent;
  }
  .tcard-band {
    position: relative;
    height: 84px;
    display: grid;
    place-items: center;
  }
  .tcard-emoji {
    font-size: 38px;
    filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.1));
  }
  .tcard-star {
    position: absolute;
    top: 8px;
    right: 10px;
    font-size: 16px;
    color: rgba(0, 0, 0, 0.28);
    line-height: 1;
    cursor: pointer;
  }
  .tcard-star.on {
    color: var(--gold);
  }
  .tcard-star:hover {
    color: var(--gold);
  }
  .tcard-body {
    padding: 14px 16px 15px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }
  .tcard-name {
    font-weight: 750;
    font-size: 15px;
    letter-spacing: -0.01em;
  }
  .tcard-desc {
    color: var(--ink-dim);
    font-size: 12.8px;
    line-height: 1.45;
    flex: 1;
  }
  .tcard-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 4px;
  }
  .tcard-tag {
    font-size: 10.5px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    padding: 3px 9px;
    border-radius: 999px;
  }
  .tcard-open {
    font-size: 12.5px;
    font-weight: 700;
  }

  .empty {
    color: var(--ink-dim);
    padding: 20px 0;
  }
  .foot {
    margin-top: 10px;
    padding-top: 18px;
    border-top: 1px solid var(--border);
    color: var(--ink-faint);
    font-size: 12.5px;
    text-align: center;
  }

  /* tool view ------------------------------------------------------------- */
  .tool-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    animation: fade 0.18s ease;
  }
  @keyframes fade {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  .tool-head {
    display: flex;
    align-items: center;
    gap: 15px;
    margin-bottom: 20px;
    flex-shrink: 0;
  }
  .back {
    width: 42px;
    height: 42px;
    border-radius: 13px;
    background: var(--surface);
    border: 1.5px solid var(--border);
    font-size: 19px;
    color: var(--ink-dim);
    display: grid;
    place-items: center;
    transition: all 0.14s ease;
  }
  .back:hover {
    color: var(--primary);
    border-color: var(--primary);
  }
  .tool-badge {
    width: 52px;
    height: 52px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    font-size: 26px;
    flex-shrink: 0;
  }
  .tool-title {
    flex: 1;
    min-width: 0;
  }
  .tool-title h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }
  .tool-title p {
    margin: 3px 0 0;
    color: var(--ink-dim);
    font-size: 13.5px;
  }
  .star {
    font-size: 22px;
    color: var(--ink-faint);
    padding: 8px;
  }
  .star.on {
    color: var(--gold);
  }
  .tool-stage {
    flex: 1;
    min-height: 0;
    background: var(--surface);
    border: 1.5px solid var(--border);
    border-radius: var(--radius-xl);
    padding: 22px;
    box-shadow: var(--shadow-sm);
    overflow: auto;
  }
</style>
