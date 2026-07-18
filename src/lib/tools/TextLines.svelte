<script lang="ts">
  import Pane from "../ui/Pane.svelte";

  let input = $state("");
  let trim = $state(true);
  let removeEmpty = $state(false);
  let dedup = $state(false);
  let numbering = $state(false);
  let reverse = $state(false);
  let sort = $state("none"); // none | az | za | len | natural

  const collator = new Intl.Collator("it", { numeric: true, sensitivity: "base" });

  const output = $derived.by(() => {
    if (!input) return "";
    let lines = input.split(/\r?\n/);
    if (trim) lines = lines.map((l) => l.trim());
    if (removeEmpty) lines = lines.filter((l) => l.length > 0);
    if (dedup) {
      const seen = new Set<string>();
      lines = lines.filter((l) => (seen.has(l) ? false : (seen.add(l), true)));
    }
    if (sort === "az") lines.sort((a, b) => a.localeCompare(b, "it"));
    else if (sort === "za") lines.sort((a, b) => b.localeCompare(a, "it"));
    else if (sort === "len") lines.sort((a, b) => a.length - b.length);
    else if (sort === "natural") lines.sort((a, b) => collator.compare(a, b));
    if (reverse) lines.reverse();
    if (numbering) {
      const w = String(lines.length).length;
      lines = lines.map((l, i) => `${String(i + 1).padStart(w, " ")}  ${l}`);
    }
    return lines.join("\n");
  });

  const stats = $derived.by(() => {
    const inLines = input ? input.split(/\r?\n/).length : 0;
    const outLines = output ? output.split(/\r?\n/).length : 0;
    return { inLines, outLines };
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <label class="chk"><input type="checkbox" bind:checked={trim} /> Trim</label>
    <label class="chk"><input type="checkbox" bind:checked={removeEmpty} /> Righe vuote</label>
    <label class="chk"><input type="checkbox" bind:checked={dedup} /> Deduplica</label>
    <label class="chk"><input type="checkbox" bind:checked={reverse} /> Inverti ordine</label>
    <label class="chk"><input type="checkbox" bind:checked={numbering} /> Numera</label>
    <select class="input sel" bind:value={sort}>
      <option value="none">Ordine originale</option>
      <option value="az">Ordina A → Z</option>
      <option value="za">Ordina Z → A</option>
      <option value="natural">Ordine naturale (1,2,10)</option>
      <option value="len">Per lunghezza</option>
    </select>
  </div>

  <div class="split">
    <Pane label="Input ({stats.inLines} righe)" bind:value={input} placeholder="Una voce per riga…" />
    <Pane label="Output ({stats.outLines} righe)" value={output} readonly showPaste={false} />
  </div>
</div>

<style>
  .chk {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    font-weight: 600;
    color: var(--ink-dim);
    cursor: pointer;
    user-select: none;
  }
  .chk input {
    width: 15px;
    height: 15px;
    accent-color: var(--primary);
  }
  .sel {
    width: auto;
    padding: 8px 12px;
    font-size: 13px;
    font-weight: 600;
  }
</style>
