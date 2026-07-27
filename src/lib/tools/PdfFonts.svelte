<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { copy } from "../api";

  interface PdfFontInfo {
    name: string;
    baseName: string;
    family: string;
    embedded: boolean;
    subset: boolean;
    standard14: boolean;
    weight: number | null;
    italic: boolean;
    style: string;
    embeddedBytes: number | null;
    pages: number[];
    glyphs: number;
  }
  interface PdfFontsReport {
    pages: number;
    scanned: number;
    fonts: PdfFontInfo[];
  }

  let path = $state<string | null>(null);
  let busy = $state(false);
  let error = $state("");
  let report = $state<PdfFontsReport | null>(null);
  let note = $state("");

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  function kb(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} kB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  /// A font that is neither embedded nor one of the standard 14 will be
  /// substituted on any machine that lacks it — the document will not look the
  /// way it does here.
  function risky(f: PdfFontInfo): boolean {
    return !f.embedded && !f.standard14;
  }

  function pagesLabel(pages: number[]): string {
    if (pages.length <= 6) return pages.join(", ");
    return `${pages.slice(0, 6).join(", ")} … (+${pages.length - 6})`;
  }

  async function load() {
    if (!path) return;
    busy = true;
    error = "";
    note = "";
    try {
      report = await invoke<PdfFontsReport>("pdf_fonts", { path });
    } catch (e) {
      error = String(e);
      report = null;
    } finally {
      busy = false;
    }
  }

  async function pick() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") {
      path = p;
      await load();
    }
  }

  async function extract(f: PdfFontInfo) {
    if (!path || !f.embedded) return;
    const dest = await save({
      defaultPath: `${f.baseName || "font"}.ttf`,
      filters: [{ name: "Font", extensions: ["ttf", "otf", "cff", "pfb"] }],
    });
    if (typeof dest !== "string") return;
    busy = true;
    error = "";
    note = "";
    try {
      const written = await invoke<number>("pdf_font_extract", {
        path,
        fontName: f.name,
        dest,
      });
      note = `Estratti ${kb(written)} in ${fileName(dest)}.`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const problems = $derived(report ? report.fonts.filter(risky).length : 0);
  const embedded = $derived(report ? report.fonts.filter((f) => f.embedded).length : 0);
  const totalBytes = $derived(
    report ? report.fonts.reduce((n, f) => n + (f.embeddedBytes ?? 0), 0) : 0,
  );

  async function copyList() {
    if (!report) return;
    const lines = report.fonts.map(
      (f) =>
        `${f.name}\t${f.embedded ? "incorporato" : "NON incorporato"}${
          f.subset ? " (subset)" : ""
        }${f.standard14 ? " [standard 14]" : ""}\tpagine ${pagesLabel(f.pages)}`,
    );
    await copy(lines.join("\n"));
    note = "Elenco copiato.";
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli PDF</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
    <span class="spacer"></span>
    {#if report}
      <button class="btn ghost sm" onclick={copyList}>⧉ Copia elenco</button>
    {/if}
    <button class="btn ghost sm" onclick={load} disabled={!path || busy}>↻ Ricarica</button>
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}
  {#if note}
    <div class="notice good">{note}</div>
  {/if}

  {#if report}
    <div class="tool-controls wrap">
      <span class="pill">{report.fonts.length} font</span>
      <span class="pill">{embedded} incorporati</span>
      {#if totalBytes > 0}
        <span class="pill">{kb(totalBytes)} di font nel file</span>
      {/if}
      {#if problems > 0}
        <span class="pill bad">{problems} a rischio sostituzione</span>
      {:else}
        <span class="pill good">nessuna sostituzione prevista</span>
      {/if}
    </div>

    {#if problems > 0}
      <div class="notice bad">
        {problems === 1 ? "Un font non è incorporato" : `${problems} font non sono incorporati`}
        e non fa parte dei 14 standard: su un'altra macchina il lettore lo sostituirà e
        l'impaginazione può cambiare.
      </div>
    {/if}

    {#if report.scanned < report.pages}
      <div class="notice">
        Analizzate le prime {report.scanned} pagine su {report.pages}.
      </div>
    {/if}

    <div class="scroll">
      {#each report.fonts as f (f.name)}
        <div class="font-row" class:risk={risky(f)}>
          <div class="main">
            <div class="name mono">{f.name}</div>
            <div class="tags">
              {#if f.embedded}
                <span class="pill good">incorporato</span>
              {:else}
                <span class="pill bad">non incorporato</span>
              {/if}
              {#if f.subset}<span class="pill">subset</span>{/if}
              {#if f.standard14}<span class="pill">standard 14</span>{/if}
              <span class="pill">{f.style}</span>
              {#if f.italic}<span class="pill">corsivo</span>{/if}
              {#if f.weight !== null}<span class="pill">peso {f.weight}</span>{/if}
              {#if f.embeddedBytes !== null}
                <span class="pill">{kb(f.embeddedBytes)}</span>
              {/if}
            </div>
            <div class="faint small">
              {#if f.family && f.family !== f.name}famiglia {f.family} · {/if}
              pagine {pagesLabel(f.pages)} · {f.glyphs.toLocaleString("it-IT")} caratteri
            </div>
          </div>
          <button
            class="btn ghost sm"
            onclick={() => extract(f)}
            disabled={!f.embedded || busy}
            title={f.embedded ? "Salva il font incorporato su file" : "Font non incorporato"}
          >
            ⤓ Estrai
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .scroll {
    display: flex;
    flex-direction: column;
    gap: 9px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    padding-right: 4px;
  }
  .font-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--radius-lg);
    border: 1.5px solid var(--border);
    background: var(--surface);
  }
  .font-row.risk {
    border-color: color-mix(in srgb, var(--red) 34%, var(--border));
    background: color-mix(in srgb, var(--red) 6%, var(--surface));
  }
  .main {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    flex: 1;
  }
  .name {
    font-size: 13.5px;
    font-weight: 700;
    word-break: break-all;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .small {
    font-size: 12px;
  }
  .pill.good {
    background: color-mix(in srgb, var(--green) 16%, var(--surface));
    color: var(--green);
  }
  .pill.bad {
    background: color-mix(in srgb, var(--red) 16%, var(--surface));
    color: var(--red);
  }
</style>
