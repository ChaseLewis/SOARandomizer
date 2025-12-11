//! Game root detection and context management.

mod root;
pub mod region;
pub mod offsets;

pub use root::GameRoot;
pub use region::{Platform, Region, GameVersion};
pub use offsets::{Offsets, id_ranges};

