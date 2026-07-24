//! Geometry-based text line reconstruction for PDF comparison.
//!
//! Pdfium does not read lines out of a PDF: it rebuilds them, deciding where to
//! emit `\r\n` and where to synthesise a space by comparing character positions
//! against thresholds derived from the *nominal* font size — the number in the
//! `Tf` operator, before the CTM is applied.
//!
//! That number is not stable across producers. Two visually identical files can
//! carry `/F1 8 Tf` with a 20x text matrix, or `/F1 160 Tf` with an identity
//! matrix under a `0.05 0 0 0.05 0 0 cm` page CTM. Pdfium reports 8 for the
//! first and 160 for the second, so its vertical threshold is 20x too large on
//! the latter and real line breaks go undetected — table rows collapse into one
//! another and a text diff of the two files reports differences that are not
//! there.
//!
//! So we ignore pdfium's `\r\n` and synthesised spaces and rebuild the lines
//! from the character geometry, which *is* consistent: the loose glyph boxes and
//! baselines of the two files above agree to within 0.01pt. Characters are
//! grouped into lines by baseline, ordered by horizontal position, and word
//! breaks are inserted where the gap between glyphs exceeds a threshold scaled
//! to the page's median glyph height rather than to a font size.

use std::path::Path;

use pdfium_render::prelude::PdfPage;

/// Thresholds, as a fraction of the page's median glyph height. Measured over a
/// corpus of Italian banking/administrative PDFs (see the notes on each one);
/// the gap distribution they cut through is strongly bimodal, so the exact
/// values matter far less than staying inside the empty band between the modes.
///
/// Maximum baseline difference for two glyphs to belong to the same line. Text
/// drawn as a second layer (an OCR or alternate-font copy over the visible one)
/// typically sits ~0.2 off, so staying well below that keeps the layers apart
/// instead of interleaving them character by character.
const LINE_TOL: f32 = 0.15;
/// Minimum horizontal gap that counts as a word break. Intra-word gaps measure
/// ~0.00 and inter-word gaps start at ~0.08, so this sits in an empty band.
const SPACE_GAP: f32 = 0.05;
/// Two identical glyphs closer than this are the same glyph drawn twice (fake
/// bold, overprint). Kept small: the advance of a narrow letter such as `l` can
/// be as little as 0.11, and eating one of those would corrupt real words.
const DUP_X: f32 = 0.02;

/// One positioned glyph on a page.
struct Glyph {
    ch: char,
    left: f32,
    right: f32,
    baseline: f32,
    height: f32,
}

/// Extracts the text of `path` as reconstructed lines, in page order.
pub fn document_lines(path: &Path) -> Result<Vec<String>, String> {
    let pdfium = pdfa_core::pdfium::istanza().map_err(|e| e.to_string())?;
    let doc = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| e.to_string())?;

    let mut lines = Vec::new();
    for page in doc.pages().iter() {
        lines.extend(page_lines(&page));
    }
    Ok(lines)
}

/// Rebuilds the lines of a single page from its glyph positions.
fn page_lines(page: &PdfPage) -> Vec<String> {
    let text = match page.text() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let mut glyphs: Vec<Glyph> = Vec::new();
    for c in text.chars().iter() {
        // Drop what pdfium synthesised: those are exactly the decisions we are
        // here to redo. Real spaces in the content stream are kept.
        if c.is_generated().unwrap_or(false) {
            continue;
        }
        let ch = match c.unicode_char() {
            Some(ch) if ch != '\r' && ch != '\n' && ch != '\0' => ch,
            _ => continue,
        };
        let (bounds, baseline) = match (c.loose_bounds(), c.origin_y()) {
            (Ok(b), Ok(y)) => (b, y),
            _ => continue,
        };
        glyphs.push(Glyph {
            ch,
            left: bounds.left().value,
            right: bounds.right().value,
            baseline: baseline.value,
            height: bounds.top().value - bounds.bottom().value,
        });
    }
    if glyphs.is_empty() {
        return Vec::new();
    }

    let height = median_height(&glyphs);
    let (line_tol, space_gap, dup_x) = (height * LINE_TOL, height * SPACE_GAP, height * DUP_X);

    // Group by baseline, top of page first, then left to right within a line.
    glyphs.sort_by(|a, b| {
        b.baseline
            .partial_cmp(&a.baseline)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.left.partial_cmp(&b.left).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut rows: Vec<Vec<&Glyph>> = Vec::new();
    for g in &glyphs {
        match rows.last_mut() {
            Some(row) if (row[0].baseline - g.baseline).abs() <= line_tol => row.push(g),
            _ => rows.push(vec![g]),
        }
    }

    let mut out = Vec::new();
    for mut row in rows {
        row.sort_by(|a, b| a.left.partial_cmp(&b.left).unwrap_or(std::cmp::Ordering::Equal));

        let mut line = String::new();
        let mut prev: Option<&Glyph> = None;
        for g in row {
            if let Some(p) = prev {
                if g.ch == p.ch && (g.left - p.left).abs() <= dup_x {
                    continue; // same glyph drawn twice
                }
                if !g.ch.is_whitespace() && !p.ch.is_whitespace() && g.left - p.right > space_gap {
                    line.push(' ');
                }
            }
            line.push(g.ch);
            prev = Some(g);
        }

        let line = line.trim();
        if !line.is_empty() {
            out.push(line.to_string());
        }
    }
    out
}

/// Regression check for the producer-independence property this module exists
/// to provide: two PDFs that render the same text must reconstruct to the same
/// lines even when their producers express font size differently.
///
/// The fixtures are bank statements, so they are not committed. Point the test
/// at a pair of your own with:
///
/// ```text
/// DEVTOYS_PDF_PAIR=/path/a.pdf:/path/b.pdf cargo test -p devtoys same_text_same_lines -- --nocapture
/// ```
#[cfg(test)]
mod tests {
    #[test]
    fn same_text_same_lines() {
        let Ok(pair) = std::env::var("DEVTOYS_PDF_PAIR") else {
            eprintln!("DEVTOYS_PDF_PAIR non impostata: test saltato");
            return;
        };
        let (a, b) = pair.split_once(':').expect("formato: /path/a.pdf:/path/b.pdf");
        pdfa_core::pdfium::inizializza(&[]).expect("pdfium non disponibile");

        let la = super::document_lines(std::path::Path::new(a)).expect("estrazione A");
        let lb = super::document_lines(std::path::Path::new(b)).expect("estrazione B");
        eprintln!("righe: A={} B={}", la.len(), lb.len());

        let differing = la.iter().zip(&lb).filter(|(x, y)| x != y).count();
        for (i, (x, y)) in la.iter().zip(&lb).enumerate().filter(|(_, (x, y))| x != y).take(5) {
            eprintln!("riga {i}:\n  A: {x}\n  B: {y}");
        }
        assert_eq!(la.len(), lb.len(), "numero di righe diverso");
        assert_eq!(differing, 0, "{differing} righe ricostruite diversamente");
    }
}

/// Median glyph height on the page; every threshold is expressed relative to it.
/// Falls back to 1.0 so a degenerate page cannot make the thresholds collapse.
fn median_height(glyphs: &[Glyph]) -> f32 {
    let mut hs: Vec<f32> = glyphs.iter().map(|g| g.height).filter(|h| *h > 0.0).collect();
    if hs.is_empty() {
        return 1.0;
    }
    hs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let m = hs[hs.len() / 2];
    if m > 0.0 {
        m
    } else {
        1.0
    }
}
