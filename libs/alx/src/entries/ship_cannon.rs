//! Ship cannon entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

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
    pub const ENTRY_SIZE: usize = 36;
    
    // Field offsets (name at 0-16 is NEVER written)
    const OFF_SHIP_FLAGS: usize = 17;
    const OFF_TYPE_ID: usize = 18;
    const OFF_ELEMENT_ID: usize = 19;
    const OFF_ATTACK: usize = 20;
    const OFF_HIT: usize = 22;
    const OFF_LIMIT: usize = 24;
    const OFF_SP: usize = 25;
    const OFF_TRAIT_ID: usize = 26;
    // 27 = pad
    const OFF_TRAIT_VALUE: usize = 28;
    const OFF_BUY_PRICE: usize = 30;
    const OFF_SELL: usize = 32;
    const OFF_ORDER1: usize = 33;
    const OFF_ORDER2: usize = 34;

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

    /// Patch a single cannon entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_SHIP_FLAGS] = self.ship_flags;
        buf[Self::OFF_TYPE_ID] = self.type_id as u8;
        buf[Self::OFF_ELEMENT_ID] = self.element_id as u8;
        buf[Self::OFF_ATTACK..Self::OFF_ATTACK+2].copy_from_slice(&self.attack.to_be_bytes());
        buf[Self::OFF_HIT..Self::OFF_HIT+2].copy_from_slice(&self.hit.to_be_bytes());
        buf[Self::OFF_LIMIT] = self.limit as u8;
        buf[Self::OFF_SP] = self.sp as u8;
        buf[Self::OFF_TRAIT_ID] = self.trait_id as u8;
        buf[Self::OFF_TRAIT_VALUE..Self::OFF_TRAIT_VALUE+2].copy_from_slice(&self.trait_value.to_be_bytes());
        buf[Self::OFF_BUY_PRICE..Self::OFF_BUY_PRICE+2].copy_from_slice(&self.buy_price.to_be_bytes());
        buf[Self::OFF_SELL] = self.sell_percent as u8;
        buf[Self::OFF_ORDER1] = self.order1 as u8;
        buf[Self::OFF_ORDER2] = self.order2 as u8;
    }

    /// Patch all cannon entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for e in entries {
            let idx = (e.id - id_ranges::SHIP_CANNON.start) as usize;
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
        assert_eq!(ShipCannon::ENTRY_SIZE, 36);
    }
}

