<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import { api, copy, type Hashes } from "../api";

  let input = $state("");
  let hashes = $state<Hashes | null>(null);
  let busy = $state(false);
  let copiedKey = $state("");

  const ALGOS: { key: keyof Hashes; label: string }[] = [
    { key: "md5", label: "MD5" },
    { key: "sha1", label: "SHA-1" },
    { key: "sha256", label: "SHA-256" },
    { key: "sha384", label: "SHA-384" },
    { key: "sha512", label: "SHA-512" },
    { key: "sha3_256", label: "SHA3-256" },
    { key: "sha3_512", label: "SHA3-512" },
    { key: "crc32", label: "CRC32" },
  ];

  $effect(() => {
    const v = input;
    if (!v) {
      hashes = null;
      return;
    }
    busy = true;
    api
      .hashText(v)
      .then((h) => (hashes = h))
      .catch(() => (hashes = null))
      .finally(() => (busy = false));
  });

  async function copyOne(key: keyof Hashes) {
    if (!hashes) return;
    await copy(hashes[key]);
    copiedKey = key;
    setTimeout(() => (copiedKey = ""), 1000);
  }
</script>

<div class="tool">
  <Pane label="Testo da hashare" bind:value={input} rows={6} placeholder="Scrivi o incolla qui…" />
  <div class="stack">
    <span class="field-label">Digest {busy ? "…" : ""}</span>
    {#each ALGOS as a (a.key)}
      <button class="hrow" onclick={() => copyOne(a.key)} disabled={!hashes}>
        <span class="halgo">{a.label}</span>
        <span class="hval mono">{hashes ? hashes[a.key] : "—"}</span>
        <span class="hcopy" class:ok={copiedKey === a.key}>{copiedKey === a.key ? "✓" : "⧉"}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .hrow {
    display: grid;
    grid-template-columns: 96px 1fr 28px;
    align-items: center;
    gap: 12px;
    text-align: left;
    padding: 11px 14px;
    border-radius: 12px;
    border: 1.5px solid var(--border);
    background: var(--surface);
    transition: border-color 0.14s ease, background 0.14s ease;
  }
  .hrow:hover:not(:disabled) {
    border-color: var(--primary);
    background: var(--row-hover);
  }
  .hrow:disabled {
    opacity: 0.55;
  }
  .halgo {
    font-weight: 700;
    font-size: 13px;
    color: var(--primary);
  }
  .hval {
    font-size: 12.5px;
    word-break: break-all;
    color: var(--ink);
  }
  .hcopy {
    color: var(--ink-faint);
    font-size: 14px;
  }
  .hcopy.ok {
    color: var(--green);
  }
</style>
