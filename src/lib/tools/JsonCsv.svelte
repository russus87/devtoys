<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let mode = $state("json2csv");
  let delim = $state(",");
  let input = $state("");

  function csvEscape(v: unknown, d: string): string {
    let s = v === null || v === undefined ? "" : String(v);
    if (typeof v === "object") s = JSON.stringify(v);
    if (s.includes(d) || s.includes('"') || s.includes("\n") || s.includes("\r")) {
      s = '"' + s.replace(/"/g, '""') + '"';
    }
    return s;
  }

  function jsonToCsv(text: string, d: string): string {
    const data = JSON.parse(text);
    const rows: Record<string, unknown>[] = Array.isArray(data) ? data : [data];
    if (rows.length === 0) return "";
    // Union of keys, preserving first-seen order.
    const cols: string[] = [];
    for (const r of rows) {
      for (const k of Object.keys(r ?? {})) if (!cols.includes(k)) cols.push(k);
    }
    const head = cols.map((c) => csvEscape(c, d)).join(d);
    const body = rows
      .map((r) => cols.map((c) => csvEscape((r ?? {})[c], d)).join(d))
      .join("\n");
    return head + "\n" + body;
  }

  // RFC 4180-ish CSV parser (handles quotes, embedded delimiters and newlines).
  function parseCsv(text: string, d: string): string[][] {
    const rows: string[][] = [];
    let row: string[] = [];
    let field = "";
    let inQuotes = false;
    for (let i = 0; i < text.length; i++) {
      const c = text[i];
      if (inQuotes) {
        if (c === '"') {
          if (text[i + 1] === '"') { field += '"'; i++; }
          else inQuotes = false;
        } else field += c;
      } else if (c === '"') {
        inQuotes = true;
      } else if (c === d) {
        row.push(field); field = "";
      } else if (c === "\n") {
        row.push(field); field = ""; rows.push(row); row = [];
      } else if (c === "\r") {
        // handled by the \n branch; skip
      } else field += c;
    }
    if (field.length > 0 || row.length > 0) { row.push(field); rows.push(row); }
    return rows.filter((r) => r.length > 1 || r[0] !== "");
  }

  function coerce(v: string): unknown {
    if (v === "") return "";
    if (v === "true") return true;
    if (v === "false") return false;
    if (v === "null") return null;
    if (/^-?\d+(\.\d+)?$/.test(v)) return Number(v);
    return v;
  }

  function csvToJson(text: string, d: string): string {
    const rows = parseCsv(text, d);
    if (rows.length === 0) return "[]";
    const [header, ...body] = rows;
    const objs = body.map((r) => {
      const o: Record<string, unknown> = {};
      header.forEach((h, i) => (o[h] = coerce(r[i] ?? "")));
      return o;
    });
    return JSON.stringify(objs, null, 2);
  }

  const computed = $derived.by(() => {
    if (!input.trim()) return { output: "", error: "" };
    try {
      const d = delim === "\\t" ? "\t" : delim;
      return { output: mode === "json2csv" ? jsonToCsv(input, d) : csvToJson(input, d), error: "" };
    } catch (e) {
      return { output: "", error: (e as Error).message };
    }
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={mode}
      options={[
        { value: "json2csv", label: "JSON → CSV" },
        { value: "csv2json", label: "CSV → JSON" },
      ]}
    />
    <div class="row">
      <span class="field-label">Delimitatore</span>
      <Segmented
        bind:value={delim}
        options={[
          { value: ",", label: "virgola" },
          { value: ";", label: "punto e virgola" },
          { value: "\\t", label: "tab" },
        ]}
      />
    </div>
  </div>
  <div class="split">
    <Pane
      label={mode === "json2csv" ? "JSON (array di oggetti)" : "CSV"}
      bind:value={input}
      placeholder={mode === "json2csv" ? '[{"nome":"Ada","età":36}]' : "nome,età\nAda,36"}
    />
    <div class="stack grow">
      <Pane label="Output" value={computed.output} readonly showPaste={false} />
      {#if computed.error}<div class="notice bad">{computed.error}</div>{/if}
    </div>
  </div>
</div>
