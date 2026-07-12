<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let path = $state<string | null>(null);
  let format = $state("json");
  let busy = $state(false);
  let error = $state("");
  let output = $state("");

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  async function pick() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") {
      path = p;
      output = "";
      error = "";
    }
  }

  async function analyze() {
    if (!path) return;
    busy = true;
    error = "";
    output = "";
    try {
      output = await invoke<string>("pdf_structure", { path, format });
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
    <span class="spacer"></span>
    <span class="field-label">Formato</span>
    <Segmented
      bind:value={format}
      options={[
        { value: "json", label: "JSON" },
        { value: "xml", label: "XML" },
      ]}
    />
    <button class="btn primary" onclick={analyze} disabled={!path || busy}>
      {busy ? "Analisi…" : "Analizza"}
    </button>
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  <Pane label="Struttura" value={output} readonly rows={20} showPaste={false} />
</div>
