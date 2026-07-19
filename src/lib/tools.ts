import type { Component } from "svelte";

import JsonYaml from "./tools/JsonYaml.svelte";
import NumberBase from "./tools/NumberBase.svelte";
import UnixTime from "./tools/UnixTime.svelte";
import ColorConv from "./tools/ColorConv.svelte";
import Cron from "./tools/Cron.svelte";
import JsonCsv from "./tools/JsonCsv.svelte";
import Roman from "./tools/Roman.svelte";
import Contrast from "./tools/Contrast.svelte";
import UrlParser from "./tools/UrlParser.svelte";
import TomlJson from "./tools/TomlJson.svelte";
import CurlToCode from "./tools/CurlToCode.svelte";

import Base64Text from "./tools/Base64Text.svelte";
import Base64Image from "./tools/Base64Image.svelte";
import UrlEncode from "./tools/UrlEncode.svelte";
import HtmlEntities from "./tools/HtmlEntities.svelte";
import Jwt from "./tools/Jwt.svelte";
import Unicode from "./tools/Unicode.svelte";
import BasicAuth from "./tools/BasicAuth.svelte";
import CertDecoder from "./tools/CertDecoder.svelte";

import JsonFormat from "./tools/JsonFormat.svelte";
import SqlFormat from "./tools/SqlFormat.svelte";
import XmlFormat from "./tools/XmlFormat.svelte";

import Uuid from "./tools/Uuid.svelte";
import Hash from "./tools/Hash.svelte";
import Hmac from "./tools/Hmac.svelte";
import Password from "./tools/Password.svelte";
import Lorem from "./tools/Lorem.svelte";
import Qr from "./tools/Qr.svelte";

import CaseConv from "./tools/CaseConv.svelte";
import Diff from "./tools/Diff.svelte";
import TextStats from "./tools/TextStats.svelte";
import Regex from "./tools/Regex.svelte";
import Markdown from "./tools/Markdown.svelte";
import Escape from "./tools/Escape.svelte";
import TextLines from "./tools/TextLines.svelte";
import Slug from "./tools/Slug.svelte";
import JsonDiff from "./tools/JsonDiff.svelte";

import PdfToImage from "./tools/PdfToImage.svelte";
import PdfToText from "./tools/PdfToText.svelte";
import PdfToDocx from "./tools/PdfToDocx.svelte";
import ImagesToPdf from "./tools/ImagesToPdf.svelte";
import PdfMerge from "./tools/PdfMerge.svelte";
import PdfSplit from "./tools/PdfSplit.svelte";
import PdfRotate from "./tools/PdfRotate.svelte";
import PdfDelete from "./tools/PdfDelete.svelte";
import PdfReorder from "./tools/PdfReorder.svelte";
import PdfCompare from "./tools/PdfCompare.svelte";
import PdfOptimize from "./tools/PdfOptimize.svelte";
import PdfSanitize from "./tools/PdfSanitize.svelte";
import PdfWatermark from "./tools/PdfWatermark.svelte";
import PdfMetadata from "./tools/PdfMetadata.svelte";
import PdfStructure from "./tools/PdfStructure.svelte";
import PdfUa from "./tools/PdfUa.svelte";
import PdfA from "./tools/PdfA.svelte";

import LoadTest from "./tools/LoadTest.svelte";
import MockServer from "./tools/MockServer.svelte";
import DataGen from "./tools/DataGen.svelte";
import SmokeRunner from "./tools/SmokeRunner.svelte";
import HttpProxy from "./tools/HttpProxy.svelte";
import HttpStatus from "./tools/HttpStatus.svelte";

export type CategoryId = "convert" | "encode" | "format" | "generate" | "text" | "pdf" | "test";

export interface Category {
  id: CategoryId;
  label: string;
  emoji: string;
}

export const CATEGORIES: Category[] = [
  { id: "convert", label: "Convertitori", emoji: "🔄" },
  { id: "encode", label: "Codifica / Decodifica", emoji: "🔐" },
  { id: "format", label: "Formattatori", emoji: "🧾" },
  { id: "generate", label: "Generatori", emoji: "🎲" },
  { id: "text", label: "Testo", emoji: "🔤" },
  { id: "pdf", label: "PDF", emoji: "📄" },
  { id: "test", label: "API & Test", emoji: "🧪" },
];

export interface ToolMeta {
  id: string;
  name: string;
  desc: string;
  category: CategoryId;
  emoji: string;
  keywords: string;
  component: Component;
}

