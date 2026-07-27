<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { copy } from "../api";

  interface Glyph {
    gid: number;
    code: number;
    ch: string;
    name: string | null;
  }
  interface GlyphsReport {
    numGlyphs: number;
    glyphs: Glyph[];
    unmapped: number;
    dataUri: string;
  }

  /// Rendering every glyph of a large CJK font at once locks the webview for
  /// seconds, so the grid stops here and says so.
  const MAX_SHOWN = 1500;

  let path = $state<string | null>(null);
  let busy = $state(false);
  let error = $state("");
  let report = $state<GlyphsReport | null>(null);
  let query = $state("");
  let size = $state(34);
  let sample = $state("");
  let copiedCode = $state<number | null>(null);

  /// Each loaded font gets its own family name, so switching files cannot show
  /// the previous font's glyphs from cache.
  let family = $state("");
  let styleEl: HTMLStyleElement | null = null;
  let seq = 0;

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  function hex(code: number): string {
    return `U+${code.toString(16).toUpperCase().padStart(4, "0")}`;
  }

  function install(dataUri: string) {
    family = `devtoys-glyphs-${++seq}`;
    if (!styleEl) {
      styleEl = document.createElement("style");
      document.head.appendChild(styleEl);
    }
    styleEl.textContent = `@font-face { font-family: "${family}"; src: url("${dataUri}"); }`;
  }

  $effect(() => () => styleEl?.remove());

  async function load() {
    if (!path) return;
    busy = true;
    error = "";
    try {
      const r = await invoke<GlyphsReport>("font_glyphs", { path, index: 0 });
      report = r;
      install(r.dataUri);
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
      await load();
    }
  }

  const filtered = $derived.by((): Glyph[] => {
    if (!report) return [];
    const q = query.trim().toLowerCase();
    if (!q) return report.glyphs;
    // A query can be the character itself, a codepoint (hex or U+), the glyph
    // id, or part of the PostScript glyph name.
    const asHex = q.replace(/^u\+/, "");
    const num = /^[0-9a-f]+$/.test(asHex) ? parseInt(asHex, 16) : NaN;
    const dec = /^\d+$/.test(q) ? parseInt(q, 10) : NaN;
    return report.glyphs.filter(
      (g) =>
        g.ch === query ||
        g.code === num ||
        g.gid === dec ||
        (g.name?.toLowerCase().includes(q) ?? false),
    );
  });

  const shown = $derived(filtered.slice(0, MAX_SHOWN));

  async function pickGlyph(g: Glyph) {
    await copy(g.ch);
    sample += g.ch;
    copiedCode = g.code;
    setTimeout(() => {
      if (copiedCode === g.code) copiedCode = null;
    }, 900);
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli font</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
    <span class="spacer"></span>
    {#if report}
      <span class="pill">{report.glyphs.length} glifi mappati</span>
      {#if report.unmapped > 0}
        <span class="pill" title="Glifi raggiungibili solo tramite legature o alternative">
          +{report.unmapped} non mappati
        </span>
      {/if}
    {/if}
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if report}
    <div class="tool-controls wrap">
      <input
        class="input search"
        placeholder="Cerca: carattere, U+00E8, 232, o nome del glifo"
        bind:value={query}
      />
      <label class="size">
        <span class="field-label">Dimensione</span>
        <input type="range" min="20" max="72" bind:value={size} />
      </label>
    </div>

    <div class="preview card-box">
      <span class="field-label">Anteprima</span>
      <input
        class="input sample"
        style="font-family: '{family}', sans-serif"
        placeholder="Scrivi qui per provare il font…"
        bind:value={sample}
      />
    </div>

    {#if filtered.length === 0}
      <div class="notice">Nessun glifo corrisponde a «{query}».</div>
    {:else}
      {#if filtered.length > MAX_SHOWN}
        <div class="notice">
          Mostrati i primi {MAX_SHOWN} glifi su {filtered.length}: restringi la ricerca per
          vedere gli altri.
        </div>
      {/if}
      <div class="grid" style="--cell: {size + 26}px; --glyph: {size}px">
        {#each shown as g (g.gid + ":" + g.code)}
          <button
            class="cell"
            class:copied={copiedCode === g.code}
            onclick={() => pickGlyph(g)}
            title="{hex(g.code)}{g.name ? ` · ${g.name}` : ''} · glifo {g.gid}"
          >
            <span class="glyph" style="font-family: '{family}', sans-serif">{g.ch}</span>
            <span class="code mono">{hex(g.code)}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .search {
    flex: 1;
    min-width: 220px;
  }
  .size {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .preview {
    padding: 14px;
  }
  .sample {
    margin-top: 8px;
    font-size: 26px;
    line-height: 1.5;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--cell), 1fr));
    gap: 6px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    padding-right: 4px;
    align-content: start;
  }
  .cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    padding: 8px 4px;
    border-radius: 11px;
    border: 1.5px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    transition: border-color 0.12s ease, background 0.12s ease;
  }
  .cell:hover {
    border-color: var(--primary);
    background: var(--primary-soft);
  }
  .cell.copied {
    border-color: var(--green);
    background: color-mix(in srgb, var(--green) 14%, var(--surface));
  }
  .glyph {
    font-size: var(--glyph);
    line-height: 1.15;
    color: var(--ink);
  }
  .code {
    font-size: 10px;
    color: var(--ink-faint);
  }
</style>
