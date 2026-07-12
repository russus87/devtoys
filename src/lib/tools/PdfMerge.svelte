<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  let paths = $state<string[]>([]);
  let busy = $state(false);
  let error = $state("");
  let success = $state("");

  function baseName(p: string): string {
    return p.split(/[\\/]/).pop() ?? p;
  }

  async function pickFiles() {
    error = "";
    success = "";
    const picked = await open({
      multiple: true,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!picked) return;
    const arr = Array.isArray(picked) ? picked : [picked];
    paths = [...paths, ...arr];
  }

  function remove(i: number) {
    paths = paths.filter((_, idx) => idx !== i);
  }

  function moveUp(i: number) {
    if (i <= 0) return;
    const next = paths.slice();
    [next[i - 1], next[i]] = [next[i], next[i - 1]];
    paths = next;
  }

  function moveDown(i: number) {
    if (i >= paths.length - 1) return;
    const next = paths.slice();
    [next[i + 1], next[i]] = [next[i], next[i + 1]];
    paths = next;
  }

  async function merge() {
    if (paths.length < 2) return;
    error = "";
    success = "";
    const dest = await save({
      defaultPath: "unito.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    try {
      await invoke("pdf_merge", { paths, dest });
      success = `Salvato in ${dest}`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="tool">
  <div class="tool-controls">
    <button class="btn" onclick={pickFiles}>📄 Scegli PDF</button>
    {#if paths.length > 0}<span class="pill gold">{paths.length} file</span>{/if}
    <span class="spacer"></span>
    <button class="btn primary" onclick={merge} disabled={paths.length < 2 || busy}>
      {busy ? "Unione…" : "Unisci"}
    </button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}
  {#if success}<div class="notice good">{success}</div>{/if}

  {#if paths.length > 0}
    <div class="stack grow file-list">
      {#each paths as p, i (p + i)}
        <div class="file-row">
          <span class="mono file-idx">{i + 1}.</span>
          <span class="file-name">{baseName(p)}</span>
          <button class="btn sm ghost" onclick={() => moveUp(i)} disabled={i === 0} aria-label="Sposta su">↑</button>
          <button
            class="btn sm ghost"
            onclick={() => moveDown(i)}
            disabled={i === paths.length - 1}
            aria-label="Sposta giù"
          >↓</button>
          <button class="btn sm ghost" onclick={() => remove(i)} aria-label="Rimuovi">✕</button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-list {
    overflow-y: auto;
    padding-right: 4px;
  }
  .file-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 12px;
    border-radius: 10px;
    border: 1.5px solid var(--border);
    background: var(--surface);
  }
  .file-idx {
    color: var(--ink-faint);
    width: 24px;
  }
  .file-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
  }
</style>
