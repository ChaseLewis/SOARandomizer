//! Swashbuckler rating entry type.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::offsets::id_ranges;
use crate::game::region::{GameVersion, Region};
use crate::io::BinaryReader;

/// A swashbuckler rating in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swashbuckler {
    /// Entry ID
    pub id: u32,
    /// Rating name (e.g., "Vyse the Legend")
    pub name: String,
    /// Rating threshold value
    pub rating: u8,
    /// Regular attack modifier
    pub regular_attack: i16,
    /// S-Move attack modifier
    pub super_move_attack: i16,
    /// Dodge modifier
    pub dodge: i16,
    /// Run modifier
    pub run: i16,
}

impl Swashbuckler {
    /// Size of one entry in bytes (US/JP).
    pub const ENTRY_SIZE: usize = 34;

    // Field offsets (name at 0-24 is NEVER written)
    const OFF_RATING: usize = 25;
    const OFF_REG_ATTACK: usize = 26;
    const OFF_SUPER_ATTACK: usize = 28;
    const OFF_DODGE: usize = 30;
    const OFF_RUN: usize = 32;

    /// Read a single swashbuckler rating from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(25)?;
        let rating = cursor.read_u8()?;

        // EU has extra padding
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }

        let regular_attack = cursor.read_i16_be()?;
        let super_move_attack = cursor.read_i16_be()?;
        let dodge = cursor.read_i16_be()?;
        let run = cursor.read_i16_be()?;

        // EU has extra padding at end
        if version.region == Region::Eu {
            let _pad = cursor.read_i16_be()?;
        }

        Ok(Self {
            id,
            name,
            rating,
            regular_attack,
            super_move_attack,
            dodge,
            run,
        })
    }

    /// Read all swashbuckler entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);

        let id_range = id_ranges::SWASHBUCKLER_GC;
        let entry_size = Self::entry_size_for_version(version);

        for id in id_range {
            if cursor.position() as usize + entry_size > data.len() {
                break;
            }
            let entry = Self::read_one(&mut cursor, id, version)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    fn entry_size_for_version(version: &GameVersion) -> usize {
        match version.region {
            Region::Eu => Self::ENTRY_SIZE + 4,
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Patch a single swashbuckler entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_RATING] = self.rating;
        buf[Self::OFF_REG_ATTACK..Self::OFF_REG_ATTACK + 2]
            .copy_from_slice(&self.regular_attack.to_be_bytes());
        buf[Self::OFF_SUPER_ATTACK..Self::OFF_SUPER_ATTACK + 2]
            .copy_from_slice(&self.super_move_attack.to_be_bytes());
        buf[Self::OFF_DODGE..Self::OFF_DODGE + 2].copy_from_slice(&self.dodge.to_be_bytes());
        buf[Self::OFF_RUN..Self::OFF_RUN + 2].copy_from_slice(&self.run.to_be_bytes());
    }

    /// Patch all swashbuckler entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for e in entries {
            let idx = (e.id - id_ranges::SWASHBUCKLER_GC.start) as usize;
            let start = idx * entry_size;
            let end = start + entry_size;
            if end <= buf.len() {
                e.patch_entry(&mut buf[start..end]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        assert_eq!(Swashbuckler::ENTRY_SIZE, 34);
    }
}
