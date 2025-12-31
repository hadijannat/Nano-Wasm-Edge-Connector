//! Nano-Wasm Edge Connector
//!
//! A lightweight Rust-based Dataspace Connector using WebAssembly
//! for dynamic policy enforcement on edge devices.
//!
//! Target: <10MB RAM operation with single binary deployment.

mod error;
mod policy_runtime;
mod watcher;

use axum::{
    body::Bytes,
    extract::State,
    routing::{get, post},
    Json, Router,
};
use policy_runtime::PolicyRuntime;
use serde_json::{json, Value};
use shared::PolicyResponse;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Application state shared across handlers
pub struct AppState {
    runtime: RwLock<Arc<PolicyRuntime>>,
    policy_version: RwLock<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║     Nano-Wasm Edge Connector v{}          ║", env!("CARGO_PKG_VERSION"));
    println!("║     Lightweight Policy Enforcement Engine        ║");
    println!("╚══════════════════════════════════════════════════╝");

    // Ensure policies directory exists
    let policies_dir = PathBuf::from("./policies");
    if !policies_dir.exists() {
        std::fs::create_dir_all(&policies_dir)?;
        println!("Created policies directory: {}", policies_dir.display());
    }

    let policy_file = "default.wasm";
    let policy_path = policies_dir.join(policy_file);

    // Load initial policy module
    let wasm_bytes = match std::fs::read(&policy_path) {
        Ok(bytes) => {
            println!("✓ Loaded policy: {} ({} bytes)", policy_path.display(), bytes.len());
            bytes
        }
        Err(e) => {
            eprintln!("✗ Failed to load policy from {}: {}", policy_path.display(), e);
            eprintln!("  Please build the guest module and copy to policies/default.wasm");
            eprintln!("  Run: cargo build -p guest --target wasm32-unknown-unknown --release");
            eprintln!("       cp target/wasm32-unknown-unknown/release/guest.wasm policies/default.wasm");
            return Err(e.into());
        }
    };

    let runtime = PolicyRuntime::new(&wasm_bytes)?;
    println!("✓ Policy runtime initialized");

    let policy_version = make_policy_version(wasm_bytes.len());
    let state = Arc::new(AppState {
        runtime: RwLock::new(Arc::new(runtime)),
        policy_version: RwLock::new(policy_version),
    });

    // Setup hot-reload watcher
    let state_clone = state.clone();
    let policies_dir_clone = policies_dir.clone();
    tokio::spawn(async move {
        watcher::watch_policies(state_clone, &policies_dir_clone, policy_file).await;
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/evaluate", post(evaluate_policy))
        .route("/reload", post(reload_policy))
        .route("/metrics", get(get_metrics))
        .with_state(state);

    // Bind listener
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("✓ Listening on http://{}", addr);
    println!("");
    println!("Endpoints:");
    println!("  GET  /health   - Health check");
    println!("  POST /evaluate - Evaluate policy");
    println!("  POST /reload   - Force policy reload");
    println!("  GET  /metrics  - Runtime metrics");
    println!("");

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!("Server shutdown complete");
    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Policy evaluation endpoint
async fn evaluate_policy(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Json<PolicyResponse> {
    let runtime = { state.runtime.read().await.clone() };
    let policy_version = { state.policy_version.read().await.clone() };
    if let Err(e) = serde_json::from_slice::<Value>(&body) {
        return Json(PolicyResponse {
            allowed: false,
            policy_version,
            error: Some(format!("Invalid JSON: {}", e)),
        });
    }

    let request_bytes = body.to_vec();

    let eval_result =
        tokio::task::spawn_blocking(move || runtime.evaluate_policy(&request_bytes)).await;

    match eval_result {
        Ok(Ok(allowed)) => Json(PolicyResponse {
            allowed,
            policy_version,
            error: None,
        }),
        Ok(Err(e)) => Json(PolicyResponse {
            allowed: false,
            policy_version,
            error: Some(e.to_string()),
        }),
        Err(e) => Json(PolicyResponse {
            allowed: false,
            policy_version,
            error: Some(format!("Policy execution join error: {}", e)),
        }),
    }
}

/// Force policy reload endpoint
async fn reload_policy(State(state): State<Arc<AppState>>) -> Json<Value> {
    let policy_path = PathBuf::from("./policies/default.wasm");

    match std::fs::read(&policy_path) {
        Ok(bytes) => match PolicyRuntime::new(&bytes) {
            Ok(new_runtime) => {
                let new_version = make_policy_version(bytes.len());
                let mut runtime = state.runtime.write().await;
                *runtime = Arc::new(new_runtime);
                let mut version = state.policy_version.write().await;
                *version = new_version.clone();
                println!("✓ Policy manually reloaded");
                Json(json!({
                    "success": true,
                    "message": "Policy reloaded successfully",
                    "size_bytes": bytes.len(),
                    "policy_version": new_version
                }))
            }
            Err(e) => Json(json!({
                "success": false,
                "error": format!("Failed to compile policy: {}", e)
            })),
        },
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to read policy file: {}", e)
        })),
    }
}

/// Runtime metrics endpoint
async fn get_metrics() -> Json<Value> {
    // Get process memory info (platform-specific)
    let memory_kb = get_memory_usage_kb();

    Json(json!({
        "memory_kb": memory_kb,
        "memory_mb": memory_kb as f64 / 1024.0,
        "target_mb": 10,
        "within_target": memory_kb < 10 * 1024
    }))
}

/// Get current process memory usage in KB
fn get_memory_usage_kb() -> u64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let pid = std::process::id();
        if let Ok(output) = Command::new("ps")
            .args(["-o", "rss=", "-p", &pid.to_string()])
            .output()
        {
            if let Ok(rss) = String::from_utf8_lossy(&output.stdout).trim().parse::<u64>() {
                return rss;
            }
        }
        0
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        return kb.parse().unwrap_or(0);
                    }
                }
            }
        }
        0
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        0
    }
}

/// Wait for shutdown signal (SIGINT/SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => println!("\nReceived CTRL+C, shutting down..."),
        _ = terminate => println!("\nReceived SIGTERM, shutting down..."),
    }
}

pub fn make_policy_version(bytes_len: usize) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}-{}b-{}", env!("CARGO_PKG_VERSION"), bytes_len, ts)
}
