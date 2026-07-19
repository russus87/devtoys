<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";

  let target = $state("fetch");
  let input = $state("");

  interface Req {
    method: string;
    url: string;
    headers: [string, string][];
    data: string | null;
    form: [string, string][];
    user: string | null;
  }

  // Tokenizer: splits a shell-ish command line honoring '…', "…" and \ escapes,
  // plus backslash line-continuations.
  function tokenize(cmd: string): string[] {
    const s = cmd.replace(/\\\r?\n/g, " ");
    const out: string[] = [];
    let i = 0;
    const n = s.length;
    while (i < n) {
      while (i < n && /\s/.test(s[i])) i++;
      if (i >= n) break;
      let tok = "";
      while (i < n && !/\s/.test(s[i])) {
        const c = s[i];
        if (c === "'") {
          i++;
          while (i < n && s[i] !== "'") { tok += s[i]; i++; }
          i++;
        } else if (c === '"') {
          i++;
          while (i < n && s[i] !== '"') {
            if (s[i] === "\\" && i + 1 < n && (s[i + 1] === '"' || s[i + 1] === "\\")) { tok += s[i + 1]; i += 2; }
            else { tok += s[i]; i++; }
          }
          i++;
        } else if (c === "\\" && i + 1 < n) {
          tok += s[i + 1]; i += 2;
        } else {
          tok += c; i++;
        }
      }
      out.push(tok);
    }
    return out;
  }

  function parse(cmd: string): Req {
    const toks = tokenize(cmd.trim());
    const req: Req = { method: "", url: "", headers: [], data: null, form: [], user: null };
    let i = 0;
    if (toks[0] === "curl") i = 1;
    const addData = (v: string) => { req.data = req.data === null ? v : req.data + "&" + v; };
    const takesArgSkip = new Set(["-o", "--output", "--connect-timeout", "--max-time", "-m", "-w", "--write-out", "-x", "--proxy", "--cacert", "--cert", "--key", "-O"]);
    for (; i < toks.length; i++) {
      const t = toks[i];
      if (t === "-X" || t === "--request") { req.method = toks[++i] ?? ""; }
      else if (t === "-H" || t === "--header") {
        const h = toks[++i] ?? "";
        const idx = h.indexOf(":");
        if (idx > -1) req.headers.push([h.slice(0, idx).trim(), h.slice(idx + 1).trim()]);
      }
      else if (t === "-d" || t === "--data" || t === "--data-raw" || t === "--data-ascii" || t === "--data-binary" || t === "--data-urlencode") { addData(toks[++i] ?? ""); }
      else if (t === "-F" || t === "--form") {
        const f = toks[++i] ?? "";
        const idx = f.indexOf("=");
        if (idx > -1) req.form.push([f.slice(0, idx), f.slice(idx + 1)]);
      }
      else if (t === "-u" || t === "--user") { req.user = toks[++i] ?? ""; }
      else if (t === "-b" || t === "--cookie") { req.headers.push(["Cookie", toks[++i] ?? ""]); }
      else if (t === "-A" || t === "--user-agent") { req.headers.push(["User-Agent", toks[++i] ?? ""]); }
      else if (t === "-e" || t === "--referer") { req.headers.push(["Referer", toks[++i] ?? ""]); }
      else if (t === "--url") { req.url = toks[++i] ?? ""; }
      else if (takesArgSkip.has(t)) { i++; }
      else if (t.startsWith("-")) { /* boolean flag: --compressed, -L, -k, -s, -i, -v … ignore */ }
      else if (!req.url) { req.url = t; }
    }
    if (!req.method) req.method = req.data !== null || req.form.length ? "POST" : "GET";
    return req;
  }

  const q = (s: string) => JSON.stringify(s);

  function genFetch(r: Req): string {
    const headers = [...r.headers];
    if (r.user) headers.push(["Authorization", "Basic " + b64(r.user)]);
    const lines: string[] = [];
    const opts: string[] = [`  method: ${q(r.method)},`];
    if (headers.length) {
      opts.push("  headers: {");
      for (const [k, v] of headers) opts.push(`    ${q(k)}: ${q(v)},`);
      opts.push("  },");
    }
    if (r.form.length) {
      lines.push("const form = new FormData();");
      for (const [k, v] of r.form) lines.push(`form.append(${q(k)}, ${q(v)});`);
      opts.push("  body: form,");
    } else if (r.data !== null) {
      opts.push(`  body: ${q(r.data)},`);
    }
    lines.push(`fetch(${q(r.url)}, {`, ...opts, "})");
    lines.push("  .then((res) => res.json())", "  .then(console.log);");
    return lines.join("\n");
  }

  function genPython(r: Req): string {
    const lines = ["import requests", ""];
    if (r.headers.length) {
      lines.push("headers = {");
      for (const [k, v] of r.headers) lines.push(`    ${q(k)}: ${q(v)},`);
      lines.push("}");
    }
    const args = [q(r.method), q(r.url)];
    if (r.headers.length) args.push("headers=headers");
    if (r.form.length) {
      lines.push("files = {");
      for (const [k, v] of r.form) lines.push(`    ${q(k)}: (None, ${q(v)}),`);
      lines.push("}");
      args.push("files=files");
    } else if (r.data !== null) {
      args.push(`data=${q(r.data)}`);
    }
    if (r.user) {
      const [u, ...rest] = r.user.split(":");
      args.push(`auth=(${q(u)}, ${q(rest.join(":"))})`);
    }
    lines.push("", `resp = requests.request(${args.join(", ")})`, "print(resp.text)");
    return lines.join("\n");
  }

  function genAxios(r: Req): string {
    const headers = [...r.headers];
    if (r.user) headers.push(["Authorization", "Basic " + b64(r.user)]);
    const lines = ['const axios = require("axios");', "", "axios({"];
    lines.push(`  method: ${q(r.method)},`, `  url: ${q(r.url)},`);
    if (headers.length) {
      lines.push("  headers: {");
      for (const [k, v] of headers) lines.push(`    ${q(k)}: ${q(v)},`);
      lines.push("  },");
    }
    if (r.data !== null && !r.form.length) lines.push(`  data: ${q(r.data)},`);
    lines.push("})", "  .then((res) => console.log(res.data))", "  .catch(console.error);");
    return lines.join("\n");
  }

  function genHttpie(r: Req): string {
    const parts = ["http", r.method, r.url];
    for (const [k, v] of r.headers) parts.push(`${k}:${q(v)}`);
    if (r.user) parts.push(`-a ${r.user}`);
    let s = parts.join(" ");
    if (r.data !== null) s += `\n# body:\n# ${r.data}`;
    return s;
  }

  function b64(s: string): string {
    // btoa over UTF-8 bytes
    const bytes = new TextEncoder().encode(s);
    let bin = "";
    for (const b of bytes) bin += String.fromCharCode(b);
    return btoa(bin);
  }

  const computed = $derived.by(() => {
    if (!input.trim()) return { output: "", error: "" };
    try {
      const r = parse(input);
      if (!r.url) return { output: "", error: "Nessun URL trovato nel comando curl." };
      const gen = target === "python" ? genPython : target === "axios" ? genAxios : target === "httpie" ? genHttpie : genFetch;
      return { output: gen(r), error: "" };
    } catch (e) {
      return { output: "", error: (e as Error).message };
    }
  });
  const output = $derived(computed.output);
  const error = $derived(computed.error);
</script>

<div class="tool">
  <div class="tool-controls">
    <Segmented
      bind:value={target}
      options={[
        { value: "fetch", label: "fetch (JS)" },
        { value: "python", label: "Python requests" },
        { value: "axios", label: "axios" },
        { value: "httpie", label: "HTTPie" },
      ]}
    />
  </div>
  <div class="split">
    <Pane
      label="Comando cURL"
      bind:value={input}
      rows={12}
      placeholder={"curl -X POST https://api.example.com/v1/login \\\n  -H 'Content-Type: application/json' \\\n  -d '{\"user\":\"ada\"}'"}
    />
    <div class="stack grow">
      <Pane label="Codice" value={output} readonly showPaste={false} rows={12} />
      {#if error}<div class="notice bad">{error}</div>{/if}
    </div>
  </div>
</div>
