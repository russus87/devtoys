//! API & Test tools backend: load tester, mock server, pragmatic reverse
//! proxy and smoke runner. All HTTP work happens here in Rust (no CORS/CSP
//! limits) and everything stays local.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::body::{Body, Bytes};
use axum::extract::State as AxState;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::Response;
use axum::Router;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;

// ---------------------------------------------------------------------------
// shared state (managed by Tauri via .manage())
// ---------------------------------------------------------------------------

pub struct ServerHandle {
    shutdown: oneshot::Sender<()>,
    port: u16,
    target: Option<String>,
}

#[derive(Default)]
pub struct TestState {
    mock: Mutex<Option<ServerHandle>>,
    proxy: Mutex<Option<ServerHandle>>,
    load_cancel: Mutex<Option<Arc<AtomicBool>>>,
}

fn now_iso() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

// ---------------------------------------------------------------------------
// load tester
// ---------------------------------------------------------------------------

#[derive(Default)]
struct LoadStats {
    ok: u64,
    failed: u64,
    latencies: Vec<f64>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LoadSnapshot {
    sent: u64,
    ok: u64,
    failed: u64,
    rps: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    min_ms: f64,
    max_ms: f64,
    avg_ms: f64,
    elapsed_ms: f64,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn snapshot(stats: &LoadStats, elapsed_ms: f64) -> LoadSnapshot {
    let sent = stats.ok + stats.failed;
    let mut lat = stats.latencies.clone();
    lat.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let sum: f64 = lat.iter().sum();
    let avg = if lat.is_empty() { 0.0 } else { sum / lat.len() as f64 };
    LoadSnapshot {
        sent,
        ok: stats.ok,
        failed: stats.failed,
        rps: if elapsed_ms > 0.0 { sent as f64 / (elapsed_ms / 1000.0) } else { 0.0 },
        p50: percentile(&lat, 50.0),
        p95: percentile(&lat, 95.0),
        p99: percentile(&lat, 99.0),
        min_ms: lat.first().copied().unwrap_or(0.0),
        max_ms: lat.last().copied().unwrap_or(0.0),
        avg_ms: avg,
        elapsed_ms,
    }
}

#[tauri::command]
pub async fn load_start(
    app: AppHandle,
    state: State<'_, TestState>,
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    concurrency: u32,
    total_requests: u32,
) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));
    if let Ok(mut g) = state.load_cancel.lock() {
        *g = Some(cancel.clone());
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .map_err(|e| e.to_string())?;
    let total = total_requests.max(1);
    let conc = concurrency.clamp(1, 1000);
    let next = Arc::new(AtomicU32::new(0));
    let stats = Arc::new(Mutex::new(LoadStats::default()));
    let done = Arc::new(AtomicBool::new(false));
    let start = Instant::now();

    let method = Arc::new(method);
    let url = Arc::new(url);
    let headers = Arc::new(headers);
    let body = Arc::new(body);

    // ticker: emit a snapshot every 250ms until done
    {
        let app = app.clone();
        let stats = stats.clone();
        let done = done.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                let snap = {
                    let s = stats.lock().unwrap();
                    snapshot(&s, start.elapsed().as_secs_f64() * 1000.0)
                };
                let _ = app.emit("load:tick", snap);
                if done.load(Ordering::Relaxed) {
                    break;
                }
            }
        });
    }

    // driver: spawn `conc` workers pulling request indices until exhausted
    tokio::spawn(async move {
        let mut workers = Vec::new();
        for _ in 0..conc {
            let client = client.clone();
            let next = next.clone();
            let stats = stats.clone();
            let cancel = cancel.clone();
            let method = method.clone();
            let url = url.clone();
            let headers = headers.clone();
            let body = body.clone();
            workers.push(tokio::spawn(async move {
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= total {
                        break;
                    }
                    let m = reqwest::Method::from_bytes(method.as_bytes())
                        .unwrap_or(reqwest::Method::GET);
                    let mut rb = client.request(m, url.as_str());
                    for (k, v) in headers.iter() {
                        rb = rb.header(k, v);
                    }
                    if let Some(b) = body.as_ref() {
                        if !b.is_empty() {
                            rb = rb.body(b.clone());
                        }
                    }
                    let t0 = Instant::now();
                    match rb.send().await {
                        Ok(resp) => {
                            let ok = resp.status().is_success();
                            let _ = resp.bytes().await;
                            let ms = t0.elapsed().as_secs_f64() * 1000.0;
                            let mut s = stats.lock().unwrap();
                            if ok {
                                s.ok += 1;
                            } else {
                                s.failed += 1;
                            }
                            s.latencies.push(ms);
                        }
                        Err(_) => {
                            let mut s = stats.lock().unwrap();
                            s.failed += 1;
                        }
                    }
                }
            }));
        }
        for w in workers {
            let _ = w.await;
        }
        done.store(true, Ordering::Relaxed);
        let snap = {
            let s = stats.lock().unwrap();
            snapshot(&s, start.elapsed().as_secs_f64() * 1000.0)
        };
        let _ = app.emit("load:done", snap);
    });

    Ok(())
}

