//! Font tools — inspect, subset and browse the glyphs of a font file, and list
//! the fonts a PDF actually uses.
//!
//! Font files are parsed with `allsorts`, which reads OpenType, TrueType,
//! TrueType collections, WOFF and WOFF2 behind one interface. The PDF side goes
//! through the Pdfium instance the PDF tools already share.
//!
//! Everything CPU-bound runs inside `tokio::task::spawn_blocking`, like the PDF
//! commands, and errors are flattened to `String` for the frontend.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use allsorts::binary::read::ReadScope;
use allsorts::font::read_cmap_subtable;
use allsorts::font_data::FontData;
use allsorts::post::PostTable;
use allsorts::subset::{subset, CmapTarget, SubsetProfile};
use allsorts::tables::cmap::Cmap;
use allsorts::tables::os2::Os2;
use allsorts::tables::variable_fonts::fvar::FvarTable;
use allsorts::tables::{FontTableProvider, HeadTable, HheaTable, MaxpTable};
use allsorts::tag::{self, DisplayTag};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Return shapes (camelCase on the wire)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameEntry {
    pub label: String,
    pub value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableEntry {
    pub tag: String,
    pub size: Option<u32>,
}

/// The `fsType` field of OS/2, decoded. This is the licence the foundry
/// attached to the file, and the reason a font that renders fine locally can
/// still be illegal to embed in a PDF you send to a customer.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Embedding {
    pub code: u16,
    pub label: String,
    /// "ok" | "warn" | "bad" | "unknown" — drives the colour of the badge.
    pub level: String,
    pub notes: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub units_per_em: u16,
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub cap_height: Option<i16>,
    pub x_height: Option<i16>,
    pub bbox: [i16; 4],
    pub italic_angle: Option<f32>,
    pub underline_position: Option<i16>,
    pub underline_thickness: Option<i16>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AxisInfo {
    pub tag: String,
    pub label: String,
    pub min: f32,
    pub default: f32,
    pub max: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockCoverage {
    pub name: String,
    pub covered: u32,
    pub total: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontReport {
    pub file_name: String,
    pub file_size: u64,
    /// Container: "TrueType", "OpenType (CFF)", "WOFF", "WOFF2", "TrueType Collection".
    pub format: String,
    /// Outline flavour: "glyf" (quadratic) or "CFF" (cubic).
    pub outlines: String,
    /// Fonts inside the file — more than 1 only for collections.
    pub font_count: usize,
    pub index: usize,
    pub names: Vec<NameEntry>,
    pub num_glyphs: u16,
    pub mapped_chars: u32,
    pub metrics: Metrics,
    pub weight_class: Option<u16>,
    pub weight_label: Option<String>,
    pub width_label: Option<String>,
    pub vendor: Option<String>,
    pub panose: Option<String>,
    pub embedding: Embedding,
    pub axes: Vec<AxisInfo>,
    pub tables: Vec<TableEntry>,
    pub features: Vec<String>,
    pub scripts: Vec<String>,
    pub flags: Vec<String>,
    pub coverage: Vec<BlockCoverage>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Glyph {
    pub gid: u16,
    pub code: u32,
    pub ch: String,
    pub name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlyphsReport {
    pub num_glyphs: u16,
    /// Glyphs reachable from the character map, in codepoint order.
    pub glyphs: Vec<Glyph>,
    /// Glyphs with no codepoint pointing at them (alternates, ligatures, …).
    pub unmapped: u16,
    /// The whole file as a `data:` URI so the webview can render it directly
    /// through `@font-face` instead of us rasterising glyphs ourselves.
    pub data_uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsetReport {
    pub before: u64,
    pub after: u64,
    pub glyphs_before: u16,
    pub glyphs_after: u16,
    /// Requested characters the font has no glyph for.
    pub missing: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfFontInfo {
    /// Name as written in the PDF, subset prefix included.
    pub name: String,
    /// Name with the `ABCDEF+` subset prefix stripped.
    pub base_name: String,
    pub family: String,
    pub embedded: bool,
    pub subset: bool,
    /// One of the 14 fonts every PDF reader must provide.
    pub standard14: bool,
    pub weight: Option<u16>,
    pub italic: bool,
    /// "serif" | "sans" | "monospaziato" | "corsivo" | "simboli" | "—"
    pub style: String,
    pub embedded_bytes: Option<u64>,
    pub pages: Vec<u32>,
    pub glyphs: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfFontsReport {
    pub pages: u32,
    /// Pages actually inspected — capped for very long documents.
    pub scanned: u32,
    pub fonts: Vec<PdfFontInfo>,
}

// ---------------------------------------------------------------------------
// Reading a font file
// ---------------------------------------------------------------------------

pub(crate) fn read_file(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("non riesco a leggere il file: {e}"))
}

pub(crate) fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

pub(crate) fn parse<'a>(data: &'a [u8]) -> Result<FontData<'a>, String> {
    ReadScope::new(data)
        .read::<FontData<'a>>()
        .map_err(|_| "formato non riconosciuto: attesi TTF, OTF, TTC, WOFF o WOFF2".to_string())
}

/// How many fonts the file holds (>1 only for a TrueType collection).
pub(crate) fn font_count(data: &FontData<'_>) -> usize {
    match data {
        FontData::OpenType(otf) => match &otf.data {
            allsorts::tables::OpenTypeData::Single(_) => 1,
            allsorts::tables::OpenTypeData::Collection(ttc) => ttc.offset_tables.len(),
        },
        _ => 1,
    }
}

/// What to call the file. WOFF and WOFF2 are named after their wrapper; a bare
/// sfnt is named after its outline flavour, because "OTF" and "TTF" are what
/// people actually call those files.
fn container_label(data: &FontData<'_>, count: usize, cff: bool) -> String {
    match data {
        FontData::Woff(_) => "WOFF".to_string(),
        FontData::Woff2(_) => "WOFF2".to_string(),
        FontData::OpenType(_) if count > 1 => "TrueType Collection (TTC)".to_string(),
        FontData::OpenType(_) if cff => "OpenType (OTF)".to_string(),
        FontData::OpenType(_) => "TrueType (TTF)".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Small decoders for the numbers OpenType stores as codes
// ---------------------------------------------------------------------------

fn weight_label(class: u16) -> String {
    let name = match class {
        0..=149 => "Thin",
        150..=249 => "Extra Light",
        250..=349 => "Light",
        350..=449 => "Regular",
        450..=549 => "Medium",
        550..=649 => "Semi Bold",
        650..=749 => "Bold",
        750..=849 => "Extra Bold",
        _ => "Black",
    };
    format!("{class} ({name})")
}

fn width_label(class: u16) -> String {
    let name = match class {
        1 => "Ultra Condensed",
        2 => "Extra Condensed",
        3 => "Condensed",
        4 => "Semi Condensed",
        5 => "Normale",
        6 => "Semi Expanded",
        7 => "Expanded",
        8 => "Extra Expanded",
        9 => "Ultra Expanded",
        _ => "sconosciuta",
    };
    format!("{class} ({name})")
}

/// Decode `fsType`. Bits 0-3 are mutually exclusive in practice but fonts in the
/// wild set several, so the strictest one wins — that is the safe reading.
fn embedding(fs_type: Option<u16>) -> Embedding {
    let Some(code) = fs_type else {
        return Embedding {
            code: 0,
            label: "sconosciuto (nessuna tabella OS/2)".to_string(),
            level: "unknown".to_string(),
            notes: vec![
                "Il font non dichiara i permessi di embedding: verifica la licenza a mano."
                    .to_string(),
            ],
        };
    };

    let mut notes = Vec::new();
    let (label, level) = if code & 0x0002 != 0 {
        (
            "Restricted License — embedding vietato",
            "bad",
        )
    } else if code & 0x0004 != 0 {
        ("Preview & Print — incorporabile solo in sola lettura", "warn")
    } else if code & 0x0008 != 0 {
        ("Editable — incorporabile e modificabile", "ok")
    } else {
        ("Installable — nessuna restrizione", "ok")
    };

    if code & 0x0002 != 0 {
        notes.push(
            "Il font non può essere incorporato in un documento senza il permesso del foundry."
                .to_string(),
        );
    }
    if code & 0x0004 != 0 {
        notes.push(
            "Chi riceve il documento può vederlo e stamparlo, ma non modificarne il testo."
                .to_string(),
        );
    }
    if code & 0x0100 != 0 {
        notes.push("Bit 8: il font non può essere ridotto a subset.".to_string());
    }
    if code & 0x0200 != 0 {
        notes.push("Bit 9: è consentito incorporare solo le bitmap, non le outline.".to_string());
    }
    if notes.is_empty() {
        notes.push("Nessuna restrizione dichiarata dal font.".to_string());
    }

    Embedding {
        code,
        label: label.to_string(),
        level: level.to_string(),
        notes,
    }
}

fn tag_string(t: u32) -> String {
    DisplayTag(t).to_string()
}

/// The Unicode blocks worth reporting on. Not the full list — that would be
/// noise — but enough to answer "does this font cover the languages I need?".
const BLOCKS: &[(&str, u32, u32)] = &[
    ("Latino base", 0x0020, 0x007E),
    ("Latino-1 supplemento", 0x00A0, 0x00FF),
    ("Latino esteso-A", 0x0100, 0x017F),
    ("Latino esteso-B", 0x0180, 0x024F),
    ("IPA", 0x0250, 0x02AF),
    ("Greco", 0x0370, 0x03FF),
    ("Cirillico", 0x0400, 0x04FF),
    ("Ebraico", 0x0590, 0x05FF),
    ("Arabo", 0x0600, 0x06FF),
    ("Devanagari", 0x0900, 0x097F),
    ("Thai", 0x0E00, 0x0E7F),
    ("Punteggiatura generale", 0x2000, 0x206F),
    ("Simboli valuta", 0x20A0, 0x20BF),
    ("Simboli tipo lettera", 0x2100, 0x214F),
    ("Operatori matematici", 0x2200, 0x22FF),
    ("Disegno di riquadri", 0x2500, 0x257F),
    ("Blocchi geometrici", 0x25A0, 0x25FF),
    ("Simboli vari", 0x2600, 0x26FF),
    ("Frecce", 0x2190, 0x21FF),
    ("Hiragana", 0x3040, 0x309F),
    ("Katakana", 0x30A0, 0x30FF),
    ("CJK unificati", 0x4E00, 0x9FFF),
    ("Hangul", 0xAC00, 0xD7A3),
    ("Emoji", 0x1F300, 0x1F5FF),
];

// ---------------------------------------------------------------------------
// GSUB / GPOS: script and feature tags
// ---------------------------------------------------------------------------

/// Read the script and feature tags out of a GSUB/GPOS table.
///
/// The header is a fixed 10 bytes (version, then three offsets), and both lists
/// are a count followed by `(tag, offset)` records — so pulling the tags out
/// directly is smaller and less brittle than walking the parsed layout tree,
/// and a malformed table just yields nothing instead of failing the whole
/// report.
fn layout_tags(table: &[u8]) -> (Vec<String>, Vec<String>) {
    fn u16at(b: &[u8], off: usize) -> Option<u16> {
        Some(u16::from_be_bytes([*b.get(off)?, *b.get(off + 1)?]))
    }
    fn tags_at(b: &[u8], list_off: usize) -> Vec<String> {
        let Some(count) = u16at(b, list_off) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for i in 0..usize::from(count) {
            let rec = list_off + 2 + i * 6;
            let Some(bytes) = b.get(rec..rec + 4) else {
                break;
            };
            let t = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            out.push(tag_string(t));
        }
        out
    }

    if table.len() < 10 {
        return (Vec::new(), Vec::new());
    }
    let script_off = u16at(table, 4).map(usize::from).unwrap_or(0);
    let feature_off = u16at(table, 6).map(usize::from).unwrap_or(0);
    let scripts = if script_off > 0 { tags_at(table, script_off) } else { Vec::new() };
    let features = if feature_off > 0 { tags_at(table, feature_off) } else { Vec::new() };
    (scripts, features)
}

// ---------------------------------------------------------------------------
// font_inspect
// ---------------------------------------------------------------------------

/// Read everything worth knowing about a font file.
#[tauri::command]
pub async fn font_inspect(path: String, index: usize) -> Result<FontReport, String> {
    tokio::task::spawn_blocking(move || inspect_blocking(&path, index))
        .await
        .map_err(|e| e.to_string())?
}

pub(crate) fn inspect_blocking(path: &str, index: usize) -> Result<FontReport, String> {
    let bytes = read_file(path)?;
    let data = parse(&bytes)?;
    let count = font_count(&data);
    let index = index.min(count.saturating_sub(1));
    let provider = data
        .table_provider(index)
        .map_err(|e| format!("font non leggibile: {e}"))?;

    // --- names -------------------------------------------------------------
    // Name IDs from the OpenType `name` table, in the order a human reads them.
    const NAME_IDS: &[(u16, &str)] = &[
        (16, "Famiglia tipografica"),
        (1, "Famiglia"),
        (17, "Stile tipografico"),
        (2, "Stile"),
        (4, "Nome completo"),
        (6, "Nome PostScript"),
        (5, "Versione"),
        (8, "Produttore"),
        (9, "Designer"),
        (0, "Copyright"),
        (7, "Marchio"),
        (13, "Licenza"),
        (14, "URL licenza"),
        (11, "URL produttore"),
        (12, "URL designer"),
        (3, "Identificativo univoco"),
    ];
    let mut names = Vec::new();
    if let Ok(Some(name_data)) = provider.table_data(tag::NAME) {
        for (id, label) in NAME_IDS {
            if let Ok(Some(cstr)) = allsorts::get_name::fontcode_get_name(&name_data, *id) {
                let value = cstr.to_string_lossy().trim().to_string();
                if !value.is_empty() && !names.iter().any(|n: &NameEntry| n.value == value) {
                    names.push(NameEntry {
                        label: label.to_string(),
                        value,
                    });
                }
            }
        }
    }

    // --- core tables -------------------------------------------------------
    let head = provider
        .table_data(tag::HEAD)
        .ok()
        .flatten()
        .and_then(|d| ReadScope::new(&d).read::<HeadTable>().ok());
    let hhea = provider
        .table_data(tag::HHEA)
        .ok()
        .flatten()
        .and_then(|d| ReadScope::new(&d).read::<HheaTable>().ok());
    let maxp = provider
        .table_data(tag::MAXP)
        .ok()
        .flatten()
        .and_then(|d| ReadScope::new(&d).read::<MaxpTable>().ok());
    let os2 = provider
        .table_data(tag::OS_2)
        .ok()
        .flatten()
        .and_then(|d| ReadScope::new(&d).read_dep::<Os2>(d.len()).ok());

    let units_per_em = head.as_ref().map(|h| h.units_per_em).unwrap_or(1000);
    let num_glyphs = maxp.as_ref().map(|m| m.num_glyphs).unwrap_or(0);

    let post = provider
        .table_data(tag::POST)
        .ok()
        .flatten()
        .and_then(|d| {
            ReadScope::new(&d).read::<PostTable<'_>>().ok().map(|p| {
                (
                    p.header.italic_angle as f32 / 65536.0,
                    p.header.underline_position,
                    p.header.underline_thickness,
                )
            })
        });

    let metrics = Metrics {
        units_per_em,
        ascender: hhea.as_ref().map(|h| h.ascender).unwrap_or(0),
        descender: hhea.as_ref().map(|h| h.descender).unwrap_or(0),
        line_gap: hhea.as_ref().map(|h| h.line_gap).unwrap_or(0),
        cap_height: os2
            .as_ref()
            .and_then(|o| o.version2to4.as_ref())
            .map(|v| v.s_cap_height),
        x_height: os2
            .as_ref()
            .and_then(|o| o.version2to4.as_ref())
            .map(|v| v.s_x_height),
        bbox: head
            .as_ref()
            .map(|h| [h.x_min, h.y_min, h.x_max, h.y_max])
            .unwrap_or([0; 4]),
        italic_angle: post.map(|p| p.0),
        underline_position: post.map(|p| p.1),
        underline_thickness: post.map(|p| p.2),
    };

    // --- tables present ----------------------------------------------------
    // WOFF/WOFF2 providers report the tags but not the uncompressed sizes, so
    // the size stays `None` there rather than showing a misleading number.
    let mut tables: Vec<TableEntry> = provider
        .table_tags()
        .unwrap_or_default()
        .into_iter()
        .map(|t| TableEntry {
            tag: tag_string(t),
            size: provider
                .table_data(t)
                .ok()
                .flatten()
                .map(|d| d.len() as u32),
        })
        .collect();
    tables.sort_by(|a, b| a.tag.cmp(&b.tag));
    let has = |t: u32| provider.has_table(t);

    // --- character coverage ------------------------------------------------
    let mut mapped: Vec<u32> = Vec::new();
    if let Ok(Some(cmap_data)) = provider.table_data(tag::CMAP) {
        if let Ok(cmap) = ReadScope::new(&cmap_data).read::<Cmap<'_>>() {
            if let Ok(Some((_, sub))) = read_cmap_subtable(&cmap) {
                let _ = sub.mappings_fn(|ch, gid| {
                    if gid != 0 {
                        mapped.push(ch);
                    }
                });
            }
        }
    }
    mapped.sort_unstable();
    mapped.dedup();
    let coverage = BLOCKS
        .iter()
        .filter_map(|(name, start, end)| {
            let lo = mapped.partition_point(|c| c < start);
            let hi = mapped.partition_point(|c| c <= end);
            let covered = (hi - lo) as u32;
            if covered == 0 {
                return None;
            }
            Some(BlockCoverage {
                name: name.to_string(),
                covered,
                total: end - start + 1,
            })
        })
        .collect();

    // --- layout features ---------------------------------------------------
    let mut features: Vec<String> = Vec::new();
    let mut scripts: Vec<String> = Vec::new();
    for t in [tag::GSUB, tag::GPOS] {
        if let Ok(Some(d)) = provider.table_data(t) {
            let (s, f) = layout_tags(&d);
            scripts.extend(s);
            features.extend(f);
        }
    }
    features.sort();
    features.dedup();
    scripts.sort();
    scripts.dedup();

    // --- variable axes -----------------------------------------------------
    let mut axes = Vec::new();
    if let Ok(Some(fvar_data)) = provider.table_data(tag::FVAR) {
        if let Ok(fvar) = ReadScope::new(&fvar_data).read::<FvarTable<'_>>() {
            let name_data = provider.table_data(tag::NAME).ok().flatten();
            for axis in fvar.axes() {
                let label = name_data
                    .as_ref()
                    .and_then(|n| {
                        allsorts::get_name::fontcode_get_name(n, axis.axis_name_id).ok().flatten()
                    })
                    .map(|c| c.to_string_lossy().into_owned())
                    .unwrap_or_else(|| tag_string(axis.axis_tag));
                axes.push(AxisInfo {
                    tag: tag_string(axis.axis_tag),
                    label,
                    min: f32::from(axis.min_value),
                    default: f32::from(axis.default_value),
                    max: f32::from(axis.max_value),
                });
            }
        }
    }

    // --- capability flags --------------------------------------------------
    let mut flags = Vec::new();
    if !axes.is_empty() {
        flags.push("variabile".to_string());
    }
    if has(tag::COLR) || has(tag::SVG) || has(tag::CBDT) {
        flags.push("a colori".to_string());
    }
    if has(tag::EBDT) || has(tag::SBIX) {
        flags.push("bitmap incorporate".to_string());
    }
    if has(tag::KERN) {
        flags.push("kerning legacy".to_string());
    }
    if has(tag::GSUB) {
        flags.push("sostituzioni (GSUB)".to_string());
    }
    if has(tag::GPOS) {
        flags.push("posizionamento (GPOS)".to_string());
    }
    if let Some(h) = head.as_ref() {
        if h.mac_style.contains(allsorts::tables::MacStyle::ITALIC) {
            flags.push("corsivo".to_string());
        }
        if h.mac_style.contains(allsorts::tables::MacStyle::BOLD) {
            flags.push("grassetto".to_string());
        }
    }

    let cff = has(tag::CFF) || has(tag::CFF2);
    let outlines = if cff {
        "CFF (curve cubiche)"
    } else if has(tag::GLYF) {
        "glyf (curve quadratiche)"
    } else {
        "sconosciute"
    };

    let panose = os2.as_ref().map(|o| {
        o.panose
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    });

    Ok(FontReport {
        file_name: file_name(path),
        file_size: bytes.len() as u64,
        format: container_label(&data, count, cff),
        outlines: outlines.to_string(),
        font_count: count,
        index,
        names,
        num_glyphs,
        mapped_chars: mapped.len() as u32,
        metrics,
        weight_class: os2.as_ref().map(|o| o.us_weight_class),
        weight_label: os2.as_ref().map(|o| weight_label(o.us_weight_class)),
        width_label: os2.as_ref().map(|o| width_label(o.us_width_class)),
        vendor: os2.as_ref().map(|o| tag_string(o.ach_vend_id)),
        panose,
        embedding: embedding(os2.as_ref().map(|o| o.fs_type)),
        axes,
        tables,
        features,
        scripts,
        flags,
        coverage,
    })
}

// ---------------------------------------------------------------------------
// font_glyphs
// ---------------------------------------------------------------------------

/// MIME type for a `data:` URI, picked from the container so the webview knows
/// how to decode it in `@font-face`.
fn mime_for(data: &FontData<'_>) -> &'static str {
    match data {
        FontData::Woff(_) => "font/woff",
        FontData::Woff2(_) => "font/woff2",
        FontData::OpenType(_) => "font/ttf",
    }
}

/// List every glyph the character map can reach, plus the font itself so the
/// frontend can render the glyphs with the real typeface.
#[tauri::command]
pub async fn font_glyphs(path: String, index: usize) -> Result<GlyphsReport, String> {
    tokio::task::spawn_blocking(move || glyphs_blocking(&path, index))
        .await
        .map_err(|e| e.to_string())?
}

fn glyphs_blocking(path: &str, index: usize) -> Result<GlyphsReport, String> {
    let bytes = read_file(path)?;
    let data = parse(&bytes)?;
    let count = font_count(&data);
    let index = index.min(count.saturating_sub(1));
    let provider = data
        .table_provider(index)
        .map_err(|e| format!("font non leggibile: {e}"))?;

    let num_glyphs = provider
        .table_data(tag::MAXP)
        .ok()
        .flatten()
        .and_then(|d| ReadScope::new(&d).read::<MaxpTable>().ok())
        .map(|m| m.num_glyphs)
        .unwrap_or(0);

    // One codepoint per glyph: when several map to the same glyph the lowest
    // wins, which is the one a reader recognises.
    let mut by_gid: BTreeMap<u32, u16> = BTreeMap::new();
    if let Ok(Some(cmap_data)) = provider.table_data(tag::CMAP) {
        if let Ok(cmap) = ReadScope::new(&cmap_data).read::<Cmap<'_>>() {
            if let Ok(Some((_, sub))) = read_cmap_subtable(&cmap) {
                let _ = sub.mappings_fn(|ch, gid| {
                    if gid != 0 {
                        by_gid.entry(ch).or_insert(gid);
                    }
                });
            }
        }
    }

    let post_data = provider.table_data(tag::POST).ok().flatten();
    let post = post_data
        .as_ref()
        .and_then(|d| ReadScope::new(d).read::<PostTable<'_>>().ok());

    let mut seen: Vec<u16> = Vec::new();
    let glyphs: Vec<Glyph> = by_gid
        .into_iter()
        .map(|(code, gid)| {
            seen.push(gid);
            Glyph {
                gid,
                code,
                ch: char::from_u32(code).map(String::from).unwrap_or_default(),
                name: post
                    .as_ref()
                    .and_then(|p| p.glyph_name(gid).ok().flatten())
                    .map(|s| s.to_string()),
            }
        })
        .collect();

    seen.sort_unstable();
    seen.dedup();
    let unmapped = num_glyphs.saturating_sub(seen.len() as u16);

    Ok(GlyphsReport {
        num_glyphs,
        glyphs,
        unmapped,
        data_uri: format!("data:{};base64,{}", mime_for(&data), B64.encode(&bytes)),
    })
}

// ---------------------------------------------------------------------------
// font_subset
// ---------------------------------------------------------------------------

/// Cut a font down to the characters in `text` and write it to `dest`.
///
/// `profile` is "web" (Unicode cmap, minimal tables — what a browser needs) or
/// "pdf" (the table set a PDF embedder wants).
#[tauri::command]
pub async fn font_subset(
    path: String,
    index: usize,
    text: String,
    dest: String,
    profile: String,
) -> Result<SubsetReport, String> {
    tokio::task::spawn_blocking(move || subset_blocking(&path, index, &text, &dest, &profile))
        .await
        .map_err(|e| e.to_string())?
}

fn subset_blocking(
    path: &str,
    index: usize,
    text: &str,
    dest: &str,
    profile: &str,
) -> Result<SubsetReport, String> {
    let bytes = read_file(path)?;
    let data = parse(&bytes)?;
    let count = font_count(&data);
    let index = index.min(count.saturating_sub(1));
    let provider = data
        .table_provider(index)
        .map_err(|e| format!("font non leggibile: {e}"))?;

    let glyphs_before = provider
        .table_data(tag::MAXP)
        .ok()
        .flatten()
        .and_then(|d| ReadScope::new(&d).read::<MaxpTable>().ok())
        .map(|m| m.num_glyphs)
        .unwrap_or(0);

    // Refuse when the foundry said no — the whole point of reading fsType is to
    // act on it.
    let fs_type = provider
        .table_data(tag::OS_2)
        .ok()
        .flatten()
        .and_then(|d| ReadScope::new(&d).read_dep::<Os2>(d.len()).ok())
        .map(|o| o.fs_type);
    if let Some(fs) = fs_type {
        if fs & 0x0100 != 0 {
            return Err(
                "Il font vieta il subsetting (OS/2 fsType, bit 8): operazione annullata."
                    .to_string(),
            );
        }
    }

    // Map the requested characters to glyph ids.
    let cmap_data = provider
        .table_data(tag::CMAP)
        .map_err(|e| e.to_string())?
        .ok_or("il font non ha una tabella cmap: impossibile fare il subset")?;
    let cmap = ReadScope::new(&cmap_data)
        .read::<Cmap<'_>>()
        .map_err(|e| e.to_string())?;
    let (_, sub) = read_cmap_subtable(&cmap)
        .map_err(|e| e.to_string())?
        .ok_or("il font non ha una sottotabella cmap utilizzabile")?;

    // Glyph 0 (.notdef) is mandatory, and `subset` rejects duplicates, so the
    // set is built ordered and unique.
    let mut ids: Vec<u16> = vec![0];
    let mut missing = String::new();
    let mut chars: Vec<char> = text.chars().collect();
    chars.sort_unstable();
    chars.dedup();
    for ch in chars {
        match sub.map_glyph(ch as u32) {
            Ok(Some(gid)) if gid != 0 => {
                if !ids.contains(&gid) {
                    ids.push(gid);
                }
            }
            _ => missing.push(ch),
        }
    }
    if ids.len() == 1 {
        return Err("nessuno dei caratteri richiesti esiste in questo font".to_string());
    }
    ids.sort_unstable();

    let (prof, target) = match profile {
        "pdf" => (SubsetProfile::Pdf, CmapTarget::Unrestricted),
        _ => (SubsetProfile::Minimal, CmapTarget::Unicode),
    };
    let out = subset(&provider, &ids, &prof, target)
        .map_err(|e| format!("subset non riuscito: {e}"))?;

    // Count the glyphs the subsetter actually emitted rather than the ones we
    // asked for: a composite glyph (an accented letter, say) drags its
    // components in with it, so the two numbers legitimately differ.
    let glyphs_after = (|| {
        let out_data = parse(&out).ok()?;
        let out_provider = out_data.table_provider(0).ok()?;
        let maxp = out_provider.table_data(tag::MAXP).ok()??;
        Some(ReadScope::new(&maxp).read::<MaxpTable>().ok()?.num_glyphs)
    })()
    .unwrap_or(ids.len() as u16);

    let dest_path = PathBuf::from(dest);
    std::fs::write(&dest_path, &out).map_err(|e| format!("non riesco a scrivere il file: {e}"))?;

    Ok(SubsetReport {
        before: bytes.len() as u64,
        after: out.len() as u64,
        glyphs_before,
        glyphs_after,
        missing,
    })
}

// ---------------------------------------------------------------------------
// PDF fonts
// ---------------------------------------------------------------------------

/// Pages scanned at most. Walking every text object of a 2000-page file to
/// collect font names costs more than the answer is worth; the font set of a
/// document is settled long before then.
const MAX_PAGES: usize = 200;

/// Fonts named `ABCDEF+Helvetica` are subsets: six uppercase letters and a plus.
fn split_subset_prefix(name: &str) -> (bool, String) {
    let bytes = name.as_bytes();
    if bytes.len() > 7
        && bytes[6] == b'+'
        && bytes[..6].iter().all(|b| b.is_ascii_uppercase())
    {
        (true, name[7..].to_string())
    } else {
        (false, name.to_string())
    }
}

const STANDARD_14: &[&str] = &[
    "Courier",
    "Courier-Bold",
    "Courier-BoldOblique",
    "Courier-Oblique",
    "Helvetica",
    "Helvetica-Bold",
    "Helvetica-BoldOblique",
    "Helvetica-Oblique",
    "Times-Roman",
    "Times-Bold",
    "Times-BoldItalic",
    "Times-Italic",
    "Symbol",
    "ZapfDingbats",
];

/// List the fonts a PDF uses, and say which of them travel inside the file.
///
/// A font that is neither embedded nor one of the standard 14 is the classic
/// cause of a document that looks right here and wrong on the recipient's
/// machine, so that is the case the report is built to make obvious.
#[tauri::command]
pub async fn pdf_fonts(path: String) -> Result<PdfFontsReport, String> {
    tokio::task::spawn_blocking(move || pdf_fonts_blocking(&path))
        .await
        .map_err(|e| e.to_string())?
}

/// What we accumulate per distinct font while walking the pages.
struct FontAcc {
    info: PdfFontInfo,
    page_set: Vec<u32>,
}

fn pdf_fonts_blocking(path: &str) -> Result<PdfFontsReport, String> {
    use pdfium_render::prelude::*;

    let pdfium = pdfa_core::pdfium::istanza().map_err(|e| e.to_string())?;
    let doc = pdfium
        .load_pdf_from_file(Path::new(path), None)
        .map_err(|e| e.to_string())?;

    let total = doc.pages().len() as u32;
    let mut found: BTreeMap<String, FontAcc> = BTreeMap::new();
    let mut scanned = 0u32;

    for (n, page) in doc.pages().iter().enumerate().take(MAX_PAGES) {
        scanned += 1;
        let page_no = n as u32 + 1;
        let objects = page.objects();
        for object in objects.iter() {
            let Some(text) = object.as_text_object() else {
                continue;
            };
            let font = text.font();
            let name = font.name();
            if name.is_empty() {
                continue;
            }
            let glyphs = text.text().chars().count() as u64;

            let entry = found.entry(name.clone()).or_insert_with(|| {
                let family = font.family();
                // The prefix can sit on either name: producers are inconsistent
                // about which one they decorate.
                let (name_subset, base) = split_subset_prefix(&name);
                let (family_subset, _) = split_subset_prefix(&family);
                let is_subset = name_subset || family_subset;
                let embedded = font.is_embedded().unwrap_or(false);
                // Symbolic comes last on purpose: PDF producers set that flag on
                // any font with a custom encoding, which is most subsetted text
                // fonts, so testing it first would label half a document
                // "simboli".
                let style = if font.is_fixed_pitch() {
                    "monospaziato"
                } else if font.is_serif() {
                    "serif"
                } else if font.is_sans_serif() {
                    "sans"
                } else if font.is_cursive() {
                    "corsivo"
                } else if font.is_symbolic() {
                    "simboli"
                } else {
                    "—"
                };
                FontAcc {
                    info: PdfFontInfo {
                        name: name.clone(),
                        standard14: STANDARD_14.contains(&base.as_str()),
                        base_name: base,
                        family,
                        embedded,
                        subset: is_subset,
                        // Pdfium reports 0 when the descriptor carries no
                        // StemV/weight at all; that is "unknown", not "weight 0".
                        weight: font
                            .weight()
                            .ok()
                            .map(|w| match w {
                                PdfFontWeight::Weight100 => 100,
                                PdfFontWeight::Weight200 => 200,
                                PdfFontWeight::Weight300 => 300,
                                PdfFontWeight::Weight400Normal => 400,
                                PdfFontWeight::Weight500 => 500,
                                PdfFontWeight::Weight600 => 600,
                                PdfFontWeight::Weight700Bold => 700,
                                PdfFontWeight::Weight800 => 800,
                                PdfFontWeight::Weight900 => 900,
                                PdfFontWeight::Custom(v) => v as u16,
                            })
                            .filter(|w| *w > 0),
                        italic: font.is_italic(),
                        style: style.to_string(),
                        embedded_bytes: if embedded {
                            font.data().ok().map(|d| d.len() as u64)
                        } else {
                            None
                        },
                        pages: Vec::new(),
                        glyphs: 0,
                    },
                    page_set: Vec::new(),
                }
            });
            entry.info.glyphs += glyphs;
            if entry.page_set.last() != Some(&page_no) {
                entry.page_set.push(page_no);
            }
        }
    }

    let mut fonts: Vec<PdfFontInfo> = found
        .into_values()
        .map(|mut acc| {
            acc.info.pages = acc.page_set;
            acc.info
        })
        .collect();
    // Problems first: not embedded and not a standard font is the row the user
    // opened this tool to find.
    fonts.sort_by_key(|f| {
        let risk = u8::from(f.embedded || f.standard14);
        (risk, f.name.clone())
    });

    Ok(PdfFontsReport {
        pages: total,
        scanned,
        fonts,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// These read whatever fonts the machine happens to have, since the point is to
/// exercise the four containers against real files rather than a fixture that
/// only proves the parser agrees with itself. A missing candidate is skipped,
/// not failed — the same convention the PDF tests use.
#[cfg(test)]
mod tests {
    use super::*;

    /// First existing path out of the candidates.
    fn find(candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .find(|p| Path::new(p).is_file())
            .map(|p| p.to_string())
    }

    const TTF: &[&str] = &[
        "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    const OTF: &[&str] = &[
        "/usr/share/fonts/cantarell/Cantarell-VF.otf",
        "/usr/share/fonts/OTF/PowerlineSymbols.otf",
    ];
    const TTC: &[&str] = &[
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc",
    ];

    #[test]
    fn inspects_a_truetype_font() {
        let Some(path) = find(TTF) else {
            eprintln!("nessun TTF di sistema: test saltato");
            return;
        };
        let r = inspect_blocking(&path, 0).expect("ispezione");
        assert!(r.num_glyphs > 0, "nessun glifo");
        assert!(r.mapped_chars > 0, "cmap vuota");
        assert!(!r.names.is_empty(), "nessun nome");
        assert_eq!(r.metrics.units_per_em, 1000.max(r.metrics.units_per_em));
        assert!(r.tables.iter().any(|t| t.tag == "cmap"), "cmap assente");
        // Basic Latin has to be covered by any text font.
        assert!(
            r.coverage.iter().any(|c| c.name == "Latino base"),
            "latino base non coperto"
        );
    }

    #[test]
    fn inspects_a_cff_font() {
        let Some(path) = find(OTF) else {
            eprintln!("nessun OTF di sistema: test saltato");
            return;
        };
        let r = inspect_blocking(&path, 0).expect("ispezione");
        assert!(r.outlines.starts_with("CFF"), "outline: {}", r.outlines);
        assert!(r.num_glyphs > 0);
    }

    #[test]
    fn reads_every_font_of_a_collection() {
        let Some(path) = find(TTC) else {
            eprintln!("nessun TTC di sistema: test saltato");
            return;
        };
        let first = inspect_blocking(&path, 0).expect("ispezione");
        assert!(first.font_count > 1, "collezione con un solo font?");
        assert_eq!(first.format, "TrueType Collection (TTC)");
        // An out-of-range index clamps instead of failing.
        let last = inspect_blocking(&path, 999).expect("ispezione ultimo");
        assert_eq!(last.index, first.font_count - 1);
    }

    #[test]
    fn glyph_list_matches_the_character_map() {
        let Some(path) = find(TTF) else {
            eprintln!("nessun TTF di sistema: test saltato");
            return;
        };
        let g = glyphs_blocking(&path, 0).expect("glifi");
        assert!(!g.glyphs.is_empty());
        assert!(g.data_uri.starts_with("data:font/"), "{}", &g.data_uri[..24]);
        // Codepoints come out ordered, and 'A' is in there.
        assert!(g.glyphs.windows(2).all(|w| w[0].code < w[1].code), "non ordinati");
        assert!(g.glyphs.iter().any(|x| x.ch == "A"), "manca la A");
    }

    #[test]
    fn subset_is_smaller_and_still_parses() {
        let Some(path) = find(TTF) else {
            eprintln!("nessun TTF di sistema: test saltato");
            return;
        };
        let dest = std::env::temp_dir().join("devtoys-subset-test.ttf");
        let report = subset_blocking(&path, 0, "Ciao mondo", dest.to_str().unwrap(), "web")
            .expect("subset");

        assert!(report.after < report.before, "il subset non ha ridotto nulla");
        assert!(report.glyphs_after < report.glyphs_before);
        assert!(report.missing.is_empty(), "caratteri mancanti: {}", report.missing);

        // The output has to be a font we can read back.
        let back = inspect_blocking(dest.to_str().unwrap(), 0).expect("rilettura del subset");
        assert_eq!(u64::from(back.num_glyphs), u64::from(report.glyphs_after));
        let _ = std::fs::remove_file(&dest);
    }

    #[test]
    fn rejects_a_file_that_is_not_a_font() {
        let dest = std::env::temp_dir().join("devtoys-not-a-font.ttf");
        std::fs::write(&dest, b"questo non e' un font").expect("scrittura");
        let err = inspect_blocking(dest.to_str().unwrap(), 0)
            .err()
            .expect("un file qualsiasi non può essere un font");
        assert!(err.contains("formato non riconosciuto"), "messaggio: {err}");
        let _ = std::fs::remove_file(&dest);
    }

    #[test]
    fn decodes_the_embedding_permissions() {
        assert_eq!(embedding(Some(0x0000)).level, "ok");
        assert_eq!(embedding(Some(0x0008)).level, "ok");
        assert_eq!(embedding(Some(0x0004)).level, "warn");
        assert_eq!(embedding(Some(0x0002)).level, "bad");
        // The strictest bit wins when a font sets several.
        assert_eq!(embedding(Some(0x0002 | 0x0008)).level, "bad");
        assert_eq!(embedding(None).level, "unknown");
        // The no-subsetting bit is reported on top of the base permission.
        let ns = embedding(Some(0x0100));
        assert!(ns.notes.iter().any(|n| n.contains("subset")));
    }

    /// Needs a real document, so it takes one from the environment:
    ///
    /// ```text
    /// DEVTOYS_PDF=/path/doc.pdf cargo test -p devtoys lists_the_fonts -- --nocapture
    /// ```
    #[test]
    fn lists_the_fonts_of_a_pdf() {
        let Ok(path) = std::env::var("DEVTOYS_PDF") else {
            eprintln!("DEVTOYS_PDF non impostata: test saltato");
            return;
        };
        pdfa_core::pdfium::inizializza(&[]).expect("pdfium non disponibile");
        let r = pdf_fonts_blocking(&path).expect("analisi font");
        eprintln!("{} pagine, {} font", r.pages, r.fonts.len());
        for f in &r.fonts {
            eprintln!(
                "  {} — incorporato={} subset={} std14={} stile={}",
                f.name, f.embedded, f.subset, f.standard14, f.style
            );
        }
        assert!(!r.fonts.is_empty(), "nessun font trovato");
        // Fonts that need attention sort to the top.
        let risky = |f: &PdfFontInfo| !f.embedded && !f.standard14;
        let first_safe = r.fonts.iter().position(|f| !risky(f)).unwrap_or(r.fonts.len());
        assert!(
            r.fonts[first_safe..].iter().all(|f| !risky(f)),
            "font a rischio non raggruppati in cima"
        );
    }

    #[test]
    fn recognises_a_subset_prefix() {
        assert_eq!(
            split_subset_prefix("ABCDEF+Helvetica"),
            (true, "Helvetica".to_string())
        );
        assert_eq!(
            split_subset_prefix("Helvetica"),
            (false, "Helvetica".to_string())
        );
        // Lowercase or the wrong length is not a subset tag.
        assert_eq!(
            split_subset_prefix("abcdef+Helvetica"),
            (false, "abcdef+Helvetica".to_string())
        );
        assert_eq!(
            split_subset_prefix("ABCDE+Helvetica"),
            (false, "ABCDE+Helvetica".to_string())
        );
    }
}

/// Save the embedded font program of `font_name` out of the PDF to `dest`.
#[tauri::command]
pub async fn pdf_font_extract(
    path: String,
    font_name: String,
    dest: String,
) -> Result<u64, String> {
    tokio::task::spawn_blocking(move || -> Result<u64, String> {
        use pdfium_render::prelude::*;

        let pdfium = pdfa_core::pdfium::istanza().map_err(|e| e.to_string())?;
        let doc = pdfium
            .load_pdf_from_file(Path::new(&path), None)
            .map_err(|e| e.to_string())?;

        for page in doc.pages().iter().take(MAX_PAGES) {
            let objects = page.objects();
            for object in objects.iter() {
                let Some(text) = object.as_text_object() else {
                    continue;
                };
                let font = text.font();
                if font.name() != font_name {
                    continue;
                }
                let data = font
                    .data()
                    .map_err(|e| format!("font non incorporato o illeggibile: {e}"))?;
                std::fs::write(&dest, &data)
                    .map_err(|e| format!("non riesco a scrivere il file: {e}"))?;
                return Ok(data.len() as u64);
            }
        }
        Err(format!("font «{font_name}» non trovato nel documento"))
    })
    .await
    .map_err(|e| e.to_string())?
}



