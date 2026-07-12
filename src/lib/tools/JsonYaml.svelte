<script lang="ts">
  import yaml from "js-yaml";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let dir = $state("j2y");
  let input = $state("");

  const computed = $derived.by(() => {
    if (!input.trim()) return { output: "", error: "" };
    try {
      if (dir === "j2y") {
        const obj = JSON.parse(input);
        return { output: yaml.dump(obj, { indent: 2, lineWidth: -1 }), error: "" };
      } else {
        const obj = yaml.load(input);
        return { output: JSON.stringify(obj, null, 2), error: "" };
      }
    } catch (e) {
      return { output: "", error: (e as Error).message };
    }
  });
  const output = $derived(computed.output);
  const error = $derived(computed.error);
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={dir}
      options={[
        { value: "j2y", label: "JSON → YAML" },
        { value: "y2j", label: "YAML → JSON" },
      ]}
    />
  </div>
  <div class="split">
    <Pane label={dir === "j2y" ? "JSON" : "YAML"} bind:value={input} placeholder="Incolla qui…" />
    <div class="stack grow">
      <Pane label={dir === "j2y" ? "YAML" : "JSON"} value={output} readonly showPaste={false} />
      {#if error}<div class="notice bad">{error}</div>{/if}
    </div>
  </div>
</div>
