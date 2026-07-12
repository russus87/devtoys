<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("escape");
  let input = $state("");
  let error = $state("");

  function escapeHtml(s: string): string {
    return s
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
  }

  function unescapeHtml(s: string): string {
    return new DOMParser().parseFromString(s, "text/html").documentElement.textContent ?? "";
  }

  const output = $derived.by(() => {
    error = "";
    if (!input) return "";
    try {
      return mode === "escape" ? escapeHtml(input) : unescapeHtml(input);
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
        { value: "escape", label: "Escape" },
        { value: "unescape", label: "Unescape" },
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
