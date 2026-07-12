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

  async function pickImages() {
    error = "";
    success = "";
    const picked = await open({
      multiple: true,
      filters: [{ name: "Immagini", extensions: ["png", "jpg", "jpeg", "webp"] }],
    });
    if (!picked) return;
    const arr = Array.isArray(picked) ? picked : [picked];
    paths = [...paths, ...arr];
  }

  function remove(i: number) {
    paths = paths.filter((_, idx) => idx !== i);
  }

  async function convert() {
    if (paths.length === 0) return;
    error = "";
    success = "";
    const dest = await save({
      defaultPath: "immagini.pdf",
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!dest) return;
    busy = true;
    try {
      await invoke("images_to_pdf", { paths, dest });
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
    <button class="btn" onclick={pickImages}>🖼️ Scegli immagini</button>
    {#if paths.length > 0}<span class="pill gold">{paths.length} immagini</span>{/if}
    <span class="spacer"></span>
    <button class="btn primary" onclick={convert} disabled={paths.length === 0 || busy}>
      {busy ? "Salvataggio…" : "Salva PDF"}
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
    gap: 10px;
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