export const TOOLS: ToolMeta[] = [
  // convert
  { id: "json-yaml", name: "JSON ⇄ YAML", desc: "Converti tra JSON, YAML e viceversa.", category: "convert", emoji: "🔄", keywords: "json yaml convert", component: JsonYaml },
  { id: "number-base", name: "Convertitore di base", desc: "Binario, ottale, decimale ed esadecimale.", category: "convert", emoji: "🔢", keywords: "base binary hex octal radix numero", component: NumberBase },
  { id: "unix-time", name: "Timestamp Unix", desc: "Epoch ⇄ data leggibile, con fuso locale e UTC.", category: "convert", emoji: "⏱️", keywords: "unix epoch timestamp date data ora", component: UnixTime },
  { id: "color", name: "Convertitore colore", desc: "HEX, RGB e HSL con anteprima.", category: "convert", emoji: "🎨", keywords: "color colore hex rgb hsl", component: ColorConv },
  { id: "cron", name: "Cron in italiano", desc: "Spiega un'espressione cron in linguaggio naturale.", category: "convert", emoji: "🗓️", keywords: "cron crontab schedule pianificazione", component: Cron },
  { id: "json-csv", name: "JSON ⇄ CSV", desc: "Converti un array di oggetti JSON in CSV e viceversa.", category: "convert", emoji: "📑", keywords: "json csv converti tabella foglio spreadsheet", component: JsonCsv },
  { id: "url-parser", name: "Analizza URL", desc: "Scompone un URL in host, percorso e parametri, poi lo ricostruisce.", category: "convert", emoji: "🧭", keywords: "url parser componenti query parametri host percorso", component: UrlParser },
  { id: "roman", name: "Numeri romani", desc: "Converti tra numeri arabi e romani.", category: "convert", emoji: "🏛️", keywords: "numeri romani roman numerals conversione", component: Roman },
  { id: "contrast", name: "Contrasto colori (WCAG)", desc: "Rapporto di contrasto e conformità AA/AAA.", category: "convert", emoji: "🌗", keywords: "contrasto wcag accessibilità colore ratio aa aaa", component: Contrast },
  { id: "toml-json", name: "TOML ⇄ JSON", desc: "Converti tra TOML e JSON in entrambe le direzioni.", category: "convert", emoji: "🧰", keywords: "toml json converti config configurazione cargo pyproject", component: TomlJson },
  { id: "curl-code", name: "cURL → codice", desc: "Trasforma un comando curl in fetch, Python, axios o HTTPie.", category: "convert", emoji: "🌀", keywords: "curl fetch python requests axios httpie http codice converti comando", component: CurlToCode },

  // encode
  { id: "base64", name: "Base64 testo", desc: "Codifica e decodifica testo in Base64 (UTF-8).", category: "encode", emoji: "🔡", keywords: "base64 encode decode testo", component: Base64Text },
  { id: "base64-image", name: "Base64 ⇄ immagine", desc: "Immagine in data-URI Base64 e viceversa.", category: "encode", emoji: "🖼️", keywords: "base64 image immagine data uri", component: Base64Image },
  { id: "url", name: "URL encode / decode", desc: "Percent-encoding di URL e componenti.", category: "encode", emoji: "🔗", keywords: "url percent encode decode", component: UrlEncode },
  { id: "html-entities", name: "Entità HTML", desc: "Escape e unescape di entità HTML.", category: "encode", emoji: "🏷️", keywords: "html entities escape amp lt gt", component: HtmlEntities },
  { id: "jwt", name: "Decoder JWT", desc: "Ispeziona header, payload, scadenza e verifica la firma di un JWT.", category: "encode", emoji: "🔑", keywords: "jwt token json web decode firma verifica hmac rsa ecdsa signature", component: Jwt },
  { id: "unicode", name: "Testo ⇄ Unicode", desc: "Codifica in escape \\uXXXX o Phoenix /UXXXX e viceversa.", category: "encode", emoji: "🔣", keywords: "unicode escape codepoint code point \\u phoenix testo caratteri", component: Unicode },
  { id: "basic-auth", name: "Basic Auth", desc: "Genera e decodifica l'header Authorization: Basic.", category: "encode", emoji: "🪪", keywords: "basic auth authorization header credenziali base64 utente password", component: BasicAuth },
  { id: "cert-decode", name: "Decoder certificati X.509", desc: "Ispeziona un certificato PEM: soggetto, emittente, validità, SAN e impronte.", category: "encode", emoji: "📜", keywords: "certificato certificate x509 pem ssl tls san impronta fingerprint decoder", component: CertDecoder },

  // format
  { id: "json-format", name: "JSON formatter", desc: "Formatta, valida e minifica JSON.", category: "format", emoji: "🧾", keywords: "json format pretty minify validate", component: JsonFormat },
  { id: "sql-format", name: "SQL formatter", desc: "Indenta query SQL per più dialetti.", category: "format", emoji: "🗄️", keywords: "sql format query oracle postgres", component: SqlFormat },
  { id: "xml-format", name: "XML formatter", desc: "Indenta e valida documenti XML.", category: "format", emoji: "📄", keywords: "xml format pretty indent", component: XmlFormat },

  // generate
  { id: "uuid", name: "Generatore UUID", desc: "UUID v4 casuali o v7 ordinati nel tempo.", category: "generate", emoji: "🆔", keywords: "uuid guid v4 v7 identificatore", component: Uuid },
  { id: "hash", name: "Hash", desc: "MD5, SHA-1/2/3 e CRC32 di un testo.", category: "generate", emoji: "#️⃣", keywords: "hash md5 sha crc checksum digest", component: Hash },
  { id: "hmac", name: "HMAC", desc: "Firma HMAC con chiave e algoritmo a scelta.", category: "generate", emoji: "🔐", keywords: "hmac sha firma mac chiave", component: Hmac },
  { id: "password", name: "Password & token", desc: "Genera password e token robusti (CSPRNG).", category: "generate", emoji: "🎲", keywords: "password token random sicuro", component: Password },
  { id: "lorem", name: "Lorem ipsum", desc: "Testo segnaposto: parole, frasi o paragrafi.", category: "generate", emoji: "📝", keywords: "lorem ipsum placeholder segnaposto", component: Lorem },
  { id: "qr", name: "QR code", desc: "Genera un QR code da testo o URL.", category: "generate", emoji: "▦", keywords: "qr code barcode url", component: Qr },

  // text
  { id: "case", name: "Convertitore maiuscole", desc: "camelCase, snake_case, kebab, UPPER e altro.", category: "text", emoji: "🔤", keywords: "case camel snake kebab maiuscole", component: CaseConv },
  { id: "diff", name: "Confronto testo", desc: "Differenze riga per riga tra due testi.", category: "text", emoji: "📊", keywords: "diff compare confronto differenze", component: Diff },
  { id: "text-stats", name: "Statistiche testo", desc: "Caratteri, parole, righe e tempo di lettura.", category: "text", emoji: "📈", keywords: "statistiche conteggio parole caratteri", component: TextStats },
  { id: "regex", name: "Tester regex", desc: "Prova espressioni regolari con evidenziazione.", category: "text", emoji: "✳️", keywords: "regex regexp espressioni regolari match", component: Regex },
  { id: "markdown", name: "Anteprima Markdown", desc: "Rendi Markdown in HTML in tempo reale.", category: "text", emoji: "📓", keywords: "markdown md preview anteprima", component: Markdown },
  { id: "escape", name: "Escape stringa", desc: "Escape/unescape per JSON, JS e altri linguaggi.", category: "text", emoji: "␛", keywords: "escape unescape backslash stringa", component: Escape },
  { id: "text-lines", name: "Utilità righe", desc: "Ordina, deduplica, numera e ripulisce elenchi di righe.", category: "text", emoji: "📋", keywords: "righe linee ordina dedup deduplica unique sort trim elenco lista", component: TextLines },
  { id: "slug", name: "Slugify", desc: "Trasforma un testo in uno slug per URL.", category: "text", emoji: "🐌", keywords: "slug slugify url friendly permalink accenti", component: Slug },
  { id: "json-diff", name: "Diff JSON", desc: "Confronto semantico tra due JSON: chiavi aggiunte, rimosse e modificate.", category: "text", emoji: "🆚", keywords: "json diff confronto differenze semantico strutturale chiavi compara", component: JsonDiff },

  // pdf
  { id: "pdf-to-image", name: "PDF → immagini", desc: "Esporta le pagine come PNG o JPEG.", category: "pdf", emoji: "🖼️", keywords: "pdf immagini png jpeg render pagine", component: PdfToImage },
  { id: "pdf-to-text", name: "PDF → testo", desc: "Estrai il testo in Testo, Markdown o HTML.", category: "pdf", emoji: "📃", keywords: "pdf testo estrai markdown html", component: PdfToText },
  { id: "pdf-to-docx", name: "PDF → Word", desc: "Esporta in .docx (beta, solo testo).", category: "pdf", emoji: "📝", keywords: "pdf word docx converti office", component: PdfToDocx },
  { id: "images-to-pdf", name: "Immagini → PDF", desc: "Unisci più immagini in un unico PDF.", category: "pdf", emoji: "🗂️", keywords: "immagini pdf jpg png unisci", component: ImagesToPdf },
  { id: "pdf-merge", name: "Unisci PDF", desc: "Combina più PDF in ordine.", category: "pdf", emoji: "➕", keywords: "pdf unisci merge combina", component: PdfMerge },
  { id: "pdf-split", name: "Dividi / estrai pagine", desc: "Estrai un intervallo di pagine.", category: "pdf", emoji: "✂️", keywords: "pdf dividi estrai split pagine", component: PdfSplit },
  { id: "pdf-rotate", name: "Ruota pagine", desc: "Ruota pagine di 90°, 180° o 270°.", category: "pdf", emoji: "🔃", keywords: "pdf ruota rotate pagine", component: PdfRotate },
  { id: "pdf-delete", name: "Elimina pagine", desc: "Rimuovi pagine da un PDF.", category: "pdf", emoji: "🗑️", keywords: "pdf elimina rimuovi pagine", component: PdfDelete },
  { id: "pdf-reorder", name: "Riordina pagine", desc: "Cambia l'ordine delle pagine.", category: "pdf", emoji: "🔀", keywords: "pdf riordina ordine pagine", component: PdfReorder },
  { id: "pdf-compare", name: "Confronta PDF", desc: "Diff testo e visivo tra due PDF.", category: "pdf", emoji: "🔍", keywords: "pdf confronta diff compara", component: PdfCompare },
  { id: "pdf-optimize", name: "Ottimizza / comprimi", desc: "Riduci il peso del PDF.", category: "pdf", emoji: "🗜️", keywords: "pdf ottimizza comprimi peso dimensione", component: PdfOptimize },
  { id: "pdf-sanitize", name: "Sanitizza PDF", desc: "Rimuovi JavaScript, metadati e allegati.", category: "pdf", emoji: "🧼", keywords: "pdf sanitizza pulisci javascript metadati", component: PdfSanitize },
  { id: "pdf-watermark", name: "Filigrana", desc: "Applica una filigrana di testo.", category: "pdf", emoji: "💧", keywords: "pdf filigrana watermark testo", component: PdfWatermark },
  { id: "pdf-metadata", name: "Metadati PDF", desc: "Leggi titolo, autore e proprietà del documento.", category: "pdf", emoji: "🏷️", keywords: "pdf metadati proprietà autore titolo", component: PdfMetadata },
  { id: "pdf-structure", name: "Struttura / tag tree", desc: "Esporta l'albero dei tag in JSON o XML.", category: "pdf", emoji: "🌳", keywords: "pdf struttura tag tree accessibilità", component: PdfStructure },
  { id: "pdf-ua", name: "Check PDF/UA", desc: "Verifica di accessibilità (Matterhorn).", category: "pdf", emoji: "♿", keywords: "pdf ua accessibilità matterhorn verifica", component: PdfUa },
  { id: "pdf-a", name: "Check PDF/A (base)", desc: "Controllo di conformità di base per l'archiviazione.", category: "pdf", emoji: "📚", keywords: "pdf a archiviazione conformità verifica", component: PdfA },

  // test
  { id: "load-test", name: "Load / performance test", desc: "Carico concorrente con p50/p95/p99 e grafici live.", category: "test", emoji: "🚀", keywords: "load test carico performance jmeter benchmark percentili", component: LoadTest },
  { id: "mock-server", name: "Mock server", desc: "Endpoint finti con regole, latenza ed errori.", category: "test", emoji: "🎭", keywords: "mock server stub api finto wiremock", component: MockServer },
  { id: "data-gen", name: "Generatore dati di test", desc: "IBAN, codice fiscale, P.IVA e persone in JSON/CSV/SQL.", category: "test", emoji: "🧬", keywords: "dati test iban codice fiscale piva fake generatore", component: DataGen },
  { id: "smoke-runner", name: "Smoke / synthetic runner", desc: "Sequenza di chiamate con asserzioni.", category: "test", emoji: "🔦", keywords: "smoke synthetic test asserzioni steps collaudo", component: SmokeRunner },
  { id: "http-proxy", name: "HTTP proxy / inspector", desc: "Reverse-proxy verso un target: logga e ispeziona.", category: "test", emoji: "🛰️", keywords: "http proxy inspector traffico reverse debug", component: HttpProxy },
  { id: "http-status", name: "Codici di stato HTTP", desc: "Riferimento ricercabile dei codici di stato HTTP.", category: "test", emoji: "📟", keywords: "http status codici stato 404 500 riferimento reference", component: HttpStatus },
];

export function toolsByCategory(cat: CategoryId): ToolMeta[] {
  return TOOLS.filter((t) => t.category === cat);
}

export function searchTools(q: string): ToolMeta[] {
  const s = q.trim().toLowerCase();
  if (!s) return TOOLS;
  return TOOLS.filter(
    (t) =>
      t.name.toLowerCase().includes(s) ||
      t.desc.toLowerCase().includes(s) ||
      t.keywords.includes(s),
  );
}
