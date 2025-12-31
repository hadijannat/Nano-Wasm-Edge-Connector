//! Hot-reload file watcher for policy modules
//!
//! Watches the policies directory and triggers atomic module swap
//! when .wasm files are modified.

use crate::{make_policy_version, policy_runtime::PolicyRuntime, AppState};
use notify_debouncer_mini::{new_debouncer, notify::*, DebounceEventResult};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Watch the policies directory and hot-reload on changes
pub async fn watch_policies(
    state: Arc<AppState>,
    policies_dir: &Path,
    policy_file: &str,
) {
    let (tx, mut rx) = mpsc::channel::<()>(10);
    let policies_path = policies_dir.to_path_buf();

    // Spawn blocking watcher thread
    std::thread::spawn(move || {
        let debouncer_tx = tx.clone();
        let mut debouncer = match new_debouncer(
            Duration::from_millis(500),
            move |res: DebounceEventResult| {
                if let Ok(events) = res {
                    for event in events {
                        if event.path.extension().map_or(false, |e| e == "wasm") {
                            let _ = debouncer_tx.blocking_send(());
                        }
                    }
                }
            },
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to create file watcher: {}", e);
                return;
            }
        };

        if let Err(e) = debouncer
            .watcher()
            .watch(&policies_path, RecursiveMode::NonRecursive)
        {
            eprintln!("Failed to watch policies directory: {}", e);
            return;
        }

        println!("Watching {} for policy changes", policies_path.display());

        // Keep thread alive
        loop {
            std::thread::park();
        }
    });

    let policy_path = policies_dir.join(policy_file);

    // Process reload events
    while rx.recv().await.is_some() {
        println!("Detected policy change, hot-reloading...");

        match tokio::fs::read(&policy_path).await {
            Ok(bytes) => match PolicyRuntime::new(&bytes) {
                Ok(new_runtime) => {
                    let new_version = make_policy_version(bytes.len());
                    let mut guard = state.runtime.write().await;
                    *guard = Arc::new(new_runtime);
                    let mut version = state.policy_version.write().await;
                    *version = new_version;
                    println!("✓ Policy hot-reload successful");
                }
                Err(e) => {
                    eprintln!("✗ Failed to compile new policy: {}", e);
                }
            },
            Err(e) => {
                eprintln!("✗ Failed to read policy file: {}", e);
            }
        }
    }
}
