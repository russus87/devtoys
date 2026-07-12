<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  interface PdfaCheck {
    label: string;
    ok: boolean;
  }
  interface PdfaResult {
    conformant: boolean;
    note: string;
    checks: PdfaCheck[];
  }

  let path = $state<string | null>(null);
  let busy = $state(false);
  let error = $state("");
  let result = $state<PdfaResult | null>(null);

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  async function pick() {
    const p = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (typeof p === "string") {
      path = p;
      result = null;
      error = "";
    }
  }

  async function run() {
    if (!path) return;
    busy = true;
    error = "";
    result = null;
    try {
      result = await invoke<PdfaResult>("pdf_check_pdfa", { path });
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli PDF</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
    <span class="spacer"></span>
    <button class="btn primary" onclick={run} disabled={!path || busy}>
      {busy ? "Verifica…" : "Verifica PDF/A"}
    </button>
  </div>

  <div class="notice">
    Verifica di base (non certificata): controlla alcuni requisiti tipici dello standard PDF/A. Per una conformità
    ufficiale è necessario uno strumento dedicato come veraPDF.
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if result}
    <div class="banner" class:ok={result.conformant} class:fail={!result.conformant}>
      {result.conformant ? "✓ Conforme (verifica di base)" : "✗ Non conforme (verifica di base)"}
    </div>
    <div class="notice">{result.note}</div>

    <div class="checks">
      {#each result.checks as c (c.label)}
        <div class="check-row">
          <span class="badge" class:ok={c.ok} class:fail={!c.ok}>{c.ok ? "✓" : "✗"}</span>
          <span class="check-label">{c.label}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .banner {
    padding: 12px 16px;
    border-radius: var(--radius-lg);
    font-weight: 700;
    font-size: 14px;
    border: 1.5px solid var(--border-2);
  }
  .banner.ok {
    background: color-mix(in srgb, var(--green) 12%, var(--surface));
    border-color: color-mix(in srgb, var(--green) 32%, var(--border));
    color: var(--green);
  }
  .banner.fail {
    background: color-mix(in srgb, var(--red) 12%, var(--surface));
    border-color: color-mix(in srgb, var(--red) 32%, var(--border));
    color: var(--red);
  }
  .checks {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow: auto;
  }
  .check-row {
    display: grid;
    grid-template-columns: 28px 1fr;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius);
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
  }
  .badge.ok {
    background: color-mix(in srgb, var(--green) 20%, var(--surface));
    color: var(--green);
  }
  .badge.fail {
    background: color-mix(in srgb, var(--red) 20%, var(--surface));
    color: var(--red);
  }
  .check-label {
    font-size: 13px;
  }
</style>
