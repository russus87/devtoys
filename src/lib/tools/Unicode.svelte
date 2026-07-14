<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("encode");
  let format = $state("std");
  let input = $state("");

  // Codifica ogni code unit UTF-16 (come Phoenix, che itera sui char).
  function encode(s: string, fmt: string): string {
    const prefix = fmt === "phoenix" ? "/U" : "\\u";
    let out = "";
    for (let i = 0; i < s.length; i++) {
      let hex = s.charCodeAt(i).toString(16).padStart(4, "0");
      hex = fmt === "phoenix" ? hex.toUpperCase() : hex;
      out += prefix + hex;
    }
    return out;
  }

  // Decodifica sia \uXXXX (standard) sia /UXXXX (Phoenix), maiusc./minusc.
  function decode(s: string): string {
    return s.replace(/[\\/][uU]([0-9a-fA-F]{4})/g, (_, h) =>
      String.fromCharCode(parseInt(h, 16)),
    );
  }

  const result = $derived.by((): { output: string; error: string } => {
    if (!input) return { output: "", error: "" };
    try {
      const output = mode === "encode" ? encode(input, format) : decode(input);
      return { output, error: "" };
    } catch (e) {
      return { output: "", error: "Errore: " + (e as Error).message };
    }
  });
</script>

<div class="tool">
  <div class="tool-controls row wrap">
    <Segmented
      bind:value={mode}
      options={[
        { value: "encode", label: "Testo → Unicode" },
        { value: "decode", label: "Unicode → Testo" },
      ]}
    />
    <Segmented
      bind:value={format}
      options={[
        { value: "std", label: "Standard \\uXXXX" },
        { value: "phoenix", label: "Phoenix /UXXXX" },
      ]}
    />
  </div>
  <div class="split">
    <Pane label="Input" bind:value={input} placeholder="Scrivi o incolla qui…" />
    <div class="stack grow">
      <Pane label="Output" value={result.output} readonly showPaste={false} />
      {#if result.error}<div class="notice bad">{result.error}</div>{/if}
    </div>
  </div>
</div>
