<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("escape");
  let target = $state("json");
  let input = $state("");

  function jsonEscape(s: string): string {
    return JSON.stringify(s).slice(1, -1);
  }
  function jsonUnescape(s: string): string {
    return JSON.parse('"' + s.replace(/\r?\n/g, "\\n") + '"');
  }
  function jsEscape(s: string): string {
    return s
      .replace(/\\/g, "\\\\")
      .replace(/'/g, "\\'")
      .replace(/\n/g, "\\n")
      .replace(/\r/g, "\\r")
      .replace(/\t/g, "\\t");
  }
  function jsUnescape(s: string): string {
    return s
      .replace(/\\n/g, "\n")
      .replace(/\\r/g, "\r")
      .replace(/\\t/g, "\t")
      .replace(/\\'/g, "'")
      .replace(/\\"/g, '"')
      .replace(/\\\\/g, "\\");
  }

  const result = $derived.by((): { output: string; error: string } => {
    if (!input) return { output: "", error: "" };
    try {
      let output: string;
      if (target === "json") {
        output = mode === "escape" ? jsonEscape(input) : jsonUnescape(input);
      } else {
        output = mode === "escape" ? jsEscape(input) : jsUnescape(input);
      }
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
        { value: "escape", label: "Escape" },
        { value: "unescape", label: "Unescape" },
      ]}
    />
    <Segmented
      bind:value={target}
      options={[
        { value: "json", label: "JSON" },
        { value: "js", label: "JavaScript" },
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
