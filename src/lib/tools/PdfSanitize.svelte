<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  interface SanitizeResult {
    removed: string[];
  }

  let path = $state<string | null>(null);
  let busy = $state(false);
  let error = $state("");
  let result = $state<SanitizeResult | null>(null);

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  async function pick() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") {
      path = p;
      result = null;
      error = "";
    }
  }

  async function run() {
    if (!path) return;
    const dest = await save({
      defaultPath: "sanitizzato.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    error = "";
    result = null;
    try {
      result = await invoke<SanitizeResult>("pdf_sanitize", { path, dest });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli PDF</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
  </div>

  <div class="notice">
    La sanitizzazione rimuove JavaScript incorporato, metadati e allegati dal documento PDF.
  </div>

  <div class="row">
    <button class="btn primary" onclick={run} disabled={!path || busy}>
      {busy ? "Sanitizzazione…" : "Sanitizza"}
    </button>
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if result}
    <div class="notice good">File sanitizzato e salvato correttamente.</div>
    {#if result.removed.length > 0}
      <div class="stack">
        <span class="field-label">Elementi rimossi</span>
        <div class="row wrap">
          {#each result.removed as item, i (i)}
            <span class="pill">{item}</span>
          {/each}
        </div>
      </div>
    {:else}
      <div class="notice">Nessun elemento indesiderato trovato.</div>
    {/if}
  {/if}
</div>
