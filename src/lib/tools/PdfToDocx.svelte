<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  let path = $state("");
  let fileName = $state("");
  let busy = $state(false);
  let error = $state("");
  let success = $state("");

  function baseName(p: string): string {
    return p.split(/[\\/]/).pop() ?? p;
  }

  async function pickFile() {
    error = "";
    success = "";
    const picked = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (!picked || Array.isArray(picked)) return;
    path = picked;
    fileName = baseName(picked);
  }

  async function convert() {
    if (!path) return;
    error = "";
    success = "";
    const dest = await save({
      defaultPath: "documento.docx",
      filters: [{ name: "Word", extensions: ["docx"] }],
    });
    if (!dest) return;
    busy = true;
    try {
      await invoke("pdf_to_docx", { path, dest });
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
    <button class="btn" onclick={pickFile}>📄 Scegli PDF</button>
    {#if fileName}<span class="pill">{fileName}</span>{/if}
  </div>

  <div class="notice">
    Beta: estrae il testo, l'impaginazione non è fedele.
  </div>

  <div class="tool-controls">
    <button class="btn primary" onclick={convert} disabled={!path || busy}>
      {busy ? "Conversione…" : "Converti in Word"}
    </button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}
  {#if success}<div class="notice good">{success}</div>{/if}
</div>
