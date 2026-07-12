<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("beautify");
  let indent = $state("2");
  let input = $state("");

  function indentValue(): string | number {
    if (indent === "tab") return "\t";
    if (indent === "4") return 4;
    return 2;
  }

  const computed = $derived.by((): { output: string; error: string; valid: boolean } => {
    const src = input.trim();
    if (!src) return { output: "", error: "", valid: false };
    try {
      const obj = JSON.parse(input);
      if (mode === "beautify") {
        return { output: JSON.stringify(obj, null, indentValue()), error: "", valid: false };
      } else if (mode === "minify") {
        return { output: JSON.stringify(obj), error: "", valid: false };
      } else {
        return { output: "", error: "", valid: true };
      }
    } catch (e) {
      return { output: "", error: (e as Error).message, valid: false };
    }
  });
  const output = $derived(computed.output);
  const error = $derived(computed.error);
  const valid = $derived(computed.valid);
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={mode}
      options={[
        { value: "beautify", label: "Abbellisci" },
        { value: "minify", label: "Minifica" },
        { value: "validate", label: "Valida" },
      ]}
    />
    {#if mode === "beautify"}
      <Segmented
        bind:value={indent}
        options={[
          { value: "2", label: "2 spazi" },
          { value: "4", label: "4 spazi" },
          { value: "tab", label: "Tab" },
        ]}
      />
    {/if}
  </div>
  <div class="split">
    <Pane label="Input" bind:value={input} placeholder="Incolla qui il JSON…" />
    <div class="stack grow">
      {#if mode === "validate"}
        {#if input.trim() && valid}
          <div class="notice good">✓ JSON valido</div>
        {:else if input.trim() && error}
          <div class="notice bad">{error}</div>
        {/if}
      {:else}
        <Pane label="Output" value={output} readonly showPaste={false} />
        {#if error}<div class="notice bad">{error}</div>{/if}
      {/if}
    </div>
  </div>
</div>
