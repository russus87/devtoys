<script lang="ts">
  import Pane from "../ui/Pane.svelte";

  let a = $state("");
  let b = $state("");

  interface DiffRow {
    type: "eq" | "add" | "del";
    text: string;
  }

  function diffLines(a: string[], b: string[]): DiffRow[] {
    const n = a.length,
      m = b.length;
    const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
    for (let i = n - 1; i >= 0; i--)
      for (let j = m - 1; j >= 0; j--)
        dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    const out: DiffRow[] = [];
    let i = 0,
      j = 0;
    while (i < n && j < m) {
      if (a[i] === b[j]) {
        out.push({ type: "eq", text: a[i] });
        i++;
        j++;
      } else if (dp[i + 1][j] >= dp[i][j + 1]) {
        out.push({ type: "del", text: a[i] });
        i++;
      } else {
        out.push({ type: "add", text: b[j] });
        j++;
      }
    }
    while (i < n) out.push({ type: "del", text: a[i++] });
    while (j < m) out.push({ type: "add", text: b[j++] });
    return out;
  }

  const rows = $derived.by(() => diffLines(a.split("\n"), b.split("\n")));
  const added = $derived(rows.filter((r) => r.type === "add").length);
  const removed = $derived(rows.filter((r) => r.type === "del").length);

  function prefix(t: DiffRow["type"]): string {
    return t === "add" ? "+" : t === "del" ? "−" : " ";
  }
</script>

<div class="tool">
  <div class="split">
    <Pane label="Originale" bind:value={a} placeholder="Testo originale…" />
    <Pane label="Modificato" bind:value={b} placeholder="Testo modificato…" />
  </div>
  <div class="diff-summary muted">
    +{added} aggiunte · −{removed} rimosse
  </div>
  <div class="diff-result">
    {#each rows as r, i (i)}
      <div class="diff-line" class:add={r.type === "add"} class:del={r.type === "del"}>
        <span class="diff-prefix">{prefix(r.type)}</span><span class="diff-text">{r.text || " "}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .diff-summary {
    margin-top: 12px;
    font-size: 13px;
  }
  .diff-result {
    margin-top: 8px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
    overflow: auto;
    max-height: 420px;
  }
  .diff-line {
    display: flex;
    gap: 8px;
    font-family: var(--mono);
    font-size: 13px;
    white-space: pre-wrap;
    word-break: break-word;
    padding: 2px 10px;
  }
  .diff-prefix {
    flex: 0 0 auto;
    opacity: 0.6;
    user-select: none;
  }
  .diff-text {
    flex: 1;
    min-width: 0;
  }
  .diff-line.del {
    background: color-mix(in srgb, var(--red) 12%, var(--surface));
  }
  .diff-line.add {
    background: color-mix(in srgb, var(--green) 12%, var(--surface));
  }
</style>
