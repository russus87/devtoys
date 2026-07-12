<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { copy } from "../api";

  let mode = $state("encode");

  // --- Encode (Immagine -> Base64) ---
  let dataUri = $state("");
  let fileName = $state("");
  let encodeError = $state("");

  function onFileChange(e: Event) {
    encodeError = "";
    const input = e.target as HTMLInputElement;
    const file: File | undefined = input.files?.[0];
    if (!file) return;
    fileName = file.name;
    const reader: FileReader = new FileReader();
    reader.onload = () => {
      dataUri = typeof reader.result === "string" ? reader.result : "";
    };
    reader.onerror = () => {
      encodeError = "Impossibile leggere il file";
      dataUri = "";
    };
    reader.readAsDataURL(file);
  }

  async function copyDataUri() {
    if (dataUri) await copy(dataUri);
  }

  // --- Decode (Base64 -> Immagine) ---
  let decodeInput = $state("");
  let decodeError = $state("");

  const decodeSrc = $derived.by(() => {
    decodeError = "";
    const v = decodeInput.trim();
    if (!v) return "";
    return v.startsWith("data:") ? v : "data:image/png;base64," + v;
  });

  function onImgError() {
    if (decodeSrc) decodeError = "Base64 non valido";
  }
  function onImgLoad() {
    decodeError = "";
  }
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={mode}
      options={[
        { value: "encode", label: "Immagine → Base64" },
        { value: "decode", label: "Base64 → Immagine" },
      ]}
    />
  </div>

  {#if mode === "encode"}
    <div class="split">
      <div class="stack">
        <span class="field-label">Immagine</span>
        <label class="btn file-btn">
          Scegli immagine…
          <input type="file" accept="image/*" onchange={onFileChange} />
        </label>
        {#if fileName}<span class="faint">{fileName}</span>{/if}
        {#if encodeError}<div class="notice bad">{encodeError}</div>{/if}
        {#if dataUri}
          <div class="preview-box">
            <img src={dataUri} alt="Anteprima" />
          </div>
          <button class="btn" onclick={copyDataUri}>Copia data URI</button>
        {/if}
      </div>
      <div class="stack grow">
        <Pane label="Output (data URI)" value={dataUri} readonly showPaste={false} rows={10} />
      </div>
    </div>
  {:else}
    <div class="split">
      <Pane
        label="Base64 o data URI"
        bind:value={decodeInput}
        placeholder="Incolla qui il base64 o la data URI…"
        rows={10}
      />
      <div class="stack grow">
        <span class="field-label">Anteprima</span>
        {#if decodeError}
          <div class="notice bad">{decodeError}</div>
        {:else if decodeSrc}
          <div class="preview-box">
            <img src={decodeSrc} alt="Anteprima" onerror={onImgError} onload={onImgLoad} />
          </div>
        {:else}
          <div class="preview-box empty">
            <span class="muted">Nessun input</span>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .file-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    width: fit-content;
  }
  .file-btn input[type="file"] {
    position: absolute;
    width: 1px;
    height: 1px;
    opacity: 0;
    overflow: hidden;
    pointer-events: none;
  }
  .preview-box {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 220px;
    max-height: 420px;
    padding: 16px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border-2);
    background-color: var(--surface-2);
    background-image:
      linear-gradient(45deg, var(--surface-3) 25%, transparent 25%),
      linear-gradient(-45deg, var(--surface-3) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, var(--surface-3) 75%),
      linear-gradient(-45deg, transparent 75%, var(--surface-3) 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    overflow: auto;
  }
  .preview-box.empty {
    background-image: none;
  }
  .preview-box img {
    max-width: 100%;
    max-height: 380px;
    object-fit: contain;
    border-radius: 8px;
  }
</style>
