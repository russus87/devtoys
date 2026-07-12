<script lang="ts">
  import { marked } from "marked";
  import Pane from "../ui/Pane.svelte";

  let input = $state(
    `# Titolo

Questo è un **esempio** di *Markdown* con [un link](https://example.com).

- primo elemento
- secondo elemento

\`\`\`js
console.log("ciao");
\`\`\`

> Una citazione.
`
  );

  const html = $derived(marked.parse(input, { async: false }) as string);
</script>

<div class="tool">
  <div class="split">
    <Pane label="Markdown" bind:value={input} placeholder="Scrivi Markdown…" />
    <div class="stack grow">
      <span class="field-label">Anteprima</span>
      <div class="md-preview">
        {@html html}
      </div>
    </div>
  </div>
</div>

<style>
  .md-preview {
    flex: 1;
    padding: 13px 18px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border-2);
    background: var(--surface);
    overflow: auto;
  }
  :global(.md-preview h1) {
    font-size: 26px;
    font-weight: 700;
    margin: 0.6em 0 0.4em;
  }
  :global(.md-preview h2) {
    font-size: 21px;
    font-weight: 700;
    margin: 0.6em 0 0.4em;
  }
  :global(.md-preview h3) {
    font-size: 17px;
    font-weight: 600;
    margin: 0.6em 0 0.4em;
  }
  :global(.md-preview p) {
    margin: 0.5em 0;
    line-height: 1.6;
  }
  :global(.md-preview a) {
    color: var(--primary);
  }
  :global(.md-preview code) {
    font-family: var(--mono);
    background: var(--surface-3);
    padding: 2px 5px;
    border-radius: 5px;
    font-size: 0.9em;
  }
  :global(.md-preview pre) {
    background: var(--surface-2);
    padding: 12px 14px;
    border-radius: var(--radius);
    overflow: auto;
  }
  :global(.md-preview pre code) {
    background: transparent;
    padding: 0;
  }
  :global(.md-preview ul),
  :global(.md-preview ol) {
    padding-left: 1.5em;
    margin: 0.5em 0;
  }
  :global(.md-preview blockquote) {
    border-left: 3px solid var(--primary);
    padding-left: 12px;
    margin: 0.6em 0;
    color: var(--ink-dim);
  }
  :global(.md-preview table) {
    border-collapse: collapse;
    margin: 0.6em 0;
  }
  :global(.md-preview th),
  :global(.md-preview td) {
    border: 1px solid var(--border);
    padding: 6px 10px;
  }
  :global(.md-preview img) {
    max-width: 100%;
  }
  :global(.md-preview hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 1em 0;
  }
</style>
