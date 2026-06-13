use crate::sync::SyncEngine;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    engine: Arc<SyncEngine>,
    tx: Option<mpsc::UnboundedSender<String>>,
}

impl FileWatcher {
    pub fn new(engine: Arc<SyncEngine>) -> Self {
        Self {
            watcher: None,
            engine,
            tx: None,
        }
    }

    fn is_ignored_ext(ext: &str) -> bool {
        let ignore = ["tmp", "temp", "swp", "~", "part", "crdownload"];
        ignore.contains(&ext)
    }

    fn file_events_from_path(path: &std::path::Path, tx: &mpsc::UnboundedSender<String>) {
        if !path.is_file() { return; }
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !Self::is_ignored_ext(&ext) {
            let _ = tx.send(path.to_string_lossy().to_string());
        }
    }

    pub fn start_watching(&mut self, root: &str) -> Result<(), String> {
        let path = Path::new(root);
        if !path.exists() {
            return Err(format!("Folder does not exist: {}", root));
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let tx_clone = tx.clone();
        let tx_scan = tx.clone();

        let engine = self.engine.clone();
        let root_owned = root.to_string();

        let mut watcher = RecommendedWatcher::new(
            move |event: Result<Event, notify::Error>| {
                if let Ok(event) = event {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            for p in event.paths {
                                Self::file_events_from_path(&p, &tx_clone);
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| format!("Watcher error: {}", e))?;

        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| format!("Watch error: {}", e))?;

        self.watcher = Some(watcher);
        self.tx = Some(tx.clone());

        // Spawn a tokio task:
        // 1) Initial scan — enqueue existing files
        // 2) Listen for real-time file events
        let engine_task = engine.clone();
        let root_task = root_owned.clone();
        tokio::spawn(async move {
            // --- Initial scan ---
            log::info!("Initial scan: {}", root_task);
            {
                let mut dirs = vec![root_task.clone()];
                while let Some(dir) = dirs.pop() {
                    match std::fs::read_dir(&dir) {
                        Ok(entries) => {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_dir() {
                                    dirs.push(p.to_string_lossy().to_string());
                                } else {
                                    Self::file_events_from_path(&p, &tx_scan);
                                }
                            }
                        }
                        Err(e) => log::warn!("scan error {}: {}", dir, e),
                    }
                }
            }
            log::info!("Initial scan done: {}", root_task);

            // --- Live events ---
            while let Some(file_path) = rx.recv().await {
                log::info!("File event detected: {}", file_path);
                engine_task.enqueue_file(file_path, &root_task).await;
                engine_task.process_queue().await;
            }
        });

        log::info!("Watching: {}", root);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.watcher = None;
        self.tx = None;
    }
}
