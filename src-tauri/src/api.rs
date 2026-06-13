use reqwest::cookie::CookieStore;
use reqwest::cookie::Jar;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderResponse {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

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

    pub async fn sync_upload(
        &self,
        file_path: &str,
        file_data: Vec<u8>,
        mime_type: &str,
        file_name: &str,
        folder_path: Option<&str>,
    ) -> Result<SyncUploadResponse, String> {
        let url = format!("{}/api/sync/upload", self.base_url);

        let file_part = multipart::Part::bytes(file_data)
            .file_name(file_name.to_string())
            .mime_str(mime_type)
            .map_err(|e| format!("MIME error: {}", e))?;

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
            .map_err(|e| format!("Upload error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Sync upload failed ({}): {}", status, text));
        }

        let data: SyncUploadResponse = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

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
            let cookies = self.get_cookie_string();
            log::error!("Sync test failed ({}): {} | cookies: {}", status, body, cookies);
            Err(format!("FAIL ({}): {} | cookies: {}", status, body, cookies))
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
            // Log the cookie we're sending for debugging
            let cookies = self.get_cookie_string();
            log::warn!("Session check failed ({}): {} | cookies: {}", status, text, cookies);
            return Err(format!("Session invalid ({}): {}", status, text));
        }

        let data: serde_json::Value = resp.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(data)
    }
}
