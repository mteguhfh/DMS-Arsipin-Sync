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

    pub fn add_watch(&mut self, root: &str) -> Result<(), String> {
        let path = Path::new(root);
        if !path.exists() {
            return Err(format!("Folder does not exist: {}", root));
        }

        if self.watcher.is_none() {
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();
            let engine = self.engine.clone();
            let tx_clone = tx.clone();
            let tx_scan_init = tx.clone();

            let watcher = RecommendedWatcher::new(
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

            self.watcher = Some(watcher);
            self.tx = Some(tx_scan_init);

            let engine_listener = engine.clone();
            tokio::spawn(async move {
                while let Some(file_path) = rx.recv().await {
                    engine_listener.enqueue_file(file_path).await;
                    engine_listener.process_queue().await;
                }
            });
        }

        if let Some(ref mut watcher) = self.watcher {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| format!("Watch error: {}", e))?;
        }

        let root_owned = root.to_string();
        let tx_scan = self.tx.clone().unwrap();
        tokio::spawn(async move {
            log::info!("Initial scan: {}", root_owned);
            let mut dirs = vec![root_owned.clone()];
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
            log::info!("Initial scan done: {}", root_owned);
        });

        log::info!("Watching: {}", root);
        Ok(())
    }

    pub fn remove_watch(&mut self, root: &str) -> Result<(), String> {
        if let Some(ref mut watcher) = self.watcher {
            let _ = watcher.unwatch(Path::new(root));
        }
        log::info!("Stopped watching: {}", root);
        Ok(())
    }

    pub fn stop_all(&mut self) {
        self.watcher = None;
        self.tx = None;
    }
}
