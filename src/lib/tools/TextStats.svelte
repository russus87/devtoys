<script lang="ts">
  import Pane from "../ui/Pane.svelte";

  let input = $state("");

  const chars = $derived(input.length);
  const charsNoSpace = $derived(input.replace(/\s/g, "").length);
  const words = $derived(input.trim() ? input.trim().split(/\s+/).length : 0);
  const lines = $derived(input ? input.split(/\r\n|\r|\n/).length : 0);
  const sentences = $derived((input.match(/[.!?]+/g) || []).length);
  const paragraphs = $derived(input.trim() ? input.trim().split(/\n\s*\n/).length : 0);
  const bytes = $derived(new TextEncoder().encode(input).length);
  const readingTime = $derived(Math.ceil(words / 200));

  interface Stat {
    label: string;
    value: string | number;
  }

  const stats = $derived.by((): Stat[] => [
    { label: "Caratteri", value: chars },
    { label: "Caratteri (no spazi)", value: charsNoSpace },
    { label: "Parole", value: words },
    { label: "Righe", value: lines },
    { label: "Frasi", value: sentences },
    { label: "Paragrafi", value: paragraphs },
    { label: "Byte (UTF-8)", value: bytes },
    { label: "Tempo di lettura", value: `${readingTime} min` },
  ]);
</script>

<div class="tool">
  <Pane label="Input" bind:value={input} placeholder="Scrivi o incolla qui…" />
  <div class="stats-grid">
    {#each stats as s (s.label)}
      <div class="card-box stat-tile">
        <div class="stat-number">{s.value}</div>
        <div class="stat-label muted">{s.label}</div>
      </div>
    {/each}
  </div>
</div>

<style>
  .stats-grid {
    margin-top: 14px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 10px;
  }
  .stat-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 16px 10px;
    text-align: center;
  }
  .stat-number {
    font-size: 26px;
    font-weight: 700;
    color: var(--primary);
    line-height: 1.1;
  }
  .stat-label {
    font-size: 12.5px;
  }
</style>
