<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { copy } from "../api";

  interface NameEntry {
    label: string;
    value: string;
  }
  interface TableEntry {
    tag: string;
    size: number | null;
  }
  interface Embedding {
    code: number;
    label: string;
    level: "ok" | "warn" | "bad" | "unknown";
    notes: string[];
  }
  interface Metrics {
    unitsPerEm: number;
    ascender: number;
    descender: number;
    lineGap: number;
    capHeight: number | null;
    xHeight: number | null;
    bbox: [number, number, number, number];
    italicAngle: number | null;
    underlinePosition: number | null;
    underlineThickness: number | null;
  }
  interface AxisInfo {
    tag: string;
    label: string;
    min: number;
    default: number;
    max: number;
  }
  interface BlockCoverage {
    name: string;
    covered: number;
    total: number;
  }
  interface FontReport {
    fileName: string;
    fileSize: number;
    format: string;
    outlines: string;
    fontCount: number;
    index: number;
    names: NameEntry[];
    numGlyphs: number;
    mappedChars: number;
    metrics: Metrics;
    weightClass: number | null;
    weightLabel: string | null;
    widthLabel: string | null;
    vendor: string | null;
    panose: string | null;
    embedding: Embedding;
    axes: AxisInfo[];
    tables: TableEntry[];
    features: string[];
    scripts: string[];
    flags: string[];
    coverage: BlockCoverage[];
  }

  let path = $state<string | null>(null);
  let index = $state(0);
  let busy = $state(false);
  let error = $state("");
  let report = $state<FontReport | null>(null);
  let copied = $state(false);

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

  async function load() {
    if (!path) return;
    busy = true;
    error = "";
    try {
      report = await invoke<FontReport>("font_inspect", { path, index });
    } catch (e) {
      error = String(e);
      report = null;
    } finally {
      busy = false;
    }
  }

  async function pick() {
    const p = await open({
      filters: [{ name: "Font", extensions: ["ttf", "otf", "ttc", "otc", "woff", "woff2"] }],
    });
    if (typeof p === "string") {
      path = p;
      index = 0;
      await load();
    }
  }

  async function pickIndex(i: number) {
    index = i;
    await load();
  }

  /// The whole report as text, for pasting into a ticket.
  async function copyReport() {
    if (!report) return;
    const r = report;
    const lines = [
      `${r.fileName} — ${r.format}, ${r.outlines}, ${kb(r.fileSize)}`,
      ...r.names.map((n) => `${n.label}: ${n.value}`),
      `Glifi: ${r.numGlyphs} (${r.mappedChars} caratteri mappati)`,
      `Unità per em: ${r.metrics.unitsPerEm}`,
      `Ascender/Descender: ${r.metrics.ascender} / ${r.metrics.descender}`,
      r.weightLabel ? `Peso: ${r.weightLabel}` : "",
      r.widthLabel ? `Larghezza: ${r.widthLabel}` : "",
      r.vendor ? `Foundry: ${r.vendor}` : "",
      `Embedding (fsType ${r.embedding.code}): ${r.embedding.label}`,
      r.axes.length
        ? `Assi: ${r.axes.map((a) => `${a.tag} ${a.min}–${a.max}`).join(", ")}`
        : "",
      r.features.length ? `Feature: ${r.features.join(" ")}` : "",
      `Tabelle: ${r.tables.map((t) => t.tag).join(" ")}`,
    ].filter(Boolean);
    await copy(lines.join("\n"));
    copied = true;
    setTimeout(() => (copied = false), 1100);
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli font</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
    <span class="spacer"></span>
    {#if report}
      <button class="btn ghost sm" class:ok={copied} onclick={copyReport}>
        {copied ? "✓ Copiato" : "⧉ Copia scheda"}
      </button>
    {/if}
    <button class="btn ghost sm" onclick={load} disabled={!path || busy}>↻ Ricarica</button>
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if report}
    {#if report.fontCount > 1}
      <div class="notice">
        Questo file è una collezione di {report.fontCount} font.
        <span class="picker">
          {#each Array(report.fontCount) as _, i (i)}
            <button
              class="btn ghost sm"
              class:active={i === report.index}
              onclick={() => pickIndex(i)}
              disabled={busy}
            >
              {i + 1}
            </button>
          {/each}
        </span>
      </div>
    {/if}

    <div class="scroll">
      <!-- Identity ------------------------------------------------------- -->
      <div class="card-box">
        <div class="head">
          <div>
            <div class="title">{report.names[0]?.value ?? report.fileName}</div>
            <div class="faint small">{report.fileName}</div>
          </div>
          <div class="pills">
            <span class="pill">{report.format}</span>
            <span class="pill">{report.outlines}</span>
            <span class="pill">{kb(report.fileSize)}</span>
          </div>
        </div>

        <div class="meta-table">
          {#each report.names as n (n.label + n.value)}
            <div class="meta-row">
              <span class="field-label">{n.label}</span>
              <span class="meta-value">{n.value}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Embedding permissions ------------------------------------------ -->
      <div class="card-box">
        <span class="field-label">Permessi di embedding</span>
        <div class="embed {report.embedding.level}">
          <strong>{report.embedding.label}</strong>
          <span class="pill mono">fsType {report.embedding.code}</span>
        </div>
        <ul class="notes">
          {#each report.embedding.notes as note (note)}
            <li>{note}</li>
          {/each}
        </ul>
      </div>

      <!-- Numbers -------------------------------------------------------- -->
      <div class="card-box">
        <span class="field-label">Numeri</span>
        <div class="grid">
          <div class="stat"><b>{report.numGlyphs}</b><span>glifi</span></div>
          <div class="stat"><b>{report.mappedChars}</b><span>caratteri mappati</span></div>
          <div class="stat"><b>{report.metrics.unitsPerEm}</b><span>unità per em</span></div>
          <div class="stat"><b>{report.metrics.ascender}</b><span>ascender</span></div>
          <div class="stat"><b>{report.metrics.descender}</b><span>descender</span></div>
          <div class="stat"><b>{report.metrics.lineGap}</b><span>line gap</span></div>
          {#if report.metrics.capHeight !== null}
            <div class="stat"><b>{report.metrics.capHeight}</b><span>altezza maiuscole</span></div>
          {/if}
          {#if report.metrics.xHeight !== null}
            <div class="stat"><b>{report.metrics.xHeight}</b><span>altezza x</span></div>
          {/if}
          {#if report.weightLabel}
            <div class="stat wide"><b>{report.weightLabel}</b><span>peso</span></div>
          {/if}
          {#if report.widthLabel}
            <div class="stat wide"><b>{report.widthLabel}</b><span>larghezza</span></div>
          {/if}
          {#if report.vendor}
            <div class="stat"><b class="mono">{report.vendor}</b><span>foundry</span></div>
          {/if}
        </div>
        <div class="faint small bbox mono">
          bounding box: {report.metrics.bbox.join(", ")}
          {#if report.metrics.italicAngle}
            · angolo corsivo {report.metrics.italicAngle.toFixed(1)}°
          {/if}
        </div>
      </div>

      {#if report.flags.length}
        <div class="card-box">
          <span class="field-label">Caratteristiche</span>
          <div class="tags">
            {#each report.flags as f (f)}
              <span class="pill gold">{f}</span>
            {/each}
          </div>
        </div>
      {/if}

      {#if report.axes.length}
        <div class="card-box">
          <span class="field-label">Assi variabili</span>
          <div class="meta-table">
            {#each report.axes as a (a.tag)}
              <div class="meta-row">
                <span class="mono">{a.tag}</span>
                <span class="meta-value">
                  {a.label} — da {a.min} a {a.max}, predefinito {a.default}
                </span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if report.coverage.length}
        <div class="card-box">
          <span class="field-label">Copertura Unicode</span>
          <div class="cov">
            {#each report.coverage as c (c.name)}
              <div class="cov-row">
                <span class="cov-name">{c.name}</span>
                <span class="bar">
                  <span class="fill" style="width: {(c.covered / c.total) * 100}%"></span>
                </span>
                <span class="mono faint small">{c.covered}/{c.total}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if report.features.length}
        <div class="card-box">
          <span class="field-label">Feature OpenType ({report.features.length})</span>
          <div class="tags">
            {#each report.features as f (f)}
              <span class="pill mono">{f}</span>
            {/each}
          </div>
          {#if report.scripts.length}
            <span class="field-label spaced">Sistemi di scrittura</span>
            <div class="tags">
              {#each report.scripts as s (s)}
                <span class="pill mono">{s}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <div class="card-box">
        <span class="field-label">Tabelle ({report.tables.length})</span>
        <div class="tags">
          {#each report.tables as t (t.tag)}
            <span class="pill mono" title={t.size !== null ? `${kb(t.size)}` : "dimensione non nota"}>
              {t.tag}
            </span>
          {/each}
        </div>
        {#if report.panose}
          <span class="field-label spaced">PANOSE</span>
          <div class="mono small faint">{report.panose}</div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .scroll {
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    padding-right: 4px;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 14px;
    margin-bottom: 14px;
    flex-wrap: wrap;
  }
  .title {
    font-size: 17px;
    font-weight: 700;
  }
  .small {
    font-size: 12px;
  }
  .pills,
  .tags,
  .picker {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .picker {
    margin-left: 10px;
    display: inline-flex;
  }
  .btn.ghost.active {
    background: var(--primary-soft);
    color: var(--primary);
  }
  .meta-table {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .meta-row {
    display: grid;
    grid-template-columns: 170px 1fr;
    align-items: baseline;
    gap: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }
  .meta-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }
  .meta-value {
    font-size: 13px;
    color: var(--ink);
    word-break: break-word;
  }
  .embed {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    margin: 10px 0;
    padding: 11px 14px;
    border-radius: 12px;
    border: 1.5px solid var(--border-2);
    background: var(--surface-2);
    font-size: 14px;
  }
  .embed.ok {
    background: color-mix(in srgb, var(--green) 10%, var(--surface));
    border-color: color-mix(in srgb, var(--green) 32%, var(--border));
    color: var(--green);
  }
  .embed.warn {
    background: var(--gold-soft);
    border-color: color-mix(in srgb, var(--gold) 40%, var(--border));
  }
  .embed.bad {
    background: color-mix(in srgb, var(--red) 10%, var(--surface));
    border-color: color-mix(in srgb, var(--red) 32%, var(--border));
    color: var(--red);
  }
  .notes {
    margin: 0;
    padding-left: 18px;
    font-size: 12.5px;
    color: var(--ink-dim);
  }
  .notes li {
    margin-top: 3px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 10px;
    margin: 12px 0;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 9px 12px;
    border-radius: 11px;
    background: var(--surface-2);
  }
  .stat.wide {
    grid-column: span 2;
  }
  .stat b {
    font-size: 15px;
  }
  .stat span {
    font-size: 11.5px;
    color: var(--ink-faint);
  }
  .bbox {
    font-size: 12px;
  }
  .spaced {
    display: block;
    margin-top: 14px;
    margin-bottom: 6px;
  }
  .cov {
    display: flex;
    flex-direction: column;
    gap: 7px;
    margin-top: 10px;
  }
  .cov-row {
    display: grid;
    grid-template-columns: 190px 1fr 70px;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }
  .cov-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .bar {
    height: 7px;
    border-radius: 999px;
    background: var(--surface-3);
    overflow: hidden;
  }
  .fill {
    display: block;
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--primary), var(--primary-2));
  }
  .cov-row .small {
    text-align: right;
  }
  @media (max-width: 620px) {
    .meta-row {
      grid-template-columns: 1fr;
      gap: 2px;
    }
    .cov-row {
      grid-template-columns: 120px 1fr 62px;
    }
  }
</style>
