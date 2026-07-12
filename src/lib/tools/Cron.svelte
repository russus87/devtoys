<script lang="ts">
  type FieldKind = "second" | "minute" | "hour" | "dom" | "month" | "dow";

  const monthNames = [
    "gennaio",
    "febbraio",
    "marzo",
    "aprile",
    "maggio",
    "giugno",
    "luglio",
    "agosto",
    "settembre",
    "ottobre",
    "novembre",
    "dicembre",
  ];
  const weekdayNames = ["domenica", "lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato"];

  const fieldLabels: Record<FieldKind, string> = {
    second: "Secondi",
    minute: "Minuti",
    hour: "Ore",
    dom: "Giorno del mese",
    month: "Mese",
    dow: "Giorno della settimana",
  };

  let input = $state("*/15 9-17 * * 1-5");

  function pad2(n: number): string {
    return String(n).padStart(2, "0");
  }

  function nameFor(kind: FieldKind, n: number): string {
    if (kind === "month") return monthNames[(((n - 1) % 12) + 12) % 12] ?? String(n);
    if (kind === "dow") return weekdayNames[((n % 7) + 7) % 7] ?? String(n);
    return String(n);
  }

  function describeGroup(group: string, kind: FieldKind): string {
    let step: number | null = null;
    let base = group;
    if (group.includes("/")) {
      const [b, s] = group.split("/");
      base = b;
      const n = Number(s);
      step = Number.isNaN(n) ? null : n;
    }
    if (base === "*") {
      return step !== null ? `ogni ${step}` : "ogni valore";
    }
    if (base.includes("-")) {
      const [a, b] = base.split("-").map(Number);
      if (Number.isNaN(a) || Number.isNaN(b)) return base;
      const label = `da ${nameFor(kind, a)} a ${nameFor(kind, b)}`;
      return step !== null ? `${label} ogni ${step}` : label;
    }
    const n = Number(base);
    if (Number.isNaN(n)) return base;
    return nameFor(kind, n);
  }

  function describeToken(token: string, kind: FieldKind): string {
    if (token === "*") return "ogni valore";
    return token
      .split(",")
      .map((g) => describeGroup(g.trim(), kind))
      .join(", ");
  }

  function stepOf(t: string): number | null {
    const m = /^\*\/(\d+)$/.exec(t);
    return m ? Number(m[1]) : null;
  }

  function singleOf(t: string): number | null {
    return /^\d+$/.test(t) ? Number(t) : null;
  }

  function buildTimeClause(min: string, hour: string, sec: string | null): string {
    const minStep = stepOf(min);
    const hourSingle = singleOf(hour);
    const minSingle = singleOf(min);

    let clause: string;
    if (min === "*" && hour === "*") {
      clause = "ogni minuto, di ogni ora";
    } else if (minStep !== null && hour === "*") {
      clause = `ogni ${minStep} minuti`;
    } else if (minSingle !== null && hourSingle !== null) {
      clause = `alle ${pad2(hourSingle)}:${pad2(minSingle)}`;
    } else if (minStep !== null && hourSingle !== null) {
      clause = `ogni ${minStep} minuti, durante l'ora delle ${pad2(hourSingle)}:00`;
    } else {
      const minDesc = min === "*" ? "ogni minuto" : describeToken(min, "minute");
      const hourDesc = hour === "*" ? "ogni ora" : describeToken(hour, "hour");
      clause = `ai minuti ${minDesc} delle ore ${hourDesc}`;
    }

    if (sec && sec !== "*") {
      const secStep = stepOf(sec);
      if (secStep !== null) clause += `, ogni ${secStep} secondi`;
      else clause += `, al secondo ${describeToken(sec, "second")}`;
    }
    return clause;
  }

  interface Row {
    kind: FieldKind;
    raw: string;
    meaning: string;
  }

  interface Result {
    error: string;
    sentence: string;
    rows: Row[];
  }

  const result = $derived.by((): Result => {
    const tokens = input.trim().split(/\s+/).filter(Boolean);
    if (tokens.length !== 5 && tokens.length !== 6) {
      return { error: "Un'espressione cron ha 5 (o 6) campi", sentence: "", rows: [] };
    }

    const hasSeconds = tokens.length === 6;
    const sec = hasSeconds ? tokens[0] : null;
    const min = hasSeconds ? tokens[1] : tokens[0];
    const hour = hasSeconds ? tokens[2] : tokens[1];
    const dom = hasSeconds ? tokens[3] : tokens[2];
    const month = hasSeconds ? tokens[4] : tokens[3];
    const dow = hasSeconds ? tokens[5] : tokens[4];

    const rows: Row[] = [];
    if (hasSeconds && sec) rows.push({ kind: "second", raw: sec, meaning: describeToken(sec, "second") });
    rows.push({ kind: "minute", raw: min, meaning: describeToken(min, "minute") });
    rows.push({ kind: "hour", raw: hour, meaning: describeToken(hour, "hour") });
    rows.push({ kind: "dom", raw: dom, meaning: describeToken(dom, "dom") });
    rows.push({ kind: "month", raw: month, meaning: describeToken(month, "month") });
    rows.push({ kind: "dow", raw: dow, meaning: describeToken(dow, "dow") });

    const clauses: string[] = [buildTimeClause(min, hour, sec)];
    if (dom !== "*") clauses.push(`nei giorni del mese: ${describeToken(dom, "dom")}`);
    if (month !== "*") clauses.push(`nei mesi: ${describeToken(month, "month")}`);
    if (dow !== "*") clauses.push(`nei giorni della settimana: ${describeToken(dow, "dow")}`);

    let sentence = clauses.filter(Boolean).join(", ") + ".";
    sentence = sentence.charAt(0).toUpperCase() + sentence.slice(1);

    return { error: "", sentence, rows };
  });
</script>

<div class="tool">
  <div class="stack">
    <span class="field-label">Espressione cron</span>
    <input
      class="input mono"
      type="text"
      bind:value={input}
      placeholder="es. */15 9-17 * * 1-5"
      spellcheck="false"
      autocapitalize="off"
      autocomplete="off"
    />
  </div>

  {#if result.error}
    <div class="notice bad">{result.error}</div>
  {:else}
    <div class="card-box cron-sentence">
      {result.sentence}
    </div>

    <div class="card-box">
      <span class="field-label">Dettaglio campi</span>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Campo</th>
              <th>Valore</th>
              <th>Significato</th>
            </tr>
          </thead>
          <tbody>
            {#each result.rows as r (r.kind)}
              <tr>
                <td>{fieldLabels[r.kind]}</td>
                <td class="mono">{r.raw}</td>
                <td>{r.meaning}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>

<style>
  .cron-sentence {
    font-size: 15px;
    font-weight: 600;
    color: var(--ink);
    line-height: 1.5;
  }
  .table-wrap {
    overflow-x: auto;
    margin-top: 10px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }
  th {
    text-align: left;
    padding: 8px 10px;
    color: var(--ink-dim);
    font-weight: 600;
    border-bottom: 1.5px solid var(--border-2);
    white-space: nowrap;
  }
  td {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
    color: var(--ink);
    vertical-align: top;
  }
  tr:last-child td {
    border-bottom: none;
  }
</style>
