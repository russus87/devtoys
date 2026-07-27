<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import Segmented from "../ui/Segmented.svelte";

  interface SubsetReport {
    before: number;
    after: number;
    glyphsBefore: number;
    glyphsAfter: number;
    missing: string;
  }

  /// Ready-made character sets, because typing out an alphabet by hand is the
  /// part of subsetting everyone gets wrong.
  const PRESETS: { label: string; chars: string }[] = [
    {
      label: "Italiano",
      chars:
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789àèéìòùÀÈÉÌÒÙ .,;:!?'\"()[]{}«»–—…€%&@#/\\+-*=<>",
    },
    {
      label: "Latino base",
      chars:
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 .,;:!?'\"()[]{}@#$%&*+-/=<>_|~^`\\",
    },
    { label: "Solo cifre", chars: "0123456789.,-+ €$%" },
  ];

  let path = $state<string | null>(null);
  let text = $state(PRESETS[0].chars);
  let profile = $state("web");
  let busy = $state(false);
  let error = $state("");
  let report = $state<SubsetReport | null>(null);
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

  async function pick() {
    const p = await open({
      filters: [{ name: "Font", extensions: ["ttf", "otf", "ttc", "otc", "woff", "woff2"] }],
    });
    if (typeof p === "string") {
      path = p;
      report = null;
      error = "";
    }
  }

  /// Distinct characters actually requested — the number that drives the result.
  const uniqueChars = $derived(new Set([...text].filter((c) => c !== "\n")).size);

  const savedPct = $derived(
    report && report.before > 0
      ? Math.max(0, Math.round((1 - report.after / report.before) * 100))
      : 0,
  );

  async function run() {
    if (!path || uniqueChars === 0) return;
    const base = fileName(path).replace(/\.[^.]+$/, "");
    const ext = profile === "pdf" ? "otf" : "ttf";
    const dest = await save({
      defaultPath: `${base}-subset.${ext}`,
      filters: [{ name: "Font", extensions: ["ttf", "otf"] }],
    });
    if (typeof dest !== "string") return;

    busy = true;
    error = "";
    report = null;
    try {
      report = await invoke<SubsetReport>("font_subset", {
        path,
        index: 0,
        text,
        dest,
        profile,
      });
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
        { value: "web", label: "Web" },
        { value: "pdf", label: "PDF" },
      ]}
      bind:value={profile}
    />
    <button class="btn primary" onclick={run} disabled={!path || busy || uniqueChars === 0}>
      {busy ? "Riduco…" : "✂️ Riduci e salva"}
    </button>
  </div>

  <div class="notice">
    {#if profile === "web"}
      Profilo <b>Web</b>: tabelle minime e mappa caratteri Unicode, quello che serve a un
      <span class="mono">@font-face</span> nel browser.
    {:else}
      Profilo <b>PDF</b>: il set di tabelle che si aspetta chi incorpora il font in un documento.
    {/if}
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  <div class="card-box chars">
    <div class="row wrap head">
      <span class="field-label">Caratteri da tenere</span>
      <span class="pill">{uniqueChars} distinti</span>
      <span class="spacer"></span>
      {#each PRESETS as p (p.label)}
        <button class="btn ghost sm" onclick={() => (text = p.chars)} disabled={busy}>
          {p.label}
        </button>
      {/each}
    </div>
    <textarea
      class="input code area"
      bind:value={text}
      spellcheck="false"
      placeholder="Incolla qui il testo o i caratteri da conservare…"
    ></textarea>
    <div class="faint small">
      Puoi incollare direttamente il testo del tuo sito o documento: contano i caratteri
      distinti, non la lunghezza.
    </div>
  </div>

  {#if report}
    <div class="card-box">
      <div class="row wrap">
        <span class="pill gold">Risparmio: {savedPct}%</span>
        <span class="pill">{kb(report.before)} → {kb(report.after)}</span>
        <span class="pill">{report.glyphsBefore} → {report.glyphsAfter} glifi</span>
        <span class="spacer"></span>
        <span class="mono faint">{destName}</span>
      </div>
      <div class="bar">
        <span class="fill" style="width: {(report.after / report.before) * 100}%"></span>
      </div>
      {#if report.glyphsAfter > uniqueChars + 1}
        <div class="faint small">
          I glifi finali sono più dei caratteri richiesti perché quelli compositi (le lettere
          accentate, per esempio) trascinano con sé i loro componenti.
        </div>
      {/if}
      {#if report.missing}
        <div class="notice bad missing">
          Caratteri assenti dal font e quindi non inclusi:
          <span class="mono">{report.missing}</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .chars {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
    min-height: 0;
  }
  .head {
    gap: 8px;
  }
  .area {
    flex: 1;
    min-height: 120px;
    resize: none;
  }
  .small {
    font-size: 12px;
  }
  .bar {
    display: block;
    height: 9px;
    margin: 12px 0;
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
  .missing {
    margin-top: 10px;
    word-break: break-all;
  }
</style>
