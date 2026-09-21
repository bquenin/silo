//! Tacitus library — shared between the Tauri app and the CLI binary.

pub mod db;
pub mod file_association;
pub mod ingest;
pub mod parser;
pub mod playback;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};

use crate::db::{Db, ReplayCursor, ReplayRow};
use crate::ingest::IngestReport;
use crate::parser::Replay;

pub struct AppState {
    pub db: Arc<Mutex<Db>>,
    pub playback_jobs: Arc<Mutex<PlaybackJobs>>,
    pub pending_replays: Arc<Mutex<VecDeque<String>>>,
}

fn replay_argument(args: impl IntoIterator<Item = String>) -> Option<String> {
    args.into_iter().find(|argument| {
        Path::new(argument)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("kwreplay"))
    })
}

#[derive(Default)]
pub struct PlaybackJobs {
    active: Option<(String, Arc<playback::automatic::Cancellation>)>,
    cancelled: std::collections::VecDeque<String>,
}

impl PlaybackJobs {
    fn start(&mut self, id: &str) -> Result<Arc<playback::automatic::Cancellation>, String> {
        if let Some(index) = self.cancelled.iter().position(|cancelled| cancelled == id) {
            self.cancelled.remove(index);
            return Err("Replay preparation cancelled.".into());
        }
        if self.active.is_some() {
            return Err("Another replay is being prepared.".into());
        }
        let cancellation = Arc::new(playback::automatic::Cancellation::default());
        self.active = Some((id.into(), cancellation.clone()));
        Ok(cancellation)
    }

    fn cancel(&mut self, id: String) -> bool {
        if let Some((active, cancellation)) = &self.active {
            if active == &id {
                return cancellation.cancel();
            }
        }
        // Cancellation can arrive before the async launch command starts.
        self.cancelled.push_back(id);
        while self.cancelled.len() > 64 {
            self.cancelled.pop_front();
        }
        true
    }
}

struct PlaybackTicket {
    id: String,
    jobs: Arc<Mutex<PlaybackJobs>>,
}
impl Drop for PlaybackTicket {
    fn drop(&mut self) {
        if let Ok(mut jobs) = self.jobs.lock() {
            if jobs.active.as_ref().is_some_and(|(id, _)| id == &self.id) {
                jobs.active = None;
            }
        }
    }
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
async fn playback_settings() -> Result<playback::Settings, String> {
    tauri::async_runtime::spawn_blocking(playback::load_settings)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_game_installation(path: String) -> Result<playback::Settings, String> {
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
) -> Result<playback::automatic::Readiness, String> {
    let target = with_db(state.db.clone(), move |db| db.replay_target(replay_id)).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let settings = playback::load_settings()?;
        Ok(playback::automatic::inspect(
            &target,
            settings.game_path.as_deref(),
            &playback::automatic::cache_root(),
        ))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e: anyhow::Error| format!("{e:#}"))
}

#[tauri::command]
async fn launch_replay(
    state: tauri::State<'_, AppState>,
    replay_id: i64,
    request_id: String,
    on_progress: tauri::ipc::Channel<playback::automatic::Progress>,
) -> Result<playback::LaunchResult, String> {
    if request_id.is_empty() || request_id.len() > 128 {
        return Err("Invalid playback request".into());
    }
    let cancellation = state
        .playback_jobs
        .lock()
        .map_err(|e| e.to_string())?
        .start(&request_id)?;
    let ticket = PlaybackTicket {
        id: request_id,
        jobs: state.playback_jobs.clone(),
    };
    let target = with_db(state.db.clone(), move |db| db.replay_target(replay_id)).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let _ticket = ticket;
        let settings = playback::load_settings()?;
        let progress = |event| {
            if on_progress.send(event).is_err() {
                cancellation.cancel();
            }
        };
        let control = playback::automatic::Control {
            cancelled: &cancellation,
            progress: &progress,
        };
        let game_path = settings
            .game_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Choose your Kane's Wrath game folder."))?;
        playback::automatic::launch(
            &target,
            game_path,
            &playback::automatic::cache_root(),
            &control,
        )
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e: anyhow::Error| format!("{e:#}"))
}

#[tauri::command]
fn cancel_playback(state: tauri::State<'_, AppState>, request_id: String) -> Result<bool, String> {
    if request_id.is_empty() || request_id.len() > 128 {
        return Err("Invalid playback request".into());
    }
    Ok(state
        .playback_jobs
        .lock()
        .map_err(|e| e.to_string())?
        .cancel(request_id))
}

#[tauri::command]
fn replay_file_association() -> Result<file_association::AssociationStatus, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    file_association::status(&executable).map_err(|error| format!("{error:#}"))
}

#[tauri::command]
fn associate_replay_files() -> Result<file_association::AssociationStatus, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    file_association::associate(&executable).map_err(|error| format!("{error:#}"))
}

#[tauri::command]
fn take_pending_replay(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state
        .pending_replays
        .lock()
        .map_err(|error| error.to_string())?
        .pop_front())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let catalogue = ingest::default_catalogue_path();
    let db = Db::open(&catalogue).expect("failed to open catalogue");
    let pending_replays = Arc::new(Mutex::new(VecDeque::new()));
    if let Some(path) = replay_argument(std::env::args().skip(1)) {
        pending_replays
            .lock()
            .expect("pending replay lock poisoned")
            .push_back(path);
    }
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
        playback_jobs: Arc::new(Mutex::new(PlaybackJobs::default())),
        pending_replays,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let Some(path) = replay_argument(args) else {
                return;
            };
            if let Ok(mut pending) = app.state::<AppState>().pending_replays.lock() {
                pending.push_back(path);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            let _ = app.emit("replay-open-requested", ());
        }))
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
            set_game_installation,
            check_playback,
            launch_replay,
            cancel_playback,
            replay_file_association,
            associate_replay_files,
            take_pending_replay,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod playback_job_tests {
    use super::*;

    #[test]
    fn early_cancel_and_ticket_drop_release_only_the_matching_request() {
        let jobs = Arc::new(Mutex::new(PlaybackJobs::default()));
        jobs.lock().unwrap().cancel("early".into());
        assert!(jobs.lock().unwrap().start("early").is_err());
        jobs.lock().unwrap().start("first").unwrap();
        assert!(jobs.lock().unwrap().start("second").is_err());
        drop(PlaybackTicket {
            id: "second".into(),
            jobs: jobs.clone(),
        });
        assert!(jobs.lock().unwrap().active.is_some());
        assert!(jobs.lock().unwrap().cancel("first".into()));
        drop(PlaybackTicket {
            id: "first".into(),
            jobs: jobs.clone(),
        });
        assert!(jobs.lock().unwrap().start("second").is_ok());
    }

    #[test]
    fn replay_argument_is_case_insensitive_and_ignores_other_arguments() {
        assert_eq!(
            replay_argument(["--flag".into(), r"C:\Replays\match.KWReplay".into()]),
            Some(r"C:\Replays\match.KWReplay".into()),
        );
        assert_eq!(
            replay_argument(["tacitus.exe".into(), "notes.txt".into()]),
            None
        );
    }
}
