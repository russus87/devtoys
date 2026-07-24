<script lang="ts">
  import Segmented from "../ui/Segmented.svelte";

  type FamilyId = "core" | "a" | "ua" | "x" | "e" | "vt" | "r" | "rel";

  interface Family {
    id: FamilyId;
    label: string;
    short: string;
    color: string;
    scope: string;
  }

  const FAMILIES: Family[] = [
    { id: "core", label: "PDF (base)", short: "PDF", color: "var(--ink-dim)", scope: "Il formato di partenza: ogni standard sotto è un sottoinsieme di una sua versione." },
    { id: "a", label: "PDF/A — archiviazione", short: "A", color: "var(--primary)", scope: "Conservazione a lungo termine: il file deve restare leggibile identico fra decenni, senza dipendere da nulla di esterno." },
    { id: "ua", label: "PDF/UA — accessibilità", short: "UA", color: "var(--green)", scope: "Universal Accessibility: il contenuto deve essere comprensibile da screen reader e tecnologie assistive." },
    { id: "x", label: "PDF/X — prestampa", short: "X", color: "var(--amber)", scope: "Scambio affidabile con la tipografia: colore prevedibile, font inclusi, nessuna sorpresa in stampa." },
    { id: "e", label: "PDF/E — ingegneria", short: "E", color: "var(--gold)", scope: "Documentazione tecnica e disegni, con modelli 3D interattivi." },
    { id: "vt", label: "PDF/VT — stampa variabile", short: "VT", color: "var(--c-convert)", scope: "Stampa transazionale e personalizzata ad alto volume (bollette, mailing)." },
    { id: "r", label: "PDF/R — raster", short: "R", color: "var(--c-test)", scope: "Scansioni: un contenitore standard per immagini raster, alternativa al TIFF." },
    { id: "rel", label: "Correlati", short: "REL", color: "var(--red)", scope: "Non sono parti ISO del PDF, ma senza di loro il quadro non si chiude." },
  ];

  const FAMILY: Record<FamilyId, Family> = Object.fromEntries(
    FAMILIES.map((f) => [f.id, f]),
  ) as Record<FamilyId, Family>;

  /** `yes` = lo standard lo impone, `partial` = lo impone solo in parte, `no` = non se ne occupa. */
  type Level = "yes" | "partial" | "no";

  interface Standard {
    code: string;
    badge: string;
    family: FamilyId;
    name: string;
    iso: string;
    year: number;
    base: string;
    archivable: Level;
    accessible: Level;
    summary: string;
    requires: string[];
    forbids: string[];
    useWhen: string;
    note?: string;
  }

  const STANDARDS: Standard[] = [
    // --- core ---------------------------------------------------------------
    {
      code: "PDF 1.4",
      badge: "1.4",
      family: "core",
      name: "Portable Document Format 1.4",
      iso: "Specifica Adobe (pre-ISO)",
      year: 2001,
      base: "—",
      archivable: "no",
      accessible: "no",
      summary:
        "Introduce la trasparenza e il tagged PDF. È la versione su cui è costruito PDF/A-1: per questo PDF/A-1 non ammette molte cose che oggi diamo per scontate.",
      requires: [],
      forbids: [],
      useWhen: "Non sceglierlo di proposito oggi: conta solo come base storica di PDF/A-1 e PDF/X-1a:2003.",
    },
    {
      code: "PDF 1.7",
      badge: "1.7",
      family: "core",
      name: "ISO 32000-1",
      iso: "ISO 32000-1:2008",
      year: 2008,
      base: "—",
      archivable: "no",
      accessible: "no",
      summary:
        "La prima versione standardizzata ISO, donata da Adobe. È la base di PDF/A-2, PDF/A-3 e PDF/UA-1, cioè della gran parte di ciò che si incontra nella pratica.",
      requires: [],
      forbids: [],
      useWhen: "È il minimo comune denominatore: qualunque lettore in circolazione lo apre senza problemi.",
    },
    {
      code: "PDF 2.0",
      badge: "2.0",
      family: "core",
      name: "ISO 32000-2",
      iso: "ISO 32000-2:2017, riveduta 2020",
      year: 2017,
      base: "—",
      archivable: "no",
      accessible: "no",
      summary:
        "Sviluppata interamente in ISO, senza Adobe. Ripulisce la specifica, aggiunge cifratura AES-256, annotazioni migliorate e la struttura dei namespace per i tag. È la base di PDF/A-4, PDF/UA-2, PDF/X-6 e PDF/R.",
      requires: [],
      forbids: [],
      useWhen: "Quando puoi permetterti di richiedere lettori recenti e vuoi le varianti moderne degli standard.",
      note: "Attenzione: alcuni strumenti datati aprono un PDF 2.0 ma ne ignorano silenziosamente le novità.",
    },

    // --- PDF/A --------------------------------------------------------------
    {
      code: "PDF/A-1b",
      badge: "A-1b",
      family: "a",
      name: "Archiviazione — livello B (Basic)",
      iso: "ISO 19005-1:2005",
      year: 2005,
      base: "PDF 1.4",
      archivable: "yes",
      accessible: "no",
      summary:
        "Il livello minimo: garantisce solo che il documento si veda uguale per sempre. Niente sul significato del contenuto — il testo può risultare illeggibile a copia-incolla e agli screen reader.",
      requires: [
        "Tutti i font incorporati e legalmente incorporabili",
        "Colore indipendente dal dispositivo: OutputIntent con profilo ICC, oppure spazi calibrati",
        "Metadati XMP con l'identificatore pdfaid (parte e livello)",
        "Ogni risorsa contenuta nel file: nulla di esterno",
      ],
      forbids: [
        "Cifratura e password",
        "JavaScript, azioni Launch, suoni e filmati",
        "Trasparenza",
        "Compressione LZW e JPEG2000",
        "Allegati e file incorporati",
        "Livelli (OCG) e moduli XFA",
      ],
      useWhen: "Solo se un fornitore lo impone per contratto. Per una scelta nuova oggi è quasi sempre superato da A-2b.",
      note: "Il divieto di trasparenza è il motivo per cui tanti PDF moderni non si convertono in A-1 senza essere rasterizzati.",
    },
    {
      code: "PDF/A-1a",
      badge: "A-1a",
      family: "a",
      name: "Archiviazione — livello A (Accessible)",
      iso: "ISO 19005-1:2005",
      year: 2005,
      base: "PDF 1.4",
      archivable: "yes",
      accessible: "partial",
      summary:
        "Tutto il livello B, più il significato del contenuto: struttura dei tag, ordine di lettura, mappatura Unicode e lingua dichiarata.",
      requires: [
        "Tutti i requisiti di A-1b",
        "Documento taggato con albero della struttura logica",
        "Ordine di lettura corretto",
        "Mappatura Unicode di tutto il testo (ToUnicode)",
        "Lingua naturale dichiarata",
      ],
      forbids: ["Gli stessi divieti di A-1b"],
      useWhen: "Quando servono insieme archiviazione e una base di accessibilità, ma il committente chiede esplicitamente la parte 1.",
      note: "Livello A non equivale a PDF/UA: copre meno casistiche. Per l'accessibilità piena serve PDF/UA-1 in aggiunta.",
    },
    {
      code: "PDF/A-2b",
      badge: "A-2b",
      family: "a",
      name: "Archiviazione parte 2 — livello B",
      iso: "ISO 19005-2:2011",
      year: 2011,
      base: "PDF 1.7 (ISO 32000-1)",
      archivable: "yes",
      accessible: "no",
      summary:
        "Riscrive PDF/A sulla base di PDF 1.7 e toglie i divieti più dolorosi della parte 1: trasparenza, JPEG2000 e livelli tornano leciti.",
      requires: ["Gli stessi requisiti fondamentali di A-1b, su base PDF 1.7"],
      forbids: [
        "Cifratura, JavaScript, azioni esterne, contenuti multimediali",
        "Font non incorporati",
        "Allegati che non siano a loro volta PDF/A",
      ],
      useWhen: "È il default sensato per l'archiviazione quando non servono allegati di altro tipo.",
      note: "Novità utili della parte 2: firme digitali PAdES, object stream per file più leggeri e la possibilità di incorporare altri PDF/A.",
    },
    {
      code: "PDF/A-2u",
      badge: "A-2u",
      family: "a",
      name: "Archiviazione parte 2 — livello U (Unicode)",
      iso: "ISO 19005-2:2011",
      year: 2011,
      base: "PDF 1.7 (ISO 32000-1)",
      archivable: "yes",
      accessible: "partial",
      summary:
        "Il livello B più l'obbligo che tutto il testo abbia una mappatura Unicode. Il documento diventa ricercabile e copiabile senza dover costruire l'albero dei tag.",
      requires: ["Tutti i requisiti di A-2b", "ToUnicode per ogni carattere di testo"],
      forbids: ["Gli stessi divieti di A-2b"],
      useWhen:
        "Ottimo compromesso: costa pochissimo in più rispetto a B e risolve il problema pratico più fastidioso, il testo che non si riesce a cercare.",
      note: "Il livello U esiste solo nelle parti 2 e 3. In PDF/A-4 l'Unicode è diventato obbligatorio e la lettera è sparita.",
    },
    {
      code: "PDF/A-2a",
      badge: "A-2a",
      family: "a",
      name: "Archiviazione parte 2 — livello A",
      iso: "ISO 19005-2:2011",
      year: 2011,
      base: "PDF 1.7 (ISO 32000-1)",
      archivable: "yes",
      accessible: "partial",
      summary: "Livello U più la struttura logica completa: tag, ordine di lettura e lingua.",
      requires: ["Tutti i requisiti di A-2u", "Documento taggato", "Ordine di lettura", "Lingua dichiarata"],
      forbids: ["Gli stessi divieti di A-2b"],
      useWhen: "Archiviazione con requisiti di accessibilità, tipicamente nella pubblica amministrazione.",
    },
    {
      code: "PDF/A-3b",
      badge: "A-3b",
      family: "a",
      name: "Archiviazione parte 3 — livello B",
      iso: "ISO 19005-3:2012",
      year: 2012,
      base: "PDF 1.7 (ISO 32000-1)",
      archivable: "yes",
      accessible: "no",
      summary:
        "Identico ad A-2b tranne per un punto, che però cambia tutto: permette di incorporare file di qualunque formato, non solo PDF/A.",
      requires: ["Tutti i requisiti di A-2b", "Relazione dichiarata per ogni allegato (Source, Data, Alternative…)"],
      forbids: ["Gli stessi divieti di A-2b, salvo il vincolo sul tipo di allegato"],
      useWhen:
        "Quando il PDF deve trasportare il proprio originale leggibile a macchina: è la base della fatturazione elettronica (ZUGFeRD, Factur-X) con l'XML dentro al PDF.",
      note: "Il rovescio della medaglia: l'archivio non può più garantire nulla sulla conservabilità di ciò che è allegato.",
    },
    {
      code: "PDF/A-3u",
      badge: "A-3u",
      family: "a",
      name: "Archiviazione parte 3 — livello U",
      iso: "ISO 19005-3:2012",
      year: 2012,
      base: "PDF 1.7 (ISO 32000-1)",
      archivable: "yes",
      accessible: "partial",
      summary: "Il livello B della parte 3 più la mappatura Unicode obbligatoria di tutto il testo.",
      requires: ["Tutti i requisiti di A-3b", "ToUnicode per ogni carattere"],
      forbids: ["Gli stessi divieti di A-3b"],
      useWhen: "Fatturazione elettronica e scambi documentali dove serve anche poter cercare nel testo.",
    },
    {
      code: "PDF/A-3a",
      badge: "A-3a",
      family: "a",
      name: "Archiviazione parte 3 — livello A",
      iso: "ISO 19005-3:2012",
      year: 2012,
      base: "PDF 1.7 (ISO 32000-1)",
      archivable: "yes",
      accessible: "partial",
      summary: "Livello U della parte 3 più struttura logica, ordine di lettura e lingua.",
      requires: ["Tutti i requisiti di A-3u", "Documento taggato", "Ordine di lettura", "Lingua dichiarata"],
      forbids: ["Gli stessi divieti di A-3b"],
      useWhen: "Serve trasportare allegati e insieme rispettare requisiti di accessibilità.",
    },
    {
      code: "PDF/A-4",
      badge: "A-4",
      family: "a",
      name: "Archiviazione parte 4",
      iso: "ISO 19005-4:2020",
      year: 2020,
      base: "PDF 2.0 (ISO 32000-2)",
      archivable: "yes",
      accessible: "no",
      summary:
        "Ripensa PDF/A su PDF 2.0 e abbandona le lettere a/b/u: c'è un solo livello base. La mappatura Unicode non è più opzionale, ed è ora obbligatoria per tutti.",
      requires: [
        "Font incorporati, OutputIntent, XMP con pdfaid parte 4",
        "Mappatura Unicode di tutto il testo (non più opzionale)",
      ],
      forbids: ["Cifratura", "JavaScript e azioni esterne", "Contenuti non incorporati"],
      useWhen: "Nuovi progetti di archiviazione che possono richiedere strumenti aggiornati a PDF 2.0.",
      note:
        "Cambio di filosofia importante: l'accessibilità non è più un livello di PDF/A, ma è delegata a PDF/UA, da dichiarare separatamente.",
    },
    {
      code: "PDF/A-4f",
      badge: "A-4f",
      family: "a",
      name: "Archiviazione parte 4 — con allegati",
      iso: "ISO 19005-4:2020",
      year: 2020,
      base: "PDF 2.0 (ISO 32000-2)",
      archivable: "yes",
      accessible: "no",
      summary: "PDF/A-4 con la facoltà di incorporare file di qualunque formato. È il successore di PDF/A-3.",
      requires: ["Tutti i requisiti di PDF/A-4", "Allegati dichiarati con la loro relazione"],
      forbids: ["Gli stessi divieti di PDF/A-4"],
      useWhen: "Fatturazione elettronica e scambio dati su base PDF 2.0.",
    },
    {
      code: "PDF/A-4e",
      badge: "A-4e",
      family: "a",
      name: "Archiviazione parte 4 — engineering",
      iso: "ISO 19005-4:2020",
      year: 2020,
      base: "PDF 2.0 (ISO 32000-2)",
      archivable: "yes",
      accessible: "no",
      summary:
        "PDF/A-4 che ammette contenuti RichMedia e modelli 3D. Nasce per prendere il posto di PDF/E-1 nell'archiviazione tecnica.",
      requires: ["Tutti i requisiti di PDF/A-4", "Annotazioni 3D conformi"],
      forbids: ["Gli stessi divieti di PDF/A-4"],
      useWhen: "Archiviazione a lungo termine di documentazione tecnica con modelli tridimensionali.",
    },

    // --- PDF/UA -------------------------------------------------------------
    {
      code: "PDF/UA-1",
      badge: "UA-1",
      family: "ua",
      name: "Universal Accessibility parte 1",
      iso: "ISO 14289-1:2014",
      year: 2014,
      base: "PDF 1.7 (ISO 32000-1)",
      archivable: "no",
      accessible: "yes",
      summary:
        "Lo standard tecnico dell'accessibilità nei PDF. Non dice come deve apparire il documento: dice come deve essere strutturato perché una tecnologia assistiva lo interpreti correttamente.",
      requires: [
        "Documento taggato con struttura semanticamente corretta",
        "Ordine di lettura logico, indipendente dalla disposizione grafica",
        "Testo alternativo per ogni immagine informativa",
        "Tabelle con intestazioni TH e associazione riga/colonna",
        "Titoli gerarchici H1…H6 senza salti di livello",
        "Lingua del documento e dei brani in lingua diversa",
        "Segnalibri per documenti lunghi, titolo nei metadati e DisplayDocTitle attivo",
        "Contenuti puramente decorativi marcati come artefatti",
      ],
      forbids: [
        "Testo come immagine senza alternativa",
        "Uso dei tag per ottenere effetti grafici anziché per descrivere la semantica",
        "Contenuto informativo non contenuto nell'albero della struttura",
      ],
      useWhen:
        "Sempre, per qualunque documento destinato al pubblico. Nella PA italiana è di fatto un obbligo di legge.",
      note:
        "Si verifica sui 31 checkpoint del Matterhorn Protocol. Una parte è automatizzabile, il resto richiede per forza un controllo umano.",
    },
    {
      code: "PDF/UA-2",
      badge: "UA-2",
      family: "ua",
      name: "Universal Accessibility parte 2",
      iso: "ISO 14289-2:2024",
      year: 2024,
      base: "PDF 2.0 (ISO 32000-2)",
      archivable: "no",
      accessible: "yes",
      summary:
        "Aggiorna PDF/UA a PDF 2.0. Sfrutta i namespace dei tag, il modello di tabella più preciso e le associazioni contenuto-struttura introdotte dalla nuova specifica.",
      requires: ["Gli stessi principi di UA-1, espressi con i costrutti di PDF 2.0"],
      forbids: ["Gli stessi divieti di UA-1"],
      useWhen: "Nuovi flussi di produzione già su PDF 2.0, spesso in coppia con PDF/A-4.",
    },

    // --- PDF/X --------------------------------------------------------------
    {
      code: "PDF/X-1a",
      badge: "X-1a",
      family: "x",
      name: "Prestampa — solo CMYK e tinte piatte",
      iso: "ISO 15930-1:2001, poi 15930-4:2003",
      year: 2001,
      base: "PDF 1.3, poi PDF 1.4",
      archivable: "no",
      accessible: "no",
      summary:
        "Il profilo di prestampa più rigido: tutto deve essere già convertito in CMYK o tinte piatte, niente gestione del colore da interpretare in tipografia.",
      requires: ["Font incorporati", "OutputIntent che dichiara la condizione di stampa", "TrimBox o ArtBox definite"],
      forbids: ["RGB e Lab", "Trasparenza viva", "Gestione colore ICC sugli oggetti", "Cifratura", "Contenuti esterni"],
      useWhen: "La tipografia lo impone, o quando vuoi zero ambiguità sul colore assumendoti tu la conversione.",
    },
    {
      code: "PDF/X-3",
      badge: "X-3",
      family: "x",
      name: "Prestampa — colore gestito ICC",
      iso: "ISO 15930-3:2002, poi 15930-6:2003",
      year: 2002,
      base: "PDF 1.3, poi PDF 1.4",
      archivable: "no",
      accessible: "no",
      summary:
        "Come X-1a ma ammette RGB e Lab con profilo ICC: la conversione finale la fa la tipografia, non tu.",
      requires: ["Font incorporati", "OutputIntent", "Profilo ICC per ogni oggetto non CMYK"],
      forbids: ["Trasparenza viva", "Cifratura", "Contenuti esterni"],
      useWhen: "Flussi con gestione del colore, quando ti fidi del profilo di destinazione della tipografia.",
    },
    {
      code: "PDF/X-4",
      badge: "X-4",
      family: "x",
      name: "Prestampa — trasparenza viva e livelli",
      iso: "ISO 15930-7:2010",
      year: 2010,
      base: "PDF 1.6",
      archivable: "no",
      accessible: "no",
      summary:
        "Il profilo di prestampa più usato oggi: conserva la trasparenza senza appiattirla e ammette i livelli, lasciando il rendering al RIP.",
      requires: ["Font incorporati", "OutputIntent", "Trasparenza dichiarata con gruppo di fusione corretto"],
      forbids: ["Cifratura", "JavaScript", "Contenuti esterni (per questo esiste la variante X-4p)"],
      useWhen: "Scelta predefinita per la stampa professionale moderna.",
      note: "La variante X-4p rimanda a un profilo ICC esterno anziché incorporarlo.",
    },
    {
      code: "PDF/X-5",
      badge: "X-5",
      family: "x",
      name: "Prestampa — riferimenti esterni",
      iso: "ISO 15930-8:2010",
      year: 2010,
      base: "PDF 1.6",
      archivable: "no",
      accessible: "no",
      summary:
        "Estende X-4 permettendo di richiamare grafica o profili che vivono fuori dal file. Tre varianti: X-5g (grafica esterna), X-5pg (grafica e profilo esterni), X-5n (profilo n-colorante esterno).",
      requires: ["Requisiti di X-4", "Riferimenti esterni risolvibili al momento della stampa"],
      forbids: ["Cifratura"],
      useWhen: "Flussi editoriali ad alto volume in cui la stessa grafica pesante è condivisa fra molti documenti.",
      note: "Proprio perché dipende da file esterni è l'opposto di un formato d'archivio.",
    },
    {
      code: "PDF/X-6",
      badge: "X-6",
      family: "x",
      name: "Prestampa su PDF 2.0",
      iso: "ISO 15930-9:2020",
      year: 2020,
      base: "PDF 2.0 (ISO 32000-2)",
      archivable: "no",
      accessible: "no",
      summary:
        "Porta PDF/X su PDF 2.0 unificando l'impianto di X-4 e X-5. Declinato in X-6 (autonomo), X-6p (profilo esterno) e X-6n (spazio n-colorante esterno).",
      requires: ["Font incorporati", "OutputIntent", "Box di rifilo definite"],
      forbids: ["Cifratura", "Contenuti non risolvibili"],
      useWhen: "Nuovi flussi di prestampa con RIP aggiornati a PDF 2.0.",
    },

    // --- PDF/E --------------------------------------------------------------
    {
      code: "PDF/E-1",
      badge: "E-1",
      family: "e",
      name: "Documenti tecnici e di ingegneria",
      iso: "ISO 24517-1:2008",
      year: 2008,
      base: "PDF 1.6",
      archivable: "partial",
      accessible: "no",
      summary:
        "Pensato per disegni tecnici, cantieristica e manifattura: ammette geometria 3D interattiva (U3D) mantenendo l'autonomia del file.",
      requires: ["Font incorporati", "Contenuti autonomi", "Annotazioni 3D conformi"],
      forbids: ["Cifratura", "Contenuti esterni", "JavaScript non necessario alla visualizzazione 3D"],
      useWhen: "Scambio di documentazione tecnica con modelli navigabili.",
      note: "Per la sola archiviazione conviene ormai PDF/A-4e, che copre lo stesso caso d'uso con garanzie migliori.",
    },
    {
      code: "PDF/E-2",
      badge: "E-2",
      family: "e",
      name: "Documenti tecnici su PDF 2.0",
      iso: "ISO 24517-2:2024",
      year: 2024,
      base: "PDF 2.0 (ISO 32000-2)",
      archivable: "partial",
      accessible: "no",
      summary:
        "Aggiorna PDF/E a PDF 2.0 e sposta il 3D sul formato PRC, molto più compatto e preciso di U3D per la geometria CAD.",
      requires: ["Font incorporati", "Modelli 3D in PRC", "Contenuti autonomi"],
      forbids: ["Cifratura", "Contenuti esterni"],
      useWhen: "Flussi CAD moderni che devono consegnare un modello navigabile dentro a un documento.",
    },

    // --- PDF/VT -------------------------------------------------------------
    {
      code: "PDF/VT-1",
      badge: "VT-1",
      family: "vt",
      name: "Stampa variabile — file autonomo",
      iso: "ISO 16612-2:2010",
      year: 2010,
      base: "PDF/X-4",
      archivable: "no",
      accessible: "no",
      summary:
        "Costruito su PDF/X-4 per la stampa transazionale: introduce i metadati DPart che descrivono dove finisce un destinatario e comincia il successivo, e la gestione delle risorse ricorrenti perché il RIP non le rielabori a ogni pagina.",
      requires: ["Requisiti di PDF/X-4", "Gerarchia DPart", "Risorse riutilizzabili dichiarate"],
      forbids: ["Riferimenti esterni"],
      useWhen: "Bollette, estratti conto e mailing personalizzati in un unico file.",
    },
    {
      code: "PDF/VT-2",
      badge: "VT-2",
      family: "vt",
      name: "Stampa variabile — riferimenti esterni",
      iso: "ISO 16612-2:2010",
      year: 2010,
      base: "PDF/X-4p, PDF/X-5",
      archivable: "no",
      accessible: "no",
      summary: "Come VT-1 ma può richiamare risorse esterne condivise fra più lavori di stampa.",
      requires: ["Requisiti di PDF/X-5", "Gerarchia DPart"],
      forbids: ["Risorse esterne non risolvibili al momento della stampa"],
      useWhen: "Produzioni ad altissimo volume con librerie grafiche condivise.",
      note: "Esiste anche la variante di flusso VT-2s, pensata per essere consumata in streaming.",
    },
    {
      code: "PDF/VT-3",
      badge: "VT-3",
      family: "vt",
      name: "Stampa variabile su PDF 2.0",
      iso: "ISO 16612-3:2020",
      year: 2020,
      base: "PDF/X-6",
      archivable: "no",
      accessible: "no",
      summary: "Porta PDF/VT su PDF 2.0 appoggiandosi a PDF/X-6.",
      requires: ["Requisiti di PDF/X-6", "Gerarchia DPart"],
      forbids: ["Cifratura"],
      useWhen: "Nuove installazioni di stampa digitale ad alto volume.",
    },

    // --- PDF/R --------------------------------------------------------------
    {
      code: "PDF/R-1",
      badge: "R-1",
      family: "r",
      name: "PDF raster",
      iso: "ISO 23504-1:2020",
      year: 2020,
      base: "PDF 2.0 (ISO 32000-2)",
      archivable: "partial",
      accessible: "no",
      summary:
        "Un profilo minimo per contenere immagini raster: una pagina, un'immagine. Nasce per sostituire il TIFF multipagina negli scanner e nei sistemi fax, con un formato che tutti sanno aprire.",
      requires: ["Contenuto raster con risoluzione e spazio colore dichiarati", "Struttura del file volutamente semplice"],
      forbids: ["Testo vettoriale", "Interattività", "Cifratura"],
      useWhen: "Digitalizzazione massiva e sistemi di acquisizione documentale.",
      note: "Senza OCR il contenuto resta immagine: non è ricercabile e non è accessibile.",
    },

    // --- correlati ----------------------------------------------------------
    {
      code: "Tagged PDF",
      badge: "TAG",
      family: "rel",
      name: "Struttura logica del documento",
      iso: "ISO 32000-1 §14.8 e ISO 32000-2",
      year: 2001,
      base: "PDF 1.4 e successivi",
      archivable: "no",
      accessible: "partial",
      summary:
        "Non è uno standard a sé, ma il meccanismo su cui poggiano tutti i livelli accessibili: un albero parallelo alla grafica che dice cosa ogni pezzo di pagina significa — titolo, paragrafo, cella, figura.",
      requires: ["StructTreeRoot con gerarchia di elementi", "Associazione fra contenuto della pagina e nodo della struttura"],
      forbids: ["Tag usati per la resa visiva anziché per la semantica"],
      useWhen: "È il prerequisito di PDF/A livello A e di tutto PDF/UA: senza, quei livelli non sono raggiungibili.",
    },
    {
      code: "PAdES",
      badge: "SIG",
      family: "rel",
      name: "Firme elettroniche avanzate su PDF",
      iso: "ETSI EN 319 142-1 e -2",
      year: 2016,
      base: "PDF 1.7 e PDF 2.0",
      archivable: "partial",
      accessible: "no",
      summary:
        "I profili europei di firma digitale dentro al PDF. I livelli B-LT e B-LTA incorporano marca temporale e informazioni di revoca, così la firma resta verificabile anche quando il certificato è scaduto.",
      requires: ["Campo firma conforme", "Marca temporale per i livelli a lungo termine"],
      forbids: ["Modifiche al documento dopo la firma che non siano incrementali"],
      useWhen: "Documenti con valore legale in ambito eIDAS; si combina con PDF/A per la conservazione.",
    },
    {
      code: "ZUGFeRD / Factur-X",
      badge: "FAT",
      family: "rel",
      name: "Fatturazione elettronica ibrida",
      iso: "Specifiche FeRD / FNFE, su base PDF/A-3",
      year: 2014,
      base: "PDF/A-3",
      archivable: "yes",
      accessible: "no",
      summary:
        "Sfruttano PDF/A-3 per mettere in un unico file la fattura leggibile dall'uomo e l'XML leggibile dalla macchina, che devono corrispondere.",
      requires: ["Conformità PDF/A-3", "XML incorporato con nome e relazione previsti dallo standard"],
      forbids: ["Divergenze fra il contenuto visibile e l'XML"],
      useWhen: "Scambio di fatture in Europa dove serve un unico file valido per entrambi i mondi.",
      note: "In Italia il canale ufficiale resta FatturaPA in XML puro via SdI: questi formati si incontrano nei rapporti con l'estero.",
    },
    {
      code: "Matterhorn Protocol",
      badge: "MAT",
      family: "rel",
      name: "Griglia di verifica per PDF/UA",
      iso: "PDF Association, versione 1.1",
      year: 2021,
      base: "PDF/UA-1",
      archivable: "no",
      accessible: "partial",
      summary:
        "Traduce PDF/UA in 31 checkpoint e 136 condizioni di errore. Ogni condizione dichiara se è verificabile da una macchina o se richiede un giudizio umano.",
      requires: ["Verifica di tutte le condizioni applicabili"],
      forbids: [],
      useWhen: "È il riferimento pratico per qualunque audit di accessibilità serio.",
      note: "Circa metà delle condizioni non è automatizzabile: nessun validatore da solo può certificare la conformità PDF/UA.",
    },
    {
      code: "WCAG / EN 301 549",
      badge: "LEX",
      family: "rel",
      name: "Cornice normativa dell'accessibilità",
      iso: "W3C WCAG 2.1/2.2, EN 301 549",
      year: 2018,
      base: "—",
      archivable: "no",
      accessible: "yes",
      summary:
        "Sono le norme che rendono obbligatoria l'accessibilità. La EN 301 549 europea rimanda a WCAG per il web e a PDF/UA per i documenti: è il ponte fra l'obbligo di legge e lo standard tecnico.",
      requires: ["Livello AA di WCAG per i contenuti pubblici", "Conformità PDF/UA per i documenti"],
      forbids: [],
      useWhen: "Definisce cosa devi rispettare; PDF/UA definisce come farlo su un PDF.",
      note: "In Italia il riferimento è la Legge Stanca e le Linee guida AgID, che recepiscono la EN 301 549.",
    },
    {
      code: "veraPDF",
      badge: "VAL",
      family: "rel",
      name: "Validatore di riferimento",
      iso: "Open source, PDF Association",
      year: 2015,
      base: "PDF/A e PDF/UA",
      archivable: "partial",
      accessible: "partial",
      summary:
        "Il validatore riconosciuto per PDF/A (tutte le parti) e per la porzione automatizzabile di PDF/UA. Quando qualcuno chiede un file «certificato PDF/A» di norma intende che passa veraPDF.",
      requires: [],
      forbids: [],
      useWhen: "Verifica finale prima di consegnare o versare in conservazione.",
      note:
        "Il check PDF/A di DevToys è euristico e non sostituisce veraPDF: guarda pochi indicatori (tag, font, lingua, marcatore XMP, cifratura).",
    },
  ];

  // --- filtri ---------------------------------------------------------------

  let tab = $state("elenco");
  let query = $state("");
  let famFilter = $state<FamilyId | "all">("all");
  let openCode = $state<string | null>(null);

  const results = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return STANDARDS.filter((s) => {
      if (famFilter !== "all" && s.family !== famFilter) return false;
      if (!q) return true;
      return (
        s.code.toLowerCase().includes(q) ||
        s.name.toLowerCase().includes(q) ||
        s.iso.toLowerCase().includes(q) ||
        s.summary.toLowerCase().includes(q) ||
        s.base.toLowerCase().includes(q) ||
        s.useWhen.toLowerCase().includes(q) ||
        s.requires.some((r) => r.toLowerCase().includes(q)) ||
        s.forbids.some((r) => r.toLowerCase().includes(q))
      );
    });
  });

  function toggle(code: string) {
    openCode = openCode === code ? null : code;
  }

  const LEVEL_LABEL: Record<Level, string> = { yes: "sì", partial: "in parte", no: "no" };
  const LEVEL_COLOR: Record<Level, string> = {
    yes: "var(--green)",
    partial: "var(--gold)",
    no: "var(--ink-faint)",
  };

  // Matrice parti x livelli di PDF/A. `null` = combinazione inesistente.
  const MATRIX: { part: string; iso: string; cells: (string | null)[] }[] = [
    { part: "PDF/A-1", iso: "2005 · PDF 1.4", cells: ["b", "—", "a", null, null] },
    { part: "PDF/A-2", iso: "2011 · PDF 1.7", cells: ["b", "u", "a", null, null] },
    { part: "PDF/A-3", iso: "2012 · PDF 1.7", cells: ["b", "u", "a", "f", null] },
    { part: "PDF/A-4", iso: "2020 · PDF 2.0", cells: ["·", "sempre", "→ UA", "f", "e"] },
  ];
  const MATRIX_COLS = ["Base (b)", "Unicode (u)", "Accessibile (a)", "Allegati (f)", "3D (e)"];
