<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let unit = $state("parole");
  let quantity = $state(3);
  let startClassic = $state(true);

  const WORDS = [
    "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
    "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore",
    "magna", "aliqua", "enim", "ad", "minim", "veniam", "quis", "nostrud",
    "exercitation", "ullamco", "laboris", "nisi", "aliquip", "ex", "ea", "commodo",
    "consequat", "duis", "aute", "irure", "in", "reprehenderit", "voluptate",
    "velit", "esse", "cillum", "eu", "fugiat", "nulla", "pariatur", "excepteur",
    "sint", "occaecat", "cupidatat", "non", "proident", "sunt", "culpa", "qui",
    "officia", "deserunt", "mollit", "anim", "id", "est", "laborum",
  ];

  function randInt(min: number, max: number): number {
    return Math.floor(Math.random() * (max - min + 1)) + min;
  }

  function randomWord(): string {
    return WORDS[randInt(0, WORDS.length - 1)];
  }

  function capitalize(w: string): string {
    return w.charAt(0).toUpperCase() + w.slice(1);
  }

  function makeSentence(): string {
    const n = randInt(6, 12);
    const words: string[] = [];
    for (let i = 0; i < n; i++) words.push(randomWord());
    let sentence = words.join(" ");
    sentence = capitalize(sentence) + ".";
    return sentence;
  }

  function makeParagraph(): string {
    const n = randInt(3, 6);
    const sentences: string[] = [];
    for (let i = 0; i < n; i++) sentences.push(makeSentence());
    return sentences.join(" ");
  }

  let output = $state("");

  function generate() {
    const qty = Math.min(100, Math.max(1, Math.trunc(quantity) || 1));
    let text = "";

    if (unit === "parole") {
      const words: string[] = [];
      for (let i = 0; i < qty; i++) words.push(randomWord());
      text = words.join(" ");
      if (text) text = capitalize(text);
    } else if (unit === "frasi") {
      const sentences: string[] = [];
      for (let i = 0; i < qty; i++) sentences.push(makeSentence());
      text = sentences.join(" ");
    } else {
      const paragraphs: string[] = [];
      for (let i = 0; i < qty; i++) paragraphs.push(makeParagraph());
      text = paragraphs.join("\n\n");
    }

    if (startClassic && text) {
      const prefix = "Lorem ipsum dolor sit amet, ";
      const rest = text.charAt(0).toLowerCase() + text.slice(1);
      text = prefix + rest;
    }

    output = text;
  }

  function regenerate() {
    generate();
  }

  $effect(() => {
    // Recompute whenever unit, quantity or the prefix checkbox changes.
    void unit;
    void quantity;
    void startClassic;
    generate();
  });
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <Segmented
      bind:value={unit}
      options={[
        { value: "parole", label: "Parole" },
        { value: "frasi", label: "Frasi" },
        { value: "paragrafi", label: "Paragrafi" },
      ]}
    />
    <div class="row">
      <span class="field-label">Quantità</span>
      <input class="input" type="number" min="1" max="100" bind:value={quantity} />
    </div>
    <label class="check">
      <input type="checkbox" bind:checked={startClassic} />
      <span>Inizia con «Lorem ipsum…»</span>
    </label>
    <div class="spacer"></div>
    <button class="btn primary" onclick={regenerate}>Rigenera</button>
  </div>

  <Pane label="Output" value={output} readonly showPaste={false} rows={14} />
</div>

<style>
  .check {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--ink-dim);
    cursor: pointer;
  }
  .check input {
    accent-color: var(--primary);
  }
</style>
