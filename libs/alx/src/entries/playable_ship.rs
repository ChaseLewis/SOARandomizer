//! Playable ship entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// A playable ship in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayableShip {
    /// Entry ID
    pub id: u32,
    /// Ship name
    pub name: String,
    /// Maximum HP
    pub max_hp: u32,
    /// Maximum SP
    pub max_sp: i16,
    /// Current SP
    pub sp: i16,
    /// Defense stat
    pub defense: i16,
    /// Magic Defense stat
    pub mag_def: i16,
    /// Quick stat
    pub quick: i16,
    /// Dodge percentage
    pub dodge: i16,
    /// Elemental resistances (Green, Red, Purple, Blue, Yellow, Silver)
    pub elements: [i16; 6],
    /// Equipped cannon IDs (up to 5)
    pub cannon_ids: [i16; 5],
    /// Equipped accessory IDs (up to 3)
    pub accessory_ids: [i16; 3],
    /// Ship value (cost)
    pub value: u32,
    /// Growth stats
    pub max_hp_growth: i32,
    pub max_sp_growth: i16,
    pub sp_growth: i16,
    pub defense_growth: i16,
    pub mag_def_growth: i16,
    pub quick_growth: i16,
    pub dodge_growth: i16,
}

impl PlayableShip {
    /// Size of one entry in bytes (US/JP).
    /// 20 + 4 + 2 + 2 + 2 + 2 + 2 + 2 + 12 + 10 + 6 + 4 + 4 + 4 + 2 + 2 + 2 + 2 + 2 + 2 + 12 = 100 bytes
    pub const ENTRY_SIZE: usize = 100;

    /// Read a single ship from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(20)?;
        let max_hp = cursor.read_u32_be()?;
        let max_sp = cursor.read_i16_be()?;
        let sp = cursor.read_i16_be()?;
        let defense = cursor.read_i16_be()?;
        let mag_def = cursor.read_i16_be()?;
        let quick = cursor.read_i16_be()?;
        let dodge = cursor.read_i16_be()?;
        
        let mut elements = [0i16; 6];
        for i in 0..6 {
            elements[i] = cursor.read_i16_be()?;
        }
        
        let mut cannon_ids = [0i16; 5];
        for i in 0..5 {
            cannon_ids[i] = cursor.read_i16_be()?;
        }
        
        let mut accessory_ids = [0i16; 3];
        for i in 0..3 {
            accessory_ids[i] = cursor.read_i16_be()?;
        }
        
        let value = cursor.read_u32_be()?;
        let _pad1 = cursor.read_i16_be()?;
        let _pad2 = cursor.read_i16_be()?;
        
        let max_hp_growth = cursor.read_i32_be()?;
        let max_sp_growth = cursor.read_i16_be()?;
        let sp_growth = cursor.read_i16_be()?;
        let defense_growth = cursor.read_i16_be()?;
        let mag_def_growth = cursor.read_i16_be()?;
        let quick_growth = cursor.read_i16_be()?;
        let dodge_growth = cursor.read_i16_be()?;
        
        // Remaining padding (6 i16 values)
        let _pad = [
            cursor.read_i16_be()?,
            cursor.read_i16_be()?,
            cursor.read_i16_be()?,
            cursor.read_i16_be()?,
            cursor.read_i16_be()?,
            cursor.read_i16_be()?,
        ];
        
        Ok(Self {
            id,
            name,
            max_hp,
            max_sp,
            sp,
            defense,
            mag_def,
            quick,
            dodge,
            elements,
            cannon_ids,
            accessory_ids,
            value,
            max_hp_growth,
            max_sp_growth,
            sp_growth,
            defense_growth,
            mag_def_growth,
            quick_growth,
            dodge_growth,
        })
    }

    /// Read all ship entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::PLAYABLE_SHIP;
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

    fn entry_size_for_version(_version: &GameVersion) -> usize {
        Self::ENTRY_SIZE
    }

    /// Write a single ship to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, _version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 20)?;
        writer.write_u32_be(self.max_hp)?;
        writer.write_i16_be(self.max_sp)?;
        writer.write_i16_be(self.sp)?;
        writer.write_i16_be(self.defense)?;
        writer.write_i16_be(self.mag_def)?;
        writer.write_i16_be(self.quick)?;
        writer.write_i16_be(self.dodge)?;
        for &e in &self.elements { writer.write_i16_be(e)?; }
        for &c in &self.cannon_ids { writer.write_i16_be(c)?; }
        for &a in &self.accessory_ids { writer.write_i16_be(a)?; }
        writer.write_u32_be(self.value)?;
        writer.write_i16_be(0)?; writer.write_i16_be(0)?;
        writer.write_i32_be(self.max_hp_growth)?;
        writer.write_i16_be(self.max_sp_growth)?;
        writer.write_i16_be(self.sp_growth)?;
        writer.write_i16_be(self.defense_growth)?;
        writer.write_i16_be(self.mag_def_growth)?;
        writer.write_i16_be(self.quick_growth)?;
        writer.write_i16_be(self.dodge_growth)?;
        for _ in 0..6 { writer.write_i16_be(0)?; }
        Ok(())
    }

    /// Write all ship entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(entries: &[Self], writer: &mut W, version: &GameVersion) -> Result<()> {
        for e in entries { e.write_one(writer, version)?; }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        assert_eq!(PlayableShip::ENTRY_SIZE, 100);
    }
}

