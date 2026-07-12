<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  interface PdfInfo {
    pages: number;
    title: string | null;
  }

  let path = $state("");
  let fileName = $state("");
  let pages = $state<number | null>(null);
  let orderText = $state("");
  let busy = $state(false);
  let error = $state("");
  let success = $state("");

  function baseName(p: string): string {
    return p.split(/[\\/]/).pop() ?? p;
  }

  /** Parse "3,1,2,4" into a 1-based page order, clamped to [1, maxPage]. Duplicates preserved as typed. */
  function parseOrder(s: string, maxPage: number): number[] {
    const out: number[] = [];
    for (const raw of s.split(",")) {
      const part = raw.trim();
      if (!part) continue;
      const n = parseInt(part, 10);
      if (!isNaN(n) && n >= 1 && n <= maxPage) out.push(n);
    }
    return out;
  }

  const newOrder = $derived.by(() => {
    if (pages === null) return [];
    return parseOrder(orderText, pages);
  });

  const isValidPermutation = $derived.by(() => {
    if (pages === null) return false;
    if (newOrder.length !== pages) return false;
    const seen = new Set(newOrder);
    return seen.size === pages;
  });

  async function pickFile() {
    error = "";
    success = "";
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

  async function reorder() {
    if (!path || newOrder.length === 0) return;
    error = "";
    success = "";
    const dest = await save({
      defaultPath: "riordinato.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    try {
      await invoke("pdf_reorder", { path, dest, order: newOrder });
      success = `Salvato in ${dest}`;
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

  <div class="stack">
    <span class="field-label">Nuovo ordine pagine (es. 3,1,2,4)</span>
    <input class="input" type="text" placeholder="3,1,2,4" bind:value={orderText} disabled={!path} />
    {#if newOrder.length > 0}
      <span class="faint">Ordine: {newOrder.join(", ")}</span>
    {/if}
    {#if orderText.trim() && pages !== null && !isValidPermutation}
      <span class="err faint">Devi indicare tutte le {pages} pagine, ciascuna una sola volta.</span>
    {/if}
  </div>

  <div class="tool-controls">
    <button class="btn primary" onclick={reorder} disabled={!path || !isValidPermutation || busy}>
      {busy ? "Riordino…" : "Riordina"}
    </button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}
  {#if success}<div class="notice good">{success}</div>{/if}
</div>
