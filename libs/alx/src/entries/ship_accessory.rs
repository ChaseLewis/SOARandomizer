//! Ship accessory entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};
use crate::entries::Trait;

/// A ship accessory in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipAccessory {
    /// Entry ID
    pub id: u32,
    /// Accessory name
    pub name: String,
    /// Ship flags (which ships can equip)
    pub ship_flags: u8,
    /// Traits (up to 4)
    pub traits: [Trait; 4],
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

impl ShipAccessory {
    /// Size of one entry in bytes (US/JP).
    /// 17 + 1 + 16 (4 traits * 4 bytes) + 2 + 1 + 1 + 1 + 1 = 40 bytes
    pub const ENTRY_SIZE: usize = 40;

    /// Read a single accessory from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let ship_flags = cursor.read_u8()?;
        
        // EU has extra padding
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let mut traits = [Trait::default(); 4];
        for i in 0..4 {
            let trait_id = cursor.read_i8()?;
            let _pad = cursor.read_u8()?;
            let trait_value = cursor.read_i16_be()?;
            traits[i] = Trait { id: trait_id, value: trait_value };
        }
        
        let buy_price = cursor.read_u16_be()?;
        let sell_percent = cursor.read_i8()?;
        let order1 = cursor.read_i8()?;
        let order2 = cursor.read_i8()?;
        let _pad = cursor.read_u8()?;
        
        Ok(Self {
            id,
            name,
            ship_flags,
            traits,
            buy_price,
            sell_percent,
            order1,
            order2,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all accessory entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::SHIP_ACCESSORY;
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

    /// Check if a ship can equip this accessory.
    pub fn can_equip_ship(&self, ship_id: u8) -> bool {
        let bit = 0x20 >> ship_id;
        (self.ship_flags & bit) != 0
    }

    /// Write a single accessory to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 17)?;
        writer.write_u8(self.ship_flags)?;
        if version.region == Region::Eu { writer.write_u8(0)?; }
        for t in &self.traits {
            writer.write_i8(t.id)?;
            writer.write_u8(0)?;
            writer.write_i16_be(t.value)?;
        }
        writer.write_u16_be(self.buy_price)?;
        writer.write_i8(self.sell_percent)?;
        writer.write_i8(self.order1)?;
        writer.write_i8(self.order2)?;
        writer.write_u8(0)?;
        Ok(())
    }

    /// Write all accessory entries to binary data.
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
        assert_eq!(ShipAccessory::ENTRY_SIZE, 40);
    }
}

