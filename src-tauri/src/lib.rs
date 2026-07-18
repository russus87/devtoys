//! DevToys backend — the handful of tools that are better done natively:
//! cryptographic hashes, HMAC and UUID generation. Everything else (text
//! transforms, formatters, converters) lives in the frontend and never
//! touches the backend, keeping the whole app fully offline.

mod apitest;
mod pdf;
mod update;

use base64::Engine;
use hmac::{Hmac, Mac};
use md5::Md5;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use sha3::{Sha3_256, Sha3_512};
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tauri_plugin_notification::NotificationExt;

#[cfg(not(target_os = "linux"))]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
#[cfg(target_os = "linux")]
use tauri::menu::Submenu;

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
// tray icon + window lifecycle
// ---------------------------------------------------------------------------

fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

// --- quick-launch popup window ---------------------------------------------

/// Show the frameless quick-launch popup near the top-right (under the tray
/// area), focused, and tell its webview to reset + refocus its search box.
fn show_quicklaunch(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("quicklaunch") {
        position_top_right(&w);
        let _ = w.show();
        let _ = w.set_focus();
        let _ = w.emit("quick-shown", ());
    }
}

/// Anchor the popup to the top-right of the active monitor, just under the bar
/// (where the tray usually sits). The app can't know the tray icon's exact
/// coordinates on Linux — the compositor owns it — so this is the closest
/// sensible placement. Tiling compositors (niri) may still override it.
fn position_top_right(w: &tauri::WebviewWindow) {
    let monitor = match w.current_monitor() {
        Ok(Some(m)) => Some(m),
        _ => w.primary_monitor().ok().flatten(),
    };
    if let (Some(m), Ok(size)) = (monitor, w.outer_size()) {
        let scale = m.scale_factor();
        let mpos = m.position();
        let msize = m.size();
        let margin = (12.0 * scale) as i32;
        let top = (44.0 * scale) as i32; // clear a typical top bar
        let x = mpos.x + msize.width as i32 - size.width as i32 - margin;
        let y = mpos.y + top;
        let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    }
}

fn hide_quicklaunch(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("quicklaunch") {
        let _ = w.hide();
    }
}

fn toggle_quicklaunch(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("quicklaunch") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_quicklaunch(app);
        }
    }
}

/// Open a tool in the main window (from the popup), then dismiss the popup.
#[tauri::command]
fn quick_open(app: AppHandle, id: String) {
    hide_quicklaunch(&app);
    show_window(&app);
    let _ = app.emit_to("main", "open-tool", id);
}

#[tauri::command]
fn quick_hide(app: AppHandle) {
    hide_quicklaunch(&app);
}

// --- tray -------------------------------------------------------------------

/// The tray handle, so its menu can be rebuilt once the frontend sends the tool
/// list (used on Linux, where the native menu is the reliable click surface).
#[derive(Default)]
pub struct TrayHandle(pub Mutex<Option<TrayIcon>>);

/// Tool metadata mirrored from the frontend for the Linux tray menu.
#[derive(Debug, Deserialize)]
pub struct TrayTool {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct TrayCategory {
    label: String,
    emoji: String,
    tools: Vec<TrayTool>,
}

/// Build the tray menu. On Linux it lists every tool by category (the native
/// menu is the reliable left-click surface there); on Windows/macOS it stays
/// minimal because left-click opens the styled quick-launch popup instead.
fn build_menu(app: &AppHandle, cats: &[TrayCategory]) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(app, "quick", "⚡ Pannello rapido", true, None::<&str>)?)?;

    #[cfg(target_os = "linux")]
    {
        if !cats.is_empty() {
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            for cat in cats {
                if cat.tools.is_empty() {
                    continue;
                }
                let items: Vec<MenuItem<tauri::Wry>> = cat
                    .tools
                    .iter()
                    .map(|t| MenuItem::with_id(app, format!("tool:{}", t.id), &t.name, true, None::<&str>))
                    .collect::<tauri::Result<_>>()?;
                let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
                    items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<_>).collect();
                let sub = Submenu::with_items(app, &format!("{} {}", cat.emoji, cat.label), true, &refs)?;
                menu.append(&sub)?;
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = cats;

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "open", "Apri DevToys", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "check_update",
        "Controlla aggiornamenti",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Esci", true, None::<&str>)?)?;
    Ok(menu)
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "quick" => show_quicklaunch(app),
        "open" => show_window(app),
        "quit" => app.exit(0),
        "check_update" => {
            show_window(app);
            let _ = app.emit_to("main", "menu-check-update", ());
        }
        other => {
            if let Some(tool) = other.strip_prefix("tool:") {
                show_window(app);
                let _ = app.emit_to("main", "open-tool", tool.to_string());
            }
        }
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app, &[])?;
    let builder = TrayIconBuilder::with_id("devtoys-tray")
        .icon(app.default_window_icon().cloned().unwrap())
        .tooltip("DevToys")
        .menu(&menu)
        .on_menu_event(|app, event| on_menu(app, event.id.as_ref()));

    // Linux: let the SNI host show the menu on left-click — the dependable
    // surface. Windows/macOS: left-click summons the styled popup instead.
    #[cfg(target_os = "linux")]
    let builder = builder.show_menu_on_left_click(true);
    #[cfg(not(target_os = "linux"))]
    let builder = builder.show_menu_on_left_click(false).on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            toggle_quicklaunch(tray.app_handle());
        }
    });

    let tray = builder.build(app)?;
    if let Some(state) = app.try_state::<TrayHandle>() {
        *state.0.lock().unwrap() = Some(tray);
    }
    Ok(())
}

