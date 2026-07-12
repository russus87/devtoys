<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import Segmented from "../ui/Segmented.svelte";

  interface ChangeRow {
    type: "eq" | "add" | "del";
    text: string;
  }
  interface CompareTextResult {
    added: number;
    removed: number;
    changes: ChangeRow[];
  }
  interface CompareImageResult {
    diffBase64: string;
    changedRatio: number;
  }

  let pathA = $state<string | null>(null);
  let pathB = $state<string | null>(null);
  let mode = $state("testo");
  let page = $state(0);
  let width = $state(900);
  let busy = $state(false);
  let error = $state("");

  let textResult = $state<CompareTextResult | null>(null);
  let imageResult = $state<CompareImageResult | null>(null);

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  async function pickA() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") pathA = p;
  }
  async function pickB() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") pathB = p;
  }

  function prefix(t: ChangeRow["type"]): string {
    return t === "add" ? "+" : t === "del" ? "−" : " ";
  }

  async function compareText() {
    if (!pathA || !pathB) return;
    busy = true;
    error = "";
    textResult = null;
    try {
      textResult = await invoke<CompareTextResult>("pdf_compare_text", { a: pathA, b: pathB });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function compareImage() {
    if (!pathA || !pathB) return;
    busy = true;
    error = "";
    imageResult = null;
    try {
      imageResult = await invoke<CompareImageResult>("pdf_compare_image", {
        a: pathA,
        b: pathB,
        page,
        width,
      });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const canRun = $derived(!!pathA && !!pathB && !busy);
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pickA} disabled={busy}>Scegli PDF A</button>
    <span class="mono faint">{pathA ? fileName(pathA) : "nessun file"}</span>
    <span class="spacer"></span>
    <button class="btn" onclick={pickB} disabled={busy}>Scegli PDF B</button>
    <span class="mono faint">{pathB ? fileName(pathB) : "nessun file"}</span>
  </div>

  <div class="tool-controls">
    <span class="field-label">Modalità</span>
    <Segmented
      bind:value={mode}
      options={[
        { value: "testo", label: "Testo" },
        { value: "immagine", label: "Immagine" },
      ]}
    />
  </div>

  {#if mode === "testo"}
    <div class="row">
      <button class="btn primary" onclick={compareText} disabled={!canRun}>
        {busy ? "Confronto…" : "Confronta testo"}
      </button>
    </div>

    {#if textResult}
      <div class="diff-summary muted">
        +{textResult.added} aggiunte · −{textResult.removed} rimosse
      </div>
      <div class="diff-result grow">
        {#each textResult.changes as r, i (i)}
          <div class="diff-line" class:add={r.type === "add"} class:del={r.type === "del"}>
            <span class="diff-prefix">{prefix(r.type)}</span><span class="diff-text">{r.text || " "}</span>
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    <div class="tool-controls wrap">
      <div class="stack" style="gap:4px">
        <span class="field-label">Pagina (0-based)</span>
        <input class="input" type="number" min="0" step="1" bind:value={page} />
      </div>
      <div class="stack" style="gap:4px">
        <span class="field-label">Larghezza</span>
        <input class="input" type="number" min="100" step="50" bind:value={width} />
      </div>
      <button class="btn primary" onclick={compareImage} disabled={!canRun}>
        {busy ? "Confronto…" : "Confronta immagine"}
      </button>
    </div>

    {#if imageResult}
      <div class="stack">
        <div class="notice">differenza {(imageResult.changedRatio * 100).toFixed(2)}%</div>
        <img class="diff-img" src="data:image/png;base64,{imageResult.diffBase64}" alt="Differenza tra pagine" />
      </div>
    {/if}
  {/if}

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}
</div>

<style>
  .diff-summary {
    font-size: 13px;
  }
  .diff-result {
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
    overflow: auto;
    max-height: 420px;
  }
  .diff-line {
    display: flex;
    gap: 8px;
    font-family: var(--mono);
    font-size: 13px;
    white-space: pre-wrap;
    word-break: break-word;
    padding: 2px 10px;
  }
  .diff-prefix {
    flex: 0 0 auto;
    opacity: 0.6;
    user-select: none;
  }
  .diff-text {
    flex: 1;
    min-width: 0;
  }
  .diff-line.del {
    background: color-mix(in srgb, var(--red) 12%, var(--surface));
  }
  .diff-line.add {
    background: color-mix(in srgb, var(--green) 12%, var(--surface));
  }
  .diff-img {
    max-width: 100%;
    border-radius: var(--radius);
    border: 1.5px solid var(--border-2);
  }
</style>
