//! Tacitus library — shared between the Tauri app and the CLI binary.

pub mod db;
pub mod ingest;
pub mod parser;
pub mod playback;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::db::{Db, ReplayCursor, ReplayRow};
use crate::ingest::IngestReport;
use crate::parser::Replay;

pub struct AppState {
    pub db: Arc<Mutex<Db>>,
}

// Locks and disk I/O stay off both the native event loop and async workers.
async fn with_db<T, F>(db: Arc<Mutex<Db>>, operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut Db) -> anyhow::Result<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let mut db = db.lock().map_err(|error| error.to_string())?;
        operation(&mut db).map_err(|error| format!("{error:#}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn parse_replay(path: String) -> Result<Replay, String> {
    tauri::async_runtime::spawn_blocking(move || parser::parse_metadata(PathBuf::from(path)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn parse_replay_full(path: String) -> Result<Replay, String> {
    tauri::async_runtime::spawn_blocking(move || parser::parse_full(PathBuf::from(path)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ingest(state: tauri::State<'_, AppState>, path: String) -> Result<IngestReport, String> {
    with_db(state.db.clone(), move |db| {
        ingest::ingest_path(db, &PathBuf::from(path))
    })
    .await
}

#[tauri::command]
async fn list_replays(
    state: tauri::State<'_, AppState>,
    limit: i64,
    before: Option<ReplayCursor>,
) -> Result<Vec<ReplayRow>, String> {
    with_db(state.db.clone(), move |db| {
        db.list_replays_before(limit.clamp(1, 1000), before)
    })
    .await
}

#[tauri::command]
async fn count_replays(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    with_db(state.db.clone(), |db| db.count_replays()).await
}

#[tauri::command]
fn playback_settings() -> Result<playback::Settings, String> {
    playback::load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_playback_config(path: String) -> Result<playback::Settings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        playback::save_settings(std::path::Path::new(&path))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
async fn check_playback(
    state: tauri::State<'_, AppState>,
    replay_id: i64,
) -> Result<playback::Report, String> {
    let target = with_db(state.db.clone(), move |db| db.replay_target(replay_id)).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let settings = playback::load_settings()?;
        playback::check(&target, settings.sku_path.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e: anyhow::Error| format!("{e:#}"))
}

#[tauri::command]
async fn launch_replay(
    state: tauri::State<'_, AppState>,
    replay_id: i64,
) -> Result<playback::LaunchResult, String> {
    let target = with_db(state.db.clone(), move |db| db.replay_target(replay_id)).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let settings = playback::load_settings()?;
        playback::launch(&target, settings.sku_path.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e: anyhow::Error| format!("{e:#}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let catalogue = ingest::default_catalogue_path();
    let db = Db::open(&catalogue).expect("failed to open catalogue");
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            parse_replay,
            parse_replay_full,
            ingest,
            list_replays,
            count_replays,
            playback_settings,
            set_playback_config,
            check_playback,
            launch_replay,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
