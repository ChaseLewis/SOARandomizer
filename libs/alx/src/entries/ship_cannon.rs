//! Ship cannon entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// A ship cannon in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipCannon {
    /// Entry ID
    pub id: u32,
    /// Cannon name
    pub name: String,
    /// Ship flags (which ships can equip)
    pub ship_flags: u8,
    /// Type ID (0=Main Cannon, 1=Secondary, 2=Torpedo)
    pub type_id: i8,
    /// Element ID
    pub element_id: i8,
    /// Attack power
    pub attack: i16,
    /// Hit percentage
    pub hit: u16,
    /// Limit (usage limit)
    pub limit: i8,
    /// SP cost
    pub sp: i8,
    /// Trait ID
    pub trait_id: i8,
    /// Trait value
    pub trait_value: i16,
    /// Buy price
    pub buy_price: u16,
    /// Sell percentage
    pub sell_percent: i8,
    /// Sort order 1
    pub order1: i8,
    /// Sort order 2
    pub order2: i8,
    /// Description text
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
}

impl ShipCannon {
    /// Size of one entry in bytes (US/JP).
    /// 17 + 1 + 1 + 1 + 2 + 2 + 1 + 1 + 1 + 1 + 2 + 2 + 1 + 1 + 1 + 1 = 36 bytes
    pub const ENTRY_SIZE: usize = 36;

    /// Get type name.
    pub fn type_name(&self) -> &'static str {
        match self.type_id {
            0 => "Main Cannon",
            1 => "Secondary Cannon",
            2 => "Torpedo",
            _ => "???",
        }
    }

    /// Read a single cannon from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let ship_flags = cursor.read_u8()?;
        let type_id = cursor.read_i8()?;
        let element_id = cursor.read_i8()?;
        
        // EU has extra padding
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let attack = cursor.read_i16_be()?;
        let hit = cursor.read_u16_be()?;
        let limit = cursor.read_i8()?;
        let sp = cursor.read_i8()?;
        let trait_id = cursor.read_i8()?;
        let _pad1 = cursor.read_u8()?;
        let trait_value = cursor.read_i16_be()?;
        let buy_price = cursor.read_u16_be()?;
        let sell_percent = cursor.read_i8()?;
        let order1 = cursor.read_i8()?;
        let order2 = cursor.read_i8()?;
        let _pad2 = cursor.read_u8()?;
        
        Ok(Self {
            id,
            name,
            ship_flags,
            type_id,
            element_id,
            attack,
            hit,
            limit,
            sp,
            trait_id,
            trait_value,
            buy_price,
            sell_percent,
            order1,
            order2,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all cannon entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::SHIP_CANNON;
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
            Region::Eu => Self::ENTRY_SIZE + 1,
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Check if a ship can equip this cannon.
    pub fn can_equip_ship(&self, ship_id: u8) -> bool {
        // Bits: L1=5, L2=4, D1=3, D2=2, D3=1
        let bit = 0x20 >> ship_id;
        (self.ship_flags & bit) != 0
    }

    /// Write a single cannon to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 17)?;
        writer.write_u8(self.ship_flags)?;
        writer.write_i8(self.type_id)?;
        writer.write_i8(self.element_id)?;
        
        if version.region == Region::Eu {
            writer.write_u8(0)?;
        }
        
        writer.write_i16_be(self.attack)?;
        writer.write_u16_be(self.hit)?;
        writer.write_i8(self.limit)?;
        writer.write_i8(self.sp)?;
        writer.write_i8(self.trait_id)?;
        writer.write_u8(0)?; // pad1
        writer.write_i16_be(self.trait_value)?;
        writer.write_u16_be(self.buy_price)?;
        writer.write_i8(self.sell_percent)?;
        writer.write_i8(self.order1)?;
        writer.write_i8(self.order2)?;
        writer.write_u8(0)?; // pad2
        
        Ok(())
    }

    /// Write all cannon entries to binary data.
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
        assert_eq!(ShipCannon::ENTRY_SIZE, 36);
    }
}

