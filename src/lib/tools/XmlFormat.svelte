<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("indent");
  let indentSize = $state("2");
  let input = $state("");
  let error = $state("");

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

  const output = $derived.by(() => {
    error = "";
    const src = input.trim();
    if (!src) return "";
    if (!isValidXml(src)) {
      error = "XML non valido";
      return "";
    }
    try {
      if (mode === "indent") {
        const pad = indentSize === "4" ? "    " : "  ";
        return formatXml(src, pad);
      } else {
        return src.replace(/>\s+</g, "><").trim();
      }
    } catch (e) {
      error = (e as Error).message;
      return "";
    }
  });
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
