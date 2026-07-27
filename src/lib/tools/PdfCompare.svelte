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
    widthPx: number;
    heightPx: number;
    changedPixels: number;
    totalPixels: number;
  }

  let pathA = $state<string | null>(null);
  let pathB = $state<string | null>(null);
  let mode = $state("testo");
  let page = $state(0);
  let dpi = $state("150");
  let preset = $state("normale");
  let busy = $state(false);
  let error = $state("");

  // Due PDF con contenuto identico prodotti da generatori diversi non danno mai
  // pixel identici: un filetto scritto come rettangolo viene agganciato alla
  // griglia dei pixel, lo stesso filetto scritto come tracciato viene sfumato, e
  // singoli glifi scivolano di frazioni di pixel. Perdonare anche il pixel
  // vicino (raggio) oltre che l'intensità (tolleranza) assorbe tutto questo
  // senza nascondere il testo aggiunto o rimosso, che un vicino non ce l'ha.
  const PRESETS: Record<string, { tolerance: number; radius: number }> = {
    esatta: { tolerance: 0, radius: 0 },
    normale: { tolerance: 128, radius: 1 },
    permissiva: { tolerance: 160, radius: 2 },
  };

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
        dpi: Number(dpi),
        tolerance: PRESETS[preset].tolerance,
        radius: PRESETS[preset].radius,
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
        <input class="input page-input" type="number" min="0" step="1" bind:value={page} />
      </div>
      <div class="stack" style="gap:4px">
        <span class="field-label">Tolleranza</span>
        <Segmented
          bind:value={preset}
          options={[
            { value: "esatta", label: "Esatta" },
            { value: "normale", label: "Normale" },
            { value: "permissiva", label: "Permissiva" },
          ]}
        />
      </div>
      <div class="stack" style="gap:4px">
        <span class="field-label">Risoluzione</span>
        <Segmented
          bind:value={dpi}
          options={[
            { value: "72", label: "72 DPI" },
            { value: "150", label: "150 DPI" },
            { value: "300", label: "300 DPI" },
          ]}
        />
      </div>
      <button class="btn primary" onclick={compareImage} disabled={!canRun}>
        {busy ? "Confronto…" : "Confronta immagine"}
      </button>
    </div>

    <div class="muted hint">
      Con tolleranza <strong>esatta</strong> anche due PDF con contenuto identico risultano diversi:
      generatori diversi rasterizzano filetti e caratteri con arrotondamenti diversi.
      <strong>Normale</strong> assorbe questo rumore e lascia emergere solo le differenze reali.
      A 72 DPI il rumore torna a farsi sentire: sotto i 150 DPI un filetto sottile è troppo vicino
      alla dimensione del pixel.
    </div>

    {#if imageResult}
      <div class="stack">
        <div class="notice">
          differenza {(imageResult.changedRatio * 100).toFixed(3)}% ·
          {imageResult.changedPixels.toLocaleString("it-IT")} px su
          {imageResult.totalPixels.toLocaleString("it-IT")} ·
          {imageResult.widthPx}×{imageResult.heightPx} a {dpi} DPI · tolleranza
          {PRESETS[preset].tolerance}/255, raggio {PRESETS[preset].radius} px
        </div>
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
  .page-input {
    width: 120px;
  }
  .hint {
    font-size: 12px;
    max-width: 70ch;
  }
  .diff-img {
    max-width: 100%;
    border-radius: var(--radius);
    border: 1.5px solid var(--border-2);
  }
</style>
