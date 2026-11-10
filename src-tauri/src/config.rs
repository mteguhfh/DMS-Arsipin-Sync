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
#[serde(rename_all = "camelCase")]
pub struct TargetFolder {
    /// Local watched root this target applies to.
    pub local_path: String,
    /// DMS folder id; empty means "derive from the local folder name".
    pub dms_folder_id: String,
    /// Human readable path in DMS, for display only.
    pub dms_folder_path: String,
}

/// Stored on disk as snake_case and sent to the frontend as camelCase.
///
/// The disk format must stay snake_case: existing installs already have
/// `server_url`/`session_cookie` written that way, and changing it silently
/// drops the saved session. Deserialize accepts both spellings so installs
/// that ran a camelCase build recover automatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
    #[serde(alias = "serverUrl")]
    pub server_url: String,
    #[serde(default, alias = "watchedFolder")]
    pub watched_folder: Option<String>,
    #[serde(default, alias = "watchedFolders")]
    pub watched_folders: Vec<String>,
    #[serde(default, alias = "folderTargets")]
    pub folder_targets: Vec<TargetFolder>,
    #[serde(alias = "autoStart")]
    pub auto_start: bool,
    #[serde(default, alias = "folderCache")]
    pub folder_cache: Vec<FolderEntry>,
    #[serde(default, alias = "syncLog")]
    pub sync_log: Vec<SyncLogEntry>,
    #[serde(default, alias = "lastEmail")]
    pub last_email: Option<String>,
    #[serde(default, alias = "sessionCookie")]
    pub session_cookie: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "https://dms.arsipin.id".to_string(),
            watched_folder: None,
            watched_folders: Vec::new(),
            folder_targets: Vec::new(),
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_legacy_snake_case() {
        let raw = r#"{"server_url":"https://a","auto_start":true,"folder_cache":[],
            "sync_log":[],"last_email":"x@y","session_cookie":"tok"}"#;
        let c: AppConfig = serde_json::from_str(raw).unwrap();
        assert_eq!(c.server_url, "https://a");
        assert_eq!(c.last_email.as_deref(), Some("x@y"));
        assert_eq!(c.session_cookie.as_deref(), Some("tok"));
    }

    #[test]
    fn reads_camel_case_written_by_earlier_build() {
        let raw = r#"{"serverUrl":"https://a","autoStart":true,"folderCache":[],
            "syncLog":[],"lastEmail":"x@y","sessionCookie":"tok","watchedFolders":["/tmp"]}"#;
        let c: AppConfig = serde_json::from_str(raw).unwrap();
        assert_eq!(c.server_url, "https://a");
        assert_eq!(c.session_cookie.as_deref(), Some("tok"));
        assert_eq!(c.watched_folders, vec!["/tmp".to_string()]);
    }

    #[test]
    fn writes_snake_case() {
        let json = serde_json::to_string(&AppConfig::default()).unwrap();
        assert!(json.contains("\"server_url\""));
        assert!(json.contains("\"session_cookie\""));
        assert!(!json.contains("\"serverUrl\""));
    }
}
