<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  interface LoadTick {
    sent: number;
    ok: number;
    failed: number;
    rps: number;
    p50: number;
    p95: number;
    p99: number;
    minMs: number;
    maxMs: number;
    avgMs: number;
    elapsedMs: number;
  }

  let url = $state("https://example.com");
  let method = $state("GET");
  let headersRaw = $state("");
  let body = $state("");
  let concurrency = $state(10);
  let totalRequests = $state(200);

  let running = $state(false);
  let finished = $state(false);
  let error = $state("");
  let stats = $state<LoadTick | null>(null);

  let unTick: UnlistenFn | null = null;
  let unDone: UnlistenFn | null = null;

  const headers = $derived.by((): [string, string][] => {
    return headersRaw
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter(Boolean)
      .map((l) => {
        const idx = l.indexOf(":");
        if (idx === -1) return [l.trim(), ""] as [string, string];
        return [l.slice(0, idx).trim(), l.slice(idx + 1).trim()] as [string, string];
      });
  });

  const progressPct = $derived.by(() => {
    if (!stats || totalRequests <= 0) return 0;
    return Math.min(100, Math.round((stats.sent / totalRequests) * 100));
  });

  async function cleanupListeners() {
    unTick?.();
    unDone?.();
    unTick = null;
    unDone = null;
  }

  async function start() {
    error = "";
    finished = false;
    stats = null;
    try {
      unTick = await listen<LoadTick>("load:tick", (e) => {
        stats = e.payload;
      });
      unDone = await listen<LoadTick>("load:done", (e) => {
        stats = e.payload;
        finished = true;
        running = false;
      });
      running = true;
      await invoke("load_start", {
        url,
        method,
        headers,
        body: method === "GET" ? "" : body,
        concurrency: Math.max(1, Math.trunc(concurrency) || 1),
        totalRequests: Math.max(1, Math.trunc(totalRequests) || 1),
      });
    } catch (e) {
      running = false;
      error = "Avvio fallito: " + (e as Error).message;
      await cleanupListeners();
    }
  }

  async function stop() {
    try {
      await invoke("load_stop");
    } catch (e) {
      error = "Arresto fallito: " + (e as Error).message;
    } finally {
      running = false;
      await cleanupListeners();
    }
  }

  $effect(() => {
    return () => {
      cleanupListeners();
    };
  });

  interface Tile {
    label: string;
    value: string | number;
    highlight?: boolean;
  }

  const tiles = $derived.by((): Tile[] => {
    const s = stats;
    if (!s) return [];
    return [
      { label: "Inviate", value: s.sent },
      { label: "OK", value: s.ok },
      { label: "Fallite", value: s.failed },
      { label: "Richieste/s", value: s.rps.toFixed(1) },
      { label: "p50 (ms)", value: s.p50.toFixed(0) },
      { label: "p95 (ms)", value: s.p95.toFixed(0), highlight: true },
      { label: "p99 (ms)", value: s.p99.toFixed(0) },
      { label: "Media (ms)", value: s.avgMs.toFixed(0) },
      { label: "Min (ms)", value: s.minMs.toFixed(0) },
      { label: "Max (ms)", value: s.maxMs.toFixed(0) },
      { label: "Trascorso", value: (s.elapsedMs / 1000).toFixed(1) + " s" },
    ];
  });
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <Segmented
      bind:value={method}
      options={[
        { value: "GET", label: "GET" },
        { value: "POST", label: "POST" },
        { value: "PUT", label: "PUT" },
        { value: "DELETE", label: "DELETE" },
      ]}
    />
    <input class="input grow" placeholder="https://…" bind:value={url} />
  </div>

  <div class="tool-controls wrap">
    <div class="row">
      <span class="field-label">Concorrenza</span>
      <input class="input" type="number" min="1" style="width:90px" bind:value={concurrency} />
    </div>
    <div class="row">
      <span class="field-label">Richieste totali</span>
      <input class="input" type="number" min="1" style="width:110px" bind:value={totalRequests} />
    </div>
    <div class="spacer"></div>
    {#if !running}
      <button class="btn primary" onclick={start}>Avvia</button>
    {:else}
      <button class="btn" onclick={stop}>Ferma</button>
    {/if}
  </div>

  <div class="split">
    <Pane label="Header (uno per riga, Chiave: Valore)" bind:value={headersRaw} rows={5} placeholder={"Authorization: Bearer …\nContent-Type: application/json"} />
    {#if method !== "GET"}
      <Pane label="Body" bind:value={body} rows={5} placeholder="{'{'} ... {'}'}" />
    {/if}
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}
  {#if finished}<div class="notice good">Test completato.</div>{/if}

  {#if stats}
    <div class="progress-track">
      <div class="progress-fill" style="width:{progressPct}%"></div>
    </div>
    <div class="muted faint" style="font-size:12px">{stats.sent} / {totalRequests} inviate</div>

    <div class="stats-grid">
      {#each tiles as t (t.label)}
        <div class="card-box stat-tile" class:highlight={t.highlight}>
          <div class="stat-number">{t.value}</div>
          <div class="stat-label muted">{t.label}</div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 10px;
  }
  .stat-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 14px 8px;
    text-align: center;
  }
  .stat-tile.highlight {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px var(--primary-soft);
  }
  .stat-number {
    font-size: 22px;
    font-weight: 700;
    color: var(--primary);
    line-height: 1.1;
  }
  .stat-label {
    font-size: 11.5px;
  }
  .progress-track {
    height: 8px;
    border-radius: 999px;
    background: var(--surface-3);
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: linear-gradient(135deg, var(--primary), var(--primary-2));
    transition: width 0.2s ease;
  }
</style>
