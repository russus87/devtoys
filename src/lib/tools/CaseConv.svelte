<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import { copy } from "../api";

  let input = $state("");

  const tokens = $derived.by(() => {
    if (!input.trim()) return [] as string[];
    return input
      .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
      .split(/[\s_\-.]+/)
      .filter(Boolean)
      .map((w) => w.toLowerCase());
  });

  function camelCase(t: string[]): string {
    if (!t.length) return "";
    return t
      .map((w, i) => (i === 0 ? w : w.charAt(0).toUpperCase() + w.slice(1)))
      .join("");
  }
  function pascalCase(t: string[]): string {
    return t.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join("");
  }
  function snakeCase(t: string[]): string {
    return t.join("_");
  }
  function constantCase(t: string[]): string {
    return t.map((w) => w.toUpperCase()).join("_");
  }
  function kebabCase(t: string[]): string {
    return t.join("-");
  }
  function titleCase(t: string[]): string {
    return t.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
  }
  function sentenceCase(t: string[]): string {
    const s = t.join(" ");
    return s ? s.charAt(0).toUpperCase() + s.slice(1) : "";
  }
  function lowerCase(t: string[]): string {
    return t.join(" ");
  }
  function upperCase(t: string[]): string {
    return t.join(" ").toUpperCase();
  }

  interface Row {
    label: string;
    value: string;
  }

  const rows = $derived.by((): Row[] => {
    const t = tokens;
    return [
      { label: "camelCase", value: camelCase(t) },
      { label: "PascalCase", value: pascalCase(t) },
      { label: "snake_case", value: snakeCase(t) },
      { label: "CONSTANT_CASE", value: constantCase(t) },
      { label: "kebab-case", value: kebabCase(t) },
      { label: "Title Case", value: titleCase(t) },
      { label: "Sentence case", value: sentenceCase(t) },
      { label: "lowercase", value: lowerCase(t) },
      { label: "UPPERCASE", value: upperCase(t) },
    ];
  });

  let copiedIdx = $state(-1);

  async function doCopy(v: string, idx: number) {
    await copy(v);
    copiedIdx = idx;
    setTimeout(() => {
      if (copiedIdx === idx) copiedIdx = -1;
    }, 1100);
  }
</script>

<div class="tool">
  <Pane label="Input" bind:value={input} placeholder="Scrivi o incolla qui…" />
  <div class="rows stack">
    {#each rows as r, i (r.label)}
      <div class="case-row">
        <span class="case-label">{r.label}</span>
        <span class="case-value mono">{r.value || "—"}</span>
        <button
          class="btn sm ghost"
          class:ok={copiedIdx === i}
          disabled={!r.value}
          onclick={() => doCopy(r.value, i)}
        >
          {copiedIdx === i ? "✓" : "Copia"}
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  .rows {
    margin-top: 14px;
    gap: 8px;
  }
  .case-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-radius: var(--radius);
    background: var(--surface-2);
    border: 1px solid var(--border);
  }
  .case-label {
    flex: 0 0 150px;
    font-size: 13px;
    font-weight: 600;
    color: var(--ink-dim);
  }
  .case-value {
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    white-space: pre;
    color: var(--ink);
    font-size: 13px;
  }
  .btn.ok {
    color: var(--green);
  }
</style>
