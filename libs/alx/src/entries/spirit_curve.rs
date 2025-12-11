//! Spirit curve entry type (SP/MAXSP per level for each character).

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::offsets::id_ranges;
use crate::game::region::GameVersion;
use crate::io::{BinaryReader, BinaryWriter};

/// SP and MAXSP at a given level.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SpiritLevel {
    /// SP at this level
    pub sp: i8,
    /// Max SP at this level
    pub max_sp: i8,
}

/// Spirit curve for a character (SP/MAXSP progression levels 1-99).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiritCurve {
    /// Entry ID (character ID)
    pub id: u32,
    /// Character name (looked up)
    pub character_name: String,
    /// Spirit values for levels 1-99 (index 0 = level 1)
    pub levels: Vec<SpiritLevel>,
}

impl SpiritCurve {
    /// Size of one entry in bytes.
    /// 99 levels * 2 bytes (SP + MAXSP) = 198 bytes
    pub const ENTRY_SIZE: usize = 198;

    /// Read a single spirit curve from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let mut levels = Vec::with_capacity(99);

        for _ in 0..99 {
            let sp = cursor.read_i8()?;
            let max_sp = cursor.read_i8()?;
            levels.push(SpiritLevel { sp, max_sp });
        }

        // Get character name from ID
        let character_name = match id {
            0 => "Vyse",
            1 => "Aika",
            2 => "Fina",
            3 => "Drachma",
            4 => "Enrique",
            5 => "Gilder",
            _ => "???",
        }
        .to_string();

        Ok(Self {
            id,
            character_name,
            levels,
        })
    }

    /// Read all spirit curve entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);

        let id_range = id_ranges::SPIRIT_CURVE;

        for id in id_range {
            if cursor.position() as usize + Self::ENTRY_SIZE > data.len() {
                break;
            }
            let entry = Self::read_one(&mut cursor, id, version)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Get SP at a specific level (1-99).
    pub fn sp_at_level(&self, level: u8) -> Option<i8> {
        if (1..=99).contains(&level) {
            Some(self.levels[(level - 1) as usize].sp)
        } else {
            None
        }
    }

    /// Get MAXSP at a specific level (1-99).
    pub fn max_sp_at_level(&self, level: u8) -> Option<i8> {
        if (1..=99).contains(&level) {
            Some(self.levels[(level - 1) as usize].max_sp)
        } else {
            None
        }
    }

    /// Write a single spirit curve to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, _version: &GameVersion) -> Result<()> {
        for i in 0..99 {
            let level = self.levels.get(i).copied().unwrap_or_default();
            writer.write_i8(level.sp)?;
            writer.write_i8(level.max_sp)?;
        }
        Ok(())
    }

    /// Patch all spirit curve entries into a buffer.
    pub fn patch_all(_entries: &[Self], _buf: &mut [u8], _version: &GameVersion) {
        // TODO: Implement proper patching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        assert_eq!(SpiritCurve::ENTRY_SIZE, 198);
    }
}
