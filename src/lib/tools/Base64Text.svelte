<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("encode");
  let input = $state("");
  let error = $state("");

  function utf8ToB64(s: string): string {
    const bytes = new TextEncoder().encode(s);
    let bin = "";
    for (const b of bytes) bin += String.fromCharCode(b);
    return btoa(bin);
  }
  function b64ToUtf8(s: string): string {
    const bin = atob(s.trim());
    const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  }

  const output = $derived.by(() => {
    error = "";
    if (!input) return "";
    try {
      return mode === "encode" ? utf8ToB64(input) : b64ToUtf8(input);
    } catch (e) {
      error = "Input non valido: " + (e as Error).message;
      return "";
    }
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={mode}
      options={[
        { value: "encode", label: "Testo → Base64" },
        { value: "decode", label: "Base64 → Testo" },
      ]}
    />
  </div>
  <div class="split">
    <Pane label="Input" bind:value={input} placeholder="Scrivi o incolla qui…" />
    <div class="stack grow">
      <Pane label="Output" value={output} readonly showPaste={false} />
      {#if error}<div class="notice bad">{error}</div>{/if}
    </div>
  </div>
</div>
