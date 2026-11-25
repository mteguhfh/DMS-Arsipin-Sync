use reqwest::cookie::CookieStore;
use reqwest::cookie::Jar;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncUploadResponse {
    pub document_id: String,
    pub document_number: Option<String>,
    pub key: String,
    pub url: String,
    pub size: i64,
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: Option<serde_json::Value>,
    pub session: Option<serde_json::Value>,
    /// Better Auth two-factor plugin sends this instead of a session.
    #[serde(default, rename = "twoFactorRedirect")]
    pub two_factor_redirect: bool,
}

#[derive(Debug, Clone)]
pub struct UploadError {
    pub status: Option<u16>,
    pub message: String,
}

impl UploadError {
    pub fn is_retryable(&self) -> bool {
        match self.status {
            None => true,
            Some(429) | Some(500..=506) | Some(508..=599) => true,
            _ => false,
        }
    }
}

impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// `GET /api/auth/get-session` mengembalikan `null` dengan status 200 bila
/// tidak ada sesi. Status saja tidak cukup untuk menyatakan login berhasil.
pub fn session_is_valid(data: &serde_json::Value) -> bool {
    data.is_object() && (data.get("session").is_some() || data.get("user").is_some())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderResponse {
    pub id: String,
    pub name: String,
    /// The DMS API sends `parentId`; older builds expected `parent_id`.
    #[serde(default, alias = "parent_id")]
    pub parent_id: Option<String>,
    /// Password-protected folders reject uploads without an unlock.
    #[serde(default)]
    pub has_password: bool,
}

#[derive(Clone)]
pub struct DmsApi {
    client: reqwest::Client,
    base_url: String,
    cookie_jar: Arc<Jar>,
}

impl DmsApi {
    pub fn new(base_url: &str) -> Self {
        let cookie_jar = Arc::new(Jar::default());
        let client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .user_agent("DMS-Sync/1.0")
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            cookie_jar,
        }
    }

    pub fn load_cookies(&self, cookie_str: &str) {
        let url = format!("{}/", self.base_url);
        let parsed = url.parse().unwrap();
        for part in cookie_str.split(';') {
            let _ = self.cookie_jar.add_cookie_str(part.trim(), &parsed);
        }
    }

    pub fn get_cookie_string(&self) -> String {
        let url = format!("{}/", self.base_url);
        self.cookie_jar
            .cookies(&url.parse().unwrap())
            .and_then(|v| v.to_str().ok().map(|s| s.to_string()))
            .unwrap_or_default()
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse, String> {
        let url = format!("{}/api/auth/sign-in/email", self.base_url);
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Login failed ({}): {}", status, text));
        }

        let data: LoginResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(data)
    }

    pub async fn verify_totp(&self, code: &str) -> Result<(), String> {
        let url = format!("{}/api/auth/two-factor/verify-totp", self.base_url);
        let body = serde_json::json!({ "code": code });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Kode 2FA ditolak ({}): {}", status, text));
        }

        // Better Auth mengembalikan JSON `{ status: true }`; cookie sesi sudah
        // masuk jar lewat response ini, jadi body tidak perlu diparse.
        let _ = resp.text().await;
        Ok(())
    }

    pub async fn sync_upload(
        &self,
        file_path: &str,
        file_data: Vec<u8>,
        mime_type: &str,
        file_name: &str,
        folder_path: Option<&str>,
    ) -> Result<SyncUploadResponse, UploadError> {
        let url = format!("{}/api/sync/upload", self.base_url);

        let file_part = multipart::Part::bytes(file_data)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| UploadError {
                status: Some(400),
                message: format!("MIME error: {}", e),
            })?;

        let mut form = multipart::Form::new().part("file", file_part);

        if let Some(fp) = folder_path {
            form = form.text("folderPath", fp.to_string());
        }

        form = form.text("name", file_name.to_string());
        form = form.text("originalPath", file_path.to_string());

        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| UploadError {
                status: None,
                message: format!("Upload error: {}", e),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(UploadError {
                status: Some(status),
                message: format!("Sync upload failed ({}): {}", status, text),
            });
        }

        let data: SyncUploadResponse = resp.json().await.map_err(|e| UploadError {
            status: Some(200),
            message: format!("Parse error: {}", e),
        })?;

        Ok(data)
    }

    pub async fn get_folders(&self) -> Result<Vec<FolderResponse>, String> {
        let url = format!("{}/api/folders", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Get folders failed: {}", resp.status()));
        }

        let folders: Vec<FolderResponse> = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(folders)
    }

    pub async fn check_connection(&self) -> Result<bool, String> {
        let url = format!("{}/api/auth/sign-in/email", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        Ok(resp.status().is_success() || resp.status().as_u16() == 405)
    }

    pub async fn test_sync_upload(&self) -> Result<String, String> {
        let url = format!("{}/api/sync/upload", self.base_url);
        let content = b"DMS Sync Test File - please delete if seen";
        let file_name = format!("_dms_sync_test_{}.txt", chrono::Utc::now().format("%Y%m%d%H%M%S"));

        let file_part = reqwest::multipart::Part::bytes(content.to_vec())
            .file_name(file_name.clone())
            .mime_str("text/plain")
            .map_err(|e| format!("MIME error: {}", e))?;

        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("name", file_name);

        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        if status.is_success() {
            Ok(format!("OK ({}): {}", status, body))
        } else {
            let has_cookie = !self.get_cookie_string().is_empty();
            log::error!("Sync test failed ({}): {} | session cookie present: {}", status, body, has_cookie);
            Err(format!("FAIL ({}): {}", status, body))
        }
    }

    pub async fn check_session(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/auth/get-session", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            // Log only whether a cookie was sent, never its value
            let has_cookie = !self.get_cookie_string().is_empty();
            log::warn!("Session check failed ({}): {} | session cookie present: {}", status, text, has_cookie);
            return Err(format!("Session invalid ({}): {}", status, text));
        }

        let data: serde_json::Value = resp.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        if !session_is_valid(&data) {
            return Err("Session invalid: no active session".to_string());
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(status: Option<u16>) -> UploadError {
        UploadError {
            status,
            message: "test".to_string(),
        }
    }

    #[test]
    fn retry_only_transient_upload_errors() {
        assert!(err(None).is_retryable());
        assert!(err(Some(429)).is_retryable());
        assert!(err(Some(500)).is_retryable());
        assert!(err(Some(503)).is_retryable());
        assert!(!err(Some(400)).is_retryable());
        assert!(!err(Some(401)).is_retryable());
        assert!(!err(Some(403)).is_retryable());
        assert!(!err(Some(507)).is_retryable());
    }

    #[test]
    fn session_requires_object_with_user_or_session() {
        assert!(!session_is_valid(&serde_json::json!(null)));
        assert!(!session_is_valid(&serde_json::json!({})));
        assert!(session_is_valid(&serde_json::json!({ "user": {}, "session": {} })));
    }
}
