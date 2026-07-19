<script lang="ts">
  import Pane from "../ui/Pane.svelte";

  let left = $state("");
  let right = $state("");

  interface Change {
    path: string;
    kind: "added" | "removed" | "changed";
    a?: unknown;
    b?: unknown;
  }

  function eq(a: unknown, b: unknown): boolean {
    if (a === b) return true;
    if (a && b && typeof a === "object" && typeof b === "object") {
      const aa = a as Record<string, unknown>;
      const bb = b as Record<string, unknown>;
      if (Array.isArray(a) !== Array.isArray(b)) return false;
      const ka = Object.keys(aa);
      const kb = Object.keys(bb);
      if (ka.length !== kb.length) return false;
      for (const k of ka) if (!(k in bb) || !eq(aa[k], bb[k])) return false;
      return true;
    }
    return false;
  }

  function join(path: string, key: string, isArr: boolean): string {
    if (!path) return isArr ? `[${key}]` : key;
    return isArr ? `${path}[${key}]` : `${path}.${key}`;
  }

  function diff(a: unknown, b: unknown, path: string, acc: Change[]): void {
    if (eq(a, b)) return;
    const bothObj =
      a && b && typeof a === "object" && typeof b === "object" &&
      Array.isArray(a) === Array.isArray(b);
    if (!bothObj) {
      acc.push({ path: path || "(radice)", kind: "changed", a, b });
      return;
    }
    const aa = a as Record<string, unknown>;
    const bb = b as Record<string, unknown>;
    const isArr = Array.isArray(a);
    const keys = new Set([...Object.keys(aa), ...Object.keys(bb)]);
    for (const k of keys) {
      const p = join(path, k, isArr);
      const hasA = k in aa;
      const hasB = k in bb;
      if (!hasA) acc.push({ path: p, kind: "added", b: bb[k] });
      else if (!hasB) acc.push({ path: p, kind: "removed", a: aa[k] });
      else diff(aa[k], bb[k], p, acc);
    }
  }

  function show(v: unknown): string {
    const s = JSON.stringify(v);
    if (s === undefined) return String(v);
    return s.length > 160 ? s.slice(0, 157) + "…" : s;
  }

  const computed = $derived.by(() => {
    if (!left.trim() || !right.trim()) return { output: "", error: "", summary: "" };
    let a: unknown, b: unknown;
    try {
      a = JSON.parse(left);
    } catch (e) {
      return { output: "", error: "JSON di sinistra: " + (e as Error).message, summary: "" };
    }
    try {
      b = JSON.parse(right);
    } catch (e) {
      return { output: "", error: "JSON di destra: " + (e as Error).message, summary: "" };
    }
    const acc: Change[] = [];
    diff(a, b, "", acc);
    acc.sort((x, y) => x.path.localeCompare(y.path));
    if (acc.length === 0) return { output: "", error: "", summary: "identici" };
    const lines = acc.map((c) => {
      if (c.kind === "added") return `+ ${c.path}: ${show(c.b)}`;
      if (c.kind === "removed") return `- ${c.path}: ${show(c.a)}`;
      return `~ ${c.path}: ${show(c.a)} → ${show(c.b)}`;
    });
    const nAdd = acc.filter((c) => c.kind === "added").length;
    const nDel = acc.filter((c) => c.kind === "removed").length;
    const nChg = acc.filter((c) => c.kind === "changed").length;
    return {
      output: lines.join("\n"),
      error: "",
      summary: `${acc.length} differenze — +${nAdd} aggiunte, −${nDel} rimosse, ~${nChg} modificate`,
    };
  });
  const output = $derived(computed.output);
  const error = $derived(computed.error);
  const summary = $derived(computed.summary);
</script>

<div class="tool">
  <div class="split">
    <Pane label="JSON A" bind:value={left} placeholder={'{ "nome": "Ada", "ruolo": "dev" }'} />
    <Pane label="JSON B" bind:value={right} placeholder={'{ "nome": "Ada", "ruolo": "lead", "team": "core" }'} />
  </div>
  {#if error}
    <div class="notice bad">{error}</div>
  {:else if summary === "identici"}
    <div class="notice good">I due JSON sono identici.</div>
  {:else if summary}
    <div class="notice">{summary}</div>
    <Pane label="Differenze" value={output} readonly showPaste={false} rows={12} />
  {/if}
</div>
