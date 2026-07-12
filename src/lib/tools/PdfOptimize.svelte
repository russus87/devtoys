<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import Segmented from "../ui/Segmented.svelte";

  interface SizeResult {
    before: number;
    after: number;
  }

  let path = $state<string | null>(null);
  let mode = $state("ottimizza");
  let maxWidth = $state(1600);
  let quality = $state(75);
  let busy = $state(false);
  let error = $state("");
  let result = $state<SizeResult | null>(null);

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / (1024 * 1024)).toFixed(2)} MB`;
  }

  async function pick() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") {
      path = p;
      result = null;
      error = "";
    }
  }

  async function runOptimize() {
    if (!path) return;
    const dest = await save({
      defaultPath: "ottimizzato.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    error = "";
    result = null;
    try {
      result = await invoke<SizeResult>("pdf_optimize", { path, dest });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function runCompress() {
    if (!path) return;
    const dest = await save({
      defaultPath: "compresso.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    error = "";
    result = null;
    try {
      result = await invoke<SizeResult>("pdf_compress_images", { path, dest, maxWidth, quality });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const savedPct = $derived(
    result && result.before > 0 ? Math.round((1 - result.after / result.before) * 1000) / 10 : 0,
  );
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli PDF</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
  </div>

  <div class="tool-controls">
    <span class="field-label">Modalità</span>
    <Segmented
      bind:value={mode}
      options={[
        { value: "ottimizza", label: "Ottimizza" },
        { value: "comprimi", label: "Comprimi immagini" },
      ]}
    />
  </div>

  {#if mode === "ottimizza"}
    <div class="row">
      <button class="btn primary" onclick={runOptimize} disabled={!path || busy}>
        {busy ? "Esecuzione…" : "Esegui"}
      </button>
    </div>
  {:else}
    <div class="tool-controls wrap">
      <div class="stack" style="gap:4px">
        <span class="field-label">Larghezza massima</span>
        <input class="input" type="number" min="100" step="50" bind:value={maxWidth} />
      </div>
      <div class="stack" style="gap:4px">
        <span class="field-label">Qualità ({quality})</span>
        <input class="input" type="range" min="30" max="95" step="1" bind:value={quality} />
      </div>
      <button class="btn primary" onclick={runCompress} disabled={!path || busy}>
        {busy ? "Compressione…" : "Comprimi"}
      </button>
    </div>
  {/if}

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if result}
    <div class="card-box stack">
      <div class="row wrap">
        <span class="pill">Prima: {formatBytes(result.before)}</span>
        <span class="pill">Dopo: {formatBytes(result.after)}</span>
        <span class="pill gold">Risparmio: {savedPct}%</span>
      </div>
      <div class="notice good">File salvato correttamente.</div>
    </div>
  {/if}
</div>
