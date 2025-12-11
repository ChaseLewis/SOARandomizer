//! # ALX - Skies of Arcadia Legends Examiner
//!
//! A Rust library for reading, modifying, and writing game data from
//! Skies of Arcadia Legends (GameCube).
//!
//! ## Features
//!
//! - Direct ISO reading/writing via `gc_fst`
//! - Parse all game data types (items, enemies, characters, etc.)
//! - Export to CSV format (compatible with original ALX)
//! - Import from CSV and write back to ISO
//!
//! ## Example
//!
//! ```no_run
//! use alx::GameRoot;
//! use std::path::Path;
//!
//! let mut game = GameRoot::open(Path::new("game.iso")).unwrap();
//! let accessories = game.read_accessories().unwrap();
//! // Modify and write back...
//! ```

pub mod error;
pub mod io;
pub mod game;
pub mod entries;
pub mod csv;
pub mod lookups;
pub mod items;

pub use items::{ItemDatabase, ItemCategory};

pub use error::{Error, Result};
pub use game::GameRoot;
