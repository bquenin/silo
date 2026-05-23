//! Tacitus library — shared between the Tauri app and the CLI binary.

pub mod db;
pub mod ingest;
pub mod parser;

use std::path::PathBuf;
use std::sync::Mutex;

use crate::db::{Db, ReplayRow};
use crate::ingest::IngestReport;
use crate::parser::Replay;

pub struct AppState {
    pub db: Mutex<Db>,
}

#[tauri::command]
fn parse_replay(path: String) -> Result<Replay, String> {
    parser::parse_metadata(PathBuf::from(path)).map_err(|e| e.to_string())
}

#[tauri::command]
fn parse_replay_full(path: String) -> Result<Replay, String> {
    parser::parse_full(PathBuf::from(path)).map_err(|e| e.to_string())
}

#[tauri::command]
fn ingest(state: tauri::State<'_, AppState>, path: String) -> Result<IngestReport, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    ingest::ingest_path(&mut db, &PathBuf::from(path)).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_replays(state: tauri::State<'_, AppState>, limit: i64) -> Result<Vec<ReplayRow>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_replays(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn count_replays(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.count_replays().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let catalogue = ingest::default_catalogue_path();
    let db = Db::open(&catalogue).expect("failed to open catalogue");
    let state = AppState { db: Mutex::new(db) };

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
