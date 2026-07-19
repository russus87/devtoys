<script lang="ts">
  import forge from "node-forge";
  import Pane from "../ui/Pane.svelte";

  let pem = $state("");

  interface KV { k: string; v: string; }
  interface CertInfo {
    subject: KV[];
    issuer: KV[];
    notBefore: string;
    notAfter: string;
    status: "valid" | "expired" | "notyet";
    version: string;
    serial: string;
    sigAlg: string;
    pubKey: string;
    san: string[];
    sha1: string;
    sha256: string;
  }

  /* eslint-disable @typescript-eslint/no-explicit-any */
  function attrs(list: any[]): KV[] {
    return list
      .map((a) => ({ k: a.shortName || a.name || a.type, v: String(a.value ?? "") }))
      .filter((x) => x.v !== "");
  }

  function colon(hex: string): string {
    return hex.toUpperCase().replace(/(.{2})(?=.)/g, "$1:");
  }

  const SAN_TYPE: Record<number, string> = { 1: "email", 2: "DNS", 6: "URI", 7: "IP" };

  function decode(text: string): CertInfo {
    const cert: any = forge.pki.certificateFromPem(text);

    const der = forge.asn1.toDer(forge.pki.certificateToAsn1(cert)).getBytes();
    const sha1 = forge.md.sha1.create(); sha1.update(der);
    const sha256 = forge.md.sha256.create(); sha256.update(der);

    let pubKey = "sconosciuta";
    try {
      if (cert.publicKey?.n) pubKey = `RSA ${cert.publicKey.n.bitLength()} bit`;
      else if (cert.publicKey) pubKey = "non-RSA (dettaglio non interpretabile)";
    } catch { /* ignore */ }

    const san: string[] = [];
    const sanExt: any = cert.getExtension("subjectAltName");
    if (sanExt?.altNames) {
      for (const n of sanExt.altNames) san.push(`${SAN_TYPE[n.type] ?? "tipo " + n.type}: ${n.value ?? n.ip ?? ""}`);
    }

    const nb: Date = cert.validity.notBefore;
    const na: Date = cert.validity.notAfter;
    const now = Date.now();
    const status: CertInfo["status"] = now < nb.getTime() ? "notyet" : now > na.getTime() ? "expired" : "valid";

    const sigAlg = (forge.pki.oids as Record<string, string>)[cert.signatureOid] ?? cert.signatureOid;

    return {
      subject: attrs(cert.subject.attributes),
      issuer: attrs(cert.issuer.attributes),
      notBefore: nb.toLocaleString("it-IT"),
      notAfter: na.toLocaleString("it-IT"),
      status,
      version: "v" + ((cert.version ?? 2) + 1),
      serial: cert.serialNumber,
      sigAlg,
      pubKey,
      san,
      sha1: colon(sha1.digest().toHex()),
      sha256: colon(sha256.digest().toHex()),
    };
  }

  const computed = $derived.by((): { info: CertInfo | null; error: string } => {
    const t = pem.trim();
    if (!t) return { info: null, error: "" };
    if (!t.includes("BEGIN CERTIFICATE")) {
      return { info: null, error: "Incolla un certificato in formato PEM (blocco -----BEGIN CERTIFICATE-----)." };
    }
    try {
      return { info: decode(t), error: "" };
    } catch (e) {
      return { info: null, error: "Certificato non valido: " + (e as Error).message };
    }
  });
  const info = $derived(computed.info);
  const error = $derived(computed.error);
</script>

<div class="tool">
  <Pane label="Certificato PEM" bind:value={pem} rows={7} placeholder={"-----BEGIN CERTIFICATE-----\nMIID…\n-----END CERTIFICATE-----"} />

  {#if error}
    <div class="notice bad">{error}</div>
  {:else if info}
    {#if info.status === "expired"}
      <div class="notice bad">Certificato scaduto ({info.notAfter})</div>
    {:else if info.status === "notyet"}
      <div class="notice bad">Certificato non ancora valido (dal {info.notBefore})</div>
    {:else}
      <div class="notice good">Certificato valido — scade il {info.notAfter}</div>
    {/if}

    <div class="split">
      <div class="card-box stack">
        <span class="field-label">Soggetto</span>
        {#each info.subject as a (a.k + a.v)}
          <div class="kv"><span class="pill">{a.k}</span><span class="mono val">{a.v}</span></div>
        {/each}
      </div>
      <div class="card-box stack">
        <span class="field-label">Emittente</span>
        {#each info.issuer as a (a.k + a.v)}
          <div class="kv"><span class="pill">{a.k}</span><span class="mono val">{a.v}</span></div>
        {/each}
      </div>
    </div>

    <div class="card-box stack">
      <span class="field-label">Dettagli</span>
      <div class="kv"><span class="pill">Versione</span><span class="mono val">{info.version}</span></div>
      <div class="kv"><span class="pill">Seriale</span><span class="mono val">{info.serial}</span></div>
      <div class="kv"><span class="pill">Firma</span><span class="mono val">{info.sigAlg}</span></div>
      <div class="kv"><span class="pill">Chiave</span><span class="mono val">{info.pubKey}</span></div>
      <div class="kv"><span class="pill">Valido dal</span><span class="mono val">{info.notBefore}</span></div>
      <div class="kv"><span class="pill">Valido al</span><span class="mono val">{info.notAfter}</span></div>
    </div>

    {#if info.san.length}
      <div class="card-box stack">
        <span class="field-label">Nomi alternativi (SAN)</span>
        {#each info.san as s (s)}
          <div class="mono val">{s}</div>
        {/each}
      </div>
    {/if}

    <div class="card-box stack">
      <span class="field-label">Impronte</span>
      <div class="kv"><span class="pill">SHA-1</span><span class="mono val">{info.sha1}</span></div>
      <div class="kv"><span class="pill">SHA-256</span><span class="mono val">{info.sha256}</span></div>
    </div>
  {/if}
</div>

<style>
  .kv {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .val {
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    white-space: nowrap;
    color: var(--ink);
    font-size: 13px;
  }
</style>
