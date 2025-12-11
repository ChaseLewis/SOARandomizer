//! Common test utilities for integration tests.
//!
//! This module provides helpers for loading the game ISO and managing test state.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use alx::{GameRoot, Result};

/// Path to the test ISO (relative to the workspace root).
pub const ISO_PATH: &str = "../../roms/Skies of Arcadia Legends (USA).iso";

/// Check if the test ISO exists.
pub fn iso_exists() -> bool {
    Path::new(ISO_PATH).exists()
}

/// Skip a test if the ISO doesn't exist.
/// Returns `true` if the test should continue, `false` if it should be skipped.
#[macro_export]
macro_rules! skip_if_no_iso {
    () => {
        if !$crate::common::iso_exists() {
            eprintln!(
                "Skipping test: ISO not found at {}",
                $crate::common::ISO_PATH
            );
            return;
        }
    };
}

/// Load the game from the test ISO.
/// Panics if the ISO cannot be loaded.
pub fn load_game() -> GameRoot {
    GameRoot::open(Path::new(ISO_PATH)).expect("Failed to open test ISO")
}

/// Try to load the game, returning a Result.
#[allow(dead_code)]
pub fn try_load_game() -> Result<GameRoot> {
    GameRoot::open(Path::new(ISO_PATH))
}

/// Cached game instance for tests that need to share state.
/// Use `get_cached_game()` to access it.
static CACHED_GAME: OnceLock<std::sync::Mutex<Option<GameRoot>>> = OnceLock::new();

/// Get a cached game instance. This is useful for expensive operations
/// that don't need a fresh game state each time.
/// 
/// Note: The returned game is wrapped in a Mutex, so you'll need to lock it.
/// For most tests, prefer `load_game()` for isolation.
#[allow(dead_code)]
pub fn get_cached_game() -> &'static std::sync::Mutex<Option<GameRoot>> {
    CACHED_GAME.get_or_init(|| {
        if iso_exists() {
            std::sync::Mutex::new(Some(load_game()))
        } else {
            std::sync::Mutex::new(None)
        }
    })
}

// =============================================================================
// Writable ISO Test Fixture
// =============================================================================

/// Path to the writable ISO copy (for roundtrip tests).
pub const WRITABLE_ISO_PATH: &str = "../../roms/Skies of Arcadia Legends (USA) copy.iso";

/// Cached state for the writable ISO fixture.
static WRITABLE_ISO_READY: OnceLock<std::sync::Mutex<WritableIsoState>> = OnceLock::new();

/// State of the writable ISO fixture.
pub struct WritableIsoState {
    /// Whether the copy has been created.
    pub is_ready: bool,
    /// Path to the writable ISO.
    pub path: PathBuf,
}

/// Check if the writable ISO copy exists.
pub fn writable_iso_exists() -> bool {
    Path::new(WRITABLE_ISO_PATH).exists()
}

/// Ensure the writable ISO copy exists.
/// If not, copies from the original ISO.
/// Returns the path to the writable ISO.
pub fn ensure_writable_iso() -> Option<PathBuf> {
    if !iso_exists() {
        return None;
    }

    let state = WRITABLE_ISO_READY.get_or_init(|| {
        let path = PathBuf::from(WRITABLE_ISO_PATH);
        
        // If the copy doesn't exist, create it
        if !path.exists() {
            eprintln!("Creating writable ISO copy (this may take a moment)...");
            if let Err(e) = fs::copy(ISO_PATH, &path) {
                eprintln!("Failed to copy ISO: {}", e);
                return std::sync::Mutex::new(WritableIsoState {
                    is_ready: false,
                    path,
                });
            }
            eprintln!("Writable ISO copy created: {}", path.display());
        }
        
        std::sync::Mutex::new(WritableIsoState {
            is_ready: true,
            path,
        })
    });

    let guard = state.lock().ok()?;
    if guard.is_ready {
        Some(guard.path.clone())
    } else {
        None
    }
}

/// Load the game from the writable ISO copy.
/// Panics if the ISO cannot be loaded.
#[allow(dead_code)]
pub fn load_writable_game() -> GameRoot {
    let path = ensure_writable_iso().expect("Writable ISO not available");
    GameRoot::open(&path).expect("Failed to open writable ISO")
}

/// Skip a test if the writable ISO doesn't exist or can't be created.
#[macro_export]
macro_rules! skip_if_no_writable_iso {
    () => {
        if $crate::common::ensure_writable_iso().is_none() {
            eprintln!(
                "Skipping test: Could not create writable ISO copy at {}",
                $crate::common::WRITABLE_ISO_PATH
            );
            return;
        }
    };
}

/// Calculate CRC32 checksum of a byte slice.
#[allow(dead_code)]
pub fn crc32_checksum(data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data);
    hasher.finalize()
}
