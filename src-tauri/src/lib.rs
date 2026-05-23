//! Tacitus library — shared between the Tauri app, the CLI binary, and tests.

pub mod parser;

use std::path::PathBuf;

#[tauri::command]
fn parse_replay(path: String) -> Result<parser::Replay, String> {
    parser::parse_metadata(PathBuf::from(path)).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![parse_replay])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