</script>

<div class="tool">
  <div class="tool-controls wrap">
    <Segmented
      bind:value={tab}
      options={[
        { value: "elenco", label: "Elenco" },
        { value: "mappa", label: "Mappa" },
        { value: "archivio", label: "Archiviabile" },
        { value: "accesso", label: "Accessibile" },
        { value: "scelta", label: "Quale scegliere" },
      ]}
    />
  </div>

  {#if tab === "elenco"}
    <div class="tool-controls">
      <input
        class="input"
        bind:value={query}
        placeholder="Cerca standard, ISO o requisito… (es. A-3, trasparenza, allegati)"
        spellcheck="false"
      />
    </div>
    <div class="chips">
      <button class="chip" class:on={famFilter === "all"} onclick={() => (famFilter = "all")}>
        Tutti <span class="n">{STANDARDS.length}</span>
      </button>
      {#each FAMILIES as f (f.id)}
        {@const n = STANDARDS.filter((s) => s.family === f.id).length}
        <button
          class="chip"
          class:on={famFilter === f.id}
          style="--chip:{f.color}"
          onclick={() => (famFilter = f.id)}
        >
          {f.label} <span class="n">{n}</span>
        </button>
      {/each}
    </div>

    {#if famFilter !== "all"}
      <p class="scope">{FAMILY[famFilter].scope}</p>
    {/if}

    <div class="list">
      {#each results as s (s.code)}
        {@const fam = FAMILY[s.family]}
        <div class="item" class:open={openCode === s.code}>
          <button class="head" onclick={() => toggle(s.code)} aria-expanded={openCode === s.code}>
            <span class="badge" style="--c:{fam.color}">{s.badge}</span>
            <span class="body">
              <span class="name">
                {s.code}
                <span class="iso">{s.iso}</span>
              </span>
              <span class="desc">{s.name}</span>
              <span class="meta">
                <span class="kv">base <b>{s.base}</b></span>
                <span class="kv" style="color:{LEVEL_COLOR[s.archivable]}">
                  archiviabile <b>{LEVEL_LABEL[s.archivable]}</b>
                </span>
                <span class="kv" style="color:{LEVEL_COLOR[s.accessible]}">
                  accessibile <b>{LEVEL_LABEL[s.accessible]}</b>
                </span>
              </span>
            </span>
            <span class="caret" aria-hidden="true">{openCode === s.code ? "−" : "+"}</span>
          </button>

          {#if openCode === s.code}
            <div class="detail">
              <p class="summary">{s.summary}</p>
              <div class="cols">
                {#if s.requires.length}
                  <div class="col">
                    <h4 class="ok">Richiede</h4>
                    <ul>
                      {#each s.requires as r}<li>{r}</li>{/each}
                    </ul>
                  </div>
                {/if}
                {#if s.forbids.length}
                  <div class="col">
                    <h4 class="ko">Vieta</h4>
                    <ul>
                      {#each s.forbids as r}<li>{r}</li>{/each}
                    </ul>
                  </div>
                {/if}
              </div>
              <p class="when"><b>Quando usarlo.</b> {s.useWhen}</p>
              {#if s.note}
                <p class="note">{s.note}</p>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
      {#if results.length === 0}
        <p class="muted">Nessuno standard corrisponde alla ricerca.</p>
      {/if}
    </div>
  {:else if tab === "mappa"}
    <div class="page">
      <h3>Come sono imparentati</h3>
      <p class="lead">
        Ogni standard è un <b>sottoinsieme</b> di una versione di PDF: prende la specifica base e ne vieta una parte,
        aggiungendo obblighi. Un file conforme a PDF/A resta un PDF normale, che qualunque lettore apre.
      </p>

      <figure>
        <svg viewBox="0 0 760 400" role="img" aria-label="Mappa degli standard PDF raggruppati per la versione base da cui derivano">
          <defs>
            <marker id="ar" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
              <path d="M0 1 L7 4 L0 7 z" fill="var(--ink-faint)" />
            </marker>
          </defs>

          <!-- colonna delle versioni base -->
          {#each [{ y: 22, t: "PDF 1.4", s: "2001" }, { y: 138, t: "PDF 1.7", s: "ISO 32000-1 · 2008" }, { y: 298, t: "PDF 2.0", s: "ISO 32000-2 · 2017" }] as b (b.t)}
            <rect x="14" y={b.y} width="120" height="44" rx="10" fill="var(--surface-3)" stroke="var(--border-2)" />
            <text x="74" y={b.y + 20} class="t b" text-anchor="middle">{b.t}</text>
            <text x="74" y={b.y + 35} class="t s" text-anchor="middle">{b.s}</text>
          {/each}

          <!-- ogni standard sotto la versione da cui deriva davvero -->
          {#each [{ from: 44, y: 16, h: 40, c: "var(--primary)", fill: "var(--primary-soft)", t: "PDF/A-1", s: "livelli b · a" }, { from: 44, y: 62, h: 32, c: "var(--amber)", fill: "var(--surface-2)", t: "PDF/X-1a · PDF/X-3", s: "" }, { from: 160, y: 104, h: 40, c: "var(--primary)", fill: "var(--primary-soft)", t: "PDF/A-2 · PDF/A-3", s: "livelli b · u · a" }, { from: 160, y: 150, h: 32, c: "var(--green)", fill: "var(--surface-2)", t: "PDF/UA-1", s: "" }, { from: 160, y: 188, h: 32, c: "var(--amber)", fill: "var(--surface-2)", t: "PDF/X-4 · PDF/X-5", s: "" }, { from: 160, y: 226, h: 32, c: "var(--gold)", fill: "var(--surface-2)", t: "PDF/E-1", s: "" }, { from: 320, y: 264, h: 40, c: "var(--primary)", fill: "var(--primary-soft)", t: "PDF/A-4", s: "· f allegati · e 3D" }, { from: 320, y: 310, h: 32, c: "var(--green)", fill: "var(--surface-2)", t: "PDF/UA-2", s: "" }, { from: 320, y: 348, h: 40, c: "var(--ink-faint)", fill: "var(--surface-2)", t: "PDF/X-6 · PDF/E-2", s: "PDF/VT-3 · PDF/R-1" }] as n (n.y)}
            {@const mid = n.y + n.h / 2}
            <path
              d="M134 {n.from} C 167 {n.from}, 167 {mid}, 196 {mid}"
              stroke="var(--ink-faint)"
              fill="none"
              marker-end="url(#ar)"
            />
            <rect x="200" y={n.y} width="190" height={n.h} rx="10" fill={n.fill} stroke={n.c} />
            <text x="295" y={n.s ? n.y + 18 : mid + 4} class="t b" text-anchor="middle" fill={n.c}>{n.t}</text>
            {#if n.s}
              <text x="295" y={n.y + 33} class="t s" text-anchor="middle">{n.s}</text>
            {/if}
          {/each}

          <!-- unica derivazione fra standard: PDF/VT è costruito su PDF/X -->
          <path d="M390 204 H 426" stroke="var(--ink-faint)" fill="none" marker-end="url(#ar)" />
          <rect x="430" y="188" width="170" height="32" rx="10" fill="var(--surface-2)" stroke="var(--c-convert)" />
          <text x="515" y="208" class="t b" text-anchor="middle" fill="var(--c-convert)">PDF/VT-1 · PDF/VT-2</text>

          <!-- legenda -->
          <text x="648" y="42" class="t s">archiviazione</text>
          <rect x="630" y="34" width="12" height="10" rx="3" fill="var(--primary)" />
          <text x="648" y="62" class="t s">accessibilità</text>
          <rect x="630" y="54" width="12" height="10" rx="3" fill="var(--green)" />
          <text x="648" y="82" class="t s">prestampa</text>
          <rect x="630" y="74" width="12" height="10" rx="3" fill="var(--amber)" />
          <text x="648" y="102" class="t s">ingegneria</text>
          <rect x="630" y="94" width="12" height="10" rx="3" fill="var(--gold)" />
          <text x="648" y="122" class="t s">stampa variabile</text>
          <rect x="630" y="114" width="12" height="10" rx="3" fill="var(--c-convert)" />
        </svg>
        <figcaption>
          Le frecce indicano «è costruito su». Quasi tutti gli standard derivano direttamente da una versione base, non
          l'uno dall'altro: l'unica eccezione qui è PDF/VT, che si appoggia a PDF/X. Un PDF/VT-1 è quindi anche un
          PDF/X-4 valido, che a sua volta è un PDF valido e basta.
        </figcaption>
      </figure>

      <h3>Si possono combinare</h3>
      <p class="lead">
        Le famiglie coprono esigenze diverse e non si escludono: un documento può dichiararsi <b>PDF/A-2a e PDF/UA-1</b>
        insieme, ed è anzi la combinazione tipica per la pubblica amministrazione. Ogni conformità viene dichiarata
        nei metadati XMP con il proprio identificatore.
      </p>

      <figure>
        <svg viewBox="0 0 640 150" role="img" aria-label="Diagramma delle conformità combinabili in un unico file">
          <rect x="20" y="20" width="600" height="110" rx="14" fill="var(--surface-2)" stroke="var(--border-2)" stroke-dasharray="5 4" />
          <text x="40" y="44" class="t s">un solo file PDF</text>

          <rect x="46" y="56" width="164" height="54" rx="11" fill="var(--primary-soft)" stroke="var(--primary)" />
          <text x="128" y="78" class="t b" text-anchor="middle" fill="var(--primary)">PDF/A-2a</text>
          <text x="128" y="95" class="t s" text-anchor="middle">pdfaid:part=2 conformance=A</text>

          <rect x="238" y="56" width="164" height="54" rx="11" fill="var(--surface)" stroke="var(--green)" />
          <text x="320" y="78" class="t b" text-anchor="middle" fill="var(--green)">PDF/UA-1</text>
          <text x="320" y="95" class="t s" text-anchor="middle">pdfuaid:part=1</text>

          <rect x="430" y="56" width="164" height="54" rx="11" fill="var(--surface)" stroke="var(--red)" />
          <text x="512" y="78" class="t b" text-anchor="middle" fill="var(--red)">PAdES B-LTA</text>
          <text x="512" y="95" class="t s" text-anchor="middle">firma con marca temporale</text>
        </svg>
        <figcaption>Le conformità si sommano nei metadati XMP; ognuna va verificata con lo strumento adatto.</figcaption>
      </figure>

      <h3>Linea del tempo</h3>
      <figure>
        <svg viewBox="0 0 760 150" role="img" aria-label="Linea del tempo degli standard PDF dal 2001 al 2024">
          <line x1="30" y1="96" x2="730" y2="96" stroke="var(--border-2)" stroke-width="2" />
          {#each [{ x: 40, y: 1, l: "2001", t: "PDF/X-1a" }, { x: 132, y: 0, l: "2005", t: "PDF/A-1" }, { x: 224, y: 1, l: "2008", t: "PDF 1.7 · PDF/E-1" }, { x: 340, y: 0, l: "2010", t: "PDF/X-4 · PDF/VT" }, { x: 448, y: 1, l: "2011-12", t: "PDF/A-2 · A-3" }, { x: 552, y: 0, l: "2014", t: "PDF/UA-1" }, { x: 638, y: 1, l: "2017", t: "PDF 2.0" }, { x: 700, y: 0, l: "2020-24", t: "A-4 · X-6 · UA-2" }] as p (p.l)}
            <circle cx={p.x} cy="96" r="5" fill="var(--primary)" />
            <line x1={p.x} y1="96" x2={p.x} y2={p.y ? 70 : 122} stroke="var(--border-2)" />
            <text x={p.x} y={p.y ? 62 : 138} class="t b" text-anchor="middle">{p.l}</text>
            <text x={p.x} y={p.y ? 48 : 152} class="t s" text-anchor="middle">{p.t}</text>
          {/each}
        </svg>
        <figcaption>Vent'anni di stratificazione: nulla è stato ritirato, per questo convivono ancora tutte le parti.</figcaption>
      </figure>
    </div>
  {:else if tab === "archivio"}
    <div class="page">
      <h3>Che cosa rende un PDF archiviabile</h3>
      <p class="lead">
        L'idea di PDF/A è una sola: il file deve bastare a sé stesso. Fra trent'anni non ci sarà il font sul sistema,
        né il profilo colore, né la rete per scaricare qualcosa. Tutto ciò che serve a mostrare la pagina deve essere
        <b>dentro al file</b>, e tutto ciò che potrebbe comportarsi in modo imprevedibile è vietato.
      </p>

      <figure>
        <svg viewBox="0 0 720 300" role="img" aria-label="Anatomia di un file PDF/A conforme, con i componenti obbligatori e vietati">
          <!-- pagina -->
          <rect x="40" y="24" width="200" height="252" rx="10" fill="var(--surface)" stroke="var(--border-2)" stroke-width="2" />
          <rect x="64" y="52" width="152" height="9" rx="4" fill="var(--ink-faint)" opacity="0.55" />
          <rect x="64" y="70" width="120" height="9" rx="4" fill="var(--ink-faint)" opacity="0.35" />
          <rect x="64" y="96" width="152" height="6" rx="3" fill="var(--ink-faint)" opacity="0.25" />
          <rect x="64" y="110" width="152" height="6" rx="3" fill="var(--ink-faint)" opacity="0.25" />
          <rect x="64" y="124" width="112" height="6" rx="3" fill="var(--ink-faint)" opacity="0.25" />
          <rect x="64" y="150" width="152" height="60" rx="6" fill="var(--primary-soft)" stroke="var(--primary)" stroke-opacity="0.4" />
          <text x="140" y="185" class="t s" text-anchor="middle">immagine</text>
          <rect x="64" y="226" width="152" height="6" rx="3" fill="var(--ink-faint)" opacity="0.25" />
          <rect x="64" y="240" width="96" height="6" rx="3" fill="var(--ink-faint)" opacity="0.25" />

          <!-- obbligatori -->
          <text x="300" y="38" class="t b" fill="var(--green)">Deve contenere</text>
          {#each [{ y: 58, t: "Font incorporati e sottoinsiemati" }, { y: 84, t: "OutputIntent con profilo ICC" }, { y: 110, t: "XMP con pdfaid: parte e livello" }, { y: 136, t: "Ogni immagine e risorsa nel file" }] as r (r.y)}
            <rect x="296" y={r.y - 15} width="392" height="22" rx="7" fill="var(--surface-2)" stroke="var(--green)" stroke-opacity="0.45" />
            <path d="M{306} {r.y - 5} l 4 4 l 7 -8" stroke="var(--green)" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
            <text x="326" y={r.y} class="t s">{r.t}</text>
          {/each}

          <!-- vietati -->
          <text x="300" y="184" class="t b" fill="var(--red)">Non può contenere</text>
          {#each [{ y: 204, t: "Cifratura o password" }, { y: 230, t: "JavaScript, azioni Launch, multimedia" }, { y: 256, t: "Riferimenti a font o file esterni" }, { y: 282, t: "Trasparenza e livelli (solo in PDF/A-1)" }] as r (r.y)}
            <rect x="296" y={r.y - 15} width="392" height="22" rx="7" fill="var(--surface-2)" stroke="var(--red)" stroke-opacity="0.4" />
            <path d="M{307} {r.y - 9} l 9 9 M{316} {r.y - 9} l -9 9" stroke="var(--red)" stroke-width="2" stroke-linecap="round" />
            <text x="326" y={r.y} class="t s">{r.t}</text>
          {/each}
        </svg>
        <figcaption>
          I quattro obblighi valgono per ogni parte di PDF/A. L'ultimo divieto riguarda solo la parte 1: dalla 2 in poi
          trasparenza e livelli sono ammessi.
        </figcaption>
      </figure>

      <h3>Parti e livelli a colpo d'occhio</h3>
      <div class="matrix" role="table">
        <div class="mrow mhead" role="row">
          <span role="columnheader">Parte</span>
          {#each MATRIX_COLS as c (c)}<span role="columnheader">{c}</span>{/each}
        </div>
        {#each MATRIX as r (r.part)}
          <div class="mrow" role="row">
            <span role="cell">
              <b>{r.part}</b>
              <em>{r.iso}</em>
            </span>
            {#each r.cells as cell, i (i)}
              <span role="cell" class="cellv" class:none={cell === null || cell === "—"}>
                {cell === null ? "" : cell}
              </span>
            {/each}
          </div>
        {/each}
      </div>
      <p class="note">
        In PDF/A-4 le lettere spariscono: l'Unicode è sempre obbligatorio e l'accessibilità non è più un livello, ma si
        ottiene dichiarando anche PDF/UA. Restano solo i suffissi <b>f</b> per gli allegati ed <b>e</b> per il 3D.
      </p>

      <h3>Come si sceglie in pratica</h3>
      <ul class="bullets">
        <li><b>Serve solo conservare?</b> PDF/A-2b. È il punto di equilibrio fra vincoli e resa.</li>
        <li><b>Il testo deve essere cercabile?</b> PDF/A-2u, che costa quasi nulla in più.</li>
        <li><b>C'è un obbligo di accessibilità?</b> PDF/A-2a più PDF/UA-1, dichiarati entrambi.</li>
        <li><b>Devi allegare un XML o un originale?</b> PDF/A-3 (o A-4f se sei già su PDF 2.0).</li>
        <li><b>Te lo impone un capitolato vecchio?</b> Solo allora PDF/A-1b.</li>
      </ul>
    </div>
  {:else if tab === "accesso"}
    <div class="page">
      <h3>Che cosa rende un PDF accessibile</h3>
      <p class="lead">
        Un PDF disegna glifi in posizioni sulla pagina: di per sé non sa che quella riga più grande è un titolo, né in
        che ordine si legge. PDF/UA impone di affiancare alla grafica un <b>albero della struttura</b> che descrive il
        significato del contenuto. Le tecnologie assistive leggono quello, non la pagina.
      </p>

      <figure>
        <svg viewBox="0 0 720 320" role="img" aria-label="Confronto fra una pagina senza tag e lo stesso contenuto con albero della struttura">
          <!-- sinistra: pagina visiva -->
          <text x="34" y="22" class="t b">Ciò che si vede</text>
          <rect x="30" y="32" width="238" height="266" rx="10" fill="var(--surface)" stroke="var(--border-2)" stroke-width="2" />
          <rect x="52" y="56" width="150" height="13" rx="4" fill="var(--ink)" opacity="0.75" />
          <rect x="52" y="86" width="194" height="7" rx="3" fill="var(--ink-faint)" opacity="0.4" />
          <rect x="52" y="100" width="194" height="7" rx="3" fill="var(--ink-faint)" opacity="0.4" />
          <rect x="52" y="114" width="128" height="7" rx="3" fill="var(--ink-faint)" opacity="0.4" />
          <rect x="52" y="136" width="90" height="60" rx="6" fill="var(--primary-soft)" stroke="var(--primary)" stroke-opacity="0.35" />
          <text x="97" y="170" class="t s" text-anchor="middle">figura</text>
          <rect x="156" y="136" width="90" height="60" rx="6" fill="var(--surface-2)" stroke="var(--border-2)" />
          <line x1="156" y1="156" x2="246" y2="156" stroke="var(--border-2)" />
          <line x1="201" y1="136" x2="201" y2="196" stroke="var(--border-2)" />
          <text x="201" y="151" class="t s" text-anchor="middle">tabella</text>
          <rect x="52" y="216" width="194" height="7" rx="3" fill="var(--ink-faint)" opacity="0.4" />
          <rect x="52" y="230" width="160" height="7" rx="3" fill="var(--ink-faint)" opacity="0.4" />

          <!-- destra: albero -->
          <text x="316" y="22" class="t b">Ciò che legge uno screen reader</text>
          <rect x="312" y="32" width="392" height="266" rx="10" fill="var(--surface-2)" stroke="var(--border-2)" />

          <text x="332" y="58" class="t b" fill="var(--green)">Document</text>
          <text x="332" y="58" class="t s" fill="var(--ink-faint)" dx="86">lang="it-IT"</text>

          <line x1="340" y1="64" x2="340" y2="262" stroke="var(--border-2)" />

          {#each [{ y: 84, tag: "H1", txt: "Titolo del documento" }, { y: 110, tag: "P", txt: "Paragrafo introduttivo" }, { y: 136, tag: "Figure", txt: "Alt: «grafico delle vendite 2026»" }, { y: 162, tag: "Table", txt: "con intestazioni TH" }, { y: 232, tag: "P", txt: "Paragrafo conclusivo" }, { y: 258, tag: "Artifact", txt: "numero di pagina — ignorato" }] as n (n.y)}
            <line x1="340" y1={n.y - 4} x2="356" y2={n.y - 4} stroke="var(--border-2)" />
            <rect x="358" y={n.y - 15} width="62" height="18" rx="6" fill="var(--surface)" stroke="var(--green)" stroke-opacity="0.5" />
            <text x="389" y={n.y - 2} class="t s" text-anchor="middle" fill="var(--green)">{n.tag}</text>
            <text x="430" y={n.y - 2} class="t s">{n.txt}</text>
          {/each}

          <!-- sotto-albero della tabella -->
          <line x1="372" y1="170" x2="372" y2="206" stroke="var(--border-2)" />
          {#each [{ y: 190, tag: "TR › TH", txt: "riga di intestazione" }, { y: 210, tag: "TR › TD", txt: "riga di dati" }] as n (n.y)}
            <line x1="372" y1={n.y - 4} x2="388" y2={n.y - 4} stroke="var(--border-2)" />
            <text x="394" y={n.y - 1} class="t s" fill="var(--ink-dim)">{n.tag} — {n.txt}</text>
          {/each}
        </svg>
        <figcaption>
          Stessa pagina, due letture. Senza l'albero a destra, la colonna di sinistra è solo un insieme di segni: la
          figura non ha descrizione, la tabella non ha intestazioni e il numero di pagina viene letto come se fosse
          contenuto.
        </figcaption>
      </figure>

      <h3>I requisiti che si sbagliano più spesso</h3>
      <ul class="bullets">
        <li><b>Ordine di lettura.</b> Deve seguire il senso del discorso, non la posizione sulla pagina. Su un layout a due colonne è l'errore più frequente.</li>
        <li><b>Testo alternativo.</b> Obbligatorio per le immagini informative; le decorative vanno invece marcate come artefatti, non descritte.</li>
        <li><b>Tabelle.</b> Servono celle TH vere e l'associazione con le celle dati. Una tabella usata per impaginare non è una tabella.</li>
        <li><b>Titoli.</b> Gerarchia senza salti: da H1 si passa a H2, mai direttamente a H3.</li>
        <li><b>Lingua.</b> Va dichiarata sul documento e su ogni brano in lingua diversa, altrimenti la sintesi vocale sbaglia pronuncia.</li>
        <li><b>Titolo nei metadati.</b> Con DisplayDocTitle attivo, altrimenti il lettore annuncia il nome del file.</li>
        <li><b>Mappatura ToUnicode.</b> Senza, il testo si vede ma non si può leggere ad alta voce né cercare.</li>
      </ul>

      <div class="callout">
        <b>Un validatore non basta.</b> Il Matterhorn Protocol elenca 136 condizioni di errore per PDF/UA e ne dichiara
        automatizzabile solo una parte: l'ordine di lettura corretto o la sensatezza di un testo alternativo può
        giudicarli solo una persona. Uno strumento può dirti che un <i>Alt</i> manca, non che è scritto male.
      </div>
    </div>
  {:else}
    <div class="page">
      <h3>Quale scegliere</h3>
      <p class="lead">Parti dallo scopo del documento, non dal formato.</p>

      <div class="cards">
        {#each [{ q: "Deve durare nel tempo", a: "PDF/A-2u", why: "Archiviazione senza dipendenze esterne, con testo cercabile. Passa a PDF/A-2a se servono anche i tag.", c: "var(--primary)" }, { q: "Deve essere accessibile", a: "PDF/UA-1", why: "Struttura logica, ordine di lettura e alternative testuali. Spesso in coppia con PDF/A-2a.", c: "var(--green)" }, { q: "Va in tipografia", a: "PDF/X-4", why: "Trasparenza conservata, colore gestito e font inclusi. X-1a solo se te lo impongono.", c: "var(--amber)" }, { q: "Trasporta un XML o un allegato", a: "PDF/A-3u", why: "L'unico modo standard di mettere dati leggibili a macchina dentro a un file d'archivio.", c: "var(--primary-2)" }, { q: "Contiene un modello 3D", a: "PDF/A-4e", why: "Archiviazione con geometria navigabile. PDF/E-2 se conta più lo scambio che la conservazione.", c: "var(--gold)" }, { q: "È una scansione", a: "PDF/R-1", why: "Contenitore raster standard. Aggiungi OCR, altrimenti resta un'immagine muta.", c: "var(--c-test)" }, { q: "Bollette o mailing personalizzati", a: "PDF/VT-1", why: "Stampa transazionale ad alto volume con risorse ricorrenti gestite dal RIP.", c: "var(--c-convert)" }, { q: "Ha valore legale", a: "PAdES B-LTA", why: "Firma con marca temporale, verificabile anche dopo la scadenza del certificato.", c: "var(--red)" }] as c (c.q)}
          <div class="card" style="--c:{c.c}">
            <div class="cq">{c.q}</div>
            <div class="ca">{c.a}</div>
            <div class="cw">{c.why}</div>
          </div>
        {/each}
      </div>

      <h3>Errori ricorrenti</h3>
      <ul class="bullets">
        <li><b>Credere che PDF/A implichi accessibilità.</b> Solo il livello <i>a</i> tocca la struttura, e comunque non equivale a PDF/UA.</li>
        <li><b>Convertire in PDF/A rasterizzando.</b> Il file passa la validazione ma il testo diventa immagine: si perde ricerca, selezione e accessibilità.</li>
        <li><b>Scegliere PDF/A-1 per abitudine.</b> Vieta la trasparenza, quindi forza appiattimenti o rasterizzazioni che A-2 non richiederebbe.</li>
        <li><b>Usare PDF/A-3 come deposito.</b> Nulla garantisce che l'allegato sia conservabile: lo standard copre il contenitore, non il contenuto.</li>
        <li><b>Fidarsi di un solo validatore.</b> Per PDF/A serve veraPDF; per PDF/UA nessuno strumento certifica da solo la conformità.</li>
        <li><b>Firmare e poi ottimizzare.</b> Qualunque riscrittura non incrementale invalida la firma.</li>
      </ul>

      <div class="callout">
        <b>Nota su questo strumento.</b> Il check PDF/A di DevToys è euristico e guarda pochi indicatori (tag, font,
        lingua, marcatore XMP, cifratura): serve a intercettare i problemi evidenti, non a certificare. Per una verifica
        formale usa veraPDF.
      </div>
    </div>
  {/if}
</div>

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 10px;
  }
  .chip {
    --chip: var(--ink-dim);
    border: 1.5px solid var(--border-2);
    background: var(--surface-2);
    color: var(--ink-dim);
    border-radius: 999px;
    padding: 5px 12px;
    font-size: 12.5px;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
  }
  .chip:hover {
    background: var(--row-hover);
  }
  .chip.on {
    border-color: var(--chip);
    color: var(--chip);
    background: var(--surface);
  }
  .chip .n {
    opacity: 0.55;
    margin-left: 4px;
    font-size: 11px;
  }
  .scope {
    margin: 0 0 10px;
    font-size: 13px;
    color: var(--ink-dim);
  }

  .list {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .item {
    border-radius: 12px;
    background: var(--surface-2);
    border: 1.5px solid var(--border);
    /* `overflow: hidden` azzera la min-height automatica di un flex item:
       senza flex-shrink le righe collassano quando la lista eccede l'altezza. */
    overflow: hidden;
    flex-shrink: 0;
  }
  .item.open {
    background: var(--surface);
    border-color: var(--border-2);
  }
  .head {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 11px 15px;
    background: none;
    border: 0;
    font-family: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .head:hover {
    background: var(--row-hover);
  }
  .badge {
    font-family: var(--mono);
    font-size: 14px;
    font-weight: 800;
    color: var(--c);
    background: color-mix(in srgb, var(--c) 12%, transparent);
    border: 1.5px solid color-mix(in srgb, var(--c) 35%, transparent);
    border-radius: 8px;
    padding: 5px 0;
    width: 62px;
    flex-shrink: 0;
    text-align: center;
  }
  .body {
    min-width: 0;
    flex: 1;
    display: block;
  }
  .name {
    font-weight: 750;
    font-size: 14.5px;
    display: flex;
    align-items: baseline;
    gap: 9px;
    flex-wrap: wrap;
  }
  .iso {
    font-size: 11px;
    font-weight: 600;
    color: var(--ink-faint);
    font-family: var(--mono);
  }
  .desc {
    display: block;
    color: var(--ink-dim);
    font-size: 13px;
    margin-top: 1px;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 5px;
    font-size: 11px;
    color: var(--ink-faint);
  }
  .kv b {
    font-weight: 750;
  }
  .caret {
    flex-shrink: 0;
    font-size: 18px;
    color: var(--ink-faint);
    width: 16px;
    text-align: center;
  }

  .detail {
    padding: 2px 15px 15px 92px;
    border-top: 1px solid var(--border);
  }
  .summary {
    margin: 12px 0;
    font-size: 13.5px;
    line-height: 1.55;
  }
  .cols {
    display: flex;
    flex-wrap: wrap;
    gap: 10px 26px;
  }
  .col {
    flex: 1 1 260px;
    min-width: 0;
  }
  .col h4 {
    margin: 0 0 5px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .col h4.ok {
    color: var(--green);
  }
  .col h4.ko {
    color: var(--red);
  }
  .col ul {
    margin: 0;
    padding-left: 17px;
  }
  .col li {
    font-size: 12.5px;
    color: var(--ink-dim);
    margin-bottom: 3px;
    line-height: 1.45;
  }
  .when {
    margin: 13px 0 0;
    font-size: 13px;
    line-height: 1.5;
  }
  .note {
    margin: 9px 0 0;
    font-size: 12.5px;
    color: var(--ink-dim);
    border-left: 3px solid var(--gold);
    padding-left: 10px;
    line-height: 1.5;
  }

  /* --- pagine della guida --- */
  .page {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding-right: 4px;
  }
  .page h3 {
    margin: 22px 0 8px;
    font-size: 15.5px;
  }
  .page h3:first-child {
    margin-top: 2px;
  }
  .lead {
    margin: 0 0 4px;
    font-size: 13.5px;
    line-height: 1.6;
    color: var(--ink-dim);
    max-width: 78ch;
  }
  figure {
    margin: 14px 0 4px;
    padding: 14px;
    border-radius: var(--radius);
    background: var(--surface-2);
    border: 1.5px solid var(--border);
  }
  figure svg {
    width: 100%;
    height: auto;
    display: block;
  }
  figcaption {
    margin-top: 10px;
    font-size: 12px;
    color: var(--ink-dim);
    line-height: 1.5;
  }
  :global(.page svg text.t) {
    font-family: var(--font);
    font-size: 11px;
    fill: var(--ink);
  }
  :global(.page svg text.t.b) {
    font-weight: 750;
    font-size: 11.5px;
  }
  :global(.page svg text.t.s) {
    font-size: 10px;
    fill: var(--ink-dim);
  }

  .matrix {
    margin-top: 10px;
    border: 1.5px solid var(--border-2);
    border-radius: 12px;
    overflow: hidden;
  }
  .mrow {
    display: grid;
    grid-template-columns: 1.6fr repeat(5, 1fr);
    align-items: center;
  }
  .mrow > span {
    padding: 9px 12px;
    font-size: 12.5px;
    border-top: 1px solid var(--border);
  }
  .mrow.mhead > span {
    border-top: 0;
    background: var(--surface-3);
    font-weight: 750;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--ink-dim);
  }
  .mrow em {
    display: block;
    font-style: normal;
    font-size: 10.5px;
    color: var(--ink-faint);
    font-family: var(--mono);
  }
  .cellv {
    font-family: var(--mono);
    font-weight: 750;
    color: var(--primary);
    text-align: center;
  }
  .cellv.none {
    color: var(--ink-faint);
    opacity: 0.4;
  }

  .bullets {
    margin: 6px 0 0;
    padding-left: 19px;
  }
  .bullets li {
    font-size: 13px;
    line-height: 1.6;
    color: var(--ink-dim);
    margin-bottom: 6px;
    max-width: 82ch;
  }
  .bullets b {
    color: var(--ink);
  }

  .callout {
    margin-top: 16px;
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--surface-2);
    border: 1.5px solid var(--border-2);
    border-left: 4px solid var(--gold);
    font-size: 13px;
    line-height: 1.6;
    color: var(--ink-dim);
    max-width: 82ch;
  }
  .callout b {
    color: var(--ink);
  }

  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(232px, 1fr));
    gap: 10px;
    margin-top: 10px;
  }
  .card {
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--surface-2);
    border: 1.5px solid var(--border);
    border-top: 3px solid var(--c);
  }
  .cq {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--ink-faint);
    font-weight: 750;
  }
  .ca {
    font-family: var(--mono);
    font-weight: 800;
    font-size: 15px;
    color: var(--c);
    margin: 3px 0 5px;
  }
  .cw {
    font-size: 12.5px;
    color: var(--ink-dim);
    line-height: 1.5;
  }
</style>
