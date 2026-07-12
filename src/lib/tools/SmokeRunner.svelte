<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  interface Step {
    name: string;
    method: string;
    url: string;
    headersRaw: string;
    body: string;
    assertStatus: number | null;
    assertContains: string;
  }

  interface SmokeResult {
    name: string;
    ok: boolean;
    status: number;
    ms: number;
    error: string | null;
  }

  function newStep(n: number): Step {
    return {
      name: `Passo ${n}`,
      method: "GET",
      url: "",
      headersRaw: "",
      body: "",
      assertStatus: 200,
      assertContains: "",
    };
  }

  let steps = $state<Step[]>([newStep(1)]);
  let results = $state<SmokeResult[] | null>(null);
  let running = $state(false);
  let error = $state("");

  function addStep() {
    steps = [...steps, newStep(steps.length + 1)];
  }
  function removeStep(i: number) {
    steps = steps.filter((_, idx) => idx !== i);
  }

  function parseHeaders(raw: string): [string, string][] {
    return raw
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter(Boolean)
      .map((l) => {
        const idx = l.indexOf(":");
        if (idx === -1) return [l.trim(), ""] as [string, string];
        return [l.slice(0, idx).trim(), l.slice(idx + 1).trim()] as [string, string];
      });
  }

  const summary = $derived.by(() => {
    if (!results) return null;
    const passed = results.filter((r) => r.ok).length;
    return { passed, total: results.length };
  });

  async function run() {
    error = "";
    results = null;
    running = true;
    try {
      const payload = steps.map((s) => ({
        name: s.name,
        method: s.method,
        url: s.url,
        headers: parseHeaders(s.headersRaw),
        body: s.method === "GET" ? "" : s.body,
        assertStatus: s.assertStatus,
        assertContains: s.assertContains || null,
      }));
      results = await invoke<SmokeResult[]>("smoke_run", { steps: payload });
    } catch (e) {
      error = "Esecuzione fallita: " + (e as Error).message;
    } finally {
      running = false;
    }
  }
</script>

<div class="tool">
  <div class="row wrap">
    <span class="field-label">Passi</span>
    <div class="spacer"></div>
    <button class="btn sm" onclick={addStep}>+ Aggiungi passo</button>
    <button class="btn primary" onclick={run} disabled={running || steps.length === 0}>
      {running ? "Esecuzione…" : "Esegui"}
    </button>
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}

  <div class="steps grow">
    {#each steps as s, i (i)}
      <div class="card-box step-row">
        <div class="row wrap">
          <input class="input" style="width:160px" placeholder="Nome passo" bind:value={s.name} />
          <Segmented
            bind:value={s.method}
            options={[
              { value: "GET", label: "GET" },
              { value: "POST", label: "POST" },
              { value: "PUT", label: "PUT" },
              { value: "DELETE", label: "DELETE" },
              { value: "PATCH", label: "PATCH" },
            ]}
          />
          <input class="input grow" placeholder="https://…" bind:value={s.url} />
          <div class="spacer"></div>
          <button class="btn sm ghost" onclick={() => removeStep(i)}>Rimuovi</button>
        </div>
        <div class="row wrap">
          <div class="row">
            <span class="field-label">Status atteso</span>
            <input class="input" type="number" style="width:90px" bind:value={s.assertStatus} />
          </div>
          <div class="row grow">
            <span class="field-label">Contiene</span>
            <input class="input" placeholder="testo atteso nel body (opzionale)" bind:value={s.assertContains} />
          </div>
        </div>
        <Pane label="Header (Chiave: Valore, uno per riga)" bind:value={s.headersRaw} rows={2} />
        {#if s.method !== "GET"}
          <Pane label="Body" bind:value={s.body} rows={3} />
        {/if}
      </div>
    {/each}
    {#if steps.length === 0}
      <div class="muted faint">Nessun passo configurato.</div>
    {/if}
  </div>

  {#if results}
    <div class="stack">
      <div class="row">
        <span class="field-label">Risultati</span>
        {#if summary}
          <span class="pill" class:pill-good={summary.passed === summary.total} class:pill-bad={summary.passed !== summary.total}>
            {summary.passed}/{summary.total} passati
          </span>
        {/if}
      </div>
      <div class="card-box results-box">
        {#each results as r, i (i)}
          <div class="result-row" class:ok={r.ok} class:bad={!r.ok}>
            <span class="mark">{r.ok ? "✓" : "✗"}</span>
            <span class="rname">{r.name}</span>
            <span class="mono muted">status {r.status}</span>
            <span class="mono muted">{r.ms} ms</span>
            {#if r.error}<span class="err">{r.error}</span>{/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .steps {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .step-row {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .pill-good {
    color: var(--green);
  }
  .pill-bad {
    color: var(--red);
  }
  .results-box {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 260px;
    overflow: auto;
  }
  .result-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 10px;
    font-size: 13px;
    flex-wrap: wrap;
  }
  .result-row.ok {
    background: color-mix(in srgb, var(--green) 8%, transparent);
  }
  .result-row.bad {
    background: color-mix(in srgb, var(--red) 8%, transparent);
  }
  .mark {
    font-weight: 700;
  }
  .result-row.ok .mark {
    color: var(--green);
  }
  .result-row.bad .mark {
    color: var(--red);
  }
  .rname {
    font-weight: 600;
    min-width: 100px;
  }
</style>
