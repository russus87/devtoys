<script lang="ts">
  import Segmented from "../ui/Segmented.svelte";
  import { copy } from "../api";

  let value = $state("");
  let fromBase = $state("10");
  let copiedKey = $state("");

  const computed = $derived.by((): { value: number | null; error: string } => {
    const v = value.trim();
    if (!v) return { value: null, error: "" };
    const n = parseInt(v, Number(fromBase));
    if (Number.isNaN(n)) {
      return { value: null, error: "Numero non valido nella base selezionata" };
    }
    return { value: n, error: "" };
  });
  const parsed = $derived(computed.value);
  const error = $derived(computed.error);

  const rows = $derived.by(() => {
    if (parsed === null) return null;
    return [
      { key: "bin", label: "BIN", val: parsed.toString(2) },
      { key: "oct", label: "OCT", val: parsed.toString(8) },
      { key: "dec", label: "DEC", val: parsed.toString(10) },
      { key: "hex", label: "HEX", val: parsed.toString(16).toUpperCase() },
    ];
  });

  async function doCopy(key: string, val: string) {
    await copy(val);
    copiedKey = key;
    setTimeout(() => {
      if (copiedKey === key) copiedKey = "";
    }, 1100);
  }
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={fromBase}
      options={[
        { value: "2", label: "BIN" },
        { value: "8", label: "OCT" },
        { value: "10", label: "DEC" },
        { value: "16", label: "HEX" },
      ]}
    />
  </div>

  <div class="stack">
    <span class="field-label">Base d'ingresso: valore</span>
    <input class="input" type="text" bind:value placeholder="Inserisci un numero…" spellcheck="false" autocapitalize="off" autocomplete="off" />
  </div>

  {#if error}
    <div class="notice bad">{error}</div>
  {/if}

  <div class="rows">
    {#each rows ?? [{ key: "bin", label: "BIN", val: "" }, { key: "oct", label: "OCT", val: "" }, { key: "dec", label: "DEC", val: "" }, { key: "hex", label: "HEX", val: "" }] as r (r.key)}
      <div class="row-item">
        <span class="pill">{r.label}</span>
        <span class="mono val">{r.val || "—"}</span>
        <button class="btn sm" class:ok={copiedKey === r.key} disabled={!r.val} onclick={() => doCopy(r.key, r.val)}>
          {copiedKey === r.key ? "✓" : "Copia"}
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .row-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border);
    background: var(--surface);
    transition: background 0.14s ease, border-color 0.14s ease;
  }
  .row-item:hover {
    background: var(--surface-2);
    border-color: var(--border-2);
  }
  .val {
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    white-space: nowrap;
    color: var(--ink);
  }
  .btn.ok {
    color: var(--green);
  }
</style>
