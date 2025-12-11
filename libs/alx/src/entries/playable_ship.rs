//! Playable ship entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

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
    pub const ENTRY_SIZE: usize = 100;
    
    // Field offsets (name at 0-19 is NEVER written)
    const OFF_MAX_HP: usize = 20;
    const OFF_MAX_SP: usize = 24;
    const OFF_SP: usize = 26;
    const OFF_DEFENSE: usize = 28;
    const OFF_MAG_DEF: usize = 30;
    const OFF_QUICK: usize = 32;
    const OFF_DODGE: usize = 34;
    const OFF_ELEMENTS: usize = 36; // 6 * 2 bytes
    const OFF_CANNONS: usize = 48; // 5 * 2 bytes
    const OFF_ACCESSORIES: usize = 58; // 3 * 2 bytes
    const OFF_VALUE: usize = 64;
    // 68-71 = pad
    const OFF_MAX_HP_GROWTH: usize = 72;
    const OFF_MAX_SP_GROWTH: usize = 76;
    const OFF_SP_GROWTH: usize = 78;
    const OFF_DEF_GROWTH: usize = 80;
    const OFF_MAG_DEF_GROWTH: usize = 82;
    const OFF_QUICK_GROWTH: usize = 84;
    const OFF_DODGE_GROWTH: usize = 86;

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

    /// Patch a single ship entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_MAX_HP..Self::OFF_MAX_HP+4].copy_from_slice(&self.max_hp.to_be_bytes());
        buf[Self::OFF_MAX_SP..Self::OFF_MAX_SP+2].copy_from_slice(&self.max_sp.to_be_bytes());
        buf[Self::OFF_SP..Self::OFF_SP+2].copy_from_slice(&self.sp.to_be_bytes());
        buf[Self::OFF_DEFENSE..Self::OFF_DEFENSE+2].copy_from_slice(&self.defense.to_be_bytes());
        buf[Self::OFF_MAG_DEF..Self::OFF_MAG_DEF+2].copy_from_slice(&self.mag_def.to_be_bytes());
        buf[Self::OFF_QUICK..Self::OFF_QUICK+2].copy_from_slice(&self.quick.to_be_bytes());
        buf[Self::OFF_DODGE..Self::OFF_DODGE+2].copy_from_slice(&self.dodge.to_be_bytes());
        for (i, &e) in self.elements.iter().enumerate() {
            let off = Self::OFF_ELEMENTS + i * 2;
            buf[off..off+2].copy_from_slice(&e.to_be_bytes());
        }
        for (i, &c) in self.cannon_ids.iter().enumerate() {
            let off = Self::OFF_CANNONS + i * 2;
            buf[off..off+2].copy_from_slice(&c.to_be_bytes());
        }
        for (i, &a) in self.accessory_ids.iter().enumerate() {
            let off = Self::OFF_ACCESSORIES + i * 2;
            buf[off..off+2].copy_from_slice(&a.to_be_bytes());
        }
        buf[Self::OFF_VALUE..Self::OFF_VALUE+4].copy_from_slice(&self.value.to_be_bytes());
        buf[Self::OFF_MAX_HP_GROWTH..Self::OFF_MAX_HP_GROWTH+4].copy_from_slice(&self.max_hp_growth.to_be_bytes());
        buf[Self::OFF_MAX_SP_GROWTH..Self::OFF_MAX_SP_GROWTH+2].copy_from_slice(&self.max_sp_growth.to_be_bytes());
        buf[Self::OFF_SP_GROWTH..Self::OFF_SP_GROWTH+2].copy_from_slice(&self.sp_growth.to_be_bytes());
        buf[Self::OFF_DEF_GROWTH..Self::OFF_DEF_GROWTH+2].copy_from_slice(&self.defense_growth.to_be_bytes());
        buf[Self::OFF_MAG_DEF_GROWTH..Self::OFF_MAG_DEF_GROWTH+2].copy_from_slice(&self.mag_def_growth.to_be_bytes());
        buf[Self::OFF_QUICK_GROWTH..Self::OFF_QUICK_GROWTH+2].copy_from_slice(&self.quick_growth.to_be_bytes());
        buf[Self::OFF_DODGE_GROWTH..Self::OFF_DODGE_GROWTH+2].copy_from_slice(&self.dodge_growth.to_be_bytes());
    }

    /// Patch all ship entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8]) {
        for e in entries {
            let idx = (e.id - id_ranges::PLAYABLE_SHIP.start) as usize;
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
        assert_eq!(PlayableShip::ENTRY_SIZE, 100);
    }
}

