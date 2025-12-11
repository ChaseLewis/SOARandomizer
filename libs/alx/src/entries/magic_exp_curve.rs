//! Magic EXP curve entry type (magic EXP per element per level for each character).

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::offsets::id_ranges;
use crate::game::region::GameVersion;
use crate::io::BinaryReader;

/// Magic EXP curve for a character (EXP requirements for each magic element).
/// Each element has 6 levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicExpCurve {
    /// Entry ID (character ID)
    pub id: u32,
    /// Character name (looked up)
    pub character_name: String,
    /// Green magic EXP (levels 1-6)
    pub green_exp: [u16; 6],
    /// Red magic EXP (levels 1-6)
    pub red_exp: [u16; 6],
    /// Purple magic EXP (levels 1-6)
    pub purple_exp: [u16; 6],
    /// Blue magic EXP (levels 1-6)
    pub blue_exp: [u16; 6],
    /// Yellow magic EXP (levels 1-6)
    pub yellow_exp: [u16; 6],
    /// Silver magic EXP (levels 1-6)
    pub silver_exp: [u16; 6],
}

impl MagicExpCurve {
    /// Size of one entry in bytes.
    /// 6 elements * 6 levels * 2 bytes (u16) = 72 bytes
    pub const ENTRY_SIZE: usize = 72;

    /// Read a single Magic EXP curve from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let mut green_exp = [0u16; 6];
        let mut red_exp = [0u16; 6];
        let mut purple_exp = [0u16; 6];
        let mut blue_exp = [0u16; 6];
        let mut yellow_exp = [0u16; 6];
        let mut silver_exp = [0u16; 6];

        for i in 0..6 {
            green_exp[i] = cursor.read_u16_be()?;
        }
        for i in 0..6 {
            red_exp[i] = cursor.read_u16_be()?;
        }
        for i in 0..6 {
            purple_exp[i] = cursor.read_u16_be()?;
        }
        for i in 0..6 {
            blue_exp[i] = cursor.read_u16_be()?;
        }
        for i in 0..6 {
            yellow_exp[i] = cursor.read_u16_be()?;
        }
        for i in 0..6 {
            silver_exp[i] = cursor.read_u16_be()?;
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
            green_exp,
            red_exp,
            purple_exp,
            blue_exp,
            yellow_exp,
            silver_exp,
        })
    }

    /// Read all Magic EXP curve entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);

        let id_range = id_ranges::MAGIC_EXP_CURVE;

        for id in id_range {
            if cursor.position() as usize + Self::ENTRY_SIZE > data.len() {
                break;
            }
            let entry = Self::read_one(&mut cursor, id, version)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Patch a single Magic EXP curve entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        let mut offset = 0;

        for &exp in &self.green_exp {
            buf[offset..offset + 2].copy_from_slice(&exp.to_be_bytes());
            offset += 2;
        }
        for &exp in &self.red_exp {
            buf[offset..offset + 2].copy_from_slice(&exp.to_be_bytes());
            offset += 2;
        }
        for &exp in &self.purple_exp {
            buf[offset..offset + 2].copy_from_slice(&exp.to_be_bytes());
            offset += 2;
        }
        for &exp in &self.blue_exp {
            buf[offset..offset + 2].copy_from_slice(&exp.to_be_bytes());
            offset += 2;
        }
        for &exp in &self.yellow_exp {
            buf[offset..offset + 2].copy_from_slice(&exp.to_be_bytes());
            offset += 2;
        }
        for &exp in &self.silver_exp {
            buf[offset..offset + 2].copy_from_slice(&exp.to_be_bytes());
            offset += 2;
        }
    }

    /// Patch all Magic EXP curve entries into a buffer.
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
        assert_eq!(MagicExpCurve::ENTRY_SIZE, 72);
    }
}
