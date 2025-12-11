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

// Allow range loops for binary reading - clippy's suggestion doesn't work with cursor reads
#![allow(clippy::needless_range_loop)]
// Allow many arguments for data-heavy functions
#![allow(clippy::too_many_arguments)]

pub mod csv;
pub mod entries;
pub mod error;
pub mod game;
pub mod io;
pub mod items;
pub mod lookups;

pub use items::{ItemCategory, ItemDatabase};

pub use error::{Error, Result};
pub use game::GameRoot;
