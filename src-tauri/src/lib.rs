mod api;
mod config;
mod folder_cache;
mod sync;
mod tray;
mod watcher;

use api::DmsApi;
use config::AppConfig;
use sync::SyncEngine;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub engine: Arc<SyncEngine>,
    pub config: Arc<Mutex<AppConfig>>,
    pub watcher: Arc<Mutex<watcher::FileWatcher>>,
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, AppState>,
    email: String,
    password: String,
    server_url: String,
) -> Result<String, String> {
    let api = DmsApi::new(&server_url);
    let _result = api.login(&email, &password).await?;

    // Verify session immediately
    match api.check_session().await {
        Ok(session) => log::info!("Session verified: {:?}", session),
        Err(e) => log::warn!("Session check after login: {}", e),
    }

    let cookie_str = api.get_cookie_string();

    {
        let mut config = state.config.lock().await;
        config.server_url = server_url.clone();
        config.last_email = Some(email.clone());
        config.session_cookie = Some(cookie_str);
        config.save();
    }

    {
        let mut api_lock = state.engine.api.lock().await;
        *api_lock = Some(api);
    }

    // Auto-start watcher if saved watched folder exists
    let watched = {
        let config = state.config.lock().await;
        config.watched_folder.clone()
    };
    if let Some(ref folder) = watched {
        let folder = folder.trim_end_matches(&['/', '\\', ' '][..]);
        let mut watcher_lock = state.watcher.lock().await;
        watcher_lock.stop();
        if let Err(e) = watcher_lock.start_watching(folder) {
            log::warn!("Failed to auto-start watcher: {}", e);
        }
    }

    Ok("Login successful".to_string())
}

#[tauri::command]
async fn check_session(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let api_lock = state.engine.api.lock().await;
    if let Some(ref api) = *api_lock {
        api.check_session().await
    } else {
        Err("Not logged in".to_string())
    }
}

#[tauri::command]
async fn test_sync(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let api_lock = state.engine.api.lock().await;
    let api = api_lock.as_ref().ok_or("Not logged in")?;
    api.test_sync_upload().await
}

#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let sync_status = {
        let status = state.engine.status.lock().await;
        format!("{:?}", status)
    };
    let queue_len = state.engine.queue.lock().await.len();
    let config = state.config.lock().await;
    let watched = config.watched_folder.clone();
    let api_available = state.engine.api.lock().await.is_some();
    let event_count = state.engine.watcher_events.lock().await.len();

    Ok(serde_json::json!({
        "status": sync_status,
        "queue_length": queue_len,
        "watched_folder": watched,
        "server_url": config.server_url,
        "last_email": config.last_email,
        "api_available": api_available,
        "event_count": event_count,
    }))
}

#[tauri::command]
async fn get_sync_log(state: tauri::State<'_, AppState>) -> Result<Vec<config::SyncLogEntry>, String> {
    let logs = state.engine.logs.lock().await;
    Ok(logs.clone())
}

#[tauri::command]
async fn set_watch_folder(state: tauri::State<'_, AppState>, path: String) -> Result<String, String> {
    // Normalize path: remove trailing slashes, use native separators
    let path = path.trim_end_matches(&['/', '\\', ' '][..]).to_string();
    let engine = state.engine.clone();

    {
        *engine.watched_root.lock().await = Some(path.clone());
    }

    {
        let mut watcher_lock = state.watcher.lock().await;
        watcher_lock.stop();
        watcher_lock.start_watching(&path)?;
    }

    {
        let mut config = state.config.lock().await;
        config.watched_folder = Some(path.clone());
        config.save();
    }

    Ok(format!("Watching: {}", path))
}

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
async fn get_folder_cache(state: tauri::State<'_, AppState>) -> Result<Vec<Vec<String>>, String> {
    let cache = state.engine.folder_cache.lock().await;
    let pairs = cache.to_pairs();
    Ok(pairs.into_iter().map(|(k, v)| vec![k, v]).collect())
}

#[tauri::command]
async fn clear_sync_log(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.engine.logs.lock().await.clear();
    Ok(())
}

#[tauri::command]
async fn get_watcher_events(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let ev = state.engine.watcher_events.lock().await;
    let last = ev.iter().rev().take(50).cloned().collect::<Vec<_>>();
    Ok(last.into_iter().rev().collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::load();
    let engine = Arc::new(SyncEngine::new());

    // Restore session from saved cookies
    if let Some(ref cookie_str) = config.session_cookie {
        if !cookie_str.is_empty() {
            let api = DmsApi::new(&config.server_url);
            api.load_cookies(cookie_str);
            *engine.api.blocking_lock() = Some(api);
            log::info!("Restored session from saved cookies");
        }
    }

    let app_state = AppState {
        engine: engine.clone(),
        config: Arc::new(Mutex::new(config)),
        watcher: Arc::new(Mutex::new(watcher::FileWatcher::new(engine.clone()))),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .setup(|app| {
            tray::build_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            login,
            check_session,
            test_sync,
            get_status,
            get_sync_log,
            set_watch_folder,
            get_config,
            get_folder_cache,
            clear_sync_log,
            get_watcher_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