#[tauri::command]
pub fn load_stop(state: State<'_, TestState>) -> Result<(), String> {
    if let Ok(g) = state.load_cancel.lock() {
        if let Some(c) = g.as_ref() {
            c.store(true, Ordering::Relaxed);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// mock server
// ---------------------------------------------------------------------------

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MockRoute {
    method: String,
    path: String,
    status: u16,
    content_type: String,
    body: String,
    delay_ms: u64,
}

struct MockShared {
    routes: Vec<MockRoute>,
    app: AppHandle,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MockRequestEvent {
    method: String,
    path: String,
    matched: bool,
    at: String,
}

async fn mock_handler(
    AxState(shared): AxState<Arc<MockShared>>,
    method: Method,
    uri: Uri,
    _headers: HeaderMap,
    _body: Bytes,
) -> Response {
    let path = uri.path().to_string();
    let m = method.as_str().to_string();
    let route = shared
        .routes
        .iter()
        .find(|r| r.method.eq_ignore_ascii_case(&m) && r.path == path);

    let _ = shared.app.emit(
        "mock:request",
        MockRequestEvent {
            method: m.clone(),
            path: path.clone(),
            matched: route.is_some(),
            at: now_iso(),
        },
    );

    match route {
        Some(r) => {
            if r.delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(r.delay_ms)).await;
            }
            Response::builder()
                .status(StatusCode::from_u16(r.status).unwrap_or(StatusCode::OK))
                .header("content-type", r.content_type.clone())
                .body(Body::from(r.body.clone()))
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "application/json")
            .body(Body::from("{\"error\":\"no matching mock route\"}"))
            .unwrap_or_else(|_| Response::new(Body::empty())),
    }
}

#[tauri::command]
pub async fn mock_start(
    app: AppHandle,
    state: State<'_, TestState>,
    port: u16,
    routes: Vec<MockRoute>,
) -> Result<(), String> {
    // stop any existing instance first
    if let Ok(mut g) = state.mock.lock() {
        if let Some(h) = g.take() {
            let _ = h.shutdown.send(());
        }
    }

    let shared = Arc::new(MockShared { routes, app });
    let router = Router::new()
        .fallback(mock_handler)
        .with_state(shared);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("impossibile ascoltare sulla porta {port}: {e}"))?;
    let (tx, rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await;
    });

    if let Ok(mut g) = state.mock.lock() {
        *g = Some(ServerHandle { shutdown: tx, port, target: None });
    }
    Ok(())
}

#[tauri::command]
pub fn mock_stop(state: State<'_, TestState>) -> Result<(), String> {
    if let Ok(mut g) = state.mock.lock() {
        if let Some(h) = g.take() {
            let _ = h.shutdown.send(());
        }
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    running: bool,
    port: Option<u16>,
    target: Option<String>,
}

#[tauri::command]
pub fn mock_status(state: State<'_, TestState>) -> ServerStatus {
    let g = state.mock.lock().ok();
    match g.as_ref().and_then(|g| g.as_ref()) {
        Some(h) => ServerStatus { running: true, port: Some(h.port), target: None },
        None => ServerStatus { running: false, port: None, target: None },
    }
}

// ---------------------------------------------------------------------------
// pragmatic reverse proxy (forwards everything to `target`, logs each exchange)
// ---------------------------------------------------------------------------

struct ProxyShared {
    target: String,
    client: reqwest::Client,
    app: AppHandle,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProxyExchange {
    method: String,
    path: String,
    status: u16,
    ms: f64,
    at: String,
}

async fn proxy_handler(
    AxState(shared): AxState<Arc<ProxyShared>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path_and_query = uri
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string());
    let target = shared.target.trim_end_matches('/');
    let url = format!("{target}{path_and_query}");
    let m = method.as_str().to_string();

    let rmethod = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);
    let mut rb = shared.client.request(rmethod, &url);
    for (k, v) in headers.iter() {
        let name = k.as_str().to_ascii_lowercase();
        if name == "host" || name == "content-length" {
            continue;
        }
        if let Ok(val) = v.to_str() {
            rb = rb.header(k.as_str(), val);
        }
    }
    if !body.is_empty() {
        rb = rb.body(body.to_vec());
    }

    let t0 = Instant::now();
    let resp = rb.send().await;
    let ms = t0.elapsed().as_secs_f64() * 1000.0;

    match resp {
        Ok(r) => {
            let status = r.status();
            let ct = r
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let bytes = r.bytes().await.unwrap_or_default();
            let _ = shared.app.emit(
                "proxy:exchange",
                ProxyExchange {
                    method: m,
                    path: path_and_query,
                    status: status.as_u16(),
                    ms,
                    at: now_iso(),
                },
            );
            Response::builder()
                .status(StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK))
                .header("content-type", ct)
                .body(Body::from(bytes))
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
        Err(e) => {
            let _ = shared.app.emit(
                "proxy:exchange",
                ProxyExchange {
                    method: m,
                    path: path_and_query,
                    status: 502,
                    ms,
                    at: now_iso(),
                },
            );
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("content-type", "text/plain")
                .body(Body::from(format!("proxy error: {e}")))
                .unwrap_or_else(|_| Response::new(Body::empty()))
        }
    }
}

