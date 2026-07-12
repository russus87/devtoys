<script lang="ts">
  import QRCode from "qrcode";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let input = $state("");
  let level = $state("M");
  let dataUrl = $state("");
  let error = $state("");

  $effect(() => {
    const text = input;
    const ec = level as "L" | "M" | "Q" | "H";
    if (!text) {
      dataUrl = "";
      error = "";
      return;
    }
    QRCode.toDataURL(text, {
      errorCorrectionLevel: ec,
      margin: 2,
      width: 512,
      color: { dark: "#2c2a47ff", light: "#ffffffff" },
    })
      .then((url: string) => {
        dataUrl = url;
        error = "";
      })
      .catch((e: Error) => {
        error = e.message;
        dataUrl = "";
      });
  });

  function download() {
    if (!dataUrl) return;
    const a = document.createElement("a");
    a.href = dataUrl;
    a.download = "qrcode.png";
    a.click();
  }
</script>

<div class="tool">
  <div class="tool-controls">
    <span class="field-label">Correzione errori</span>
    <Segmented
      bind:value={level}
      options={[
        { value: "L", label: "L" },
        { value: "M", label: "M" },
        { value: "Q", label: "Q" },
        { value: "H", label: "H" },
      ]}
    />
  </div>
  <div class="split">
    <Pane label="Testo o URL" bind:value={input} placeholder="https://esempio.it" />
    <div class="stack grow">
      <span class="field-label">QR code</span>
      <div class="qrbox">
        {#if dataUrl}
          <img src={dataUrl} alt="QR code" />
        {:else}
          <span class="qr-empty">Il QR apparirà qui</span>
        {/if}
      </div>
      {#if error}<div class="notice bad">{error}</div>{/if}
      <button class="btn primary" onclick={download} disabled={!dataUrl}>⬇ Scarica PNG</button>
    </div>
  </div>
</div>

<style>
  .qrbox {
    flex: 1;
    display: grid;
    place-items: center;
    background: var(--surface-2);
    border: 1.5px solid var(--border-2);
    border-radius: var(--radius);
    padding: 20px;
    min-height: 220px;
  }
  .qrbox img {
    width: 240px;
    height: 240px;
    image-rendering: pixelated;
    border-radius: 10px;
    background: #fff;
  }
  .qr-empty {
    color: var(--ink-faint);
    font-size: 13px;
  }
</style>
