<script lang="ts">
  import Pane from "../ui/Pane.svelte";
  import Segmented from "../ui/Segmented.svelte";
  import { copy } from "../api";

  type Row = Record<string, string>;

  // --- helpers --------------------------------------------------------------
  function randInt(min: number, max: number): number {
    return Math.floor(Math.random() * (max - min + 1)) + min;
  }
  function randItem<T>(arr: T[]): T {
    return arr[Math.floor(Math.random() * arr.length)];
  }
  function pad(n: number, len: number): string {
    return String(n).padStart(len, "0");
  }

  // --- IBAN (IT) --------------------------------------------------------------
  function genIban(): string {
    const cin = randItem("ABCDEFGHIJKLMNOPQRSTUVWXYZ".split(""));
    const abi = pad(randInt(0, 99999), 5);
    const cab = pad(randInt(0, 99999), 5);
    const account = pad(randInt(0, 999999999999), 12);
    const bban = cin + abi + cab + account;
    // ISO 13616: move "IT00" to the end, convert letters (A=10..Z=35), mod 97
    const rearranged = bban + "IT00";
    const numeric = rearranged.replace(/[A-Z]/g, (ch) => (ch.charCodeAt(0) - 55).toString());
    let remainder = 0;
    for (const ch of numeric) remainder = (remainder * 10 + Number(ch)) % 97;
    const check = pad(98 - remainder, 2);
    return "IT" + check + bban;
  }

  // --- Codice Fiscale -----------------------------------------------------
  const CF_ODD: Record<string, number> = {
    "0": 1, "1": 0, "2": 5, "3": 7, "4": 9, "5": 13, "6": 15, "7": 17, "8": 19, "9": 21,
    A: 1, B: 0, C: 5, D: 7, E: 9, F: 13, G: 15, H: 17, I: 19, J: 21, K: 2, L: 4, M: 18,
    N: 20, O: 11, P: 3, Q: 6, R: 8, S: 12, T: 14, U: 16, V: 10, W: 22, X: 25, Y: 24, Z: 23,
  };
  const CF_EVEN: Record<string, number> = {
    "0": 0, "1": 1, "2": 2, "3": 3, "4": 4, "5": 5, "6": 6, "7": 7, "8": 8, "9": 9,
    A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12,
    N: 13, O: 14, P: 15, Q: 16, R: 17, S: 18, T: 19, U: 20, V: 21, W: 22, X: 23, Y: 24, Z: 25,
  };
  const CF_REMAINDER = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

  function cfCheckChar(code15: string): string {
    let sum = 0;
    for (let i = 0; i < 15; i++) {
      const ch = code15[i];
      const pos = i + 1; // 1-indexed
      sum += pos % 2 === 1 ? CF_ODD[ch] : CF_EVEN[ch];
    }
    return CF_REMAINDER[sum % 26];
  }

  function genCf(): string {
    const consonants = "BCDFGHJKLMNPQRSTVWXYZ".split("");
    const surname = Array.from({ length: 3 }, () => randItem(consonants)).join("");
    const name = Array.from({ length: 3 }, () => randItem(consonants)).join("");
    const year = pad(randInt(0, 99), 2);
    const months = ["A", "B", "C", "D", "E", "H", "L", "M", "P", "R", "S", "T"];
    const monthLetter = randItem(months);
    const female = Math.random() < 0.5;
    let day = randInt(1, 28);
    if (female) day += 40;
    const dayStr = pad(day, 2);
    const cadastralLetter = randItem("ABCDEFGHIJKLMNOPQRSTUVWXYZ".split(""));
    const cadastralNum = pad(randInt(0, 999), 3);
    const code15 = surname + name + year + monthLetter + dayStr + cadastralLetter + cadastralNum;
    const check = cfCheckChar(code15);
    return (code15 + check).toUpperCase();
  }

  // --- Partita IVA ----------------------------------------------------------
  function genPiva(): string {
    const digits = Array.from({ length: 10 }, () => randInt(0, 9));
    let sum = 0;
    digits.forEach((d, i) => {
      const pos = i + 1;
      if (pos % 2 === 1) {
        sum += d;
      } else {
        let v = d * 2;
        if (v > 9) v -= 9;
        sum += v;
      }
    });
    const check = (10 - (sum % 10)) % 10;
    return digits.join("") + String(check);
  }

  // --- Persona --------------------------------------------------------------
  const NOMI = [
    "Marco", "Giulia", "Luca", "Sara", "Andrea", "Elena", "Davide", "Francesca",
    "Matteo", "Chiara", "Alessandro", "Valentina", "Simone", "Martina", "Federico",
    "Giorgia", "Riccardo", "Silvia", "Lorenzo", "Alice",
  ];
  const COGNOMI = [
    "Rossi", "Russo", "Ferrari", "Esposito", "Bianchi", "Romano", "Colombo", "Ricci",
    "Marino", "Greco", "Bruno", "Gallo", "Conti", "DeLuca", "Costa", "Giordano",
    "Mancini", "Rizzo", "Lombardi", "Moretti",
  ];
  function genPersona(): Row {
    const nome = randItem(NOMI);
    const cognome = randItem(COGNOMI);
    const email = `${nome.toLowerCase()}.${cognome.toLowerCase()}@example.it`;
    const telefono = "3" + Array.from({ length: 9 }, () => randInt(0, 9)).join("");
    return { nome, cognome, email, telefono };
  }

  function generateRows(kind: string, count: number): Row[] {
    const rows: Row[] = [];
    for (let i = 0; i < count; i++) {
      if (kind === "iban") rows.push({ iban: genIban() });
      else if (kind === "cf") rows.push({ codice_fiscale: genCf() });
      else if (kind === "piva") rows.push({ partita_iva: genPiva() });
      else rows.push(genPersona());
    }
    return rows;
  }

  function tableNameFor(kind: string): string {
    switch (kind) {
      case "iban": return "iban_generati";
      case "cf": return "codici_fiscali";
      case "piva": return "partite_iva";
      default: return "persone";
    }
  }

  function toList(kind: string, rows: Row[]): string {
    if (kind === "person") {
      return rows.map((r) => `${r.nome} ${r.cognome} <${r.email}> ${r.telefono}`).join("\n");
    }
    return rows.map((r) => Object.values(r)[0]).join("\n");
  }
  function toJson(rows: Row[]): string {
    return JSON.stringify(rows, null, 2);
  }
  function toCsv(rows: Row[]): string {
    if (rows.length === 0) return "";
    const cols = Object.keys(rows[0]);
    const header = cols.join(",");
    const body = rows.map((r) => cols.map((c) => r[c]).join(",")).join("\n");
    return header + "\n" + body;
  }
  function toSql(kind: string, rows: Row[]): string {
    if (rows.length === 0) return "";
    const cols = Object.keys(rows[0]);
    const table = tableNameFor(kind);
    return rows
      .map((r) => {
        const vals = cols.map((c) => `'${String(r[c]).replace(/'/g, "''")}'`).join(", ");
        return `INSERT INTO ${table} (${cols.join(", ")}) VALUES (${vals});`;
      })
      .join("\n");
  }

  let kind = $state("iban");
  let count = $state(10);
  let format = $state("list");
  let rows = $state<Row[]>([]);
  let copied = $state(false);

  function generate() {
    const n = Math.min(500, Math.max(1, Math.trunc(count) || 1));
    count = n;
    rows = generateRows(kind, n);
  }

  $effect(() => {
    // regenerate whenever the kind or the requested count changes
    kind;
    count;
    generate();
  });

  const output = $derived.by((): string => {
    switch (format) {
      case "json": return toJson(rows);
      case "csv": return toCsv(rows);
      case "sql": return toSql(kind, rows);
      default: return toList(kind, rows);
    }
  });

  async function copyOutput() {
    if (!output) return;
    await copy(output);
    copied = true;
    setTimeout(() => (copied = false), 1100);
  }
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <Segmented
      bind:value={kind}
      options={[
        { value: "iban", label: "IBAN (IT)" },
        { value: "cf", label: "Codice fiscale" },
        { value: "piva", label: "Partita IVA" },
        { value: "person", label: "Persona" },
      ]}
    />
    <div class="row">
      <span class="field-label">Quantità</span>
      <input class="input" type="number" min="1" max="500" style="width:90px" bind:value={count} />
    </div>
  </div>

  <div class="tool-controls wrap">
    <Segmented
      bind:value={format}
      options={[
        { value: "list", label: "Elenco" },
        { value: "json", label: "JSON" },
        { value: "csv", label: "CSV" },
        { value: "sql", label: "SQL" },
      ]}
    />
    <div class="spacer"></div>
    <button class="btn primary" onclick={generate}>Genera</button>
    <button class="btn" class:ok={copied} onclick={copyOutput}>{copied ? "Copiato ✓" : "Copia"}</button>
  </div>

  <Pane label="Dati generati" value={output} readonly showPaste={false} rows={16} />
</div>

<style>
  .btn.ok {
    color: var(--green);
  }
</style>
