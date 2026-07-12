<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  interface Check {
    id: string;
    description: string;
    passed: boolean;
  }
  interface UaResult {
    total: number;
    passed: number;
    failed: number;
    checks: Check[];
  }

  let path = $state<string | null>(null);
  let busy = $state(false);
  let error = $state("");
  let result = $state<UaResult | null>(null);

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
      result = await invoke<UaResult>("pdf_validate_ua", { path });
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
      {busy ? "Verifica…" : "Verifica PDF/UA"}
    </button>
  </div>

  <div class="notice">
    Controllo di accessibilità basato sulle regole Matterhorn Protocol (verifica euristica, non certificazione ufficiale).
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if result}
    <div class="card-box">
      <div class="row wrap">
        <span class="pill">Totali: {result.total}</span>
        <span class="pill ok">Superati: {result.passed}</span>
        <span class="pill fail">Falliti: {result.failed}</span>
      </div>
    </div>

    <div class="checks">
      {#each result.checks as c (c.id)}
        <div class="check-row">
          <span class="badge" class:ok={c.passed} class:fail={!c.passed}>{c.passed ? "✓" : "✗"}</span>
          <span class="check-id mono">{c.id}</span>
          <span class="check-desc">{c.description}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .pill.ok {
    background: color-mix(in srgb, var(--green) 16%, var(--surface-3));
    color: var(--green);
  }
  .pill.fail {
    background: color-mix(in srgb, var(--red) 16%, var(--surface-3));
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
    grid-template-columns: 28px 140px 1fr;
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
  .check-id {
    font-size: 12px;
    color: var(--ink-dim);
  }
  .check-desc {
    font-size: 13px;
  }
</style>
