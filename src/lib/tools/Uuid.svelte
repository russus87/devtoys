<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { api, copy } from "../api";

  let version = $state("v4");
  let count = $state(5);
  let uppercase = $state("off");
  let uuids = $state<string[]>([]);
  let error = $state("");
  let started = false;

  const output = $derived.by(() => {
    const joined = uuids.join("\n");
    return uppercase === "on" ? joined.toUpperCase() : joined;
  });

  async function generate() {
    error = "";
    const n = Math.min(1000, Math.max(1, Math.trunc(count) || 1));
    count = n;
    try {
      uuids = await api.genUuids(version, n);
    } catch (e) {
      uuids = [];
      error = "Generazione fallita: " + (e as Error).message;
    }
  }

  function onGenerateClick() {
    generate().catch(() => {
      error = "Generazione fallita";
    });
  }

  $effect(() => {
    if (!started) {
      started = true;
      generate().catch(() => {
        error = "Generazione fallita";
      });
    }
  });

  async function copyAll() {
    if (output) await copy(output);
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <Segmented
      bind:value={version}
      options={[
        { value: "v4", label: "v4 (casuale)" },
        { value: "v7", label: "v7 (ordinato nel tempo)" },
      ]}
    />
    <div class="row">
      <span class="field-label">Quantità</span>
      <input class="input" type="number" min="1" max="1000" bind:value={count} />
    </div>
    <Segmented
      bind:value={uppercase}
      options={[
        { value: "off", label: "Maiuscolo: off" },
        { value: "on", label: "Maiuscolo: on" },
      ]}
    />
    <div class="spacer"></div>
    <button class="btn primary" onclick={onGenerateClick}>Genera</button>
    <button class="btn" onclick={copyAll}>Copia tutti</button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}

  <Pane label="UUID generati" value={output} readonly showPaste={false} rows={12} />
</div>
