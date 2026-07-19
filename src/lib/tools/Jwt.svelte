<script lang="ts">
  import Pane from "../ui/Pane.svelte";

  let token = $state("");
  let key = $state("");
  let verifyState = $state<{ status: "idle" | "checking" | "valid" | "invalid" | "error"; msg: string }>({
    status: "idle",
    msg: "",
  });

  function b64urlDecode(s: string): string {
    return new TextDecoder().decode(b64urlToBytes(s));
  }

  function b64urlToBytes(s: string): Uint8Array {
    const pad = s.replace(/-/g, "+").replace(/_/g, "/");
    const padded = pad + "=".repeat((4 - (pad.length % 4)) % 4);
    const bin = atob(padded);
    return Uint8Array.from(bin, (c) => c.charCodeAt(0));
  }

  function toBuf(u: Uint8Array): ArrayBuffer {
    return u.buffer.slice(u.byteOffset, u.byteOffset + u.byteLength) as ArrayBuffer;
  }

  function pemToDer(pem: string): ArrayBuffer {
    const b64 = pem
      .replace(/-----BEGIN [^-]+-----/, "")
      .replace(/-----END [^-]+-----/, "")
      .replace(/\s+/g, "");
    const bin = atob(b64);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return bytes.buffer;
  }

  interface Decoded {
    header: string;
    payload: string;
    payloadObj: Record<string, unknown>;
    alg: string;
  }

  const computed = $derived.by((): { decoded: Decoded | null; error: string } => {
    const t = token.trim();
    if (!t) return { decoded: null, error: "" };
    const parts = t.split(".");
    if (parts.length !== 3) {
      return { decoded: null, error: "Il token deve avere 3 parti separate da '.'" };
    }
    try {
      const headerObj = JSON.parse(b64urlDecode(parts[0]));
      const payloadObj = JSON.parse(b64urlDecode(parts[1])) as Record<string, unknown>;
      return {
        decoded: {
          header: JSON.stringify(headerObj, null, 2),
          payload: JSON.stringify(payloadObj, null, 2),
          payloadObj,
          alg: String(headerObj.alg ?? ""),
        },
        error: "",
      };
    } catch (e) {
      return { decoded: null, error: "Token non valido: " + (e as Error).message };
    }
  });
  const decoded = $derived(computed.decoded);
  const error = $derived(computed.error);

  async function verifySig() {
    verifyState = { status: "checking", msg: "" };
    try {
      const parts = token.trim().split(".");
      if (parts.length !== 3) throw new Error("Token non valido.");
      const header = JSON.parse(b64urlDecode(parts[0])) as { alg?: string };
      const alg = String(header.alg ?? "");
      if (alg === "none") throw new Error("alg 'none': non c'è firma da verificare.");
      if (!key.trim()) throw new Error("Inserisci il segreto (HS) o la chiave pubblica PEM (RS/ES/PS).");
      const data = toBuf(new TextEncoder().encode(parts[0] + "." + parts[1]));
      const sig = toBuf(b64urlToBytes(parts[2]));
      const bits = alg.slice(2);
      const hash = "SHA-" + bits;
      let ok = false;

      if (alg.startsWith("HS")) {
        const k = await crypto.subtle.importKey(
          "raw",
          new TextEncoder().encode(key),
          { name: "HMAC", hash: { name: hash } },
          false,
          ["verify"],
        );
        ok = await crypto.subtle.verify("HMAC", k, sig, data);
      } else if (alg.startsWith("RS") || alg.startsWith("PS")) {
        const name = alg.startsWith("PS") ? "RSA-PSS" : "RSASSA-PKCS1-v1_5";
        const k = await crypto.subtle.importKey("spki", pemToDer(key), { name, hash: { name: hash } }, false, ["verify"]);
        const params = name === "RSA-PSS" ? { name, saltLength: parseInt(bits, 10) / 8 } : { name };
        ok = await crypto.subtle.verify(params, k, sig, data);
      } else if (alg.startsWith("ES")) {
        const curve = bits === "256" ? "P-256" : bits === "384" ? "P-384" : "P-521";
        const k = await crypto.subtle.importKey("spki", pemToDer(key), { name: "ECDSA", namedCurve: curve }, false, ["verify"]);
        ok = await crypto.subtle.verify({ name: "ECDSA", hash: { name: hash } }, k, sig, data);
      } else {
        throw new Error("Algoritmo non supportato: " + (alg || "assente"));
      }

      verifyState = ok
        ? { status: "valid", msg: "Firma valida (" + alg + ")." }
        : { status: "invalid", msg: "Firma NON valida (" + alg + ")." };
    } catch (e) {
      verifyState = { status: "error", msg: (e as Error).message };
    }
  }

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
            <div class="row-item"><span class="pill">iat</span><span class="mono val">{fmt(iat)}</span></div>
          {/if}
          {#if nbf !== null}
            <div class="row-item"><span class="pill">nbf</span><span class="mono val">{fmt(nbf)}</span></div>
          {/if}
          {#if exp !== null}
            <div class="row-item"><span class="pill">exp</span><span class="mono val">{fmt(exp)}</span></div>
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

    <div class="card-box stack">
      <span class="field-label">Verifica firma {decoded.alg ? "(" + decoded.alg + ")" : ""}</span>
      <Pane
        label="Segreto HMAC (HS…) o chiave pubblica PEM (RS/ES/PS…)"
        bind:value={key}
        rows={3}
        placeholder={decoded.alg.startsWith("HS") ? "il-tuo-segreto" : "-----BEGIN PUBLIC KEY-----\n…\n-----END PUBLIC KEY-----"}
      />
      <div class="row">
        <button class="btn primary" onclick={verifySig} disabled={verifyState.status === "checking"}>
          {verifyState.status === "checking" ? "Verifica…" : "Verifica firma"}
        </button>
      </div>
      {#if verifyState.status === "valid"}
        <div class="notice good">✓ {verifyState.msg}</div>
      {:else if verifyState.status === "invalid"}
        <div class="notice bad">✕ {verifyState.msg}</div>
      {:else if verifyState.status === "error"}
        <div class="notice bad">{verifyState.msg}</div>
      {/if}
    </div>
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
