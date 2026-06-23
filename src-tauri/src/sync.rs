use crate::api::DmsApi;
use crate::config::SyncLogEntry;
use crate::folder_cache::FolderCache;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct QueuedFile {
    pub file_path: String,
    pub relative_path: String,
    pub file_data: Vec<u8>,
    pub mime_type: String,
    pub file_hash: String,
    pub watched_root: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Paused,
    Error(String),
}

pub struct SyncEngine {
    pub status: Arc<Mutex<SyncStatus>>,
    pub queue: Arc<Mutex<Vec<QueuedFile>>>,
    pub processed: Arc<Mutex<HashMap<String, String>>>,
    pub logs: Arc<Mutex<Vec<SyncLogEntry>>>,
    pub folder_cache: Arc<Mutex<FolderCache>>,
    pub api: Arc<Mutex<Option<DmsApi>>>,
    pub watched_roots: Arc<Mutex<Vec<String>>>,
    pub watcher_events: Arc<Mutex<Vec<String>>>,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(SyncStatus::Idle)),
            queue: Arc::new(Mutex::new(Vec::new())),
            processed: Arc::new(Mutex::new(HashMap::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            folder_cache: Arc::new(Mutex::new(FolderCache::new())),
            api: Arc::new(Mutex::new(None)),
            watched_roots: Arc::new(Mutex::new(Vec::new())),
            watcher_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn relative_path(root: &str, full_path: &str, events: &mut Vec<String>) -> String {
        // Normalize root: trim trailing separators
        let root_trimmed = root.trim_end_matches(&['/', '\\', ' '][..]);
        let root_path = Path::new(root_trimmed);
        let full = Path::new(full_path);

        events.push(format!(
            "relative_path: root=[{}] full=[{}]",
            root_path.display(),
            full.display()
        ));

        // Try Path::strip_prefix first (cross-platform)
        if let Ok(rel) = full.strip_prefix(root_path) {
            let result = rel.to_string_lossy().replace('\\', "/");
            events.push(format!("relative_path OK: {}", result));
            return result;
        }

        // Fallback: manual prefix check
        // On Windows, do case-insensitive comparison
        // On macOS, APFS can be either case-sensitive or case-insensitive,
        // but try case-insensitive as a fallback too
        #[cfg(target_os = "windows")]
        {
            let root_str = root_path.to_string_lossy().to_lowercase();
            let full_str = full.to_string_lossy().to_lowercase();
            if full_str.starts_with(&root_str) {
                let suffix = &full.to_string_lossy()[root_str.len()..];
                let suffix = suffix.trim_start_matches(&['/', '\\'][..]);
                let result = suffix.replace('\\', "/");
                events.push(format!("relative_path OK (case-insensitive): {}", result));
                return result;
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let root_str = root_path.to_string_lossy();
            let full_str = full.to_string_lossy();
            if full_str.starts_with(root_str.as_ref()) {
                let suffix = &full_str[root_str.len()..];
                let suffix = suffix.trim_start_matches('/');
                let result = suffix.to_string();
                events.push(format!("relative_path OK (manual): {}", result));
                return result;
            }
        }

        // Last resort: just use filename
        let result = full
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        events.push(format!("relative_path FAIL: root=[{}] full=[{}] -> {}", root, full_path, result));
        log::warn!(
            "relative_path FAIL: root={} full={} -> {}",
            root, full_path, result
        );
        result
    }

    pub fn folder_path_from_relative(rel_path: &str) -> Option<String> {
        let p = Path::new(rel_path);
        p.parent().and_then(|parent| {
            let s = parent.to_string_lossy();
            if s.is_empty() || s == "." {
                None
            } else {
                Some(s.replace('\\', "/"))
            }
        })
    }

    pub fn mime_from_ext(path: &str) -> String {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "pdf" => "application/pdf".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "png" => "image/png".to_string(),
            "gif" => "image/gif".to_string(),
            "webp" => "image/webp".to_string(),
            "doc" | "docx" => "application/msword".to_string(),
            "xls" | "xlsx" => "application/vnd.ms-excel".to_string(),
            "ppt" | "pptx" => "application/vnd.ms-powerpoint".to_string(),
            "txt" => "text/plain".to_string(),
            "csv" => "text/csv".to_string(),
            "mp4" => "video/mp4".to_string(),
            "mp3" => "audio/mpeg".to_string(),
            "zip" => "application/zip".to_string(),
            "rar" => "application/vnd.rar".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    pub async fn enqueue_file(&self, file_path: String) {
        // Track raw event for debugging
        {
            let mut ev = self.watcher_events.lock().await;
            ev.push(format!("raw_event: {}", file_path));
            if ev.len() > 500 { ev.remove(0); }
        }
        log::info!("enqueue_file called: {}", file_path);

        // Find matching root (must match on directory boundary)
        let root = {
            let roots = self.watched_roots.lock().await;
            roots.iter().find(|r| {
                let r_trimmed = r.trim_end_matches(&['/', '\\', ' '][..]);
                let fp_lower = file_path.to_lowercase();
                let r_lower = r_trimmed.to_lowercase();
                if !fp_lower.starts_with(&r_lower) { return false; }
                let after = &fp_lower[r_lower.len()..];
                after.is_empty() || after.starts_with('\\') || after.starts_with('/')
            }).cloned()
        };
        let root = match root {
            Some(r) => r,
            None => {
                let msg = format!("no matching root for: {}", file_path);
                log::warn!("{}", msg);
                let mut ev = self.watcher_events.lock().await;
                ev.push(msg);
                return;
            }
        };

        let data = tokio::fs::read(&file_path).await;
        if let Err(e) = data {
            let msg = format!("read error {}: {}", file_path, e);
            log::error!("{}", msg);
            let mut ev = self.watcher_events.lock().await;
            ev.push(msg);
            return;
        }
        let data = data.unwrap();
        let hash = Self::compute_hash(&data);

        {
            let processed = self.processed.lock().await;
            if processed.contains_key(&file_path) && processed.get(&file_path) == Some(&hash) {
                let msg = format!("skip (already processed): {}", file_path);
                log::info!("{}", msg);
                let mut ev = self.watcher_events.lock().await;
                ev.push(msg);
                return;
            }
        }

        let rel = {
            let mut ev = self.watcher_events.lock().await;
            Self::relative_path(&root, &file_path, &mut *ev)
        };
        let mime = Self::mime_from_ext(&file_path);

        let qf = QueuedFile {
            file_path: file_path.clone(),
            relative_path: rel.clone(),
            file_data: data,
            mime_type: mime,
            file_hash: hash,
            watched_root: root,
        };

        {
            let mut queue = self.queue.lock().await;
            queue.push(qf);
        }

        let msg = format!("enqueued: {} ({})", file_path, rel);
        log::info!("{}", msg);
        {
            let mut ev = self.watcher_events.lock().await;
            ev.push(msg);
        }
    }

    pub async fn process_queue(&self) {
        loop {
            {
                let mut ev = self.watcher_events.lock().await;
                ev.push("process_queue loop start".to_string());
            }

            let item = {
                let mut queue = self.queue.lock().await;
                if queue.is_empty() {
                    let mut ev = self.watcher_events.lock().await;
                    ev.push("queue empty, returning".to_string());
                    return;
                }
                queue.remove(0)
            };

            {
                let mut status = self.status.lock().await;
                *status = SyncStatus::Syncing;
            }

            let mut folder_path = Self::folder_path_from_relative(&item.relative_path);

            // If file is at root of watched folder (no parent in relative path),
            // use the watched folder's own name as the folderPath
            if folder_path.is_none() {
                folder_path = Path::new(&item.watched_root)
                    .file_name()
                    .map(|n| n.to_string_lossy().replace('\\', "/"));
            }

            let fp_for_log = folder_path.clone();

            {
                let mut ev = self.watcher_events.lock().await;
                ev.push(format!("processing: {} folderPath={:?}", item.relative_path, fp_for_log));
            }

            let result = {
                let api_lock = self.api.lock().await;
                if let Some(ref api) = *api_lock {
                    {
                        let mut ev = self.watcher_events.lock().await;
                        ev.push("api available, calling sync_upload".to_string());
                    }
                    api.sync_upload(
                        &item.file_path,
                        item.file_data.clone(),
                        &item.mime_type,
                        &item.relative_path,
                        folder_path.as_deref(),
                    )
                    .await
                } else {
                    let mut ev = self.watcher_events.lock().await;
                    ev.push("ERROR: api not available (not logged in)".to_string());
                    Err("Not logged in".to_string())
                }
            };

            match result {
                Ok(response) => {
                    {
                        let mut processed = self.processed.lock().await;
                        processed.insert(item.file_path.clone(), item.file_hash);
                    }

                    {
                        let mut ev = self.watcher_events.lock().await;
                        ev.push(format!("upload success: {} -> {}", item.relative_path, response.document_id));
                    }

                    let log_entry = SyncLogEntry {
                        file_path: item.file_path.clone(),
                        document_id: response.document_id.clone(),
                        status: "success".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        folder_path: folder_path.clone(),
                        error: None,
                    };

                    {
                        let mut logs = self.logs.lock().await;
                        logs.push(log_entry);
                        if logs.len() > 1000 {
                            logs.remove(0);
                        }
                    }

                    log::info!(
                        "Synced: {} -> doc {}",
                        item.relative_path,
                        response.document_id
                    );
                }
                Err(e) => {
                    {
                        let mut ev = self.watcher_events.lock().await;
                        ev.push(format!("upload FAIL: {} -> {}", item.relative_path, e));
                    }

                    let log_entry = SyncLogEntry {
                        file_path: item.file_path.clone(),
                        document_id: String::new(),
                        status: "error".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        folder_path: folder_path.clone(),
                        error: Some(e.clone()),
                    };

                    {
                        let mut logs = self.logs.lock().await;
                        logs.push(log_entry);
                    }

                    log::error!("Failed to sync {}: {}", item.relative_path, e);
                }
            }
        }
    }
}
