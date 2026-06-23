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

    // Start watcher for all saved watched folders (after login session restored)
    let folders = {
        let config = state.config.lock().await;
        config.watched_folders()
    };
    {
        let mut watcher_lock = state.watcher.lock().await;
        for folder in &folders {
            let folder = folder.trim_end_matches(&['/', '\\', ' '][..]);
            if let Err(e) = watcher_lock.add_watch(folder) {
                log::warn!("Failed to start watcher for {}: {}", folder, e);
            }
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
    let watched_folders = config.watched_folders();
    let api_available = state.engine.api.lock().await.is_some();
    let event_count = state.engine.watcher_events.lock().await.len();

    Ok(serde_json::json!({
        "status": sync_status,
        "queue_length": queue_len,
        "watched_folders": watched_folders,
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
async fn add_watch_folder(state: tauri::State<'_, AppState>, path: String) -> Result<String, String> {
    let path = path.trim_end_matches(&['/', '\\', ' '][..]).to_string();

    // Check duplicate
    {
        let roots = state.engine.watched_roots.lock().await;
        if roots.contains(&path) {
            return Err("Folder already being watched".to_string());
        }
    }

    // Add to engine roots
    {
        let mut roots = state.engine.watched_roots.lock().await;
        roots.push(path.clone());
    }

    // Start watching
    {
        let mut watcher_lock = state.watcher.lock().await;
        watcher_lock.add_watch(&path)?;
    }

    // Save to config
    {
        let mut config = state.config.lock().await;
        if !config.watched_folders.contains(&path) {
            config.watched_folders.push(path.clone());
            config.save();
        }
    }

    Ok(format!("Now watching: {}", path))
}

#[tauri::command]
async fn remove_watch_folder(state: tauri::State<'_, AppState>, path: String) -> Result<String, String> {
    let path = path.trim_end_matches(&['/', '\\', ' '][..]).to_string();

    // Remove from watcher
    {
        let mut watcher_lock = state.watcher.lock().await;
        watcher_lock.remove_watch(&path)?;
    }

    // Remove from engine roots
    {
        let mut roots = state.engine.watched_roots.lock().await;
        roots.retain(|r| r != &path);
    }

    // Remove from config
    {
        let mut config = state.config.lock().await;
        config.watched_folders.retain(|f| f != &path);
        config.save();
    }

    Ok(format!("Stopped watching: {}", path))
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

    // Start watcher for all saved folders, then store roots in engine
    let mut file_watcher = watcher::FileWatcher::new(engine.clone());
    {
        let folders = config.watched_folders();
        for folder in &folders {
            if let Err(e) = file_watcher.add_watch(folder) {
                log::warn!("Failed to start watcher for {}: {}", folder, e);
            }
        }
        let mut roots = engine.watched_roots.blocking_lock();
        *roots = folders;
    }

    let app_state = AppState {
        engine: engine.clone(),
        config: Arc::new(Mutex::new(config)),
        watcher: Arc::new(Mutex::new(file_watcher)),
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
            add_watch_folder,
            remove_watch_folder,
            get_config,
            get_folder_cache,
            clear_sync_log,
            get_watcher_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
