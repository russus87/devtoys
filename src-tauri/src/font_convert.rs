//! Font conversion — TrueType ⇄ OpenType/CFF outlines, and AFM metrics.
//!
//! The container is opened with `allsorts` (so TTC, WOFF and WOFF2 work as
//! input like in the other font tools), its tables are copied out into a plain
//! sfnt, and that is what `ttf-parser` reads outlines, names and kerning from.
//!
//! The converters rewrite only what the outline format dictates — `glyf`/`loca`
//! against `CFF `, plus `maxp`, `post`, `head` and `hmtx` — and copy every
//! other table byte for byte. Glyph order never changes, so `cmap`, GSUB, GPOS
//! and GDEF stay valid as they are.
//!
//! What is lost, deliberately: TrueType hinting going to CFF, and nothing is
//! re-hinted either way. Variable fonts are refused: flattening them to one
//! instance is a different tool.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::font::{file_name, font_count, parse, read_file};

const fn tag(b: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*b)
}

const T_CFF: u32 = tag(b"CFF ");
const T_CFF2: u32 = tag(b"CFF2");
const T_GLYF: u32 = tag(b"glyf");
const T_LOCA: u32 = tag(b"loca");
const T_HEAD: u32 = tag(b"head");
const T_HHEA: u32 = tag(b"hhea");
const T_HMTX: u32 = tag(b"hmtx");
const T_MAXP: u32 = tag(b"maxp");
const T_POST: u32 = tag(b"post");
const T_FVAR: u32 = tag(b"fvar");

/// TrueType-only tables: instructions and the caches built from them. They
/// mean nothing next to CFF outlines.
const TT_ONLY: &[&[u8; 4]] = &[
    b"glyf", b"loca", b"cvt ", b"fpgm", b"prep", b"hdmx", b"LTSH", b"VDMX",
];

/// CFF-only tables.
const CFF_ONLY: &[&[u8; 4]] = &[b"CFF ", b"CFF2", b"VORG"];

// ---------------------------------------------------------------------------
// Return shapes
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConvertReport {
    pub from: String,
    pub to: String,
    pub glyphs: u16,
    pub before: u64,
    pub after: u64,
    /// What changed besides the outlines, in words.
    pub notes: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AfmReport {
    pub font_name: String,
    pub glyphs: usize,
    pub encoded: usize,
    pub kern_pairs: usize,
    pub bytes: u64,
    /// The beginning of the file, for the preview pane.
    pub preview: String,
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

struct Source {
    size: u64,
    tables: BTreeMap<u32, Vec<u8>>,
    /// The chosen font as a bare sfnt, whatever the container was.
    sfnt: Vec<u8>,
    cff: bool,
    label: String,
}

fn load(path: &str, index: usize) -> Result<Source, String> {
    let bytes = read_file(path)?;
    let data = parse(&bytes)?;
    let count = font_count(&data);
    let index = index.min(count.saturating_sub(1));
    let provider = data
        .table_provider(index)
        .map_err(|e| format!("font non leggibile: {e}"))?;
    let tags = allsorts::tables::FontTableProvider::table_tags(&provider)
        .ok_or("non riesco a elencare le tabelle del font")?;

    let mut tables = BTreeMap::new();
    for t in tags {
        if let Some(d) = allsorts::tables::FontTableProvider::table_data(&provider, t)
            .map_err(|e| format!("tabella illeggibile: {e}"))?
        {
            tables.insert(t, d.into_owned());
        }
    }
    let cff = tables.contains_key(&T_CFF) || tables.contains_key(&T_CFF2);
    if !cff && !tables.contains_key(&T_GLYF) {
        return Err("il font non ha contorni (né glyf né CFF): niente da convertire".into());
    }
    let sfnt = write_sfnt(if cff { tag(b"OTTO") } else { 0x0001_0000 }, &tables);
    let label = match &data {
        allsorts::font_data::FontData::Woff(_) => "WOFF",
        allsorts::font_data::FontData::Woff2(_) => "WOFF2",
        _ if cff => "OTF",
        _ => "TTF",
    }
    .to_string();
    Ok(Source {
        size: bytes.len() as u64,
        tables,
        sfnt,
        cff,
        label,
    })
}

fn face(sfnt: &[u8]) -> Result<Face<'_>, String> {
    Face::parse(sfnt, 0).map_err(|e| format!("font non leggibile: {e}"))
}

// ---------------------------------------------------------------------------
// Binary helpers
// ---------------------------------------------------------------------------

fn u16_at(d: &[u8], at: usize) -> Option<u16> {
    d.get(at..at + 2).map(|b| u16::from_be_bytes([b[0], b[1]]))
}

fn put_u16(d: &mut [u8], at: usize, v: u16) {
    if let Some(b) = d.get_mut(at..at + 2) {
        b.copy_from_slice(&v.to_be_bytes());
    }
}

fn put_i16(d: &mut [u8], at: usize, v: i16) {
    put_u16(d, at, v as u16);
}

fn checksum(d: &[u8]) -> u32 {
    d.chunks(4).fold(0u32, |acc, c| {
        let mut w = [0u8; 4];
        w[..c.len()].copy_from_slice(c);
        acc.wrapping_add(u32::from_be_bytes(w))
    })
}

/// Serialise a table map into an sfnt: sorted directory, 4-byte padding,
/// per-table checksums and the `head` checksum adjustment.
fn write_sfnt(version: u32, tables: &BTreeMap<u32, Vec<u8>>) -> Vec<u8> {
    let n = tables.len() as u16;
    let mut pow = 1u16;
    let mut sel = 0u16;
    while pow * 2 <= n {
        pow *= 2;
        sel += 1;
    }
    let range = pow * 16;
    let mut out = Vec::new();
    out.extend(version.to_be_bytes());
    out.extend(n.to_be_bytes());
    out.extend(range.to_be_bytes());
    out.extend(sel.to_be_bytes());
    out.extend((n * 16 - range).to_be_bytes());

    let mut offset = 12 + 16 * tables.len();
    let mut body = Vec::new();
    let mut head_at = None;
    for (&t, data) in tables {
        let mut data = data.clone();
        if t == T_HEAD && data.len() >= 12 {
            data[8..12].fill(0);
            head_at = Some(offset);
        }
        out.extend(t.to_be_bytes());
        out.extend(checksum(&data).to_be_bytes());
        out.extend((offset as u32).to_be_bytes());
        out.extend((data.len() as u32).to_be_bytes());
        let padded = (data.len() + 3) & !3;
        body.extend(&data);
        body.resize(body.len() + padded - data.len(), 0);
        offset += padded;
    }
    out.extend(body);
    if let Some(at) = head_at {
        let adj = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        out[at + 8..at + 12].copy_from_slice(&adj.to_be_bytes());
    }
    out
}

// ---------------------------------------------------------------------------
// Outlines
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
struct P {
    x: f64,
    y: f64,
}

impl P {
    fn new(x: f64, y: f64) -> Self {
        P { x, y }
    }
    fn lerp(self, o: P, t: f64) -> P {
        P::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }
    fn round(self) -> (i32, i32) {
        (self.x.round() as i32, self.y.round() as i32)
    }
}

#[derive(Clone, Copy, Debug)]
enum Seg {
    Line(P),
    Quad(P, P),
    Cubic(P, P, P),
}

impl Seg {
    fn end(&self) -> P {
        match *self {
            Seg::Line(p) | Seg::Quad(_, p) | Seg::Cubic(_, _, p) => p,
        }
    }
}

/// One closed contour. The closing segment is always explicit: the last
/// segment ends on `start`.
#[derive(Clone, Debug)]
struct Contour {
    start: P,
    segs: Vec<Seg>,
}

impl Contour {
    /// Same shape, opposite direction. TrueType winds outer contours
    /// clockwise and PostScript counter-clockwise, so both conversions flip.
    fn reversed(&self) -> Contour {
        let mut ends = vec![self.start];
        ends.extend(self.segs.iter().map(Seg::end));
        let n = self.segs.len();
        let segs = (0..n)
            .rev()
            .map(|i| match self.segs[i] {
                Seg::Line(_) => Seg::Line(ends[i]),
                Seg::Quad(c, _) => Seg::Quad(c, ends[i]),
                Seg::Cubic(a, b, _) => Seg::Cubic(b, a, ends[i]),
            })
            .collect();
        Contour {
            start: ends[n],
            segs,
        }
    }
}

#[derive(Default)]
struct Collector {
    done: Vec<Contour>,
    cur: Option<Contour>,
}

impl Collector {
    fn finish(&mut self) {
        if let Some(mut c) = self.cur.take() {
            if c.segs.is_empty() {
                return;
            }
            let last = c.segs.last().map(Seg::end).unwrap_or(c.start);
            if last != c.start {
                c.segs.push(Seg::Line(c.start));
            }
            self.done.push(c);
        }
    }
    fn push(&mut self, s: Seg) {
        if let Some(c) = self.cur.as_mut() {
            c.segs.push(s);
        }
    }
}

impl OutlineBuilder for Collector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish();
        self.cur = Some(Contour {
            start: P::new(x as f64, y as f64),
            segs: Vec::new(),
        });
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.push(Seg::Line(P::new(x as f64, y as f64)));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.push(Seg::Quad(
            P::new(x1 as f64, y1 as f64),
            P::new(x as f64, y as f64),
        ));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.push(Seg::Cubic(
            P::new(x1 as f64, y1 as f64),
            P::new(x2 as f64, y2 as f64),
            P::new(x as f64, y as f64),
        ));
    }
    fn close(&mut self) {
        self.finish();
    }
}

