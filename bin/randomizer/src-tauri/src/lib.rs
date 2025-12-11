//! Skies of Arcadia Legends Randomizer - Tauri Backend

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

use alx::game::GameRoot;

/// Application state holding the loaded game
pub struct AppState {
    pub game: Mutex<Option<GameRoot>>,
    pub iso_path: Mutex<Option<PathBuf>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            game: Mutex::new(None),
            iso_path: Mutex::new(None),
        }
    }
}

/// Game info returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub version: String,
    pub region: String,
    pub path: String,
}

/// Result type for commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Load an ISO file and detect its version
#[tauri::command]
fn load_iso(path: String, state: State<AppState>) -> CommandResult<GameInfo> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return CommandResult::err("File does not exist");
    }

    match GameRoot::open(&path_buf) {
        Ok(game) => {
            let version = game.version();
            let info = GameInfo {
                version: version.to_string(),
                region: format!("{} ({})", version.region, version.platform),
                path: path.clone(),
            };

            // Store in state
            *state.game.lock().unwrap() = Some(game);
            *state.iso_path.lock().unwrap() = Some(path_buf);

            CommandResult::ok(info)
        }
        Err(e) => CommandResult::err(format!("Failed to load ISO: {}", e)),
    }
}

/// Check if an ISO is currently loaded
#[tauri::command]
fn is_iso_loaded(state: State<AppState>) -> bool {
    state.game.lock().unwrap().is_some()
}

/// Get info about the currently loaded ISO
#[tauri::command]
fn get_game_info(state: State<AppState>) -> CommandResult<GameInfo> {
    let game_lock = state.game.lock().unwrap();
    let path_lock = state.iso_path.lock().unwrap();

    match (&*game_lock, &*path_lock) {
        (Some(game), Some(path)) => {
            let version = game.version();
            CommandResult::ok(GameInfo {
                version: version.to_string(),
                region: format!("{} ({})", version.region, version.platform),
                path: path.display().to_string(),
            })
        }
        _ => CommandResult::err("No ISO loaded"),
    }
}

/// Close the currently loaded ISO
#[tauri::command]
fn close_iso(state: State<AppState>) -> CommandResult<()> {
    *state.game.lock().unwrap() = None;
    *state.iso_path.lock().unwrap() = None;
    CommandResult::ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            load_iso,
            is_iso_loaded,
            get_game_info,
            close_iso,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
