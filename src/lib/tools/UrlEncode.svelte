<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("encode");
  let input = $state("");
  let error = $state("");

  const output = $derived.by(() => {
    error = "";
    if (!input) return "";
    try {
      return mode === "encode" ? encodeURIComponent(input) : decodeURIComponent(input);
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
        { value: "encode", label: "Codifica" },
        { value: "decode", label: "Decodifica" },
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