/// The glyph's contours, already flipped to the other format's direction.
/// Composite TrueType glyphs come out flattened, components resolved.
fn outline(face: &Face<'_>, gid: u16) -> Vec<Contour> {
    let mut c = Collector::default();
    face.outline_glyph(GlyphId(gid), &mut c);
    c.finish();
    c.done.iter().map(Contour::reversed).collect()
}

// ---------------------------------------------------------------------------
// Horizontal metrics
// ---------------------------------------------------------------------------

struct Hmtx {
    advances: Vec<u16>,
    num_h_metrics: u16,
}

fn read_hmtx(tables: &BTreeMap<u32, Vec<u8>>, num_glyphs: u16) -> Result<Hmtx, String> {
    let hhea = tables.get(&T_HHEA).ok_or("tabella hhea assente")?;
    let hmtx = tables.get(&T_HMTX).ok_or("tabella hmtx assente")?;
    let num_h_metrics = u16_at(hhea, 34).ok_or("hhea troncata")?.max(1);
    let mut advances = Vec::with_capacity(num_glyphs as usize);
    let mut last = 0;
    for g in 0..num_glyphs {
        if g < num_h_metrics {
            last = u16_at(hmtx, g as usize * 4).unwrap_or(last);
        }
        advances.push(last);
    }
    Ok(Hmtx {
        advances,
        num_h_metrics: num_h_metrics.min(num_glyphs.max(1)),
    })
}

/// Rebuild `hmtx` with the same advances and left side bearings equal to the
/// new outlines' xMin, and refresh the extent fields of `hhea` to match.
fn rewrite_metrics(tables: &mut BTreeMap<u32, Vec<u8>>, h: &Hmtx, bboxes: &[Option<[i32; 4]>]) {
    let mut out = Vec::new();
    let (mut min_lsb, mut min_rsb, mut max_ext) = (i32::MAX, i32::MAX, i32::MIN);
    for (g, adv) in h.advances.iter().enumerate() {
        let lsb = bboxes[g].map(|b| b[0]).unwrap_or(0);
        if let Some(b) = bboxes[g] {
            min_lsb = min_lsb.min(b[0]);
            min_rsb = min_rsb.min(*adv as i32 - b[2]);
            max_ext = max_ext.max(b[2]);
        }
        if g < h.num_h_metrics as usize {
            out.extend(adv.to_be_bytes());
        }
        out.extend((clamp16(lsb)).to_be_bytes());
    }
    tables.insert(T_HMTX, out);
    if let Some(hhea) = tables.get_mut(&T_HHEA) {
        if min_lsb != i32::MAX {
            put_i16(hhea, 10, clamp16(min_lsb));
            put_i16(hhea, 12, clamp16(min_rsb));
            put_i16(hhea, 14, clamp16(max_ext));
        }
        put_u16(hhea, 34, h.num_h_metrics);
    }
}

