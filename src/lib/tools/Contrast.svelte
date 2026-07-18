<script lang="ts">
  let fg = $state("#2c2a47");
  let bg = $state("#ffffff");

  function parseHex(h: string): [number, number, number] | null {
    let s = h.trim().replace(/^#/, "");
    if (s.length === 3) s = s.split("").map((c) => c + c).join("");
    if (!/^[0-9a-fA-F]{6}$/.test(s)) return null;
    return [
      parseInt(s.slice(0, 2), 16),
      parseInt(s.slice(2, 4), 16),
      parseInt(s.slice(4, 6), 16),
    ];
  }

  function luminance([r, g, b]: [number, number, number]): number {
    const a = [r, g, b].map((v) => {
      const c = v / 255;
      return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
    });
    return 0.2126 * a[0] + 0.7152 * a[1] + 0.0722 * a[2];
  }

  const result = $derived.by(() => {
    const f = parseHex(fg);
    const b = parseHex(bg);
    if (!f || !b) return null;
    const l1 = luminance(f);
    const l2 = luminance(b);
    const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
    return {
      ratio,
      aaNormal: ratio >= 4.5,
      aaLarge: ratio >= 3,
      aaaNormal: ratio >= 7,
      aaaLarge: ratio >= 4.5,
    };
  });

  interface Check {
    label: string;
    pass: boolean;
  }
  const checks = $derived.by((): Check[] => {
    if (!result) return [];
    return [
      { label: "AA · testo normale (≥ 4.5)", pass: result.aaNormal },
      { label: "AA · testo grande (≥ 3)", pass: result.aaLarge },
      { label: "AAA · testo normale (≥ 7)", pass: result.aaaNormal },
      { label: "AAA · testo grande (≥ 4.5)", pass: result.aaaLarge },
    ];
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <div class="row">
      <span class="field-label">Testo</span>
      <input type="color" class="swatch" bind:value={fg} />
      <input class="input hex" bind:value={fg} spellcheck="false" />
    </div>
    <div class="row">
      <span class="field-label">Sfondo</span>
      <input type="color" class="swatch" bind:value={bg} />
      <input class="input hex" bind:value={bg} spellcheck="false" />
    </div>
  </div>

  <div class="preview" style="background:{bg};color:{fg}">
    <div class="big">Testo grande di esempio</div>
    <div class="small">Testo normale di esempio — Lorem ipsum dolor sit amet.</div>
  </div>

  {#if result}
    <div class="ratio">
      <span class="ratio-num">{result.ratio.toFixed(2)}</span>
      <span class="ratio-x">:1</span>
      <span class="muted">rapporto di contrasto</span>
    </div>
    <div class="checks">
      {#each checks as c (c.label)}
        <div class="check" class:pass={c.pass}>
          <span class="dot">{c.pass ? "✓" : "✕"}</span>
          {c.label}
        </div>
      {/each}
    </div>
  {:else}
    <div class="notice bad">Inserisci due colori HEX validi (es. #1a2b3c).</div>
  {/if}
</div>

<style>
  .swatch {
    width: 38px;
    height: 34px;
    padding: 0;
    border: 1.5px solid var(--border-2);
    border-radius: 9px;
    background: none;
    cursor: pointer;
  }
  .hex {
    width: 110px;
    font-family: var(--mono);
  }
  .preview {
    border-radius: var(--radius-lg);
    border: 1.5px solid var(--border);
    padding: 26px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .preview .big {
    font-size: 26px;
    font-weight: 800;
  }
  .preview .small {
    font-size: 14px;
  }
  .ratio {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .ratio-num {
    font-size: 34px;
    font-weight: 800;
    font-family: var(--mono);
  }
  .ratio-x {
    font-size: 18px;
    font-weight: 700;
    color: var(--ink-dim);
  }
  .checks {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 10px;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 11px 14px;
    border-radius: 12px;
    font-size: 13px;
    font-weight: 600;
    background: color-mix(in srgb, var(--red) 9%, var(--surface));
    border: 1.5px solid color-mix(in srgb, var(--red) 28%, var(--border));
    color: var(--red);
  }
  .check.pass {
    background: color-mix(in srgb, var(--green) 9%, var(--surface));
    border-color: color-mix(in srgb, var(--green) 28%, var(--border));
    color: var(--green);
  }
  .dot {
    font-weight: 800;
  }
</style>
