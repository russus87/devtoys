//! Self-updater, modeled on russus_launcher: check the project's GitHub
//! releases, and — when a newer version is published — download the platform
//! package and install it (Arch: `pkexec pacman -U`, Windows: elevated
//! msiexec/NSIS, macOS: mount the dmg and copy the .app). Fully optional and
//! offline-friendly: it only reaches the network when the user (or the periodic
//! check) explicitly asks.

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;

const OWNER: &str = "russus87";
const REPO: &str = "devtoys";
const UA: &str = "devtoys-updater";

// ---------------------------------------------------------------------------
// GitHub release model
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<Asset>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Result of an update check, returned to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: Option<String>,
    pub available: bool,
    pub notes: Option<String>,
    pub published_at: Option<String>,
    pub asset_name: Option<String>,
    pub asset_url: Option<String>,
    /// True when a package for this platform exists; false means "update
    /// exists but we can't install it here — open the release page instead".
    pub installable: bool,
}

/// Progress of a running download/install, streamed as `update-progress`.
#[derive(Debug, Clone, Serialize)]
struct Progress {
    phase: String, // download | install | done | error
    message: String,
    percent: i32, // -1 = indeterminate
}

fn emit(app: &AppHandle, phase: &str, message: &str, percent: i32) {
    let _ = app.emit(
        "update-progress",
        Progress {
            phase: phase.to_string(),
            message: message.to_string(),
            percent,
        },
    );
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(UA)
        .build()
        .map_err(|e| e.to_string())
}

/// Best-effort GitHub token: honour GITHUB_TOKEN, else fall back to the gh CLI.
/// Purely to dodge the 60/hour anonymous rate limit; never required.
async fn resolve_token() -> Option<String> {
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    let out = tokio::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .await
        .ok()?;
    if out.status.success() {
        let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !t.is_empty() {
            return Some(t);
        }
    }
    None
}

async fn latest_release() -> Result<Option<Release>, String> {
    let url = format!("https://api.github.com/repos/{OWNER}/{REPO}/releases?per_page=10");
    let mut req = client()?
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(t) = resolve_token().await {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err("Limite di richieste GitHub raggiunto, riprova più tardi.".into());
    }
    if !status.is_success() {
        return Err(format!("GitHub API: {status}"));
    }
    let releases: Vec<Release> = resp.json().await.map_err(|e| e.to_string())?;
    Ok(releases.into_iter().find(|r| !r.draft && !r.prerelease))
}

/// Pick the installable asset for the platform we are running on.
fn pick_platform_asset(assets: &[Asset]) -> Option<&Asset> {
    #[cfg(target_os = "linux")]
    {
        // Arch package: DevToys ships for Arch as .pkg.tar.zst.
        return assets.iter().find(|a| a.name.ends_with(".pkg.tar.zst"));
    }
    #[cfg(target_os = "macos")]
    {
        let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else { "x64" };
        return assets
            .iter()
            .find(|a| a.name.ends_with(".dmg") && a.name.contains(arch))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".dmg")));
    }
    #[cfg(target_os = "windows")]
    {
        // Prefer the NSIS `-setup.exe` (per-user, silent), else the WiX `.msi`.
        return assets
            .iter()
            .find(|a| a.name.ends_with("-setup.exe"))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".msi")));
    }
    #[allow(unreachable_code)]
    {
        let _ = assets;
        None
    }
}

// ---------------------------------------------------------------------------
// version comparison (dotted, numeric: 1.10 > 1.9)
// ---------------------------------------------------------------------------

fn normalize(v: &str) -> String {
    v.trim().trim_start_matches(['v', 'V']).to_string()
}

