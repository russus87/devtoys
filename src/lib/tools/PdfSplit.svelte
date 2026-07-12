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
  let rangeText = $state("");
  let busy = $state(false);
  let error = $state("");
  let success = $state("");

  function baseName(p: string): string {
    return p.split(/[\\/]/).pop() ?? p;
  }

  /** Parse "1-3,5,8" into a deduped, in-order list of 1-based page numbers, clamped to [1, maxPage]. */
  function parseRanges(s: string, maxPage: number): number[] {
    const out: number[] = [];
    for (const raw of s.split(",")) {
      const part = raw.trim();
      if (!part) continue;
      const m = part.match(/^(\d+)\s*-\s*(\d+)$/);
      if (m) {
        let a = parseInt(m[1], 10);
        let b = parseInt(m[2], 10);
        if (a > b) [a, b] = [b, a];
        for (let n = a; n <= b; n++) {
          if (n >= 1 && n <= maxPage && !out.includes(n)) out.push(n);
        }
      } else {
        const n = parseInt(part, 10);
        if (!isNaN(n) && n >= 1 && n <= maxPage && !out.includes(n)) out.push(n);
      }
    }
    return out;
  }

  const selectedPages = $derived.by(() => {
    if (pages === null) return [];
    return parseRanges(rangeText, pages);
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

  async function extract() {
    if (!path || selectedPages.length === 0) return;
    error = "";
    success = "";
    const dest = await save({
      defaultPath: "estratto.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    try {
      await invoke("pdf_extract_pages", { path, dest, pages: selectedPages });
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
    <span class="field-label">Pagine da estrarre (es. 1-3,5,8)</span>
    <input class="input" type="text" placeholder="1-3,5,8" bind:value={rangeText} disabled={!path} />
    {#if selectedPages.length > 0}
      <span class="faint">Selezionate: {selectedPages.join(", ")}</span>
    {/if}
  </div>

  <div class="tool-controls">
    <button class="btn primary" onclick={extract} disabled={!path || selectedPages.length === 0 || busy}>
      {busy ? "Estrazione…" : "Estrai pagine"}
    </button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}
  {#if success}<div class="notice good">{success}</div>{/if}
</div>
