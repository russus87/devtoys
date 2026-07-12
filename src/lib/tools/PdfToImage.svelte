<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import Segmented from "../ui/Segmented.svelte";

  interface PdfInfo {
    pages: number;
    title: string | null;
  }

  let path = $state("");
  let fileName = $state("");
  let pages = $state<number | null>(null);
  let format = $state("png");
  let width = $state(1240);
  let busy = $state(false);
  let error = $state("");
  let images = $state<string[]>([]);

  const jpeg = $derived(format === "jpeg");
  const mime = $derived(jpeg ? "image/jpeg" : "image/png");
  const ext = $derived(jpeg ? "jpg" : "png");

  function baseName(p: string): string {
    return p.split(/[\\/]/).pop() ?? p;
  }

  async function pickFile() {
    error = "";
    images = [];
    pages = null;
    const picked = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!picked || Array.isArray(picked)) return;
    path = picked;
    fileName = baseName(picked);
    try {
      const info = await invoke<PdfInfo>("pdf_info", { path });
      pages = info.pages;
    } catch (e) {
      error = String(e);
    }
  }

  async function convert() {
    if (!path) return;
    busy = true;
    error = "";
    images = [];
    try {
      images = await invoke<string[]>("pdf_to_images", { path, width, jpeg });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="tool">
  <div class="tool-controls">
    <button class="btn" onclick={pickFile}>📄 Scegli PDF</button>
    {#if fileName}<span class="pill">{fileName}</span>{/if}
    {#if pages !== null}<span class="pill gold">{pages} pagine</span>{/if}
  </div>

  <div class="tool-controls">
    <span class="field-label">Formato</span>
    <Segmented
      bind:value={format}
      options={[
        { value: "png", label: "PNG" },
        { value: "jpeg", label: "JPEG" },
      ]}
    />
    <span class="field-label">Larghezza (px)</span>
    <input class="input width-input" type="number" min="100" step="10" bind:value={width} />
    <button class="btn primary" onclick={convert} disabled={!path || busy}>
      {busy ? "Conversione…" : "Converti"}
    </button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}

  {#if images.length > 0}
    <div class="grow img-grid">
      {#each images as img, i (i)}
        <div class="img-card">
          <img src="data:{mime};base64,{img}" alt="Pagina {i + 1}" />
          <div class="img-card-foot">
            <span class="faint">Pagina {i + 1}</span>
            <a class="btn sm" href="data:{mime};base64,{img}" download="pagina-{i + 1}.{ext}">⬇ Scarica</a>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .width-input {
    width: 100px;
  }
  .img-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
    overflow-y: auto;
    padding-right: 4px;
  }
  .img-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border);
    background: var(--surface);
  }
  .img-card img {
    width: 100%;
    border-radius: 8px;
    border: 1px solid var(--border-2);
    background: #fff;
  }
  .img-card-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 12px;
  }
</style>
