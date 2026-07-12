<script lang="ts">
  import Pane from "../ui/Pane.svelte";

  let token = $state("");
  let error = $state("");

  function b64urlDecode(s: string): string {
    const pad = s.replace(/-/g, "+").replace(/_/g, "/");
    const padded = pad + "=".repeat((4 - (pad.length % 4)) % 4);
    const bin = atob(padded);
    const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  }

  interface Decoded {
    header: string;
    payload: string;
    payloadObj: Record<string, unknown>;
  }

  const decoded = $derived.by((): Decoded | null => {
    error = "";
    const t = token.trim();
    if (!t) return null;
    const parts = t.split(".");
    if (parts.length !== 3) {
      error = "Il token deve avere 3 parti separate da '.'";
      return null;
    }
    try {
      const headerObj = JSON.parse(b64urlDecode(parts[0]));
      const payloadObj = JSON.parse(b64urlDecode(parts[1])) as Record<string, unknown>;
      return {
        header: JSON.stringify(headerObj, null, 2),
        payload: JSON.stringify(payloadObj, null, 2),
        payloadObj,
      };
    } catch (e) {
      error = "Token non valido: " + (e as Error).message;
      return null;
    }
  });

  function asUnixSeconds(v: unknown): number | null {
    if (typeof v === "number" && Number.isFinite(v)) return v;
    return null;
  }

  function fmt(v: unknown): string {
    const n = asUnixSeconds(v);
    if (n === null) return "-";
    return new Date(n * 1000).toLocaleString("it-IT");
  }

  const exp = $derived(decoded ? asUnixSeconds(decoded.payloadObj["exp"]) : null);
  const iat = $derived(decoded ? asUnixSeconds(decoded.payloadObj["iat"]) : null);
  const nbf = $derived(decoded ? asUnixSeconds(decoded.payloadObj["nbf"]) : null);
  const isExpired = $derived(exp !== null && exp * 1000 < Date.now());
</script>

<div class="tool">
  <Pane label="Token JWT" bind:value={token} placeholder="Incolla qui il token JWT…" rows={4} />
  <div class="notice">La firma non viene verificata: qui il token viene solo decodificato.</div>

  {#if error}
    <div class="notice bad">{error}</div>
  {:else if decoded}
    <div class="split">
      <Pane label="Header" value={decoded.header} readonly showPaste={false} />
      <Pane label="Payload" value={decoded.payload} readonly showPaste={false} />
    </div>

    {#if exp !== null || iat !== null || nbf !== null}
      <div class="card-box stack">
        <span class="field-label">Date del token</span>
        <div class="rows">
          {#if iat !== null}
            <div class="row-item">
              <span class="pill">iat</span>
              <span class="mono val">{fmt(iat)}</span>
            </div>
          {/if}
          {#if nbf !== null}
            <div class="row-item">
              <span class="pill">nbf</span>
              <span class="mono val">{fmt(nbf)}</span>
            </div>
          {/if}
          {#if exp !== null}
            <div class="row-item">
              <span class="pill">exp</span>
              <span class="mono val">{fmt(exp)}</span>
            </div>
          {/if}
        </div>
        {#if exp !== null}
          {#if isExpired}
            <div class="notice bad">Token scaduto</div>
          {:else}
            <div class="notice good">Token valido (non scaduto)</div>
          {/if}
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .row-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border);
    background: var(--surface);
  }
  .val {
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    white-space: nowrap;
    color: var(--ink);
  }
</style>