fn parse_parts(v: &str) -> Vec<u64> {
    normalize(v)
        .split(['.', '-', '+'])
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

fn compare(a: &str, b: &str) -> Ordering {
    let pa = parse_parts(a);
    let pb = parse_parts(b);
    for i in 0..pa.len().max(pb.len()) {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ---------------------------------------------------------------------------
// commands
// ---------------------------------------------------------------------------

/// Check GitHub for a newer release. Never fails silently — returns an error
/// string the UI can surface, or an `UpdateInfo` describing the state.
#[tauri::command]
pub async fn update_check() -> Result<UpdateInfo, String> {
    let current = current_version();
    let Some(rel) = latest_release().await? else {
        return Ok(UpdateInfo {
            current,
            latest: None,
            available: false,
            notes: None,
            published_at: None,
            asset_name: None,
            asset_url: None,
            installable: false,
        });
    };
    let latest = normalize(&rel.tag_name);
    let available = compare(&current, &latest) == Ordering::Less;
    let asset = pick_platform_asset(&rel.assets);
    Ok(UpdateInfo {
        current,
        available,
        installable: asset.is_some(),
        asset_name: asset.map(|a| a.name.clone()),
        asset_url: asset.map(|a| a.browser_download_url.clone()),
        notes: rel.body.clone(),
        published_at: rel.published_at.clone(),
        latest: Some(latest),
    })
}

fn cache_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("updates")
}

/// Download the platform package for the latest release and install it.
/// Streams `update-progress` events throughout.
#[tauri::command]
pub async fn update_install(app: AppHandle) -> Result<(), String> {
    let run = async {
        let Some(rel) = latest_release().await? else {
            return Err("Nessuna release trovata.".to_string());
        };
        let asset = pick_platform_asset(&rel.assets)
            .ok_or("Nessun pacchetto disponibile per questa piattaforma.")?;
        let url = asset.browser_download_url.clone();
        let name = asset.name.clone();

        let file = download(&app, &url, &name).await?;
        install_package(&app, &file).await?;
        Ok::<(), String>(())
    };

    match run.await {
        Ok(()) => {
            emit(&app, "done", "Aggiornamento installato — riavvia DevToys.", 100);
            Ok(())
        }
        Err(e) => {
            emit(&app, "error", &e, -1);
            Err(e)
        }
    }
}

async fn download(app: &AppHandle, url: &str, filename: &str) -> Result<PathBuf, String> {
    let dir = cache_dir(app);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    let dest = dir.join(filename);

    let resp = client()?.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Download fallito: {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);
    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        let pct = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0) as i32
        } else {
            -1
        };
        emit(
            app,
            "download",
            &format!("Scaricamento… {}", human(downloaded, total)),
            pct,
        );
    }
    file.flush().await.map_err(|e| e.to_string())?;
    Ok(dest)
}

fn human(done: u64, total: u64) -> String {
    let mb = |b: u64| format!("{:.1} MB", b as f64 / 1_048_576.0);
    if total > 0 {
        format!("{} / {}", mb(done), mb(total))
    } else {
        mb(done)
    }
}

async fn install_package(app: &AppHandle, file: &PathBuf) -> Result<(), String> {
    emit(app, "install", "Installazione…", -1);

    #[cfg(target_os = "linux")]
    {
        // pacman needs root: pkexec shows the polkit GUI prompt.
        let status = tokio::process::Command::new("pkexec")
            .args(["pacman", "-U", "--noconfirm"])
            .arg(file)
            .status()
            .await
            .map_err(|e| format!("Impossibile avviare pkexec/pacman: {e}"))?;
        if !status.success() {
            return Err("Installazione annullata o fallita (pacman -U).".to_string());
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        return install_dmg(file).await;
    }

    #[cfg(target_os = "windows")]
    {
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let file_str = file.to_string_lossy().to_string();
        if ext == "msi" {
            return win::run_elevated(
                "msiexec",
                &["/i".into(), file_str, "/quiet".into(), "/norestart".into()],
            )
            .await;
        }
        return win::run_elevated(&file_str, &["/S".into()]).await;
    }

    #[allow(unreachable_code)]
    {
        let _ = file;
        Err("Piattaforma non supportata".into())
    }
}

#[cfg(target_os = "macos")]
async fn install_dmg(file: &PathBuf) -> Result<(), String> {
    let mount = format!("/Volumes/devtoys-update-{}", std::process::id());
    let out = tokio::process::Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-mountpoint", &mount])
        .arg(file)
        .output()
        .await
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("hdiutil attach fallito".into());
    }
    let res = (|| async {
        let mut entries = tokio::fs::read_dir(&mount).await.map_err(|e| e.to_string())?;
        let mut app_path = None;
        while let Some(e) = entries.next_entry().await.map_err(|e| e.to_string())? {
            if e.file_name().to_string_lossy().ends_with(".app") {
                app_path = Some(e.path());
                break;
            }
        }
        let app_path = app_path.ok_or("Nessuna .app nel dmg")?;
        let status = tokio::process::Command::new("cp")
            .arg("-R")
            .arg(&app_path)
            .arg("/Applications/")
            .status()
            .await
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("Copia in /Applications fallita".to_string());
        }
        Ok::<(), String>(())
    })()
    .await;
    let _ = tokio::process::Command::new("hdiutil")
        .args(["detach", &mount])
        .status()
        .await;
    res
}

#[cfg(target_os = "windows")]
mod win {
    pub async fn run_elevated(program: &str, args: &[String]) -> Result<(), String> {
        fn ps_quote(s: &str) -> String {
            format!("'{}'", s.replace('\'', "''"))
        }
        let mut script =
            String::from("$ErrorActionPreference='Stop'; $p = Start-Process -FilePath ");
        script.push_str(&ps_quote(program));
        script.push_str(" -Verb RunAs -Wait -PassThru");
        if !args.is_empty() {
            let list = args.iter().map(|a| ps_quote(a)).collect::<Vec<_>>().join(",");
            script.push_str(" -ArgumentList ");
            script.push_str(&list);
        }
        script.push_str("; exit $p.ExitCode");

        let status = tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .await
            .map_err(|e| format!("Impossibile avviare PowerShell: {e}"))?;
        match status.code() {
            Some(0) => Ok(()),
            _ => Err("Installazione annullata o non riuscita.".into()),
        }
    }
}
