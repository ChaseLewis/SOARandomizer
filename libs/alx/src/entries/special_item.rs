//! Special item entry type (key items, moon crystals, etc.)

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// Special item entry (key items, moon crystals, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialItem {
    /// Entry ID
    pub id: u32,
    /// Entry name (US region)
    pub name: String,
    /// Sell percentage
    pub sell_percent: i8,
    /// Sort order 1
    pub order1: i8,
    /// Sort order 2
    pub order2: i8,
    /// Buy price
    pub buy_price: u16,
    /// Description text
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
}

impl Default for SpecialItem {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            sell_percent: 0,
            order1: -1,
            order2: -1,
            buy_price: 0,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        }
    }
}

impl SpecialItem {
    /// Size of one special item entry in bytes (JP/US).
    /// 17 + 1 + 1 + 1 + 2 = 22 bytes
    pub const ENTRY_SIZE: usize = 22;

    /// Read a single special item entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let sell_percent = cursor.read_i8()?;
        let order1 = cursor.read_i8()?;
        let order2 = cursor.read_i8()?;
        
        // EU has extra padding here
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let buy_price = cursor.read_u16_be()?;
        
        // EU has extra padding at the end
        if version.region == Region::Eu {
            let _pad1 = cursor.read_u8()?;
            let _pad2 = cursor.read_u8()?;
        }
        
        Ok(Self {
            id,
            name,
            sell_percent,
            order1,
            order2,
            buy_price,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all special item entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut items = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::SPECIAL_ITEM;
        let entry_size = Self::entry_size_for_version(version);
        
        for id in id_range {
            if cursor.position() as usize + entry_size > data.len() {
                break;
            }
            let item = Self::read_one(&mut cursor, id, version)?;
            items.push(item);
        }
        
        Ok(items)
    }

    fn entry_size_for_version(version: &GameVersion) -> usize {
        match version.region {
            Region::Eu => Self::ENTRY_SIZE + 4, // EU has extra padding
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Write a single special item entry to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 17)?;
        writer.write_i8(self.sell_percent)?;
        writer.write_i8(self.order1)?;
        writer.write_i8(self.order2)?;
        
        if version.region == Region::Eu {
            writer.write_u8(0)?;
        }
        
        writer.write_u16_be(self.buy_price)?;
        
        if version.region == Region::Eu {
            writer.write_u8(0)?;
            writer.write_u8(0)?;
        }
        
        Ok(())
    }

    /// Write all special item entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(items: &[Self], writer: &mut W, version: &GameVersion) -> Result<()> {
        for item in items {
            item.write_one(writer, version)?;
        }
        Ok(())
    }
}
