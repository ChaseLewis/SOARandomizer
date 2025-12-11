//! EXP curve entry type (EXP required per level for each character).

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// EXP curve for a character (EXP requirements for levels 1-99).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpCurve {
    /// Entry ID (character ID)
    pub id: u32,
    /// Character name (looked up)
    pub character_name: String,
    /// EXP required for levels 1-99 (index 0 = level 1)
    pub exp_values: Vec<i32>,
}

impl ExpCurve {
    /// Size of one entry in bytes.
    /// 99 levels * 4 bytes (i32) = 396 bytes
    pub const ENTRY_SIZE: usize = 396;

    /// Read a single EXP curve from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let mut exp_values = Vec::with_capacity(99);
        
        for _ in 0..99 {
            let exp = cursor.read_i32_be()?;
            exp_values.push(exp);
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
        }.to_string();
        
        Ok(Self {
            id,
            character_name,
            exp_values,
        })
    }

    /// Read all EXP curve entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::EXP_CURVE;
        
        for id in id_range {
            if cursor.position() as usize + Self::ENTRY_SIZE > data.len() {
                break;
            }
            let entry = Self::read_one(&mut cursor, id, version)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }

    /// Get EXP required at a specific level (1-99).
    pub fn exp_at_level(&self, level: u8) -> Option<i32> {
        if level >= 1 && level <= 99 {
            Some(self.exp_values[(level - 1) as usize])
        } else {
            None
        }
    }

    /// Write a single EXP curve entry to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W) -> Result<()> {
        for i in 0..99 {
            let exp = self.exp_values.get(i).copied().unwrap_or(0);
            writer.write_i32_be(exp)?;
        }
        Ok(())
    }

    /// Patch a single EXP curve entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        for (i, &exp) in self.exp_values.iter().take(99).enumerate() {
            let offset = i * 4;
            if offset + 4 <= buf.len() {
                buf[offset..offset+4].copy_from_slice(&exp.to_be_bytes());
            }
        }
    }

    /// Patch all EXP curve entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8]) {
        for e in entries {
            let idx = e.id as usize;
            let start = idx * Self::ENTRY_SIZE;
            let end = start + Self::ENTRY_SIZE;
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
        assert_eq!(ExpCurve::ENTRY_SIZE, 396);
    }
}