#[tauri::command]
pub async fn proxy_start(
    app: AppHandle,
    state: State<'_, TestState>,
    port: u16,
    target: String,
) -> Result<(), String> {
    if let Ok(mut g) = state.proxy.lock() {
        if let Some(h) = g.take() {
            let _ = h.shutdown.send(());
        }
    }

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;
    let shared = Arc::new(ProxyShared { target: target.clone(), client, app });
    let router = Router::new().fallback(proxy_handler).with_state(shared);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("impossibile ascoltare sulla porta {port}: {e}"))?;
    let (tx, rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await;
    });

    if let Ok(mut g) = state.proxy.lock() {
        *g = Some(ServerHandle { shutdown: tx, port, target: Some(target) });
    }
    Ok(())
}

#[tauri::command]
pub fn proxy_stop(state: State<'_, TestState>) -> Result<(), String> {
    if let Ok(mut g) = state.proxy.lock() {
        if let Some(h) = g.take() {
            let _ = h.shutdown.send(());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn proxy_status(state: State<'_, TestState>) -> ServerStatus {
    let g = state.proxy.lock().ok();
    match g.as_ref().and_then(|g| g.as_ref()) {
        Some(h) => ServerStatus {
            running: true,
            port: Some(h.port),
            target: h.target.clone(),
        },
        None => ServerStatus { running: false, port: None, target: None },
    }
}

// ---------------------------------------------------------------------------
// smoke / synthetic runner
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeStep {
    name: String,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    assert_status: Option<u16>,
    assert_contains: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeResult {
    name: String,
    ok: bool,
    status: u16,
    ms: f64,
    error: Option<String>,
}

#[tauri::command]
pub async fn smoke_run(steps: Vec<SmokeStep>) -> Result<Vec<SmokeResult>, String> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;
    let _ = AtomicU64::new(0); // (kept for potential future counters)
    let mut out = Vec::with_capacity(steps.len());

    for step in steps {
        let m = reqwest::Method::from_bytes(step.method.as_bytes()).unwrap_or(reqwest::Method::GET);
        let mut rb = client.request(m, &step.url);
        for (k, v) in &step.headers {
            rb = rb.header(k, v);
        }
        if let Some(b) = &step.body {
            if !b.is_empty() {
                rb = rb.body(b.clone());
            }
        }
        let t0 = Instant::now();
        let res = rb.send().await;
        let ms = t0.elapsed().as_secs_f64() * 1000.0;

        let result = match res {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();
                let mut err: Option<String> = None;
                if let Some(exp) = step.assert_status {
                    if status != exp {
                        err = Some(format!("status atteso {exp}, ricevuto {status}"));
                    }
                }
                if err.is_none() {
                    if let Some(needle) = &step.assert_contains {
                        if !needle.is_empty() && !text.contains(needle.as_str()) {
                            err = Some(format!("il corpo non contiene «{needle}»"));
                        }
                    }
                }
                SmokeResult {
                    name: step.name,
                    ok: err.is_none(),
                    status,
                    ms,
                    error: err,
                }
            }
            Err(e) => SmokeResult {
                name: step.name,
                ok: false,
                status: 0,
                ms,
                error: Some(e.to_string()),
            },
        };
        out.push(result);
    }

    Ok(out)
}
