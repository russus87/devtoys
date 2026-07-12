<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { copy } from "../api";

  interface MetadataResult {
    title: string | null;
    author: string | null;
    subject: string | null;
    keywords: string | null;
    creator: string | null;
    producer: string | null;
    lang: string | null;
    pages: number;
  }

  interface Row {
    label: string;
    value: string;
  }

  let path = $state<string | null>(null);
  let busy = $state(false);
  let error = $state("");
  let meta = $state<MetadataResult | null>(null);
  let copiedIdx = $state<number | null>(null);

  function fileName(p: string | null): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] ?? p;
  }

  async function load() {
    if (!path) return;
    busy = true;
    error = "";
    meta = null;
    try {
      meta = await invoke<MetadataResult>("pdf_metadata", { path });
    } catch (e) {
      error = String(e);
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

  const rows = $derived.by((): Row[] => {
    if (!meta) return [];
    return [
      { label: "Titolo", value: meta.title ?? "—" },
      { label: "Autore", value: meta.author ?? "—" },
      { label: "Oggetto", value: meta.subject ?? "—" },
      { label: "Parole chiave", value: meta.keywords ?? "—" },
      { label: "Creatore", value: meta.creator ?? "—" },
      { label: "Producer", value: meta.producer ?? "—" },
      { label: "Lingua", value: meta.lang ?? "—" },
      { label: "Pagine", value: String(meta.pages) },
    ];
  });

  async function doCopy(row: Row, i: number) {
    if (row.value === "—") return;
    await copy(row.value);
    copiedIdx = i;
    setTimeout(() => {
      if (copiedIdx === i) copiedIdx = null;
    }, 1100);
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <button class="btn" onclick={pick} disabled={busy}>Scegli PDF</button>
    <span class="mono faint">{path ? fileName(path) : "nessun file"}</span>
    <span class="spacer"></span>
    <button class="btn ghost sm" onclick={load} disabled={!path || busy}>↻ Ricarica</button>
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  {#if meta}
    <div class="card-box">
      <div class="meta-table">
        {#each rows as row, i (row.label)}
          <div class="meta-row">
            <span class="field-label">{row.label}</span>
            <span class="meta-value mono">{row.value}</span>
            <button
              class="btn ghost sm"
              class:ok={copiedIdx === i}
              onclick={() => doCopy(row, i)}
              disabled={row.value === "—"}
              aria-label="Copia {row.label}"
            >
              {copiedIdx === i ? "✓" : "⧉"}
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .meta-table {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .meta-row {
    display: grid;
    grid-template-columns: 140px 1fr auto;
    align-items: center;
    gap: 12px;
    padding-bottom: 10px;
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
  .btn.ghost.ok {
    color: var(--green);
  }
</style>
