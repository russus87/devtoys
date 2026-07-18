<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let input = $state("");
  let sep = $state("-");
  let lower = $state(true);
  let stripAccents = $state(true);

  const output = $derived.by(() => {
    if (!input) return "";
    return input
      .split(/\r?\n/)
      .map((line) => {
        let s = line;
        if (stripAccents) {
          // Decompose accented characters and drop the diacritics.
          s = s.normalize("NFKD").replace(/[\u0300-\u036f]/g, "");
        }
        if (lower) s = s.toLowerCase();
        s = s
          .replace(/[^a-zA-Z0-9]+/g, sep) // non-alnum -> separator
          .replace(new RegExp(`\\${sep}{2,}`, "g"), sep) // collapse repeats
          .replace(new RegExp(`^\\${sep}+|\\${sep}+$`, "g"), ""); // trim
        return s;
      })
      .join("\n");
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <div class="row">
      <span class="field-label">Separatore</span>
      <Segmented
        bind:value={sep}
        options={[
          { value: "-", label: "trattino -" },
          { value: "_", label: "underscore _" },
        ]}
      />
    </div>
    <label class="chk"><input type="checkbox" bind:checked={lower} /> Minuscolo</label>
    <label class="chk"><input type="checkbox" bind:checked={stripAccents} /> Rimuovi accenti</label>
  </div>

  <div class="split">
    <Pane label="Testo" bind:value={input} placeholder="Il Mio Fantàstico Titolo!" />
    <Pane label="Slug" value={output} readonly showPaste={false} />
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
</style>
