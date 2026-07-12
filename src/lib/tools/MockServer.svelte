<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { onMount, onDestroy } from "svelte";

  interface Route {
    method: string;
    path: string;
    status: number;
    contentType: string;
    body: string;
    delayMs: number;
  }

  interface MockRequestEvent {
    method: string;
    path: string;
    matched: boolean;
    at: string;
  }

  interface MockStatus {
    running: boolean;
    port: number | null;
  }

  let port = $state(8081);
  let routes = $state<Route[]>([
    { method: "GET", path: "/health", status: 200, contentType: "application/json", body: '{"ok":true}', delayMs: 0 },
  ]);

  let running = $state(false);
  let error = $state("");
  let log = $state<MockRequestEvent[]>([]);

  let unReq: UnlistenFn | null = null;

  function addRoute() {
    routes = [
      ...routes,
      { method: "GET", path: "/", status: 200, contentType: "application/json", body: "{}", delayMs: 0 },
    ];
  }
  function removeRoute(i: number) {
    routes = routes.filter((_, idx) => idx !== i);
  }

  async function refreshStatus() {
    try {
      const s = await invoke<MockStatus>("mock_status");
      running = s.running;
      if (s.port) port = s.port;
    } catch {
      /* ignore */
    }
  }

  async function start() {
    error = "";
    log = [];
    try {
      unReq = await listen<MockRequestEvent>("mock:request", (e) => {
        log = [e.payload, ...log].slice(0, 200);
      });
      await invoke("mock_start", { port, routes });
      running = true;
    } catch (e) {
      error = "Avvio fallito: " + (e as Error).message;
      unReq?.();
      unReq = null;
      running = false;
    }
  }

  async function stop() {
    try {
      await invoke("mock_stop");
    } catch (e) {
      error = "Arresto fallito: " + (e as Error).message;
    } finally {
      running = false;
      unReq?.();
      unReq = null;
    }
  }

  onMount(() => {
    refreshStatus();
  });
  onDestroy(() => {
    unReq?.();
  });
</script>

<div class="tool">
  <div class="notice">
    Attenzione: avviare un server mock apre una porta in ascolto su localhost, raggiungibile da qualsiasi
    processo sulla macchina.
  </div>

  <div class="tool-controls wrap">
    <div class="row">
      <span class="field-label">Porta</span>
      <input class="input" type="number" min="1" max="65535" style="width:100px" bind:value={port} disabled={running} />
    </div>
    <div class="spacer"></div>
    {#if !running}
      <button class="btn primary" onclick={start}>Avvia server</button>
    {:else}
      <button class="btn" onclick={stop}>Ferma</button>
      <span class="pill gold">In ascolto su :{port}</span>
    {/if}
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}

  <div class="stack grow">
    <div class="row wrap">
      <span class="field-label">Rotte</span>
      <div class="spacer"></div>
      <button class="btn sm" onclick={addRoute} disabled={running}>+ Aggiungi rotta</button>
    </div>

    <div class="routes">
      {#each routes as r, i (i)}
        <div class="card-box route-row">
          <div class="row wrap">
            <Segmented
              bind:value={r.method}
              options={[
                { value: "GET", label: "GET" },
                { value: "POST", label: "POST" },
                { value: "PUT", label: "PUT" },
                { value: "DELETE", label: "DELETE" },
                { value: "PATCH", label: "PATCH" },
              ]}
            />
            <input class="input" style="width:180px" placeholder="/percorso" bind:value={r.path} disabled={running} />
            <div class="row">
              <span class="field-label">Status</span>
              <input class="input" type="number" style="width:80px" bind:value={r.status} disabled={running} />
            </div>
            <div class="row">
              <span class="field-label">Ritardo (ms)</span>
              <input class="input" type="number" min="0" style="width:90px" bind:value={r.delayMs} disabled={running} />
            </div>
            <input class="input" style="width:180px" placeholder="Content-Type" bind:value={r.contentType} disabled={running} />
            <div class="spacer"></div>
            <button class="btn sm ghost" onclick={() => removeRoute(i)} disabled={running}>Rimuovi</button>
          </div>
          <Pane label="Body risposta" bind:value={r.body} rows={3} readonly={running} showPaste={!running} />
        </div>
      {/each}
      {#if routes.length === 0}
        <div class="muted faint">Nessuna rotta configurata.</div>
      {/if}
    </div>
  </div>

  <div class="stack">
    <span class="field-label">Log richieste</span>
    <div class="card-box log-box">
      {#if log.length === 0}
        <div class="muted faint">Nessuna richiesta ricevuta.</div>
      {:else}
        <table class="log-table">
          <thead>
            <tr>
              <th>Ora</th>
              <th>Metodo</th>
              <th>Percorso</th>
              <th>Esito</th>
            </tr>
          </thead>
          <tbody>
            {#each log as l, i (i)}
              <tr>
                <td class="mono faint">{l.at}</td>
                <td class="mono">{l.method}</td>
                <td class="mono">{l.path}</td>
                <td>
                  {#if l.matched}
                    <span class="pill" style="color:var(--green)">trovata</span>
                  {:else}
                    <span class="pill" style="color:var(--red)">404</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </div>
</div>

<style>
  .routes {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .route-row {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .log-box {
    max-height: 260px;
    overflow: auto;
    padding: 10px;
  }
  .log-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12.5px;
  }
  .log-table th {
    text-align: left;
    padding: 6px 8px;
    color: var(--ink-faint);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }
  .log-table td {
    padding: 6px 8px;
    border-top: 1px solid var(--border);
  }
</style>
