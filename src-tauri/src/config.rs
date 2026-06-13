use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderEntry {
    pub path: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLogEntry {
    pub file_path: String,
    pub document_id: String,
    pub status: String,
    pub timestamp: String,
    pub folder_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub watched_folder: Option<String>,
    pub auto_start: bool,
    pub folder_cache: Vec<FolderEntry>,
    pub sync_log: Vec<SyncLogEntry>,
    pub last_email: Option<String>,
    pub session_cookie: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "https://dms.arsipin.id".to_string(),
            watched_folder: None,
            auto_start: true,
            folder_cache: Vec::new(),
            sync_log: Vec::new(),
            last_email: None,
            session_cookie: None,
        }
    }
}

impl AppConfig {
    pub fn path() -> PathBuf {
        let mut p = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        p.push("dms-sync-config.json");
        p
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            fs::read_to_string(&p)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let p = Self::path();
            if let Some(dir) = p.parent() {
                let _ = fs::create_dir_all(dir);
            }
            let _ = fs::write(&p, &json);
        }
    }
}

fn dirs_next() -> Option<PathBuf> {
    let dir = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .ok()?;
    Some(PathBuf::from(dir))
}
