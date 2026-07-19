<script lang="ts">
  import { parse as parseToml, stringify as stringifyToml } from "smol-toml";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("toml2json");
  let input = $state("");

  const computed = $derived.by(() => {
    if (!input.trim()) return { output: "", error: "" };
    try {
      if (mode === "toml2json") {
        return { output: JSON.stringify(parseToml(input), null, 2), error: "" };
      }
      const obj = JSON.parse(input);
      if (obj === null || typeof obj !== "object" || Array.isArray(obj)) {
        return { output: "", error: "TOML richiede un oggetto JSON alla radice (non un array o un valore semplice)." };
      }
      return { output: stringifyToml(obj), error: "" };
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
      bind:value={mode}
      options={[
        { value: "toml2json", label: "TOML → JSON" },
        { value: "json2toml", label: "JSON → TOML" },
      ]}
    />
  </div>
  <div class="split">
    <Pane
      label={mode === "toml2json" ? "TOML" : "JSON (oggetto)"}
      bind:value={input}
      placeholder={mode === "toml2json"
        ? 'titolo = "Config"\n[server]\nhost = "localhost"\nporta = 8080'
        : '{\n  "titolo": "Config",\n  "server": { "host": "localhost", "porta": 8080 }\n}'}
    />
    <div class="stack grow">
      <Pane label="Output" value={output} readonly showPaste={false} />
      {#if error}<div class="notice bad">{error}</div>{/if}
    </div>
  </div>
</div>
