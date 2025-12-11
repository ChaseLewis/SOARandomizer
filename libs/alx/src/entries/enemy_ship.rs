//! Enemy ship entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// An armament slot on an enemy ship.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ShipArmament {
    /// Type ID (0=Main Cannon, 1=Secondary, 2=Torpedo)
    pub type_id: i16,
    /// Attack power
    pub attack: i16,
    /// Range
    pub range: i16,
    /// Hit percentage
    pub hit: i16,
    /// Element ID
    pub element_id: i16,
}

/// An item drop from an enemy ship.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ShipItemDrop {
    /// Drop rate ID
    pub drop_id: i16,
    /// Item ID
    pub item_id: i16,
}

/// An enemy ship in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyShip {
    /// Entry ID
    pub id: u32,
    /// Ship name
    pub name: String,
    /// Maximum HP
    pub max_hp: i32,
    /// Will stat
    pub will: i16,
    /// Defense stat
    pub defense: i16,
    /// Magic Defense stat
    pub mag_def: i16,
    /// Quick stat
    pub quick: i16,
    /// Agile stat
    pub agile: i16,
    /// Dodge percentage
    pub dodge: i16,
    /// Elemental resistances (Green, Red, Purple, Blue, Yellow, Silver)
    pub elements: [i16; 6],
    /// Armaments (up to 4)
    pub armaments: [ShipArmament; 4],
    /// Experience reward
    pub exp: i32,
    /// Gold reward
    pub gold: i32,
    /// Item drops (up to 3)
    pub item_drops: [ShipItemDrop; 3],
}

impl EnemyShip {
    /// Size of one entry in bytes (US/JP).
    /// 20 + 4 + 2 + 2 + 2 + 2 + 2 + 2 + 12 + 40 (4 arms * 10 bytes) + 12 + 4 + 4 + 12 = 120 bytes
    pub const ENTRY_SIZE: usize = 120;

    /// Read a single enemy ship from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(20)?;
        let max_hp = cursor.read_i32_be()?;
        let will = cursor.read_i16_be()?;
        let defense = cursor.read_i16_be()?;
        let mag_def = cursor.read_i16_be()?;
        let quick = cursor.read_i16_be()?;
        let agile = cursor.read_i16_be()?;
        let dodge = cursor.read_i16_be()?;
        
        let mut elements = [0i16; 6];
        for i in 0..6 {
            elements[i] = cursor.read_i16_be()?;
        }
        
        let mut armaments = [ShipArmament::default(); 4];
        for i in 0..4 {
            armaments[i] = ShipArmament {
                type_id: cursor.read_i16_be()?,
                attack: cursor.read_i16_be()?,
                range: cursor.read_i16_be()?,
                hit: cursor.read_i16_be()?,
                element_id: cursor.read_i16_be()?,
            };
        }
        
        // Padding (6 i16 values)
        for _ in 0..6 {
            let _pad = cursor.read_i16_be()?;
        }
        
        let exp = cursor.read_i32_be()?;
        let gold = cursor.read_i32_be()?;
        
        let mut item_drops = [ShipItemDrop::default(); 3];
        for i in 0..3 {
            item_drops[i] = ShipItemDrop {
                drop_id: cursor.read_i16_be()?,
                item_id: cursor.read_i16_be()?,
            };
        }
        
        Ok(Self {
            id,
            name,
            max_hp,
            will,
            defense,
            mag_def,
            quick,
            agile,
            dodge,
            elements,
            armaments,
            exp,
            gold,
            item_drops,
        })
    }

    /// Read all enemy ship entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::ENEMY_SHIP;
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

    /// Write a single enemy ship to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, _version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 20)?;
        writer.write_i32_be(self.max_hp)?;
        writer.write_i16_be(self.will)?;
        writer.write_i16_be(self.defense)?;
        writer.write_i16_be(self.mag_def)?;
        writer.write_i16_be(self.quick)?;
        writer.write_i16_be(self.agile)?;
        writer.write_i16_be(self.dodge)?;
        for &e in &self.elements { writer.write_i16_be(e)?; }
        for a in &self.armaments {
            writer.write_i16_be(a.type_id)?;
            writer.write_i16_be(a.attack)?;
            writer.write_i16_be(a.range)?;
            writer.write_i16_be(a.hit)?;
            writer.write_i16_be(a.element_id)?;
        }
        for _ in 0..6 { writer.write_i16_be(0)?; }
        writer.write_i32_be(self.exp)?;
        writer.write_i32_be(self.gold)?;
        for d in &self.item_drops {
            writer.write_i16_be(d.drop_id)?;
            writer.write_i16_be(d.item_id)?;
        }
        Ok(())
    }

    /// Write all enemy ship entries to binary data.
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
        assert_eq!(EnemyShip::ENTRY_SIZE, 120);
    }
}

