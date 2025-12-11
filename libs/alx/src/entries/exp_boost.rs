//! Experience boost entry type (starting EXP for late-joining characters).

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// Experience boost for a character that joins late.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpBoost {
    /// Entry ID (character ID: 3=Drachma, 4=Enrique, 5=Gilder)
    pub id: u32,
    /// Character name
    pub character_name: String,
    /// Base EXP
    pub exp: u32,
    /// Green element EXP
    pub green_exp: u32,
    /// Red element EXP
    pub red_exp: u32,
    /// Purple element EXP
    pub purple_exp: u32,
    /// Blue element EXP
    pub blue_exp: u32,
    /// Yellow element EXP
    pub yellow_exp: u32,
    /// Silver element EXP
    pub silver_exp: u32,
}

impl ExpBoost {
    /// Size of one entry in bytes.
    /// 7 * 4 bytes (u32) = 28 bytes
    pub const ENTRY_SIZE: usize = 28;

    /// Read a single exp boost from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let exp = cursor.read_u32_be()?;
        let green_exp = cursor.read_u32_be()?;
        let red_exp = cursor.read_u32_be()?;
        let purple_exp = cursor.read_u32_be()?;
        let blue_exp = cursor.read_u32_be()?;
        let yellow_exp = cursor.read_u32_be()?;
        let silver_exp = cursor.read_u32_be()?;
        
        // Get character name from ID
        let character_name = match id {
            3 => "Drachma",
            4 => "Enrique",
            5 => "Gilder",
            _ => "???",
        }.to_string();
        
        Ok(Self {
            id,
            character_name,
            exp,
            green_exp,
            red_exp,
            purple_exp,
            blue_exp,
            yellow_exp,
            silver_exp,
        })
    }

    /// Read all exp boost entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::EXP_BOOST;
        
        for id in id_range {
            if cursor.position() as usize + Self::ENTRY_SIZE > data.len() {
                break;
            }
            let entry = Self::read_one(&mut cursor, id, version)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }

    /// Write a single exp boost to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, _version: &GameVersion) -> Result<()> {
        writer.write_u32_be(self.exp)?;
        writer.write_u32_be(self.green_exp)?;
        writer.write_u32_be(self.red_exp)?;
        writer.write_u32_be(self.purple_exp)?;
        writer.write_u32_be(self.blue_exp)?;
        writer.write_u32_be(self.yellow_exp)?;
        writer.write_u32_be(self.silver_exp)?;
        Ok(())
    }

    /// Patch all exp boost entries into a buffer.
    pub fn patch_all(_entries: &[Self], _buf: &mut [u8], _version: &GameVersion) {
        // TODO: Implement proper patching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        assert_eq!(ExpBoost::ENTRY_SIZE, 28);
    }
}

