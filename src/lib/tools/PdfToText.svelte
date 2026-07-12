<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let path = $state("");
  let fileName = $state("");
  let format = $state("testo");
  let busy = $state(false);
  let error = $state("");
  let result = $state("");

  function baseName(p: string): string {
    return p.split(/[\\/]/).pop() ?? p;
  }

  async function pickFile() {
    error = "";
    result = "";
    const picked = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!picked || Array.isArray(picked)) return;
    path = picked;
    fileName = baseName(picked);
  }

  async function extract() {
    if (!path) return;
    busy = true;
    error = "";
    try {
      result = await invoke<string>("pdf_to_text", { path, format });
    } catch (e) {
      error = String(e);
      result = "";
    } finally {
      busy = false;
    }
  }
</script>

<div class="tool">
  <div class="tool-controls">
    <button class="btn" onclick={pickFile}>📄 Scegli PDF</button>
    {#if fileName}<span class="pill">{fileName}</span>{/if}
  </div>

  <div class="tool-controls">
    <span class="field-label">Formato</span>
    <Segmented
      bind:value={format}
      options={[
        { value: "testo", label: "Testo" },
        { value: "markdown", label: "Markdown" },
        { value: "html", label: "HTML" },
      ]}
    />
    <button class="btn primary" onclick={extract} disabled={!path || busy}>
      {busy ? "Estrazione…" : "Estrai"}
    </button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}

  <Pane label="Testo estratto" value={result} readonly showPaste={false} rows={18} />
</div>
