<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("indent");
  let indentSize = $state("2");
  let input = $state("");

  function isValidXml(xml: string): boolean {
    const doc = new DOMParser().parseFromString(xml, "application/xml");
    return doc.querySelector("parsererror") === null;
  }

  function formatXml(xml: string, pad: string): string {
    const reg = /(>)(<)(\/*)/g;
    const xml2 = xml.replace(reg, "$1\n$2$3");
    let indent = 0;
    return xml2
      .split("\n")
      .map((node) => {
        let out = "";
        if (/^<\/\w/.test(node)) indent = Math.max(indent - 1, 0);
        out = pad.repeat(indent) + node;
        if (/^<\w[^>]*[^/]>.*$/.test(node) && !/^<\w[^>]*>.*<\/\w/.test(node)) indent++;
        return out;
      })
      .join("\n")
      .trim();
  }

  const computed = $derived.by(() => {
    const src = input.trim();
    if (!src) return { output: "", error: "" };
    if (!isValidXml(src)) {
      return { output: "", error: "XML non valido" };
    }
    try {
      if (mode === "indent") {
        const pad = indentSize === "4" ? "    " : "  ";
        return { output: formatXml(src, pad), error: "" };
      } else {
        return { output: src.replace(/>\s+</g, "><").trim(), error: "" };
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
      bind:value={mode}
      options={[
        { value: "indent", label: "Indenta" },
        { value: "minify", label: "Minifica" },
      ]}
    />
    {#if mode === "indent"}
      <Segmented
        bind:value={indentSize}
        options={[
          { value: "2", label: "2 spazi" },
          { value: "4", label: "4 spazi" },
        ]}
      />
    {/if}
  </div>
  <div class="split">
    <Pane label="Input" bind:value={input} placeholder="Incolla qui l'XML…" />
    <div class="stack grow">
      <Pane label="Output" value={output} readonly showPaste={false} />
      {#if error}<div class="notice bad">{error}</div>{/if}
    </div>
  </div>
</div>
