<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("toRoman");
  let input = $state("");

  const TABLE: [number, string][] = [
    [1000, "M"], [900, "CM"], [500, "D"], [400, "CD"],
    [100, "C"], [90, "XC"], [50, "L"], [40, "XL"],
    [10, "X"], [9, "IX"], [5, "V"], [4, "IV"], [1, "I"],
  ];

  function toRoman(n: number): string {
    let out = "";
    for (const [v, sym] of TABLE) {
      while (n >= v) { out += sym; n -= v; }
    }
    return out;
  }

  function fromRoman(s: string): number {
    const map: Record<string, number> = { I: 1, V: 5, X: 10, L: 50, C: 100, D: 500, M: 1000 };
    let total = 0;
    for (let i = 0; i < s.length; i++) {
      const cur = map[s[i]];
      const next = map[s[i + 1]];
      if (next && cur < next) { total -= cur; } else { total += cur; }
    }
    return total;
  }

  const computed = $derived.by(() => {
    const s = input.trim();
    if (!s) return { output: "", error: "" };
    if (mode === "toRoman") {
      const n = Number(s);
      if (!Number.isInteger(n) || n < 1 || n > 3999) {
        return { output: "", error: "Inserisci un intero tra 1 e 3999." };
      }
      return { output: toRoman(n), error: "" };
    } else {
      const up = s.toUpperCase();
      if (!/^[IVXLCDM]+$/.test(up)) {
        return { output: "", error: "Numero romano non valido (usa I V X L C D M)." };
      }
      const n = fromRoman(up);
      // Round-trip check catches malformed sequences like "IIII" or "VX".
      if (toRoman(n) !== up) {
        return { output: "", error: "Sequenza romana non canonica." };
      }
      return { output: String(n), error: "" };
    }
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={mode}
      options={[
        { value: "toRoman", label: "Numero → Romano" },
        { value: "fromRoman", label: "Romano → Numero" },
      ]}
    />
  </div>
  <div class="split">
    <Pane
      label={mode === "toRoman" ? "Numero (1–3999)" : "Numero romano"}
      bind:value={input}
      placeholder={mode === "toRoman" ? "2024" : "MMXXIV"}
      rows={3}
    />
    <div class="stack grow">
      <Pane label="Risultato" value={computed.output} readonly showPaste={false} rows={3} />
      {#if computed.error}<div class="notice bad">{computed.error}</div>{/if}
    </div>
  </div>
</div>
