//! PDF tools — thin Tauri command wrappers over the `pdfa-core` crate.
//!
//! Every command runs the (blocking/CPU-bound) `pdfa-core` call inside
//! `tokio::task::spawn_blocking`, converts path `String`s to `Path`/`PathBuf`,
//! and maps `pdfa_core::Errore` (via `Display`) to `String` for the frontend.

use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Return shapes (camelCase on the wire)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfInfo {
    pub pages: u32,
    pub title: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeReport {
    pub before: u64,
    pub after: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SanitizeResult {
    pub removed: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub lang: Option<String>,
    pub pages: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UaCheck {
    pub id: String,
    pub description: String,
    pub passed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UaReport {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub checks: Vec<UaCheck>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfaCheck {
    pub label: String,
    pub ok: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfaReport {
    pub conformant: bool,
    pub note: String,
    pub checks: Vec<PdfaCheck>,
}

#[derive(Serialize)]
pub struct DiffChange {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDiff {
    pub added: u32,
    pub removed: u32,
    pub changes: Vec<DiffChange>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDiff {
    pub diff_base64: String,
    pub changed_ratio: f64,
    pub width_px: u32,
    pub height_px: u32,
    pub changed_pixels: u64,
    pub total_pixels: u64,
}

// ---------------------------------------------------------------------------
// Basics
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn pdf_info(path: String) -> Result<PdfInfo, String> {
    tokio::task::spawn_blocking(move || -> Result<PdfInfo, String> {
        let p = Path::new(&path);
        let info = pdfa_core::apri(p).map_err(|e| e.to_string())?;
        Ok(PdfInfo { pages: info.pagine as u32, title: info.titolo })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_render_page(path: String, page: u32, width: u32) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let p = Path::new(&path);
        let png = pdfa_core::render_pagina(p, page as i32, width as i32).map_err(|e| e.to_string())?;
        Ok(B64.encode(&png))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_to_text(path: String, format: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let p = Path::new(&path);
        let formato = pdfa_core::estrazione::Formato::da_str(&format);
        pdfa_core::estrazione::esporta(p, formato).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_to_images(path: String, width: u32, jpeg: bool) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || -> Result<Vec<String>, String> {
        let p = Path::new(&path);
        let pagine = pdfa_core::conversione::pdf_a_immagini(p, width as i32, jpeg).map_err(|e| e.to_string())?;
        Ok(pagine.iter().map(|b| B64.encode(b)).collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Page operations
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn pdf_merge(paths: Vec<String>, dest: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let origini: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        pdfa_core::pagine::unisci(&origini, Path::new(&dest)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_extract_pages(path: String, dest: String, pages: Vec<u32>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        pdfa_core::pagine::estrai(Path::new(&path), Path::new(&dest), &pages).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_delete_pages(path: String, dest: String, pages: Vec<u32>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        pdfa_core::pagine::elimina(Path::new(&path), Path::new(&dest), &pages).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// `pages` empty means "all pages" (per contract); resolve that against the
/// real page count before calling `pagine::ruota`, which otherwise only
/// rotates the pages explicitly listed.
#[tauri::command]
pub async fn pdf_rotate(path: String, dest: String, pages: Vec<u32>, degrees: i64) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let p = Path::new(&path);
        let elenco = if pages.is_empty() {
            let info = pdfa_core::apri(p).map_err(|e| e.to_string())?;
            (1..=info.pagine as u32).collect::<Vec<u32>>()
        } else {
            pages
        };
        pdfa_core::pagine::ruota(p, Path::new(&dest), &elenco, degrees).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_reorder(path: String, dest: String, order: Vec<u32>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        pdfa_core::pagine::riordina(Path::new(&path), Path::new(&dest), &order).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Optimization / sanitization
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn pdf_optimize(path: String, dest: String) -> Result<SizeReport, String> {
    tokio::task::spawn_blocking(move || -> Result<SizeReport, String> {
        let (before, after) =
            pdfa_core::ottimizzazione::ottimizza(Path::new(&path), Path::new(&dest)).map_err(|e| e.to_string())?;
        Ok(SizeReport { before, after })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_compress_images(
    path: String,
    dest: String,
    max_width: u32,
    quality: u8,
) -> Result<SizeReport, String> {
    tokio::task::spawn_blocking(move || -> Result<SizeReport, String> {
        let (before, after) =
            pdfa_core::ottimizzazione::comprimi_immagini(Path::new(&path), Path::new(&dest), max_width, quality)
                .map_err(|e| e.to_string())?;
        Ok(SizeReport { before, after })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_sanitize(path: String, dest: String) -> Result<SanitizeResult, String> {
    tokio::task::spawn_blocking(move || -> Result<SanitizeResult, String> {
        let opzioni = pdfa_core::sanitizza::Opzioni::default();
        let esito = pdfa_core::sanitizza::pulisci(Path::new(&path), Path::new(&dest), &opzioni)
            .map_err(|e| e.to_string())?;
        let mut removed = Vec::new();
        if esito.javascript {
            removed.push("JavaScript e azioni automatiche".to_string());
        }
        if esito.metadati {
            removed.push("Metadati (Info + XMP)".to_string());
        }
        if esito.allegati {
            removed.push("Allegati incorporati".to_string());
        }
        Ok(SanitizeResult { removed })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Metadata / structure
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn pdf_metadata(path: String) -> Result<PdfMetadata, String> {
    tokio::task::spawn_blocking(move || -> Result<PdfMetadata, String> {
        let p = Path::new(&path);
        let m = pdfa_core::metadati::leggi(p).map_err(|e| e.to_string())?;
        let info = pdfa_core::apri(p).map_err(|e| e.to_string())?;
        Ok(PdfMetadata {
            title: m.titolo,
            author: m.autore,
            subject: m.soggetto,
            keywords: m.parole_chiave,
            creator: m.creatore,
            producer: m.produttore,
            lang: m.lang,
            pages: info.pagine as u32,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_structure(path: String, format: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let p = Path::new(&path);
        match format.as_str() {
            "xml" => pdfa_core::export::esporta_xml(p).map_err(|e| e.to_string()),
            _ => pdfa_core::export::esporta_json(p).map_err(|e| e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn pdf_validate_ua(path: String) -> Result<UaReport, String> {
    tokio::task::spawn_blocking(move || -> Result<UaReport, String> {
        let report = pdfa_core::matterhorn::analizza(Path::new(&path)).map_err(|e| e.to_string())?;
        let checks: Vec<UaCheck> = report
            .voci
            .iter()
            .map(|v| {
                let passed = matches!(v.stato, pdfa_core::matterhorn::StatoMh::Superato);
                let description = if v.dettaglio.trim().is_empty() {
                    v.titolo.clone()
                } else {
                    format!("{} — {}", v.titolo, v.dettaglio)
                };
                UaCheck { id: v.checkpoint.clone(), description, passed }
            })
            .collect();
        let total = checks.len() as u32;
        let passed = checks.iter().filter(|c| c.passed).count() as u32;
        let failed = total - passed;
        Ok(UaReport { total, passed, failed, checks })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Best-effort, non-certified PDF/A check: `pdfa-core` has no PDF/A validator,
/// so this inspects a handful of heuristics (tag tree, ToUnicode coverage,
/// declared language, a PDF/A XMP identifier, absence of encryption) directly
/// from the structural info plus a raw byte scan of the file. It is NOT a
/// substitute for a real validator such as veraPDF.
#[tauri::command]
pub async fn pdf_check_pdfa(path: String) -> Result<PdfaReport, String> {
    tokio::task::spawn_blocking(move || -> Result<PdfaReport, String> {
        let p = Path::new(&path);
        let info = pdfa_core::struttura::analizza(p).map_err(|e| e.to_string())?;
        let bytes = std::fs::read(p).map_err(|e| e.to_string())?;

        let contains = |needle: &[u8]| -> bool {
            !needle.is_empty() && needle.len() <= bytes.len() && bytes.windows(needle.len()).any(|w| w == needle)
        };

        let tagged = info.taggato && info.ha_struct_tree;
        let fonts_mapped = info.font_totali == 0 || info.font_senza_tounicode == 0;
        let lang_declared = info.lang.is_some();
        let pdfa_id_present = contains(b"pdfaid") || contains(b"GTS_PDFA1");
        let not_encrypted = !contains(b"/Encrypt");

        let checks = vec![
            PdfaCheck { label: "Documento taggato (structure tree)".to_string(), ok: tagged },
            PdfaCheck { label: "Font con mappatura ToUnicode".to_string(), ok: fonts_mapped },
            PdfaCheck { label: "Lingua del documento dichiarata".to_string(), ok: lang_declared },
            PdfaCheck { label: "Identificatore PDF/A nei metadati XMP".to_string(), ok: pdfa_id_present },
            PdfaCheck { label: "Nessuna cifratura rilevata".to_string(), ok: not_encrypted },
        ];
        let conformant = checks.iter().all(|c| c.ok);

        Ok(PdfaReport {
            conformant,
            note: "Controllo di base, non certificato: verifica solo alcuni indicatori euristici \
                   (tag, font, lingua, marcatore XMP, cifratura) letti dal file. Non è una validazione \
                   PDF/A completa: per una verifica certificata usa un validatore dedicato come veraPDF."
                .to_string(),
            checks,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Compare
// ---------------------------------------------------------------------------

/// Text comparison. Lines come from `pdftext::document_lines`, which rebuilds
/// them from glyph geometry rather than trusting pdfium's own line breaks —
/// those depend on the nominal font size and diverge between producers, so two
/// identical documents written by, say, Aspose and PDFBox would otherwise
/// report hundreds of phantom differences on table rows. See `pdftext`.
#[tauri::command]
pub async fn pdf_compare_text(a: String, b: String) -> Result<TextDiff, String> {
    tokio::task::spawn_blocking(move || -> Result<TextDiff, String> {
        let la = crate::pdftext::document_lines(Path::new(&a))?;
        let lb = crate::pdftext::document_lines(Path::new(&b))?;

        let (ra, rb): (Vec<&str>, Vec<&str>) =
            (la.iter().map(String::as_str).collect(), lb.iter().map(String::as_str).collect());
        let diff = similar::TextDiff::from_slices(&ra, &rb);
        let (mut added, mut removed) = (0u32, 0u32);
        let mut changes = Vec::new();
        for change in diff.iter_all_changes() {
            let kind = match change.tag() {
                similar::ChangeTag::Insert => {
                    added += 1;
                    "add"
                }
                similar::ChangeTag::Delete => {
                    removed += 1;
                    "del"
                }
                similar::ChangeTag::Equal => "eq",
            };
            changes.push(DiffChange { kind: kind.to_string(), text: change.value().to_string() });
        }
        Ok(TextDiff { added, removed, changes })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Render width in pixels for `page` of `path` at `dpi`, from the page's own
/// width in PDF points (1 pt = 1/72"). Deliberately *not* a fixed pixel width:
/// rendering both files at the same DPI keeps them at true physical scale, so a
/// page-size mismatch (A4 vs Letter) shows up as a difference instead of being
/// silently normalised away.
fn width_for_dpi(path: &Path, page: u32, dpi: u32) -> Result<u32, String> {
    let (w_pt, _) = pdfa_core::dimensioni_pagina(path, page as i32).map_err(|e| e.to_string())?;
    let px = (w_pt as f64 / 72.0 * dpi as f64).round();
    Ok((px as u32).clamp(100, 8000))
}

/// True when two pixels differ by more than `tol` on any channel.
fn channels_differ(pa: [u8; 4], pb: [u8; 4], tol: u8) -> bool {
    (0..4).any(|i| pa[i].abs_diff(pb[i]) > tol)
}

/// True when no pixel within `radius` of (x, y) in `other` matches `px`.
///
/// A plain intensity tolerance is not enough on its own: two producers emitting
/// byte-identical geometry still rasterise it differently. A 0.5 pt rule written
/// as an `re` rectangle gets snapped to two full black rows, while the same rule
/// written as a generic path is antialiased across two rows — a delta of up to
/// 245/255 on content that is provably identical. Allowing the match to come
/// from a neighbouring pixel absorbs that, plus sub-pixel glyph shifts and
/// whole-row baseline shifts, while genuinely added or removed content still has
/// no match anywhere nearby.
fn unmatched(px: [u8; 4], other: &image::RgbaImage, x: u32, y: u32, radius: u32, tol: u8) -> bool {
    let (w, h) = (other.width(), other.height());
    let x0 = x.saturating_sub(radius);
    let y0 = y.saturating_sub(radius);
    for ny in y0..=(y + radius).min(h.saturating_sub(1)) {
        for nx in x0..=(x + radius).min(w.saturating_sub(1)) {
            if !channels_differ(px, other.get_pixel(nx, ny).0, tol) {
                return false;
            }
        }
    }
    true
}

#[tauri::command]
pub async fn pdf_compare_image(
    a: String,
    b: String,
    page: u32,
    dpi: u32,
    tolerance: u8,
    radius: u32,
) -> Result<ImageDiff, String> {
    tokio::task::spawn_blocking(move || -> Result<ImageDiff, String> {
        use image::{Rgba, RgbaImage};

        let dpi = dpi.clamp(24, 600);
        let (pa, pb) = (Path::new(&a), Path::new(&b));
        let ia = pdfa_core::documento::render_immagine(pa, page as i32, width_for_dpi(pa, page, dpi)? as i32)
            .map_err(|e| e.to_string())?
            .to_rgba8();
        let ib = pdfa_core::documento::render_immagine(pb, page as i32, width_for_dpi(pb, page, dpi)? as i32)
            .map_err(|e| e.to_string())?
            .to_rgba8();

        // Union of the two canvases: whatever only one of them covers counts as
        // a difference rather than being cropped out of the comparison.
        let w = ia.width().max(ib.width());
        let h = ia.height().max(ib.height());
        let mut diff = RgbaImage::new(w, h);
        let mut changed: u64 = 0;

        for y in 0..h {
            for x in 0..w {
                let inside_a = x < ia.width() && y < ia.height();
                let inside_b = x < ib.width() && y < ib.height();
                let va = inside_a.then(|| ia.get_pixel(x, y).0);
                let vb = inside_b.then(|| ib.get_pixel(x, y).0);
                let differs = match (va, vb) {
                    // Fast path: most pixels are byte-identical, so only fall back
                    // to the neighbourhood scan where the two actually disagree.
                    (Some(ca), Some(cb)) if !channels_differ(ca, cb, tolerance) => false,
                    // Symmetric: matching A against B alone would hide removals.
                    (Some(ca), Some(cb)) => {
                        radius == 0
                            || unmatched(ca, &ib, x, y, radius, tolerance)
                            || unmatched(cb, &ia, x, y, radius, tolerance)
                    }
                    _ => true,
                };
                if differs {
                    changed += 1;
                    diff.put_pixel(x, y, Rgba([220, 40, 40, 255]));
                } else {
                    // Original, lightened, so the red of the differences stands out.
                    let base = va.unwrap_or([255, 255, 255, 255]);
                    diff.put_pixel(x, y, Rgba([128 + base[0] / 2, 128 + base[1] / 2, 128 + base[2] / 2, 255]));
                }
            }
        }

        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(diff)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;

        let total = w as u64 * h as u64;
        Ok(ImageDiff {
            diff_base64: B64.encode(&buf),
            changed_ratio: if total == 0 { 0.0 } else { changed as f64 / total as f64 },
            width_px: w,
            height_px: h,
            changed_pixels: changed,
            total_pixels: total,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// Watermark / office / images
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn pdf_watermark(path: String, dest: String, text: String, opacity: f64) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        use pdfa_core::sovrapposizione::{Colore, Filigrana};
        let opacita = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let filigrana = Filigrana::Testo {
            testo: text,
            dim_pt: 60.0,
            colore: Colore { r: 128, g: 128, b: 128 },
            opacita,
            rotazione: 45.0,
        };
        pdfa_core::sovrapposizione::applica(Path::new(&path), Path::new(&dest), &[], Some(&filigrana))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_to_docx(path: String, dest: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        pdfa_core::office::pdf_a_docx(Path::new(&path), Path::new(&dest)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn images_to_pdf(paths: Vec<String>, dest: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut immagini = Vec::with_capacity(paths.len());
        for p in &paths {
            let bytes = std::fs::read(p).map_err(|e| format!("lettura {p}: {e}"))?;
            immagini.push(bytes);
        }
        pdfa_core::conversione::immagini_a_pdf(&immagini, Path::new(&dest)).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
