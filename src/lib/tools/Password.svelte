<script lang="ts">
  import { copy } from "../api";

  let length = $state(20);
  let useLower = $state(true);
  let useUpper = $state(true);
  let useDigits = $state(true);
  let useSymbols = $state(false);
  let excludeAmbiguous = $state(false);

  let password = $state("");
  let copied = $state(false);

  const LOWER = "abcdefghijklmnopqrstuvwxyz";
  const UPPER = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
  const DIGITS = "0123456789";
  const SYMBOLS = "!@#$%^&*()-_=+[]{};:,.?";
  const AMBIGUOUS = "il1Lo0O";

  const charset = $derived.by(() => {
    let s = "";
    if (useLower) s += LOWER;
    if (useUpper) s += UPPER;
    if (useDigits) s += DIGITS;
    if (useSymbols) s += SYMBOLS;
    if (excludeAmbiguous) {
      s = Array.from(s)
        .filter((c) => !AMBIGUOUS.includes(c))
        .join("");
    }
    return s;
  });

  const entropyBits = $derived.by(() => {
    if (!charset || !password) return 0;
    return Math.round(length * Math.log2(charset.length));
  });

  function generate() {
    const cs = charset;
    if (!cs) {
      password = "";
      return;
    }
    const n = Math.min(64, Math.max(4, Math.trunc(length) || 4));
    length = n;
    const rand = new Uint32Array(n);
    crypto.getRandomValues(rand);
    let out = "";
    for (let i = 0; i < n; i++) {
      out += cs[rand[i] % cs.length];
    }
    password = out;
  }

  $effect(() => {
    // Recompute whenever any option changes.
    void length;
    void useLower;
    void useUpper;
    void useDigits;
    void useSymbols;
    void excludeAmbiguous;
    generate();
  });

  async function copyPassword() {
    if (!password) return;
    await copy(password);
    copied = true;
    setTimeout(() => (copied = false), 1100);
  }
</script>

<div class="tool">
  <div class="stack">
    <div class="row">
      <span class="field-label">Lunghezza</span>
      <input class="range" type="range" min="4" max="64" bind:value={length} />
      <span class="pill">{length}</span>
    </div>

    <div class="row wrap checks">
      <label class="check">
        <input type="checkbox" bind:checked={useLower} />
        <span>minuscole (a-z)</span>
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={useUpper} />
        <span>MAIUSCOLE (A-Z)</span>
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={useDigits} />
        <span>numeri (0-9)</span>
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={useSymbols} />
        <span>simboli (!@#$…)</span>
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={excludeAmbiguous} />
        <span>Escludi caratteri ambigui</span>
      </label>
    </div>

    {#if !charset}
      <div class="notice bad">Seleziona almeno un tipo</div>
    {:else}
      <div class="card-box mono password-display">{password}</div>
      <div class="row">
        <span class="faint">~{entropyBits} bit</span>
        <div class="spacer"></div>
        <button class="btn" onclick={generate}>Rigenera</button>
        <button class="btn primary" onclick={copyPassword}>{copied ? "✓ Copiato" : "Copia"}</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .range {
    flex: 1;
    accent-color: var(--primary);
  }
  .checks {
    gap: 16px;
  }
  .check {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--ink-dim);
    cursor: pointer;
  }
  .check input {
    accent-color: var(--primary);
  }
  .password-display {
    font-size: 22px;
    letter-spacing: 1px;
    text-align: center;
    padding: 22px 16px;
    word-break: break-all;
  }
</style>
