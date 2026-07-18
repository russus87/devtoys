<script lang="ts">
  interface Status {
    code: number;
    name: string;
    desc: string;
  }

  const STATUSES: Status[] = [
    // 1xx
    { code: 100, name: "Continue", desc: "Il client può proseguire con la richiesta." },
    { code: 101, name: "Switching Protocols", desc: "Il server cambia protocollo come richiesto." },
    { code: 102, name: "Processing", desc: "Richiesta in elaborazione (WebDAV)." },
    { code: 103, name: "Early Hints", desc: "Header preliminari, utili per il preload." },
    // 2xx
    { code: 200, name: "OK", desc: "Richiesta riuscita." },
    { code: 201, name: "Created", desc: "Risorsa creata con successo." },
    { code: 202, name: "Accepted", desc: "Accettata ma non ancora elaborata." },
    { code: 204, name: "No Content", desc: "Riuscita, nessun corpo nella risposta." },
    { code: 206, name: "Partial Content", desc: "Contenuto parziale (range)." },
    // 3xx
    { code: 301, name: "Moved Permanently", desc: "Risorsa spostata in modo permanente." },
    { code: 302, name: "Found", desc: "Redirect temporaneo." },
    { code: 303, name: "See Other", desc: "Recupera la risorsa a un altro URI con GET." },
    { code: 304, name: "Not Modified", desc: "Risorsa non modificata (cache)." },
    { code: 307, name: "Temporary Redirect", desc: "Redirect temporaneo, metodo invariato." },
    { code: 308, name: "Permanent Redirect", desc: "Redirect permanente, metodo invariato." },
    // 4xx
    { code: 400, name: "Bad Request", desc: "Richiesta malformata." },
    { code: 401, name: "Unauthorized", desc: "Autenticazione richiesta o non valida." },
    { code: 403, name: "Forbidden", desc: "Accesso negato." },
    { code: 404, name: "Not Found", desc: "Risorsa non trovata." },
    { code: 405, name: "Method Not Allowed", desc: "Metodo HTTP non consentito." },
    { code: 406, name: "Not Acceptable", desc: "Nessuna rappresentazione accettabile." },
    { code: 408, name: "Request Timeout", desc: "Timeout in attesa della richiesta." },
    { code: 409, name: "Conflict", desc: "Conflitto con lo stato corrente." },
    { code: 410, name: "Gone", desc: "Risorsa rimossa in modo permanente." },
    { code: 413, name: "Payload Too Large", desc: "Corpo della richiesta troppo grande." },
    { code: 415, name: "Unsupported Media Type", desc: "Tipo di media non supportato." },
    { code: 418, name: "I'm a teapot", desc: "Sono una teiera (RFC 2324)." },
    { code: 422, name: "Unprocessable Entity", desc: "Semantica non valida (validazione)." },
    { code: 425, name: "Too Early", desc: "Rischio di replay, riprova più tardi." },
    { code: 429, name: "Too Many Requests", desc: "Troppe richieste (rate limit)." },
    // 5xx
    { code: 500, name: "Internal Server Error", desc: "Errore generico del server." },
    { code: 501, name: "Not Implemented", desc: "Funzionalità non implementata." },
    { code: 502, name: "Bad Gateway", desc: "Risposta non valida da un upstream." },
    { code: 503, name: "Service Unavailable", desc: "Servizio non disponibile / sovraccarico." },
    { code: 504, name: "Gateway Timeout", desc: "Timeout da un server upstream." },
    { code: 511, name: "Network Authentication Required", desc: "Autenticazione di rete richiesta." },
  ];

  const CLASSES: Record<number, { label: string; color: string }> = {
    1: { label: "Informativo", color: "var(--primary)" },
    2: { label: "Successo", color: "var(--green)" },
    3: { label: "Redirect", color: "var(--gold)" },
    4: { label: "Errore client", color: "var(--red)" },
    5: { label: "Errore server", color: "var(--red)" },
  };

  let query = $state("");

  const results = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return STATUSES;
    return STATUSES.filter(
      (s) =>
        String(s.code).includes(q) ||
        s.name.toLowerCase().includes(q) ||
        s.desc.toLowerCase().includes(q),
    );
  });
</script>

<div class="tool">
  <div class="tool-controls">
    <input class="input" bind:value={query} placeholder="Cerca codice o nome… (es. 404, timeout)" spellcheck="false" />
  </div>
  <div class="list">
    {#each results as s (s.code)}
      {@const cls = CLASSES[Math.floor(s.code / 100)]}
      <div class="item">
        <span class="code" style="color:{cls.color}">{s.code}</span>
        <div class="body">
          <div class="name">{s.name} <span class="tag" style="color:{cls.color}">{cls.label}</span></div>
          <div class="desc">{s.desc}</div>
        </div>
      </div>
    {/each}
    {#if results.length === 0}
      <p class="muted">Nessun codice trovato.</p>
    {/if}
  </div>
</div>

<style>
  .list {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 11px 15px;
    border-radius: 12px;
    background: var(--surface-2);
    border: 1.5px solid var(--border);
  }
  .code {
    font-family: var(--mono);
    font-size: 22px;
    font-weight: 800;
    width: 52px;
    flex-shrink: 0;
  }
  .body {
    min-width: 0;
  }
  .name {
    font-weight: 750;
    font-size: 14.5px;
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .tag {
    font-size: 10.5px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .desc {
    color: var(--ink-dim);
    font-size: 13px;
    margin-top: 2px;
  }
</style>
