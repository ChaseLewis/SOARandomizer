//! Ship item entry type.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::offsets::id_ranges;
use crate::game::region::{GameVersion, Region};
use crate::io::BinaryReader;

/// A ship item (consumable) in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipItem {
    /// Entry ID
    pub id: u32,
    /// Item name
    pub name: String,
    /// Occasion flags (Menu/Battle/Ship)
    pub occasion_flags: u8,
    /// Ship effect ID
    pub ship_effect_id: i8,
    /// Ship effect duration in turns
    pub ship_effect_turns: i8,
    /// Consume percentage
    pub consume: i8,
    /// Buy price
    pub buy_price: u16,
    /// Sell percentage
    pub sell_percent: i8,
    /// Sort order 1
    pub order1: i8,
    /// Sort order 2
    pub order2: i8,
    /// Ship effect base value
    pub ship_effect_base: i16,
    /// Element ID
    pub element_id: i8,
    /// Unknown value 1
    pub unknown1: i8,
    /// Unknown value 2
    pub unknown2: i16,
    /// Hit percentage
    pub hit: i16,
    /// Description text
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
}

impl ShipItem {
    /// Size of one entry in bytes (US/JP).
    pub const ENTRY_SIZE: usize = 36;

    // Field offsets (name at 0-16 is NEVER written)
    const OFF_OCCASION: usize = 17;
    const OFF_SHIP_EFFECT_ID: usize = 18;
    const OFF_SHIP_EFFECT_TURNS: usize = 19;
    const OFF_CONSUME: usize = 20;
    // 21 = pad
    const OFF_BUY_PRICE: usize = 22;
    const OFF_SELL: usize = 24;
    const OFF_ORDER1: usize = 25;
    const OFF_ORDER2: usize = 26;
    // 27 = pad
    const OFF_SHIP_EFFECT_BASE: usize = 28;
    const OFF_ELEMENT_ID: usize = 30;
    const OFF_UNK1: usize = 31;
    const OFF_UNK2: usize = 32;
    const OFF_HIT: usize = 34;

    /// Read a single ship item from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let occasion_flags = cursor.read_u8()?;
        let ship_effect_id = cursor.read_i8()?;
        let ship_effect_turns = cursor.read_i8()?;
        let consume = cursor.read_i8()?;
        let _pad1 = cursor.read_u8()?;
        let buy_price = cursor.read_u16_be()?;
        let sell_percent = cursor.read_i8()?;
        let order1 = cursor.read_i8()?;
        let order2 = cursor.read_i8()?;
        let _pad2 = cursor.read_u8()?;

        // EU has extra padding
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
            let _pad = cursor.read_u8()?;
        }

        let ship_effect_base = cursor.read_i16_be()?;
        let element_id = cursor.read_i8()?;
        let unknown1 = cursor.read_i8()?;
        let unknown2 = cursor.read_i16_be()?;
        let hit = cursor.read_i16_be()?;

        Ok(Self {
            id,
            name,
            occasion_flags,
            ship_effect_id,
            ship_effect_turns,
            consume,
            buy_price,
            sell_percent,
            order1,
            order2,
            ship_effect_base,
            element_id,
            unknown1,
            unknown2,
            hit,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all ship item entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);

        let id_range = id_ranges::SHIP_ITEM;
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
            Region::Eu => Self::ENTRY_SIZE + 2,
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Check if usable in menu
    pub fn usable_in_menu(&self) -> bool {
        (self.occasion_flags & 0x04) != 0
    }

    /// Check if usable in battle
    pub fn usable_in_battle(&self) -> bool {
        (self.occasion_flags & 0x02) != 0
    }

    /// Check if usable on ship
    pub fn usable_on_ship(&self) -> bool {
        (self.occasion_flags & 0x01) != 0
    }

    /// Patch a single ship item entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_OCCASION] = self.occasion_flags;
        buf[Self::OFF_SHIP_EFFECT_ID] = self.ship_effect_id as u8;
        buf[Self::OFF_SHIP_EFFECT_TURNS] = self.ship_effect_turns as u8;
        buf[Self::OFF_CONSUME] = self.consume as u8;
        buf[Self::OFF_BUY_PRICE..Self::OFF_BUY_PRICE + 2]
            .copy_from_slice(&self.buy_price.to_be_bytes());
        buf[Self::OFF_SELL] = self.sell_percent as u8;
        buf[Self::OFF_ORDER1] = self.order1 as u8;
        buf[Self::OFF_ORDER2] = self.order2 as u8;
        buf[Self::OFF_SHIP_EFFECT_BASE..Self::OFF_SHIP_EFFECT_BASE + 2]
            .copy_from_slice(&self.ship_effect_base.to_be_bytes());
        buf[Self::OFF_ELEMENT_ID] = self.element_id as u8;
        buf[Self::OFF_UNK1] = self.unknown1 as u8;
        buf[Self::OFF_UNK2..Self::OFF_UNK2 + 2].copy_from_slice(&self.unknown2.to_be_bytes());
        buf[Self::OFF_HIT..Self::OFF_HIT + 2].copy_from_slice(&self.hit.to_be_bytes());
    }

    /// Patch all ship item entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for e in entries {
            let idx = (e.id - id_ranges::SHIP_ITEM.start) as usize;
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
        assert_eq!(ShipItem::ENTRY_SIZE, 36);
    }
}
