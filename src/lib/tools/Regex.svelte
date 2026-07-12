<script lang="ts">
  import Pane from "../ui/Pane.svelte";

  let pattern = $state("");
  let flags = $state("g");
  let text = $state("");

  interface MatchInfo {
    index: number;
    match: string;
    groups: string[];
  }

  function escapeHtml(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  const result = $derived.by((): {
    error: string;
    highlighted: string;
    matches: MatchInfo[];
  } => {
    if (!pattern) {
      return { error: "", highlighted: escapeHtml(text), matches: [] };
    }
    let re: RegExp;
    try {
      re = new RegExp(pattern, flags);
    } catch (e) {
      return { error: "Regex non valida: " + (e as Error).message, highlighted: "", matches: [] };
    }

    const gFlags = flags.includes("g") ? flags : flags + "g";
    let gRe: RegExp;
    try {
      gRe = new RegExp(pattern, gFlags);
    } catch (e) {
      return { error: "Regex non valida: " + (e as Error).message, highlighted: "", matches: [] };
    }

    const escaped = escapeHtml(text);
    let highlighted = "";
    let lastIndex = 0;
    const matches: MatchInfo[] = [];

    try {
      for (const m of escaped.matchAll(gRe)) {
        const idx = m.index ?? 0;
        highlighted += escaped.slice(lastIndex, idx);
        highlighted += `<mark>${m[0]}</mark>`;
        lastIndex = idx + m[0].length;
      }
      highlighted += escaped.slice(lastIndex);

      for (const m of text.matchAll(new RegExp(pattern, gFlags))) {
        matches.push({
          index: m.index ?? 0,
          match: m[0],
          groups: (m.slice(1) as (string | undefined)[]).map((g) => (g === undefined ? "—" : g)),
        });
      }
    } catch (e) {
      return { error: "Regex non valida: " + (e as Error).message, highlighted: "", matches: [] };
    }

    return { error: "", highlighted, matches };
  });
</script>

<div class="tool">
  <div class="tool-controls row wrap">
    <input class="input grow" bind:value={pattern} placeholder="Pattern regex, es. \d+" />
    <input class="input" style="width: 90px" bind:value={flags} placeholder="flags" />
  </div>

  <Pane label="Testo di prova" bind:value={text} placeholder="Scrivi o incolla qui…" />

  {#if result.error}
    <div class="notice bad">{result.error}</div>
  {:else}
    <div class="highlight-box mono">{@html result.highlighted || "&nbsp;"}</div>
    <div class="match-summary muted">
      {result.matches.length} corrispondenz{result.matches.length === 1 ? "a" : "e"}
    </div>
    {#if result.matches.length}
      <div class="stack match-list">
        {#each result.matches as m, i (i)}
          <div class="card-box match-item">
            <div class="row">
              <span class="pill">#{i + 1}</span>
              <span class="mono">indice {m.index}</span>
            </div>
            <div class="mono match-full">{m.match}</div>
            {#if m.groups.length}
              <div class="groups faint">
                gruppi: {m.groups.join(", ")}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .highlight-box {
    margin-top: 12px;
    padding: 13px 15px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border-2);
    background: var(--surface);
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 13px;
  }
  :global(.highlight-box mark) {
    background: var(--gold);
    color: #3a2a05;
    border-radius: 4px;
    padding: 0 2px;
  }
  .match-summary {
    margin-top: 8px;
    font-size: 13px;
  }
  .match-list {
    margin-top: 8px;
    gap: 8px;
  }
  .match-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .match-full {
    font-size: 13px;
  }
  .groups {
    font-size: 12px;
  }
</style>
