//! Swashbuckler rating entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

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
    /// 25 + 1 + 2 + 2 + 2 + 2 = 34 bytes
    pub const ENTRY_SIZE: usize = 34;

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

    /// Write a single swashbuckler entry to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 25)?;
        writer.write_u8(self.rating)?;
        
        if version.region == Region::Eu {
            writer.write_u8(0)?;
        }
        
        writer.write_i16_be(self.regular_attack)?;
        writer.write_i16_be(self.super_move_attack)?;
        writer.write_i16_be(self.dodge)?;
        writer.write_i16_be(self.run)?;
        
        if version.region == Region::Eu {
            writer.write_i16_be(0)?;
        }
        
        Ok(())
    }

    /// Write all swashbuckler entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(entries: &[Self], writer: &mut W, version: &GameVersion) -> Result<()> {
        for entry in entries {
            entry.write_one(writer, version)?;
        }
        Ok(())
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

