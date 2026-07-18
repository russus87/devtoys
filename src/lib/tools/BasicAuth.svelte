<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("encode");
  let user = $state("");
  let pass = $state("");
  let header = $state("");

  function b64encodeUtf8(s: string): string {
    const bytes = new TextEncoder().encode(s);
    let bin = "";
    for (const b of bytes) bin += String.fromCharCode(b);
    return btoa(bin);
  }
  function b64decodeUtf8(s: string): string {
    const bin = atob(s);
    const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  }

  const encoded = $derived.by(() => {
    if (!user && !pass) return { header: "", raw: "" };
    const token = b64encodeUtf8(`${user}:${pass}`);
    return { header: `Authorization: Basic ${token}`, raw: token };
  });

  const decoded = $derived.by(() => {
    const s = header.trim();
    if (!s) return { user: "", pass: "", error: "" };
    const m = s.replace(/^Authorization:\s*/i, "").replace(/^Basic\s+/i, "").trim();
    try {
      const dec = b64decodeUtf8(m);
      const idx = dec.indexOf(":");
      if (idx === -1) return { user: dec, pass: "", error: "" };
      return { user: dec.slice(0, idx), pass: dec.slice(idx + 1), error: "" };
    } catch {
      return { user: "", pass: "", error: "Base64 non valido." };
    }
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={mode}
      options={[
        { value: "encode", label: "Genera" },
        { value: "decode", label: "Decodifica" },
      ]}
    />
  </div>

  {#if mode === "encode"}
    <div class="fields">
      <label class="field">
        <span class="field-label">Utente</span>
        <input class="input" bind:value={user} placeholder="aladdin" spellcheck="false" autocomplete="off" />
      </label>
      <label class="field">
        <span class="field-label">Password</span>
        <input class="input" bind:value={pass} placeholder="opensesame" spellcheck="false" autocomplete="off" />
      </label>
    </div>
    <Pane label="Header Authorization" value={encoded.header} readonly showPaste={false} rows={2} />
    <Pane label="Solo token Base64" value={encoded.raw} readonly showPaste={false} rows={2} />
  {:else}
    <Pane label="Header o token Base64" bind:value={header} placeholder="Authorization: Basic YWxhZGRpbjpvcGVuc2VzYW1l" rows={3} />
    {#if decoded.error}
      <div class="notice bad">{decoded.error}</div>
    {:else}
      <div class="fields">
        <label class="field">
          <span class="field-label">Utente</span>
          <input class="input mono" value={decoded.user} readonly />
        </label>
        <label class="field">
          <span class="field-label">Password</span>
          <input class="input mono" value={decoded.pass} readonly />
        </label>
      </div>
    {/if}
  {/if}
</div>

<style>
  .fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }
  @media (max-width: 640px) {
    .fields {
      grid-template-columns: 1fr;
    }
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
</style>
