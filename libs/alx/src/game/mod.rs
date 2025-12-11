//! Game root detection and context management.

pub mod offsets;
pub mod region;
mod root;

pub use offsets::{id_ranges, Offsets};
pub use region::{GameVersion, Platform, Region};
pub use root::GameRoot;
