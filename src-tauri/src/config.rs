use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    let base = dirs::data_dir()
        .or_else(|| dirs::home_dir())
        .unwrap_or_else(|| PathBuf::from("."));
    
    // Windows:   C:\Users\Alice\AppData\Roaming\dms-sync
    // macOS:     /Users/Alice/Library/Application Support/dms-sync
    // Linux:     /home/alice/.local/share/dms-sync
    base.join("dms-sync")
}

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
    #[serde(default)]
    pub watched_folder: Option<String>,
    #[serde(default)]
    pub watched_folders: Vec<String>,
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
            watched_folders: Vec::new(),
            auto_start: true,
            folder_cache: Vec::new(),
            sync_log: Vec::new(),
            last_email: None,
            session_cookie: None,
        }
    }
}

impl AppConfig {
    pub fn watched_folders(&self) -> Vec<String> {
        if !self.watched_folders.is_empty() {
            self.watched_folders.clone()
        } else if let Some(ref old) = self.watched_folder {
            vec![old.clone()]
        } else {
            Vec::new()
        }
    }
}

impl AppConfig {
    pub fn path() -> PathBuf {
        let mut p = config_dir();
        p.push("config.json");
        p
    }

    pub fn load() -> Self {
        let p = Self::path();
        let mut config = if p.exists() {
            fs::read_to_string(&p)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        };
        // Migrate old watched_folder to watched_folders
        if config.watched_folders.is_empty() {
            if let Some(ref old) = config.watched_folder {
                config.watched_folders = vec![old.clone()];
            }
        }
        config
    }

    pub fn save(&self) -> Result<(), String> {
        // For save, always clear old watched_folder, use watched_folders only
        let mut data = self.clone();
        data.watched_folder = None;
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        let p = Self::path();
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        fs::write(&p, &json).map_err(|e| format!("Failed to write config: {}", e))?;
        restrict_permissions(&p);
        Ok(())
    }
}

/// The config file holds a live session cookie, so keep it owner-only.
#[cfg(unix)]
fn restrict_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &std::path::Path) {}


