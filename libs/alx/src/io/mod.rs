//! Binary I/O utilities for reading/writing game data.

mod aklz;
mod binary;
mod enp;
mod enp_builder;
mod enp_dump;
mod iso;
mod strings;

pub use aklz::{compress as compress_aklz, decompress as decompress_aklz, is_aklz};
pub use binary::{BinaryReader, BinaryWriter};
pub use enp::{parse_dat_file, parse_enp, parse_evp, patch_enp_encounters, EnpData};
pub use enp_builder::{build_enp, EnemyDatabase, GlobalEnemyDatabase, RawEnemyData};
pub use enp_dump::{
    dump_enp, dump_enp_editable, EncounterDefinition, EncounterDump, EnemyDefinition, EnemyDump,
    EnemyStatsDef, EnpDefinition, EnpDump, HeaderEntry, ItemDropDef,
};
pub use iso::{IsoFile, IsoFileEntry};
pub use strings::{decode_windows1252, read_description_strings};
