<script lang="ts">
  import { copy } from "../api";

  let text = $state("#3366ff");
  let copiedKey = $state("");

  type RGB = { r: number; g: number; b: number };
  type HSL = { h: number; s: number; l: number };

  function clamp255(n: number): number {
    return Math.max(0, Math.min(255, n));
  }

  function parseColor(input: string): RGB | null {
    const s = input.trim();
    if (!s) return null;

    // #rgb or #rrggbb
    const hexMatch = s.match(/^#?([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/);
    if (hexMatch) {
      let h = hexMatch[1];
      if (h.length === 3) {
        h = h
          .split("")
          .map((c) => c + c)
          .join("");
      }
      const r = parseInt(h.slice(0, 2), 16);
      const g = parseInt(h.slice(2, 4), 16);
      const b = parseInt(h.slice(4, 6), 16);
      return { r, g, b };
    }

    // rgb(r, g, b) or rgba(r, g, b, a)
    const rgbMatch = s.match(/^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*(?:,\s*[\d.]+\s*)?\)$/i);
    if (rgbMatch) {
      const r = clamp255(Math.round(Number(rgbMatch[1])));
      const g = clamp255(Math.round(Number(rgbMatch[2])));
      const b = clamp255(Math.round(Number(rgbMatch[3])));
      return { r, g, b };
    }

    // hsl(h, s%, l%) or hsla(...)
    const hslMatch = s.match(/^hsla?\(\s*([\d.]+)\s*,\s*([\d.]+)%\s*,\s*([\d.]+)%\s*(?:,\s*[\d.]+\s*)?\)$/i);
    if (hslMatch) {
      const h = Number(hslMatch[1]);
      const sat = Number(hslMatch[2]);
      const l = Number(hslMatch[3]);
      return hslToRgb({ h, s: sat, l });
    }

    return null;
  }

  function rgbToHex({ r, g, b }: RGB): string {
    const h = (n: number) => n.toString(16).padStart(2, "0");
    return "#" + h(r) + h(g) + h(b);
  }

  function rgbToHsl({ r, g, b }: RGB): HSL {
    const rn = r / 255;
    const gn = g / 255;
    const bn = b / 255;
    const max = Math.max(rn, gn, bn);
    const min = Math.min(rn, gn, bn);
    const l = (max + min) / 2;
    let h = 0;
    let s = 0;
    const d = max - min;
    if (d !== 0) {
      s = d / (1 - Math.abs(2 * l - 1));
      switch (max) {
        case rn:
          h = 60 * (((gn - bn) / d) % 6);
          break;
        case gn:
          h = 60 * ((bn - rn) / d + 2);
          break;
        case bn:
          h = 60 * ((rn - gn) / d + 4);
          break;
      }
    }
    if (h < 0) h += 360;
    return { h: Math.round(h), s: Math.round(s * 100), l: Math.round(l * 100) };
  }

  function hslToRgb({ h, s, l }: HSL): RGB {
    const sn = s / 100;
    const ln = l / 100;
    const c = (1 - Math.abs(2 * ln - 1)) * sn;
    const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
    const m = ln - c / 2;
    let rp = 0;
    let gp = 0;
    let bp = 0;
    if (h < 60) {
      rp = c;
      gp = x;
    } else if (h < 120) {
      rp = x;
      gp = c;
    } else if (h < 180) {
      gp = c;
      bp = x;
    } else if (h < 240) {
      gp = x;
      bp = c;
    } else if (h < 300) {
      rp = x;
      bp = c;
    } else {
      rp = c;
      bp = x;
    }
    return {
      r: clamp255(Math.round((rp + m) * 255)),
      g: clamp255(Math.round((gp + m) * 255)),
      b: clamp255(Math.round((bp + m) * 255)),
    };
  }

  const rgb = $derived.by(() => parseColor(text));
  const hex = $derived.by(() => (rgb ? rgbToHex(rgb) : null));
  const hsl = $derived.by(() => (rgb ? rgbToHsl(rgb) : null));

  const rgbStr = $derived.by(() => (rgb ? `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})` : ""));
  const hslStr = $derived.by(() => (hsl ? `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)` : ""));

  function onPicker(e: Event) {
    text = (e.target as HTMLInputElement).value;
  }

  async function doCopy(key: string, val: string) {
    if (!val) return;
    await copy(val);
    copiedKey = key;
    setTimeout(() => {
      if (copiedKey === key) copiedKey = "";
    }, 1100);
  }
</script>

<div class="tool">
  <div class="split">
    <div class="stack">
      <div class="stack">
        <span class="field-label">Colore</span>
        <div class="row">
          <input
            class="input"
            type="text"
            bind:value={text}
            placeholder="#3366ff, rgb(51,102,255), hsl(225,100%,60%)…"
            spellcheck="false"
            autocapitalize="off"
            autocomplete="off"
          />
          <input class="picker" type="color" value={hex ?? "#000000"} oninput={onPicker} aria-label="Selettore colore" />
        </div>
      </div>

      {#if !rgb}
        <div class="notice bad">Colore non valido</div>
      {:else}
        <div class="swatch" style="background: {hex};"></div>
      {/if}
    </div>

    <div class="stack">
      <div class="rows">
        <div class="row-item">
          <span class="pill">HEX</span>
          <span class="mono val">{hex ?? "—"}</span>
          <button class="btn sm" class:ok={copiedKey === "hex"} disabled={!hex} onclick={() => doCopy("hex", hex ?? "")}>
            {copiedKey === "hex" ? "✓" : "Copia"}
          </button>
        </div>
        <div class="row-item">
          <span class="pill">RGB</span>
          <span class="mono val">{rgbStr || "—"}</span>
          <button class="btn sm" class:ok={copiedKey === "rgb"} disabled={!rgbStr} onclick={() => doCopy("rgb", rgbStr)}>
            {copiedKey === "rgb" ? "✓" : "Copia"}
          </button>
        </div>
        <div class="row-item">
          <span class="pill">HSL</span>
          <span class="mono val">{hslStr || "—"}</span>
          <button class="btn sm" class:ok={copiedKey === "hsl"} disabled={!hslStr} onclick={() => doCopy("hsl", hslStr)}>
            {copiedKey === "hsl" ? "✓" : "Copia"}
          </button>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .picker {
    width: 44px;
    height: 40px;
    padding: 2px;
    border-radius: var(--radius);
    border: 1.5px solid var(--border-2);
    background: var(--surface);
    cursor: pointer;
  }
  .swatch {
    height: 120px;
    border-radius: var(--radius-lg);
    border: 1.5px solid var(--border-2);
    box-shadow: var(--shadow-sm);
  }
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
