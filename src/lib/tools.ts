import type { Component } from "svelte";

import JsonYaml from "./tools/JsonYaml.svelte";
import NumberBase from "./tools/NumberBase.svelte";
import UnixTime from "./tools/UnixTime.svelte";
import ColorConv from "./tools/ColorConv.svelte";
import Cron from "./tools/Cron.svelte";

import Base64Text from "./tools/Base64Text.svelte";
import Base64Image from "./tools/Base64Image.svelte";
import UrlEncode from "./tools/UrlEncode.svelte";
import HtmlEntities from "./tools/HtmlEntities.svelte";
import Jwt from "./tools/Jwt.svelte";

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

export type CategoryId = "convert" | "encode" | "format" | "generate" | "text";

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

  // encode
  { id: "base64", name: "Base64 testo", desc: "Codifica e decodifica testo in Base64 (UTF-8).", category: "encode", emoji: "🔡", keywords: "base64 encode decode testo", component: Base64Text },
  { id: "base64-image", name: "Base64 ⇄ immagine", desc: "Immagine in data-URI Base64 e viceversa.", category: "encode", emoji: "🖼️", keywords: "base64 image immagine data uri", component: Base64Image },
  { id: "url", name: "URL encode / decode", desc: "Percent-encoding di URL e componenti.", category: "encode", emoji: "🔗", keywords: "url percent encode decode", component: UrlEncode },
  { id: "html-entities", name: "Entità HTML", desc: "Escape e unescape di entità HTML.", category: "encode", emoji: "🏷️", keywords: "html entities escape amp lt gt", component: HtmlEntities },
  { id: "jwt", name: "Decoder JWT", desc: "Ispeziona header, payload e scadenza di un JWT.", category: "encode", emoji: "🔑", keywords: "jwt token json web decode", component: Jwt },

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
