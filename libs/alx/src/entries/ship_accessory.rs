//! Ship accessory entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;
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
    pub const ENTRY_SIZE: usize = 40;
    
    // Field offsets (name at 0-16 is NEVER written)
    const OFF_SHIP_FLAGS: usize = 17;
    const OFF_TRAITS: usize = 18; // 4 traits * 4 bytes
    const OFF_BUY_PRICE: usize = 34;
    const OFF_SELL: usize = 36;
    const OFF_ORDER1: usize = 37;
    const OFF_ORDER2: usize = 38;

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

    /// Patch a single accessory entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_SHIP_FLAGS] = self.ship_flags;
        for (i, t) in self.traits.iter().enumerate() {
            let off = Self::OFF_TRAITS + i * 4;
            buf[off] = t.id as u8;
            buf[off+2..off+4].copy_from_slice(&t.value.to_be_bytes());
        }
        buf[Self::OFF_BUY_PRICE..Self::OFF_BUY_PRICE+2].copy_from_slice(&self.buy_price.to_be_bytes());
        buf[Self::OFF_SELL] = self.sell_percent as u8;
        buf[Self::OFF_ORDER1] = self.order1 as u8;
        buf[Self::OFF_ORDER2] = self.order2 as u8;
    }

    /// Patch all accessory entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for e in entries {
            let idx = (e.id - id_ranges::SHIP_ACCESSORY.start) as usize;
            let start = idx * entry_size;
            let end = start + entry_size;
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
        assert_eq!(ShipAccessory::ENTRY_SIZE, 40);
    }
}

