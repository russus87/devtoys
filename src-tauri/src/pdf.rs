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

#[tauri::command]
pub async fn pdf_compare_text(a: String, b: String) -> Result<TextDiff, String> {
    tokio::task::spawn_blocking(move || -> Result<TextDiff, String> {
        let diff = pdfa_core::confronto::confronta_testo(Path::new(&a), Path::new(&b)).map_err(|e| e.to_string())?;
        let changes = diff
            .righe
            .iter()
            .map(|r| {
                let kind = match r.tipo.as_str() {
                    "aggiunta" => "add",
                    "rimossa" => "del",
                    _ => "eq",
                };
                DiffChange { kind: kind.to_string(), text: r.testo.clone() }
            })
            .collect();
        Ok(TextDiff { added: diff.aggiunte as u32, removed: diff.rimosse as u32, changes })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pdf_compare_image(a: String, b: String, page: u32, width: u32) -> Result<ImageDiff, String> {
    tokio::task::spawn_blocking(move || -> Result<ImageDiff, String> {
        let cfr = pdfa_core::confronto::confronta_immagine(Path::new(&a), Path::new(&b), page as i32, width as i32)
            .map_err(|e| e.to_string())?;
        Ok(ImageDiff { diff_base64: cfr.diff_png_base64, changed_ratio: cfr.percentuale / 100.0 })
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
