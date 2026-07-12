<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  interface ProxyExchange {
    method: string;
    path: string;
    status: number;
    ms: number;
    at: string;
  }

  interface ProxyStatus {
    running: boolean;
    port: number | null;
    target: string | null;
  }

  let port = $state(8899);
  let target = $state("https://api.example.com");
  let running = $state(false);
  let error = $state("");
  let exchanges = $state<ProxyExchange[]>([]);

  let unExchange: UnlistenFn | null = null;

  async function refreshStatus() {
    try {
      const s = await invoke<ProxyStatus>("proxy_status");
      running = s.running;
      if (s.port) port = s.port;
      if (s.target) target = s.target;
    } catch {
      /* ignore */
    }
  }

  async function start() {
    error = "";
    exchanges = [];
    try {
      unExchange = await listen<ProxyExchange>("proxy:exchange", (e) => {
        exchanges = [e.payload, ...exchanges].slice(0, 300);
      });
      await invoke("proxy_start", { port, target });
      running = true;
    } catch (e) {
      error = "Avvio fallito: " + (e as Error).message;
      unExchange?.();
      unExchange = null;
      running = false;
    }
  }

  async function stop() {
    try {
      await invoke("proxy_stop");
    } catch (e) {
      error = "Arresto fallito: " + (e as Error).message;
    } finally {
      running = false;
      unExchange?.();
      unExchange = null;
    }
  }

  function statusColor(status: number): string {
    if (status >= 500) return "var(--red)";
    if (status >= 400) return "var(--amber)";
    if (status >= 300) return "var(--gold)";
    if (status >= 200) return "var(--green)";
    return "var(--ink-dim)";
  }

  onMount(() => {
    refreshStatus();
  });
  onDestroy(() => {
    unExchange?.();
  });
</script>

<div class="tool">
  <div class="notice">
    Punta l'URL di base della tua app a <span class="mono">http://localhost:{port}</span> e le richieste
    verranno inoltrate al target, registrando ogni scambio. È un reverse-proxy pragmatico, <strong>non</strong>
    un man-in-the-middle capace di intercettare HTTPS.
  </div>

  <div class="tool-controls wrap">
    <div class="row">
      <span class="field-label">Porta locale</span>
      <input class="input" type="number" min="1" max="65535" style="width:100px" bind:value={port} disabled={running} />
    </div>
    <input class="input grow" placeholder="https://api.example.com" bind:value={target} disabled={running} />
    <div class="spacer"></div>
    {#if !running}
      <button class="btn primary" onclick={start}>Avvia</button>
    {:else}
      <button class="btn" onclick={stop}>Ferma</button>
      <span class="pill gold">In ascolto su :{port}</span>
    {/if}
  </div>

  {#if error}<div class="notice bad">{error}</div>{/if}

  <div class="stack grow">
    <span class="field-label">Scambi</span>
    <div class="card-box exch-box">
      {#if exchanges.length === 0}
        <div class="muted faint">Nessuno scambio registrato.</div>
      {:else}
        <table class="exch-table">
          <thead>
            <tr>
              <th>Ora</th>
              <th>Metodo</th>
              <th>Percorso</th>
              <th>Status</th>
              <th>ms</th>
            </tr>
          </thead>
          <tbody>
            {#each exchanges as ex, i (i)}
              <tr>
                <td class="mono faint">{ex.at}</td>
                <td class="mono">{ex.method}</td>
                <td class="mono">{ex.path}</td>
                <td class="mono" style="color:{statusColor(ex.status)}">{ex.status}</td>
                <td class="mono muted">{ex.ms}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  </div>
</div>

<style>
  .exch-box {
    max-height: 340px;
    overflow: auto;
    padding: 10px;
  }
  .exch-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12.5px;
  }
  .exch-table th {
    text-align: left;
    padding: 6px 8px;
    color: var(--ink-faint);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.02em;
  }
  .exch-table td {
    padding: 6px 8px;
    border-top: 1px solid var(--border);
  }
</style>
