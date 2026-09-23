<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import Segmented from "../ui/Segmented.svelte";

  interface ConvertReport {
    from: string;
    to: string;
    glyphs: number;
    before: number;
    after: number;
    notes: string[];
  }

  interface AfmReport {
    fontName: string;
    glyphs: number;
    encoded: number;
    kernPairs: number;
    bytes: number;
    preview: string;
  }

  let path = $state<string | null>(null);
  let target = $state("otf");
  let busy = $state(false);
  let error = $state("");
  let convert = $state<ConvertReport | null>(null);
  let afm = $state<AfmReport | null>(null);
  let destName = $state("");

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

  function reset() {
    convert = null;
    afm = null;
    error = "";
  }

  async function pick() {
    const p = await open({
      filters: [{ name: "Font", extensions: ["ttf", "otf", "ttc", "otc", "woff", "woff2"] }],
    });
    if (typeof p === "string") {
      path = p;
      reset();
      // Suggest the conversion that makes sense for the file picked.
      if (/\.otf$/i.test(p)) target = "ttf";
      else if (/\.ttf$/i.test(p)) target = "otf";
    }
  }

  async function run() {
    if (!path) return;
    const base = fileName(path).replace(/\.[^.]+$/, "");
    const dest = await save({
      defaultPath: `${base}.${target}`,
      filters: [
        target === "afm"
          ? { name: "Adobe Font Metrics", extensions: ["afm"] }
          : { name: "Font", extensions: [target] },
      ],
    });
    if (typeof dest !== "string") return;

    busy = true;
    reset();
    try {
      if (target === "afm") {
        afm = await invoke<AfmReport>("font_afm", { path, index: 0, dest });
      } else {
        convert = await invoke<ConvertReport>("font_convert", { path, index: 0, dest, target });
      }
      destName = fileName(dest);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli font</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
    <span class="spacer"></span>
    <Segmented
      options={[
        { value: "otf", label: "→ OTF" },
        { value: "ttf", label: "→ TTF" },
        { value: "afm", label: "→ AFM" },
      ]}
      bind:value={target}
    />
    <button class="btn primary" onclick={run} disabled={!path || busy}>
      {busy ? "Converto…" : "🔁 Converti e salva"}
    </button>
  </div>

  <div class="notice">
    {#if target === "otf"}
      <b>OTF</b>: contorni TrueType (quadratici) riscritti come CFF (cubici). La conversione
      delle curve è esatta; l'hinting TrueType non ha equivalente e viene tolto.
    {:else if target === "ttf"}
      <b>TTF</b>: contorni CFF (cubici) approssimati con curve quadratiche, entro 1/1000 di em.
      I nomi dei glifi passano nella tabella <span class="mono">post</span>.
    {:else}
      <b>AFM</b>: le metriche Adobe (larghezze, riquadri, kerning) scalate a 1000 unità, per chi
      usa il font con strumenti PostScript, TeX o generatori PDF. Il font resta com'è.
    {/if}
    Tabelle di layout (GSUB, GPOS, GDEF), mappa caratteri e nomi sono copiati identici.
    Da un file TTC si converte il primo font della collezione.
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if convert}
    <div class="card-box">
      <div class="row wrap">
        <span class="pill gold">{convert.from} → {convert.to}</span>
        <span class="pill">{kb(convert.before)} → {kb(convert.after)}</span>
        <span class="pill">{convert.glyphs} glifi</span>
        <span class="spacer"></span>
        <span class="mono faint">{destName}</span>
      </div>
      <ul class="notes">
        {#each convert.notes as n (n)}
          <li>{n}</li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if afm}
    <div class="card-box afm">
      <div class="row wrap">
        <span class="pill gold">{afm.fontName}</span>
        <span class="pill">{afm.glyphs} glifi</span>
        <span class="pill">{afm.encoded} con codice 32–255</span>
        <span class="pill">{afm.kernPairs} coppie di kerning</span>
        <span class="pill">{kb(afm.bytes)}</span>
        <span class="spacer"></span>
        <span class="mono faint">{destName}</span>
      </div>
      <pre class="input code preview">{afm.preview}</pre>
      <div class="faint small">
        Codici carattere: i glifi raggiunti da un code point fra 32 e 255 (Latin-1) hanno quel
        codice, tutti gli altri <span class="mono">C -1</span>. Il kerning viene dalla feature
        <span class="mono">kern</span> di GPOS, oppure dalla vecchia tabella
        <span class="mono">kern</span>.
      </div>
    </div>
  {/if}
</div>

<style>
  .notes {
    margin: 12px 0 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .afm {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
    min-height: 0;
  }
  .preview {
    flex: 1;
    min-height: 160px;
    margin: 0;
    overflow: auto;
    white-space: pre;
  }
  .small {
    font-size: 12px;
  }
</style>
