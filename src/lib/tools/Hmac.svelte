<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { api, copy } from "../api";

  let message = $state("");
  let key = $state("");
  let algo = $state("sha256");
  let hmac = $state("");
  let error = $state("");
  let busy = $state(false);
  let copied = $state(false);

  $effect(() => {
    const m = message;
    const k = key;
    const a = algo;
    if (!m || !k) {
      hmac = "";
      error = "";
      return;
    }
    busy = true;
    api
      .hmacText(a, k, m)
      .then((r) => {
        hmac = r;
        error = "";
      })
      .catch((e: unknown) => {
        hmac = "";
        error = "Calcolo HMAC fallito: " + (e as Error).message;
      })
      .finally(() => {
        busy = false;
      });
  });

  async function copyHmac() {
    if (!hmac) return;
    await copy(hmac);
    copied = true;
    setTimeout(() => (copied = false), 1100);
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <Segmented
      bind:value={algo}
      options={[
        { value: "sha1", label: "SHA-1" },
        { value: "sha256", label: "SHA-256" },
        { value: "sha384", label: "SHA-384" },
        { value: "sha512", label: "SHA-512" },
      ]}
    />
    <div class="row grow">
      <span class="field-label">Chiave</span>
      <input class="input" type="text" bind:value={key} placeholder="Chiave segreta…" />
    </div>
  </div>

  <div class="split">
    <Pane label="Messaggio" bind:value={message} placeholder="Scrivi o incolla qui…" rows={10} />
    <div class="stack grow">
      <div class="row">
        <span class="field-label">HMAC {busy ? "…" : ""}</span>
        <div class="spacer"></div>
        <button class="btn sm" onclick={copyHmac} disabled={!hmac}>{copied ? "✓ Copiato" : "Copia"}</button>
      </div>
      <div class="card-box mono hmac-out">{hmac || "—"}</div>
      {#if error}<div class="notice bad">{error}</div>{/if}
    </div>
  </div>
</div>

<style>
  .hmac-out {
    word-break: break-all;
    font-size: 13px;
    min-height: 48px;
  }
</style>
