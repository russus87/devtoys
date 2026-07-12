//! DevToys backend — the handful of tools that are better done natively:
//! cryptographic hashes, HMAC and UUID generation. Everything else (text
//! transforms, formatters, converters) lives in the frontend and never
//! touches the backend, keeping the whole app fully offline.

mod apitest;
mod pdf;

use base64::Engine;
use hmac::{Hmac, Mac};
use md5::Md5;
use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use sha3::{Sha3_256, Sha3_512};

// ---------------------------------------------------------------------------
// hashing
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Hashes {
    md5: String,
    sha1: String,
    sha256: String,
    sha384: String,
    sha512: String,
    sha3_256: String,
    sha3_512: String,
    crc32: String,
}

fn hex_lower(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Compute a bundle of common digests for the given text (UTF-8 bytes).
#[tauri::command]
fn hash_text(input: String) -> Hashes {
    let data = input.as_bytes();
    Hashes {
        md5: hex_lower(&Md5::digest(data)),
        sha1: hex_lower(&Sha1::digest(data)),
        sha256: hex_lower(&Sha256::digest(data)),
        sha384: hex_lower(&Sha384::digest(data)),
        sha512: hex_lower(&Sha512::digest(data)),
        sha3_256: hex_lower(&Sha3_256::digest(data)),
        sha3_512: hex_lower(&Sha3_512::digest(data)),
        crc32: format!("{:08x}", crc32fast::hash(data)),
    }
}

// ---------------------------------------------------------------------------
// HMAC
// ---------------------------------------------------------------------------

fn hmac_hex<M: Mac + hmac::digest::KeyInit>(key: &[u8], data: &[u8]) -> String {
    let mut mac = <M as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

/// Compute an HMAC of `message` with `key` under the named algorithm.
/// `algo` is one of: sha1, sha256, sha384, sha512.
#[tauri::command]
fn hmac_text(algo: String, key: String, message: String) -> Result<String, String> {
    let k = key.as_bytes();
    let m = message.as_bytes();
    let out = match algo.as_str() {
        "sha1" => hmac_hex::<Hmac<Sha1>>(k, m),
        "sha256" => hmac_hex::<Hmac<Sha256>>(k, m),
        "sha384" => hmac_hex::<Hmac<Sha384>>(k, m),
        "sha512" => hmac_hex::<Hmac<Sha512>>(k, m),
        other => return Err(format!("algoritmo HMAC sconosciuto: {other}")),
    };
    Ok(out)
}

// ---------------------------------------------------------------------------
// UUID
// ---------------------------------------------------------------------------

/// Generate `count` UUIDs of the given version ("v4" random, "v7" time-ordered).
#[tauri::command]
fn gen_uuids(version: String, count: u32) -> Result<Vec<String>, String> {
    let n = count.clamp(1, 1000);
    let mut out = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let id = match version.as_str() {
            "v4" => uuid::Uuid::new_v4(),
            "v7" => uuid::Uuid::now_v7(),
            other => return Err(format!("versione UUID non supportata: {other}")),
        };
        out.push(id.to_string());
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Base64 of raw bytes (used by the image/file tools where JS btoa struggles)
// ---------------------------------------------------------------------------

/// Standard Base64-encode arbitrary bytes.
#[tauri::command]
fn base64_encode_bytes(bytes: Vec<u8>) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

// ---------------------------------------------------------------------------
// entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(all(desktop, not(debug_assertions)))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
        }));
    }
    #[cfg(all(desktop, debug_assertions))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}));
    }

    builder
        .setup(|app| {
            // Load the bundled Pdfium native library so the PDF tools can work.
            use tauri::Manager;
            let mut cands: Vec<std::path::PathBuf> = Vec::new();
            if let Ok(res) = app.path().resource_dir() {
                cands.push(res.join("pdfium"));
            }
            cands.push(std::path::PathBuf::from("pdfium"));
            cands.push(std::path::PathBuf::from("src-tauri/pdfium"));
            let _ = pdfa_core::pdfium::inizializza(&cands);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(apitest::TestState::default())
        .invoke_handler(tauri::generate_handler![
            hash_text,
            hmac_text,
            gen_uuids,
            base64_encode_bytes,
            apitest::load_start,
            apitest::load_stop,
            apitest::mock_start,
            apitest::mock_stop,
            apitest::mock_status,
            apitest::proxy_start,
            apitest::proxy_stop,
            apitest::proxy_status,
            apitest::smoke_run,
            pdf::pdf_info,
            pdf::pdf_render_page,
            pdf::pdf_to_text,
            pdf::pdf_to_images,
            pdf::pdf_merge,
            pdf::pdf_extract_pages,
            pdf::pdf_delete_pages,
            pdf::pdf_rotate,
            pdf::pdf_reorder,
            pdf::pdf_optimize,
            pdf::pdf_compress_images,
            pdf::pdf_sanitize,
            pdf::pdf_metadata,
            pdf::pdf_structure,
            pdf::pdf_validate_ua,
            pdf::pdf_check_pdfa,
            pdf::pdf_compare_text,
            pdf::pdf_compare_image,
            pdf::pdf_watermark,
            pdf::pdf_to_docx,
            pdf::images_to_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevToys");
}