fn clamp16(v: i32) -> i16 {
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

// ---------------------------------------------------------------------------
// Glyph names
// ---------------------------------------------------------------------------

/// Unicode code point → glyph, the lowest code point winning per glyph. The
/// Microsoft Symbol cmap (codes in U+F0xx) is folded to its low byte, which is
/// what those fonts mean by it.
fn unicode_of_glyphs(face: &Face<'_>) -> HashMap<u16, u32> {
    let mut by_gid: HashMap<u16, u32> = HashMap::new();
    let Some(cmap) = face.tables().cmap else {
        return by_gid;
    };
    for sub in cmap.subtables {
        let symbol = sub.platform_id == ttf_parser::PlatformId::Windows && sub.encoding_id == 0;
        if !sub.is_unicode() && !symbol {
            continue;
        }
        sub.codepoints(|cp| {
            if let Some(g) = sub.glyph_index(cp) {
                let cp = if symbol && (0xF000..=0xF0FF).contains(&cp) {
                    cp & 0xFF
                } else {
                    cp
                };
                by_gid
                    .entry(g.0)
                    .and_modify(|v| *v = (*v).min(cp))
                    .or_insert(cp);
            }
        });
    }
    by_gid
}

/// A PostScript-safe name for every glyph: the font's own where it has one,
/// otherwise `uniXXXX` from the cmap, otherwise `glyphN`. Always unique.
fn glyph_names(face: &Face<'_>, uni: &HashMap<u16, u32>) -> Vec<String> {
    let n = face.number_of_glyphs();
    let mut seen: HashSet<String> = HashSet::new();
    let mut names = Vec::with_capacity(n as usize);
    for g in 0..n {
        let own = face.glyph_name(GlyphId(g)).map(sanitize_name);
        let mut name = match own {
            _ if g == 0 => ".notdef".to_string(),
            Some(s) if !s.is_empty() && s != ".notdef" => s,
            _ => match uni.get(&g) {
                Some(&cp) if cp <= 0xFFFF => format!("uni{cp:04X}"),
                Some(&cp) => format!("u{cp:05X}"),
                None => format!("glyph{g}"),
            },
        };
        if seen.contains(&name) {
            let base = name.clone();
            let mut k = 1;
            while seen.contains(&name) {
                name = format!("{base}.{k}");
                k += 1;
            }
        }
        seen.insert(name.clone());
        names.push(name);
    }
    names
}

fn sanitize_name(s: &str) -> String {
    s.chars()
        .take(63)
        .map(|c| {
            if c.is_ascii_graphic() && !"()[]{}<>/%".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// A name-table string, preferring Unicode records.
fn name_string(face: &Face<'_>, id: u16) -> Option<String> {
    let names = face.names();
    let mut fallback = None;
    for n in names {
        if n.name_id != id {
            continue;
        }
        if let Some(s) = n.to_string() {
            if n.is_unicode() {
                return Some(s);
            }
            fallback.get_or_insert(s);
        }
    }
    fallback
}

/// Printable ASCII on one line — what CFF strings and AFM values tolerate.
fn ascii_line(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '©' => "(c)".to_string(),
            '®' => "(r)".to_string(),
            c if c.is_ascii_graphic() || c == ' ' => c.to_string(),
            c if c.is_whitespace() => " ".to_string(),
            _ => String::new(),
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn postscript_name(face: &Face<'_>) -> String {
    let raw = name_string(face, 6)
        .or_else(|| name_string(face, 4))
        .unwrap_or_else(|| "Untitled".into());
    let s: String = raw
        .chars()
        .filter(|c| c.is_ascii_graphic() && !"[](){}<>/%".contains(*c))
        .take(63)
        .collect();
    if s.is_empty() {
        "Untitled".into()
    } else {
        s
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Convert the outlines of a font. `target` is "otf" (CFF) or "ttf" (glyf).
#[tauri::command]
pub async fn font_convert(
    path: String,
    index: usize,
    dest: String,
    target: String,
) -> Result<ConvertReport, String> {
    tokio::task::spawn_blocking(move || convert_blocking(&path, index, &dest, &target))
        .await
        .map_err(|e| e.to_string())?
}

fn convert_blocking(
    path: &str,
    index: usize,
    dest: &str,
    target: &str,
) -> Result<ConvertReport, String> {
    let src = load(path, index)?;
    let out = convert_bytes(&src, target)?;
    std::fs::write(dest, &out.0).map_err(|e| format!("non riesco a scrivere il file: {e}"))?;
    Ok(ConvertReport {
        from: src.label.clone(),
        to: if target == "otf" { "OTF (CFF)" } else { "TTF" }.to_string(),
        glyphs: out.1,
        before: src.size,
        after: out.0.len() as u64,
        notes: out.2,
    })
}

fn convert_bytes(src: &Source, target: &str) -> Result<(Vec<u8>, u16, Vec<String>), String> {
    let face = face(&src.sfnt)?;
    if src.tables.contains_key(&T_FVAR) || src.tables.contains_key(&T_CFF2) {
        return Err(
            "È un font variabile: la conversione dei font variabili non è supportata \
             (servirebbe prima estrarne un'istanza statica)."
                .into(),
        );
    }
    match target {
        "otf" if src.cff => Err("Il font ha già contorni CFF: è già un OTF.".into()),
        "ttf" if !src.cff => Err("Il font ha già contorni TrueType: è già un TTF.".into()),
        "otf" => to_otf(src, &face),
        "ttf" => to_ttf(src, &face),
        _ => Err(format!("formato di destinazione sconosciuto: {target}")),
    }
}

/// Write the AFM metrics file of a font.
#[tauri::command]
pub async fn font_afm(path: String, index: usize, dest: String) -> Result<AfmReport, String> {
    tokio::task::spawn_blocking(move || afm_blocking(&path, index, &dest))
        .await
        .map_err(|e| e.to_string())?
}

fn afm_blocking(path: &str, index: usize, dest: &str) -> Result<AfmReport, String> {
    let src = load(path, index)?;
    let face = face(&src.sfnt)?;
    let (text, mut report) = build_afm(&face, &file_name(path));
    std::fs::write(dest, &text).map_err(|e| format!("non riesco a scrivere il file: {e}"))?;
    report.bytes = text.len() as u64;
    Ok(report)
}

// ---------------------------------------------------------------------------
// TrueType → CFF
// ---------------------------------------------------------------------------

fn to_otf(src: &Source, face: &Face<'_>) -> Result<(Vec<u8>, u16, Vec<String>), String> {
    let n = face.number_of_glyphs();
    let upem = face.units_per_em();
    let h = read_hmtx(&src.tables, n)?;
    let uni = unicode_of_glyphs(face);
    let names = glyph_names(face, &uni);

    let mut charstrings = Vec::with_capacity(n as usize);
    let mut bboxes = Vec::with_capacity(n as usize);
    for g in 0..n {
        let contours: Vec<Contour> = outline(face, g).iter().map(to_cubic).collect();
        let (cs, bbox) = charstring(&contours, h.advances[g as usize]);
        charstrings.push(cs);
        bboxes.push(bbox);
    }

    let ps = postscript_name(face);
    let bbox = face.global_bounding_box();
    let underline = face.underline_metrics();
    let top = TopDict {
        full_name: ascii_line(&name_string(face, 4).unwrap_or_else(|| ps.clone())),
        family: ascii_line(&name_string(face, 1).unwrap_or_else(|| ps.clone())),
        notice: ascii_line(&name_string(face, 0).unwrap_or_default()),
        fixed: face.is_monospaced(),
        italic_angle: face.italic_angle() as f64,
        underline_pos: underline.map(|u| u.position as i32).unwrap_or(-100),
        underline_thick: underline.map(|u| u.thickness as i32).unwrap_or(50),
        bbox: [
            bbox.x_min as i32,
            bbox.y_min as i32,
            bbox.x_max as i32,
            bbox.y_max as i32,
        ],
        upem,
    };
    let cff = build_cff(&ps, &top, &names, &charstrings);

    let mut tables = src.tables.clone();
    let mut notes = Vec::new();
    let hinted = [b"fpgm", b"prep", b"cvt "]
        .iter()
        .any(|t| tables.contains_key(&tag(t)));
    for t in TT_ONLY {
        tables.remove(&tag(t));
    }
    if hinted {
        notes.push("Hinting TrueType rimosso (fpgm, prep, cvt): non ha equivalente in CFF, e il font non è stato ri-hintato.".into());
    }
    if tables.remove(&tag(b"DSIG")).is_some() {
        notes.push(
            "Firma digitale (DSIG) rimossa: dopo la conversione non sarebbe più valida.".into(),
        );
    }
    tables.insert(T_CFF, cff);

    // maxp 0.5: just the glyph count, which is all CFF fonts carry.
    let mut maxp = 0x0000_5000u32.to_be_bytes().to_vec();
    maxp.extend(n.to_be_bytes());
    tables.insert(T_MAXP, maxp);
    tables.insert(T_POST, post_table(tables.get(&T_POST), 3, &[]));
    rewrite_metrics(&mut tables, &h, &bboxes);

    notes.push("Curve quadratiche convertite in cubiche (conversione esatta, coordinate arrotondate all'unità).".into());
    notes.push(format!(
        "Nomi dei glifi scritti nel charset CFF: {} glifi, tabella post in formato 3.",
        n
    ));
    Ok((write_sfnt(tag(b"OTTO"), &tables), n, notes))
}

/// Replace every quadratic segment with the cubic that draws the same curve.
fn to_cubic(c: &Contour) -> Contour {
    let mut prev = c.start;
    let segs = c
        .segs
        .iter()
        .map(|s| {
            let out = match *s {
                Seg::Quad(q, p) => Seg::Cubic(prev.lerp(q, 2.0 / 3.0), p.lerp(q, 2.0 / 3.0), p),
                other => other,
            };
            prev = s.end();
            out
        })
        .collect();
    Contour {
        start: c.start,
        segs,
    }
}

struct TopDict {
    full_name: String,
    family: String,
    notice: String,
    fixed: bool,
    italic_angle: f64,
    underline_pos: i32,
    underline_thick: i32,
    bbox: [i32; 4],
    upem: u16,
}

/// Type 2 charstring number.
fn cs_num(out: &mut Vec<u8>, v: i32) {
    let v = v.clamp(-32768, 32767);
    match v {
        -107..=107 => out.push((v + 139) as u8),
        108..=1131 => {
            let w = v - 108;
            out.push((w >> 8) as u8 + 247);
            out.push((w & 0xFF) as u8);
        }
        -1131..=-108 => {
            let w = -v - 108;
            out.push((w >> 8) as u8 + 251);
            out.push((w & 0xFF) as u8);
        }
        _ => {
            out.push(28);
            out.extend((v as i16).to_be_bytes());
        }
    }
}

/// Encode one glyph (cubic contours only) as a Type 2 charstring, with its
/// exact bounding box. Width is always written against nominalWidthX = 0.
fn charstring(contours: &[Contour], advance: u16) -> (Vec<u8>, Option<[i32; 4]>) {
    const RMOVETO: u8 = 21;
    const RLINETO: u8 = 5;
    const RRCURVETO: u8 = 8;
    const ENDCHAR: u8 = 14;

    let mut out = Vec::new();
    let mut width = (advance != 0).then_some(advance as i32);
    let mut cur = (0i32, 0i32);
    let mut bb = BBox::default();

    for c in contours {
        // Drop the explicit closing line: a charstring contour closes itself.
        let mut segs: &[Seg] = &c.segs;
        if let Some(Seg::Line(p)) = segs.last() {
            if p.round() == c.start.round() {
                segs = &segs[..segs.len() - 1];
            }
        }
        if segs.is_empty() {
            continue;
        }
        let start = c.start.round();
        if let Some(w) = width.take() {
            cs_num(&mut out, w);
        }
        cs_num(&mut out, start.0 - cur.0);
        cs_num(&mut out, start.1 - cur.1);
        out.push(RMOVETO);
        cur = start;
        bb.add(start);

        // Runs of the same operator share it, up to the 48-argument stack.
        let mut pending: Option<u8> = None;
        let mut args = 0;
        let mut prev_f = c.start;
        for s in segs {
            let (op, pts): (u8, Vec<(i32, i32)>) = match *s {
                Seg::Line(p) => (RLINETO, vec![p.round()]),
                Seg::Cubic(a, b, p) => {
                    bb.add_cubic(prev_f, a, b, p);
                    (RRCURVETO, vec![a.round(), b.round(), p.round()])
                }
                Seg::Quad(..) => unreachable!("contorni già convertiti in cubiche"),
            };
            prev_f = s.end();
            let end = *pts.last().unwrap_or(&cur);
            if op == RLINETO && end == cur {
                continue;
            }
            bb.add(end);
            if pending.is_some_and(|p| p != op) || args + pts.len() * 2 > 48 {
                out.push(pending.take().unwrap_or(op));
                args = 0;
            }
            for p in pts {
                cs_num(&mut out, p.0 - cur.0);
                cs_num(&mut out, p.1 - cur.1);
                cur = p;
                args += 2;
            }
            pending = Some(op);
        }
        if let Some(op) = pending {
            out.push(op);
        }
    }
    if let Some(w) = width {
        cs_num(&mut out, w);
    }
    out.push(ENDCHAR);
    (out, bb.get())
}

#[derive(Default)]
struct BBox {
    b: Option<[f64; 4]>,
}

impl BBox {
    fn add_f(&mut self, x: f64, y: f64) {
        let b = self.b.get_or_insert([x, y, x, y]);
        b[0] = b[0].min(x);
        b[1] = b[1].min(y);
        b[2] = b[2].max(x);
        b[3] = b[3].max(y);
    }
    fn add(&mut self, p: (i32, i32)) {
        self.add_f(p.0 as f64, p.1 as f64);
    }
    /// Add the extremes of a cubic, found where its derivative vanishes.
    fn add_cubic(&mut self, p0: P, p1: P, p2: P, p3: P) {
        let axis = |a: f64, b: f64, c: f64, d: f64| -> Vec<f64> {
            // B'(t)/3 = at² + bt + c
            let qa = -a + 3.0 * b - 3.0 * c + d;
            let qb = 2.0 * (a - 2.0 * b + c);
            let qc = b - a;
            let mut ts = Vec::new();
            if qa.abs() < 1e-12 {
                if qb.abs() > 1e-12 {
                    ts.push(-qc / qb);
                }
            } else {
                let disc = qb * qb - 4.0 * qa * qc;
                if disc >= 0.0 {
                    let r = disc.sqrt();
                    ts.push((-qb + r) / (2.0 * qa));
                    ts.push((-qb - r) / (2.0 * qa));
                }
            }
            ts.retain(|t| *t > 0.0 && *t < 1.0);
            ts
        };
        let mut ts = axis(p0.x, p1.x, p2.x, p3.x);
        ts.extend(axis(p0.y, p1.y, p2.y, p3.y));
        for t in ts {
            let p = cubic_at(p0, p1, p2, p3, t);
            self.add_f(p.x.round(), p.y.round());
        }
    }
    fn get(&self) -> Option<[i32; 4]> {
        self.b.map(|b| b.map(|v| v.round() as i32))
    }
}

fn cubic_at(p0: P, p1: P, p2: P, p3: P, t: f64) -> P {
    let a = p0.lerp(p1, t);
    let b = p1.lerp(p2, t);
    let c = p2.lerp(p3, t);
    let d = a.lerp(b, t);
    let e = b.lerp(c, t);
    d.lerp(e, t)
}

/// CFF INDEX structure.
fn cff_index(items: &[Vec<u8>]) -> Vec<u8> {
    let mut out = (items.len() as u16).to_be_bytes().to_vec();
    if items.is_empty() {
        return out;
    }
    let total: usize = items.iter().map(Vec::len).sum::<usize>() + 1;
    let off_size: u8 = match total {
        0..=0xFF => 1,
        0x100..=0xFFFF => 2,
        0x1_0000..=0xFF_FFFF => 3,
        _ => 4,
    };
    out.push(off_size);
    let put = |out: &mut Vec<u8>, v: usize| {
        let b = (v as u32).to_be_bytes();
        out.extend(&b[4 - off_size as usize..]);
    };
    let mut off = 1;
    put(&mut out, off);
    for it in items {
        off += it.len();
        put(&mut out, off);
    }
    for it in items {
        out.extend(it);
    }
    out
}

/// DICT integer, always the 5-byte form when `fixed` so that offsets can be
/// patched in without changing the dictionary's size.
fn dict_int(out: &mut Vec<u8>, v: i32, fixed: bool) {
    if fixed {
        out.push(29);
        out.extend(v.to_be_bytes());
        return;
    }
    match v {
        -107..=107 => out.push((v + 139) as u8),
        108..=1131 => {
            let w = v - 108;
            out.push((w >> 8) as u8 + 247);
            out.push((w & 0xFF) as u8);
        }
        -1131..=-108 => {
            let w = -v - 108;
            out.push((w >> 8) as u8 + 251);
            out.push((w & 0xFF) as u8);
        }
        -32768..=32767 => {
            out.push(28);
            out.extend((v as i16).to_be_bytes());
        }
        _ => {
            out.push(29);
            out.extend(v.to_be_bytes());
        }
    }
}

/// DICT real number, nibble-encoded.
fn dict_real(out: &mut Vec<u8>, v: f64) {
    if v.fract() == 0.0 && v.abs() < 1e9 {
        dict_int(out, v as i32, false);
        return;
    }
    let s = format!("{v}");
    let mut nibbles: Vec<u8> = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '0'..='9' => nibbles.push(c as u8 - b'0'),
            '.' => nibbles.push(0xA),
            '-' => nibbles.push(0xE),
            'e' | 'E' => {
                if chars.peek() == Some(&'-') {
                    chars.next();
                    nibbles.push(0xC);
                } else {
                    nibbles.push(0xB);
                }
            }
            _ => {}
        }
    }
    nibbles.push(0xF);
    if nibbles.len() % 2 == 1 {
        nibbles.push(0xF);
    }
    out.push(30);
    for pair in nibbles.chunks(2) {
        out.push((pair[0] << 4) | pair[1]);
    }
}

fn build_cff(ps_name: &str, top: &TopDict, names: &[String], charstrings: &[Vec<u8>]) -> Vec<u8> {
    // Every string is custom (SID ≥ 391): valid, and it spares carrying the
    // 391-entry standard strings table around.
    let mut strings: Vec<Vec<u8>> = Vec::new();
    let sid = |s: &str, strings: &mut Vec<Vec<u8>>| -> i32 {
        strings.push(s.as_bytes().to_vec());
        390 + strings.len() as i32
    };
    let notice = (!top.notice.is_empty()).then(|| sid(&top.notice, &mut strings));
    let full = sid(&top.full_name, &mut strings);
    let family = sid(&top.family, &mut strings);
    let glyph_sids: Vec<u16> = names
        .iter()
        .skip(1)
        .map(|n| sid(n, &mut strings) as u16)
        .collect();

    let mut charset = vec![0u8];
    for s in &glyph_sids {
        charset.extend(s.to_be_bytes());
    }
    let mut private = Vec::new();
    dict_int(&mut private, 0, false);
    private.push(20); // defaultWidthX
    dict_int(&mut private, 0, false);
    private.push(21); // nominalWidthX

    let build_top = |charset_off: i32, cs_off: i32, priv_off: i32| -> Vec<u8> {
        let mut d = Vec::new();
        if let Some(n) = notice {
            dict_int(&mut d, n, false);
            d.push(1);
        }
        dict_int(&mut d, full, false);
        d.push(2);
        dict_int(&mut d, family, false);
        d.push(3);
        if top.fixed {
            dict_int(&mut d, 1, false);
            d.extend([12, 1]);
        }
        if top.italic_angle != 0.0 {
            dict_real(&mut d, top.italic_angle);
            d.extend([12, 2]);
        }
        dict_int(&mut d, top.underline_pos, false);
        d.extend([12, 3]);
        dict_int(&mut d, top.underline_thick, false);
        d.extend([12, 4]);
        if top.upem != 1000 {
            let s = 1.0 / top.upem as f64;
            for v in [s, 0.0, 0.0, s, 0.0, 0.0] {
                dict_real(&mut d, v);
            }
            d.extend([12, 7]);
        }
        for v in top.bbox {
            dict_int(&mut d, v, false);
        }
        d.push(5);
        dict_int(&mut d, charset_off, true);
        d.push(15);
        dict_int(&mut d, cs_off, true);
        d.push(17);
        dict_int(&mut d, private.len() as i32, true);
        dict_int(&mut d, priv_off, true);
        d.push(18);
        d
    };

    let header = [1u8, 0, 4, 4];
    let name_index = cff_index(&[ps_name.as_bytes().to_vec()]);
    let string_index = cff_index(&strings);
    let gsubrs = cff_index(&[]);
    let cs_index = cff_index(charstrings);
    let top_len = cff_index(&[build_top(0, 0, 0)]).len();

    let charset_off = header.len() + name_index.len() + top_len + string_index.len() + gsubrs.len();
    let cs_off = charset_off + charset.len();
    let priv_off = cs_off + cs_index.len();
    let top_index = cff_index(&[build_top(
        charset_off as i32,
        cs_off as i32,
        priv_off as i32,
    )]);
    debug_assert_eq!(top_index.len(), top_len);

    let mut out = header.to_vec();
    out.extend(name_index);
    out.extend(top_index);
    out.extend(string_index);
    out.extend(gsubrs);
    out.extend(charset);
    out.extend(cs_index);
    out.extend(private);
    out
}

/// A `post` table in format 2 (with names) or 3 (without), keeping the
/// header fields of the original.
fn post_table(orig: Option<&Vec<u8>>, format: u8, names: &[String]) -> Vec<u8> {
    let mut out = vec![0u8; 32];
    if let Some(o) = orig.filter(|o| o.len() >= 32) {
        out.copy_from_slice(&o[..32]);
    }
    out[0..4].copy_from_slice(&(format as u32 * 0x1_0000).to_be_bytes());
    if format == 2 {
        // All names custom (index ≥ 258): no need for the Macintosh set.
        out.extend((names.len() as u16).to_be_bytes());
        for i in 0..names.len() {
            out.extend((258 + i as u16).to_be_bytes());
        }
        for n in names {
            let b = &n.as_bytes()[..n.len().min(255)];
            out.push(b.len() as u8);
            out.extend(b);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// CFF → TrueType
// ---------------------------------------------------------------------------

fn to_ttf(src: &Source, face: &Face<'_>) -> Result<(Vec<u8>, u16, Vec<String>), String> {
    let n = face.number_of_glyphs();
    let upem = face.units_per_em();
    let h = read_hmtx(&src.tables, n)?;
    let uni = unicode_of_glyphs(face);
    let names = glyph_names(face, &uni);
    let tolerance = (upem as f64 / 1000.0).max(0.5);

    let mut glyf = Vec::new();
    let mut loca: Vec<u32> = vec![0];
    let mut bboxes = Vec::with_capacity(n as usize);
    let (mut max_points, mut max_contours) = (0usize, 0usize);
    let mut head_bb: Option<[i32; 4]> = None;
    let mut quads = 0usize;
    for g in 0..n {
        let contours: Vec<Vec<(i32, i32, bool)>> = outline(face, g)
            .iter()
            .map(|c| tt_points(c, tolerance, &mut quads))
            .filter(|pts| pts.len() >= 3)
            .collect();
        let (data, bbox) = simple_glyph(&contours);
        let points: usize = contours.iter().map(Vec::len).sum();
        max_points = max_points.max(points);
        max_contours = max_contours.max(contours.len());
        if let Some(b) = bbox {
            let hb = head_bb.get_or_insert(b);
            hb[0] = hb[0].min(b[0]);
            hb[1] = hb[1].min(b[1]);
            hb[2] = hb[2].max(b[2]);
            hb[3] = hb[3].max(b[3]);
        }
        bboxes.push(bbox);
        glyf.extend(&data);
        glyf.resize((glyf.len() + 3) & !3, 0);
        loca.push(glyf.len() as u32);
    }

    let mut tables = src.tables.clone();
    let mut notes = Vec::new();
    for t in CFF_ONLY {
        tables.remove(&tag(t));
    }
    if tables.remove(&tag(b"DSIG")).is_some() {
        notes.push(
            "Firma digitale (DSIG) rimossa: dopo la conversione non sarebbe più valida.".into(),
        );
    }
    tables.insert(T_GLYF, glyf);
    tables.insert(T_LOCA, loca.iter().flat_map(|v| v.to_be_bytes()).collect());

    let mut maxp = vec![0u8; 32];
    maxp[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    put_u16(&mut maxp, 4, n);
    put_u16(&mut maxp, 6, max_points.min(u16::MAX as usize) as u16);
    put_u16(&mut maxp, 8, max_contours.min(u16::MAX as usize) as u16);
    put_u16(&mut maxp, 14, 2); // maxZones
    tables.insert(T_MAXP, maxp);

    if let Some(head) = tables.get_mut(&T_HEAD) {
        let flags = u16_at(head, 16).unwrap_or(0);
        put_u16(head, 16, flags | 0x0002); // lsb == xMin, which is how hmtx is written
        if let Some(b) = head_bb {
            put_i16(head, 36, clamp16(b[0]));
            put_i16(head, 38, clamp16(b[1]));
            put_i16(head, 40, clamp16(b[2]));
            put_i16(head, 42, clamp16(b[3]));
        }
        put_i16(head, 50, 1); // long loca
        put_i16(head, 52, 0);
    }
    tables.insert(T_POST, post_table(tables.get(&T_POST), 2, &names));
    rewrite_metrics(&mut tables, &h, &bboxes);

    notes.push(format!(
        "Curve cubiche approssimate con {quads} segmenti quadratici, errore massimo {tolerance:.1} unità (1/1000 di em)."
    ));
    notes.push(
        "Nessun hinting TrueType generato: il font si affida all'autohinting del sistema.".into(),
    );
    notes.push("Nomi dei glifi portati dal charset CFF nella tabella post (formato 2).".into());
    Ok((write_sfnt(0x0001_0000, &tables), n, notes))
}

/// Turn one contour into TrueType points (x, y, on-curve), approximating
/// cubics with quadratics within `tol` font units.
fn tt_points(c: &Contour, tol: f64, quads: &mut usize) -> Vec<(i32, i32, bool)> {
    let mut pts: Vec<(i32, i32, bool)> = Vec::new();
    let push_on = |pts: &mut Vec<(i32, i32, bool)>, p: P| {
        let (x, y) = p.round();
        if !matches!(pts.last(), Some(&(px, py, true)) if px == x && py == y) {
            pts.push((x, y, true));
        }
    };
    push_on(&mut pts, c.start);
    let mut prev = c.start;
    for s in &c.segs {
        match *s {
            Seg::Line(p) => push_on(&mut pts, p),
            Seg::Quad(q, p) => {
                let (x, y) = q.round();
                pts.push((x, y, false));
                push_on(&mut pts, p);
                *quads += 1;
            }
            Seg::Cubic(a, b, p) => {
                for (q, e) in cubic_to_quads(prev, a, b, p, tol) {
                    let (x, y) = q.round();
                    pts.push((x, y, false));
                    push_on(&mut pts, e);
                    *quads += 1;
                }
            }
        }
        prev = s.end();
    }
    // The last point is the start again: TrueType closes contours itself.
    if pts.len() > 1 && pts.last() == pts.first() {
        pts.pop();
    }
    // Drop on-curve points sitting exactly halfway between two off-curve
    // ones: TrueType implies them.
    let mut i = 1;
    while i + 1 < pts.len() {
        let (a, m, b) = (pts[i - 1], pts[i], pts[i + 1]);
        if m.2 && !a.2 && !b.2 && a.0 + b.0 == 2 * m.0 && a.1 + b.1 == 2 * m.1 {
            pts.remove(i);
        } else {
            i += 1;
        }
    }
    pts
}

/// Split a cubic into as many pieces as needed for each one's best single
/// quadratic to stay within `tol`. The quadratic of a piece is the midpoint
/// approximation, whose error is exactly √3/36 · |p3 − 3p2 + 3p1 − p0|; that
/// third difference shrinks with the cube of the piece count, so the count
/// comes straight out of the bound instead of from trial and error.
fn cubic_to_quads(p0: P, p1: P, p2: P, p3: P, tol: f64) -> Vec<(P, P)> {
    let dx = p3.x - 3.0 * p2.x + 3.0 * p1.x - p0.x;
    let dy = p3.y - 3.0 * p2.y + 3.0 * p1.y - p0.y;
    let err = (dx * dx + dy * dy).sqrt() * 3f64.sqrt() / 36.0;
    let n = ((err / tol).cbrt().ceil() as usize).clamp(1, 32);

    let deriv = |t: f64| -> P {
        let u = 1.0 - t;
        P::new(
            3.0 * (u * u * (p1.x - p0.x) + 2.0 * u * t * (p2.x - p1.x) + t * t * (p3.x - p2.x)),
            3.0 * (u * u * (p1.y - p0.y) + 2.0 * u * t * (p2.y - p1.y) + t * t * (p3.y - p2.y)),
        )
    };
    (0..n)
        .map(|i| {
            let (t0, t1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
            let dt = (t1 - t0) / 3.0;
            let b0 = if i == 0 {
                p0
            } else {
                cubic_at(p0, p1, p2, p3, t0)
            };
            let b3 = if i + 1 == n {
                p3
            } else {
                cubic_at(p0, p1, p2, p3, t1)
            };
            let (d0, d1) = (deriv(t0), deriv(t1));
            let b1 = P::new(b0.x + d0.x * dt, b0.y + d0.y * dt);
            let b2 = P::new(b3.x - d1.x * dt, b3.y - d1.y * dt);
            let q = P::new(
                (3.0 * (b1.x + b2.x) - b0.x - b3.x) / 4.0,
                (3.0 * (b1.y + b2.y) - b0.y - b3.y) / 4.0,
            );
            (q, b3)
        })
        .collect()
}

/// A simple `glyf` entry (no instructions), and its bounding box. An empty
/// glyph is zero bytes.
fn simple_glyph(contours: &[Vec<(i32, i32, bool)>]) -> (Vec<u8>, Option<[i32; 4]>) {
    let all: Vec<&(i32, i32, bool)> = contours.iter().flatten().collect();
    if all.is_empty() {
        return (Vec::new(), None);
    }
    let bbox = [
        all.iter().map(|p| p.0).min().unwrap_or(0),
        all.iter().map(|p| p.1).min().unwrap_or(0),
        all.iter().map(|p| p.0).max().unwrap_or(0),
        all.iter().map(|p| p.1).max().unwrap_or(0),
    ];
    let mut out = Vec::new();
    out.extend((contours.len() as i16).to_be_bytes());
    for v in bbox {
        out.extend(clamp16(v).to_be_bytes());
    }
    let mut end = 0usize;
    for c in contours {
        end += c.len();
        out.extend(((end - 1) as u16).to_be_bytes());
    }
    out.extend(0u16.to_be_bytes()); // no instructions

    const ON: u8 = 0x01;
    const X_SHORT: u8 = 0x02;
    const Y_SHORT: u8 = 0x04;
    const REPEAT: u8 = 0x08;
    const X_SAME: u8 = 0x10;
    const Y_SAME: u8 = 0x20;
    let (mut flags, mut xs, mut ys) = (Vec::new(), Vec::new(), Vec::new());
    let (mut cx, mut cy) = (0i32, 0i32);
    for p in &all {
        let (dx, dy) = (p.0 - cx, p.1 - cy);
        (cx, cy) = (p.0, p.1);
        let mut f = if p.2 { ON } else { 0 };
        for (d, short, same, bytes) in [
            (dx, X_SHORT, X_SAME, &mut xs),
            (dy, Y_SHORT, Y_SAME, &mut ys),
        ] {
            if d == 0 {
                f |= same;
            } else if d.abs() < 256 {
                f |= short;
                if d > 0 {
                    f |= same;
                }
                bytes.push(d.unsigned_abs() as u8);
            } else {
                bytes.extend(clamp16(d).to_be_bytes());
            }
        }
        flags.push(f);
    }
    let mut i = 0;
    while i < flags.len() {
        let f = flags[i];
        let mut run = 0;
        while i + 1 + run < flags.len() && flags[i + 1 + run] == f && run < 255 {
            run += 1;
        }
        if run > 0 {
            out.push(f | REPEAT);
            out.push(run as u8);
        } else {
            out.push(f);
        }
        i += 1 + run;
    }
    out.extend(xs);
    out.extend(ys);
    (out, Some(bbox))
}

// ---------------------------------------------------------------------------
// AFM
// ---------------------------------------------------------------------------

/// Glyphs considered for kerning pairs out of GPOS: the ones a code point
/// reaches, lowest code points first. GPOS pair sets can only be queried, not
/// listed, so pairs are found by asking; this caps the n² questions.
const MAX_KERN_GLYPHS: usize = 1500;

fn weight_name(class: u16) -> &'static str {
    match class {
        0..=149 => "Thin",
        150..=249 => "ExtraLight",
        250..=349 => "Light",
        350..=449 => "Regular",
        450..=549 => "Medium",
        550..=649 => "SemiBold",
        650..=749 => "Bold",
        750..=849 => "ExtraBold",
        _ => "Black",
    }
}

fn build_afm(face: &Face<'_>, source_name: &str) -> (String, AfmReport) {
    use std::fmt::Write as _;

    let upem = face.units_per_em().max(1) as f64;
    let sc = |v: f64| -> i32 { (v * 1000.0 / upem).round() as i32 };
    let n = face.number_of_glyphs();
    let uni = unicode_of_glyphs(face);
    let names = glyph_names(face, &uni);
    let ps = postscript_name(face);

    let mut s = String::new();
    let _ = writeln!(s, "StartFontMetrics 4.1");
    let _ = writeln!(
        s,
        "Comment Generated by DevToys from {}",
        ascii_line(source_name)
    );
    let _ = writeln!(
        s,
        "Comment Units per em of the source: {}",
        face.units_per_em()
    );
    let _ = writeln!(s, "FontName {ps}");
    if let Some(v) = name_string(face, 4) {
        let _ = writeln!(s, "FullName {}", ascii_line(&v));
    }
    if let Some(v) = name_string(face, 1) {
        let _ = writeln!(s, "FamilyName {}", ascii_line(&v));
    }
    let _ = writeln!(s, "Weight {}", weight_name(face.weight().to_number()));
    let angle = face.italic_angle();
    let _ = writeln!(s, "ItalicAngle {}", (angle * 10.0).round() / 10.0);
    let _ = writeln!(s, "IsFixedPitch {}", face.is_monospaced());
    let bb = face.global_bounding_box();
    let _ = writeln!(
        s,
        "FontBBox {} {} {} {}",
        sc(bb.x_min as f64),
        sc(bb.y_min as f64),
        sc(bb.x_max as f64),
        sc(bb.y_max as f64)
    );
    if let Some(u) = face.underline_metrics() {
        let _ = writeln!(s, "UnderlinePosition {}", sc(u.position as f64));
        let _ = writeln!(s, "UnderlineThickness {}", sc(u.thickness as f64));
    }
    if let Some(v) = name_string(face, 5) {
        let v = ascii_line(&v);
        let v = v.strip_prefix("Version ").unwrap_or(&v);
        let _ = writeln!(s, "Version {v}");
    }
    if let Some(v) = name_string(face, 0) {
        let _ = writeln!(s, "Notice {}", ascii_line(&v));
    }
    let _ = writeln!(s, "EncodingScheme FontSpecific");
    if let Some(v) = face.capital_height() {
        let _ = writeln!(s, "CapHeight {}", sc(v as f64));
    }
    if let Some(v) = face.x_height() {
        let _ = writeln!(s, "XHeight {}", sc(v as f64));
    }
    let _ = writeln!(s, "Ascender {}", sc(face.ascender() as f64));
    let _ = writeln!(s, "Descender {}", sc(face.descender() as f64));

    // Codes 32–255 are the Latin-1 code points; everything else is -1.
    let code = |g: u16| -> i32 {
        match uni.get(&g) {
            Some(&cp) if (32..=255).contains(&cp) => cp as i32,
            _ => -1,
        }
    };
    let mut order: Vec<u16> = (0..n).collect();
    order.sort_by_key(|&g| {
        let c = code(g);
        (c < 0, if c < 0 { g as i32 } else { c })
    });
    let encoded = order.iter().filter(|&&g| code(g) >= 0).count();
    let _ = writeln!(s, "StartCharMetrics {n}");
    for g in order {
        let w = face.glyph_hor_advance(GlyphId(g)).unwrap_or(0) as f64;
        let b = face
            .glyph_bounding_box(GlyphId(g))
            .map(|r| {
                [
                    sc(r.x_min as f64),
                    sc(r.y_min as f64),
                    sc(r.x_max as f64),
                    sc(r.y_max as f64),
                ]
            })
            .unwrap_or([0; 4]);
        let _ = writeln!(
            s,
            "C {} ; WX {} ; N {} ; B {} {} {} {} ;",
            code(g),
            sc(w),
            names[g as usize],
            b[0],
            b[1],
            b[2],
            b[3]
        );
    }
    let _ = writeln!(s, "EndCharMetrics");

    let pairs: Vec<(u16, u16, i32)> = kerning(face, &uni)
        .into_iter()
        .map(|((l, r), v)| (l, r, sc(v as f64)))
        .filter(|p| p.2 != 0)
        .collect();
    if !pairs.is_empty() {
        let _ = writeln!(s, "StartKernData");
        let _ = writeln!(s, "StartKernPairs {}", pairs.len());
        for (l, r, v) in &pairs {
            let _ = writeln!(s, "KPX {} {} {v}", names[*l as usize], names[*r as usize]);
        }
        let _ = writeln!(s, "EndKernPairs");
        let _ = writeln!(s, "EndKernData");
    }
    let _ = writeln!(s, "EndFontMetrics");

    let preview = s.lines().take(60).collect::<Vec<_>>().join("\n");
    let report = AfmReport {
        font_name: ps,
        glyphs: n as usize,
        encoded,
        kern_pairs: pairs.len(),
        bytes: 0,
        preview,
    };
    (s, report)
}

/// Horizontal kerning pairs in font units: from the GPOS `kern` feature when
/// the font has one, otherwise from the old `kern` table.
fn kerning(face: &Face<'_>, uni: &HashMap<u16, u32>) -> BTreeMap<(u16, u16), i32> {
    let gpos = gpos_kerning(face, uni);
    if !gpos.is_empty() {
        return gpos;
    }
    let mut out = BTreeMap::new();
    if let Some(kern) = face.tables().kern {
        for sub in kern.subtables {
            if !sub.horizontal || sub.variable || sub.has_cross_stream {
                continue;
            }
            if let ttf_parser::kern::Format::Format0(f0) = sub.format {
                for p in f0.pairs {
                    *out.entry((p.left().0, p.right().0)).or_insert(0) += p.value as i32;
                }
            }
        }
    }
    out.retain(|_, v| *v != 0);
    out
}

fn gpos_kerning(face: &Face<'_>, uni: &HashMap<u16, u32>) -> BTreeMap<(u16, u16), i32> {
    use ttf_parser::gpos::{PairAdjustment, PositioningSubtable};

    let mut out = BTreeMap::new();
    let Some(gpos) = face.tables().gpos else {
        return out;
    };
    let mut lookup_ids: Vec<u16> = Vec::new();
    for i in 0..gpos.features.len() {
        if let Some(f) = gpos.features.get(i) {
            if f.tag == ttf_parser::Tag::from_bytes(b"kern") {
                lookup_ids.extend(f.lookup_indices);
            }
        }
    }
    lookup_ids.sort_unstable();
    lookup_ids.dedup();
    let lookups: Vec<Vec<PairAdjustment<'_>>> = lookup_ids
        .iter()
        .filter_map(|&i| gpos.lookups.get(i))
        .map(|l| {
            (0..l.subtables.len())
                .filter_map(|i| match l.subtables.get::<PositioningSubtable>(i) {
                    Some(PositioningSubtable::Pair(p)) => Some(p),
                    _ => None,
                })
                .collect()
        })
        .collect();
    if lookups.iter().all(Vec::is_empty) {
        return out;
    }

    let mut glyphs: Vec<(u32, u16)> = uni.iter().map(|(&g, &cp)| (cp, g)).collect();
    glyphs.sort_unstable();
    let glyphs: Vec<u16> = glyphs
        .into_iter()
        .map(|(_, g)| g)
        .take(MAX_KERN_GLYPHS)
        .collect();

    for &l in &glyphs {
        // Per lookup, the subtables that cover `l` and at which index.
        let covering: Vec<Vec<(&PairAdjustment<'_>, u16)>> = lookups
            .iter()
            .map(|subs| {
                subs.iter()
                    .filter_map(|st| st.coverage().get(GlyphId(l)).map(|ci| (st, ci)))
                    .collect()
            })
            .collect();
        if covering.iter().all(Vec::is_empty) {
            continue;
        }
        for &r in &glyphs {
            let mut total = 0i32;
            for subs in &covering {
                for (st, ci) in subs {
                    let v = match st {
                        PairAdjustment::Format1 { sets, .. } => sets
                            .get(*ci)
                            .and_then(|set| set.get(GlyphId(r)))
                            .map(|(a, _)| a.x_advance),
                        PairAdjustment::Format2 {
                            classes, matrix, ..
                        } => matrix
                            .get((classes.0.get(GlyphId(l)), classes.1.get(GlyphId(r))))
                            .map(|(a, _)| a.x_advance),
                    };
                    if let Some(v) = v {
                        total += v as i32;
                        break;
                    }
                }
            }
            if total != 0 {
                out.insert((l, r), total);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn find(candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .find(|p| Path::new(p).is_file())
            .map(|p| p.to_string())
    }

    const TTF: &[&str] = &[
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
    ];
    const OTF: &[&str] = &[
        "/usr/share/fonts/OTF/PowerlineSymbols.otf",
        "/usr/share/fonts/adobe-source-code-pro/SourceCodePro-Regular.otf",
    ];

    fn scratch(name: &str) -> String {
        std::env::temp_dir()
            .join(format!("devtoys-test-{name}"))
            .to_string_lossy()
            .into_owned()
    }

    /// The exact bounding box of a glyph's drawn outline. ttf-parser reports
    /// the `glyf` header box, which counts off-curve points, so it cannot be
    /// compared with a CFF box as is.
    fn exact_bbox(f: &Face<'_>, g: u16) -> Option<[i32; 4]> {
        let mut bb = BBox::default();
        for c in outline(f, g).iter().map(to_cubic) {
            // Single-point contours (hinting anchors) draw nothing.
            if c.segs
                .iter()
                .all(|s| matches!(s, Seg::Line(p) if *p == c.start))
            {
                continue;
            }
            bb.add(c.start.round());
            let mut prev = c.start;
            for s in &c.segs {
                if let Seg::Cubic(a, b, p) = *s {
                    bb.add_cubic(prev, a, b, p);
                }
                bb.add(s.end().round());
                prev = s.end();
            }
        }
        bb.get()
    }

    /// Every glyph of `a` and `b` has the same advance and an outline whose
    /// bounding box is within `tol` units.
    fn same_glyphs(a: &Face<'_>, b: &Face<'_>, tol: i32) {
        assert_eq!(a.number_of_glyphs(), b.number_of_glyphs());
        for g in 0..a.number_of_glyphs() {
            let id = GlyphId(g);
            assert_eq!(
                a.glyph_hor_advance(id),
                b.glyph_hor_advance(id),
                "avanzamento glifo {g}"
            );
            match (exact_bbox(a, g), exact_bbox(b, g)) {
                (Some(x), Some(y)) => {
                    for k in 0..4 {
                        assert!((x[k] - y[k]).abs() <= tol, "bbox glifo {g}: {x:?} vs {y:?}");
                    }
                }
                (None, None) => {}
                (x, y) => {
                    // A degenerate contour may vanish; only a real shape counts.
                    let real = |r: Option<[i32; 4]>| r.is_some_and(|r| r[2] > r[0] && r[3] > r[1]);
                    assert!(!real(x) && !real(y), "glifo {g}: {x:?} vs {y:?}");
                }
            }
        }
    }

    #[test]
    fn ttf_to_otf_and_back() {
        let Some(path) = find(TTF) else {
            eprintln!("nessun TTF di sistema: test saltato");
            return;
        };
        let otf = scratch("conv.otf");
        let r = convert_blocking(&path, 0, &otf, "otf").expect("TTF → OTF");
        assert_eq!(r.to, "OTF (CFF)");

        let src = load(&path, 0).unwrap();
        let out = load(&otf, 0).expect("l'OTF prodotto si rilegge");
        assert!(out.cff);
        let (fa, fb) = (face(&src.sfnt).unwrap(), face(&out.sfnt).unwrap());
        same_glyphs(&fa, &fb, 1);
        assert_eq!(fa.glyph_name(GlyphId(36)), fb.glyph_name(GlyphId(36)));
        // allsorts must accept it too: that is what the other tools read with.
        crate::font::inspect_blocking(&otf, 0).expect("allsorts rilegge l'OTF");

        let ttf = scratch("back.ttf");
        convert_blocking(&otf, 0, &ttf, "ttf").expect("OTF → TTF");
        let back = load(&ttf, 0).unwrap();
        assert!(!back.cff);
        let fc = face(&back.sfnt).unwrap();
        same_glyphs(&fb, &fc, 2);
        assert_eq!(fb.glyph_name(GlyphId(36)), fc.glyph_name(GlyphId(36)));
        crate::font::inspect_blocking(&ttf, 0).expect("allsorts rilegge il TTF");
    }

    #[test]
    fn otf_to_ttf() {
        let Some(path) = find(OTF) else {
            eprintln!("nessun OTF statico di sistema: test saltato");
            return;
        };
        let ttf = scratch("conv.ttf");
        convert_blocking(&path, 0, &ttf, "ttf").expect("OTF → TTF");
        let (a, b) = (load(&path, 0).unwrap(), load(&ttf, 0).unwrap());
        same_glyphs(&face(&a.sfnt).unwrap(), &face(&b.sfnt).unwrap(), 2);
    }

    #[test]
    fn refuses_same_format_and_variable() {
        if let Some(path) = find(TTF) {
            let e = convert_blocking(&path, 0, &scratch("x.ttf"), "ttf").unwrap_err();
            assert!(e.contains("già"), "{e}");
        }
        if Path::new("/usr/share/fonts/cantarell/Cantarell-VF.otf").is_file() {
            let e = convert_blocking(
                "/usr/share/fonts/cantarell/Cantarell-VF.otf",
                0,
                &scratch("x.ttf"),
                "ttf",
            )
            .unwrap_err();
            assert!(e.contains("variabile"), "{e}");
        }
    }

    #[test]
    fn writes_an_afm() {
        let Some(path) = find(TTF) else {
            eprintln!("nessun TTF di sistema: test saltato");
            return;
        };
        let dest = scratch("m.afm");
        let r = afm_blocking(&path, 0, &dest).expect("AFM");
        let text = std::fs::read_to_string(&dest).unwrap();
        assert!(text.starts_with("StartFontMetrics 4.1\n"));
        assert!(text.trim_end().ends_with("EndFontMetrics"));
        assert!(text.contains("C 65 ; WX "), "manca la A");
        assert!(text.contains(" N space ;") || text.contains(" N uni0020 ;"));
        assert!(r.encoded >= 95, "codificati: {}", r.encoded);
        let declared: usize = text
            .lines()
            .find_map(|l| l.strip_prefix("StartCharMetrics "))
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(
            declared,
            text.lines().filter(|l| l.starts_with("C ")).count()
        );
        eprintln!("{} coppie di kerning", r.kern_pairs);
    }

    #[test]
    fn cubic_split_respects_tolerance() {
        let (p0, p1, p2, p3) = (
            P::new(0.0, 0.0),
            P::new(0.0, 800.0),
            P::new(900.0, 800.0),
            P::new(1000.0, -300.0),
        );
        let qs = cubic_to_quads(p0, p1, p2, p3, 1.0);
        assert!(qs.len() > 1);
        // Sample each quadratic piece against the cubic on the same interval.
        let n = qs.len();
        let mut start = p0;
        for (i, (q, e)) in qs.iter().enumerate() {
            for k in 1..10 {
                let s = k as f64 / 10.0;
                let quad = start.lerp(*q, s).lerp(q.lerp(*e, s), s);
                let cub = cubic_at(p0, p1, p2, p3, (i as f64 + s) / n as f64);
                let d = ((quad.x - cub.x).powi(2) + (quad.y - cub.y).powi(2)).sqrt();
                assert!(d <= 1.0 + 1e-6, "errore {d} al pezzo {i}");
            }
            start = *e;
        }
    }

    #[test]
    fn dict_real_encoding() {
        let mut v = Vec::new();
        dict_real(&mut v, 0.00048828125);
        assert_eq!(v, vec![30, 0x0a, 0x00, 0x04, 0x88, 0x28, 0x12, 0x5f]);
        let mut v = Vec::new();
        dict_real(&mut v, -12.5);
        assert_eq!(v, vec![30, 0xe1, 0x2a, 0x5f]);
    }
}
