<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  let path = $state<string | null>(null);
  let text = $state("");
  let opacity = $state(0.15);
  let busy = $state(false);
  let error = $state("");
  let success = $state(false);

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  async function pick() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") {
      path = p;
      success = false;
      error = "";
    }
  }

  async function run() {
    if (!path || !text.trim()) return;
    const dest = await save({
      defaultPath: "filigrana.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    error = "";
    success = false;
    try {
      await invoke<null>("pdf_watermark", { path, dest, text, opacity });
      success = true;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const canRun = $derived(!!path && text.trim().length > 0 && !busy);
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli PDF</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
  </div>

  <div class="stack" style="gap:4px">
    <span class="field-label">Testo filigrana</span>
    <input class="input" type="text" placeholder="es. RISERVATO" bind:value={text} />
  </div>

  <div class="stack" style="gap:4px">
    <span class="field-label">Opacità ({opacity.toFixed(2)})</span>
    <input class="input" type="range" min="0.05" max="1" step="0.05" bind:value={opacity} />
  </div>

  <div class="row">
    <button class="btn primary" onclick={run} disabled={!canRun}>
      {busy ? "Applicazione…" : "Applica filigrana"}
    </button>
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}
  {#if success}
    <div class="notice good">Filigrana applicata e file salvato correttamente.</div>
  {/if}
</div>
