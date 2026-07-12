<script lang="ts">
  import { copy } from "../api";

  let epochInput = $state("");
  let dateInput = $state("");
  let copiedKey = $state("");

  function now() {
    epochInput = String(Math.floor(Date.now() / 1000));
  }

  const epochDate = $derived.by(() => {
    const v = epochInput.trim();
    if (!v) return null;
    const n = Number(v);
    if (Number.isNaN(n)) return null;
    const ms = Math.abs(n) > 1e12 ? n : n * 1000;
    const d = new Date(ms);
    if (Number.isNaN(d.getTime())) return null;
    return d;
  });

  const parsedDate = $derived.by(() => {
    const v = dateInput.trim();
    if (!v) return null;
    const d = new Date(v);
    if (Number.isNaN(d.getTime())) return null;
    return d;
  });

  async function doCopy(key: string, val: string) {
    await copy(val);
    copiedKey = key;
    setTimeout(() => {
      if (copiedKey === key) copiedKey = "";
    }, 1100);
  }
</script>

<div class="tool">
  <div class="card-box stack">
    <span class="field-label">Timestamp → Data</span>
    <div class="row">
      <input
        class="input"
        type="text"
        bind:value={epochInput}
        placeholder="Timestamp Unix (secondi o millisecondi)…"
        spellcheck="false"
        autocapitalize="off"
        autocomplete="off"
      />
      <button class="btn" onclick={now}>Adesso</button>
    </div>

    {#if epochInput.trim() && !epochDate}
      <div class="notice bad">Timestamp non valido</div>
    {:else if epochDate}
      <div class="rows">
        <div class="row-item">
          <span class="pill">Locale</span>
          <span class="mono val">{epochDate.toLocaleString("it-IT")}</span>
          <button class="btn sm" class:ok={copiedKey === "locale"} onclick={() => doCopy("locale", epochDate.toLocaleString("it-IT"))}>
            {copiedKey === "locale" ? "✓" : "Copia"}
          </button>
        </div>
        <div class="row-item">
          <span class="pill">UTC</span>
          <span class="mono val">{epochDate.toUTCString()}</span>
          <button class="btn sm" class:ok={copiedKey === "utc"} onclick={() => doCopy("utc", epochDate.toUTCString())}>
            {copiedKey === "utc" ? "✓" : "Copia"}
          </button>
        </div>
        <div class="row-item">
          <span class="pill">ISO 8601</span>
          <span class="mono val">{epochDate.toISOString()}</span>
          <button class="btn sm" class:ok={copiedKey === "iso"} onclick={() => doCopy("iso", epochDate.toISOString())}>
            {copiedKey === "iso" ? "✓" : "Copia"}
          </button>
        </div>
      </div>
    {/if}
  </div>

  <div class="card-box stack">
    <span class="field-label">Data → Timestamp</span>
    <input
      class="input"
      type="text"
      bind:value={dateInput}
      placeholder="es. 2026-07-12T10:00:00 oppure 12/07/2026…"
      spellcheck="false"
      autocapitalize="off"
      autocomplete="off"
    />

    {#if dateInput.trim() && !parsedDate}
      <div class="notice bad">Data non valida</div>
    {:else if parsedDate}
      <div class="rows">
        <div class="row-item">
          <span class="pill">Secondi</span>
          <span class="mono val">{Math.floor(parsedDate.getTime() / 1000)}</span>
          <button
            class="btn sm"
            class:ok={copiedKey === "sec"}
            onclick={() => doCopy("sec", String(Math.floor(parsedDate.getTime() / 1000)))}
          >
            {copiedKey === "sec" ? "✓" : "Copia"}
          </button>
        </div>
        <div class="row-item">
          <span class="pill">Millisecondi</span>
          <span class="mono val">{parsedDate.getTime()}</span>
          <button class="btn sm" class:ok={copiedKey === "ms"} onclick={() => doCopy("ms", String(parsedDate.getTime()))}>
            {copiedKey === "ms" ? "✓" : "Copia"}
          </button>
        </div>
      </div>
    {/if}
  </div>
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
    transition: background 0.14s ease, border-color 0.14s ease;
  }
  .row-item:hover {
    background: var(--surface-2);
    border-color: var(--border-2);
  }
  .val {
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    white-space: nowrap;
    color: var(--ink);
  }
  .btn.ok {
    color: var(--green);
  }
</style>