/// Rebuild the tray menu from the frontend's tool list (Linux only; elsewhere
/// the popup carries the tools and the menu stays minimal).
#[tauri::command]
fn set_tray_menu(app: AppHandle, tray: State<TrayHandle>, categories: Vec<TrayCategory>) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let menu = build_menu(&app, &categories).map_err(|e| e.to_string())?;
        let guard = tray.0.lock().map_err(|_| "tray bloccato")?;
        if let Some(t) = guard.as_ref() {
            t.set_menu(Some(menu)).map_err(|e| e.to_string())?;
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = (&app, &tray, &categories);
    Ok(())
}

/// Background loop: check for a new release every 6 hours and notify once.
async fn periodic_update_check(app: AppHandle) {
    let mut notified: Option<String> = None;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(6 * 60 * 60)).await;
        if let Ok(info) = update::update_check().await {
            if info.available {
                if let Some(latest) = &info.latest {
                    let _ = app.emit("update-available", info.clone());
                    if notified.as_deref() != Some(latest.as_str()) {
                        notified = Some(latest.clone());
                        let _ = app
                            .notification()
                            .builder()
                            .title("DevToys")
                            .body(format!("Nuova versione disponibile: v{latest}"))
                            .show();
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
                // `devtoys --quick` (or `--panel`) from a second launch tells the
                // running instance to pop the quick-launch panel — handy to bind
                // to a tray/bar left-click on Linux, where the app can't intercept
                // the click itself. Otherwise just surface the main window.
                if argv.iter().any(|a| a == "--quick" || a == "--panel") {
                    show_quicklaunch(app);
                } else {
                    show_window(app);
                }
            }))
            .plugin(tauri_plugin_global_shortcut::Builder::new().build());
    }

    builder
        .setup(|app| {
            let handle = app.handle().clone();

            // Load the bundled Pdfium native library so the PDF tools can work.
            let mut cands: Vec<std::path::PathBuf> = Vec::new();
            if let Ok(res) = app.path().resource_dir() {
                cands.push(res.join("pdfium"));
            }
            cands.push(std::path::PathBuf::from("pdfium"));
            cands.push(std::path::PathBuf::from("src-tauri/pdfium"));
            // Arch package ships it in a private dir to avoid clashing with
            // other packages that own /usr/lib/libpdfium.so.
            cands.push(std::path::PathBuf::from("/usr/lib/devtoys"));
            let _ = pdfa_core::pdfium::inizializza(&cands);

            // System tray: left-click / menu open the quick-launch popup.
            build_tray(&handle)?;

            // Closing the main window hides it to the tray instead of quitting;
            // "Esci" in the tray menu is the way to fully quit.
            if let Some(w) = app.get_webview_window("main") {
                let wc = w.clone();
                w.on_window_event(move |e| {
                    if let WindowEvent::CloseRequested { api, .. } = e {
                        api.prevent_close();
                        let _ = wc.hide();
                    }
                });
            }

            // The quick-launch popup dismisses itself when it loses focus or is
            // closed, Spotlight-style.
            if let Some(w) = app.get_webview_window("quicklaunch") {
                let wc = w.clone();
                w.on_window_event(move |e| match e {
                    WindowEvent::Focused(false) => {
                        let _ = wc.hide();
                    }
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = wc.hide();
                    }
                    _ => {}
                });
            }

            // Global shortcut to summon the popup — the reliable trigger on
            // Linux, where the tray left-click event isn't delivered.
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
                if let Ok(shortcut) = "CommandOrControl+Shift+Space".parse::<Shortcut>() {
                    let h = handle.clone();
                    let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _sc, event| {
                        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            toggle_quicklaunch(&h);
                        }
                    });
                }
            }

            tauri::async_runtime::spawn(periodic_update_check(handle));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .manage(apitest::TestState::default())
        .manage(TrayHandle::default())
        .invoke_handler(tauri::generate_handler![
            hash_text,
            hmac_text,
            gen_uuids,
            base64_encode_bytes,
            quick_open,
            quick_hide,
            set_tray_menu,
            update::update_check,
            update::update_install,
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
