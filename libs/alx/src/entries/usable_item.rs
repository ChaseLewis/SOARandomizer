//! Usable item entry type (potions, crystals, etc.)

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// Occasion flags for when an item can be used.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct OccasionFlags(pub u8);

impl OccasionFlags {
    /// Can be used in menu.
    pub fn can_use_menu(&self) -> bool { (self.0 & 0x04) != 0 }
    /// Can be used in battle.
    pub fn can_use_battle(&self) -> bool { (self.0 & 0x02) != 0 }
    /// Can be used in ship battle.
    pub fn can_use_ship(&self) -> bool { (self.0 & 0x01) != 0 }
    
    /// Format as binary string (e.g., "0b0110").
    pub fn as_binary_string(&self) -> String {
        format!("0b{:04b}", self.0 & 0x07)
    }
}

/// Usable item entry (potions, crystals, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsableItem {
    /// Entry ID
    pub id: u32,
    /// Entry name (US region)
    pub name: String,
    /// Occasion flags (Menu/Battle/Ship)
    pub occasion_flags: OccasionFlags,
    /// Effect ID
    pub effect_id: i8,
    /// Scope ID (single/all targets)
    pub scope_id: u8,
    /// Consume percentage
    pub consume_percent: i8,
    /// Sell percentage
    pub sell_percent: i8,
    /// Sort order 1
    pub order1: i8,
    /// Sort order 2
    pub order2: i8,
    /// Buy price
    pub buy_price: u16,
    /// Effect base value
    pub effect_base: i16,
    /// Element ID
    pub element_id: i8,
    /// Type ID
    pub type_id: i8,
    /// State ID
    pub state_id: i16,
    /// State miss percentage
    pub state_miss: i16,
    /// Description text
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
}

impl Default for UsableItem {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            occasion_flags: OccasionFlags::default(),
            effect_id: -1,
            scope_id: 0,
            consume_percent: 0,
            sell_percent: 0,
            order1: -1,
            order2: -1,
            buy_price: 0,
            effect_base: 0,
            element_id: -1,
            type_id: -1,
            state_id: 0,
            state_miss: 0,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        }
    }
}

impl UsableItem {
    /// Size of one usable item entry in bytes (JP/US).
    /// 17 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 1 + 1 + 2 + 1 + 1 + 2 + 2 = 36 bytes
    pub const ENTRY_SIZE: usize = 36;

    /// Get effect name for this item's effect ID.
    pub fn effect_name(&self) -> &'static str {
        match self.effect_id {
            -1 => "None",
            31 => "Recover HP",
            32 => "Recover HP of 100%",
            48 => "Recover MP",
            49 => "Recover MP of 100%",
            50 => "Revive",
            51 => "Cure Poison",
            52 => "Cure Stone",
            53 => "Cure Unconsciousness",
            54 => "Cure Sleep",
            55 => "Cure Confusion",
            56 => "Cure Silence",
            57 => "Cure Fatigue",
            58 => "Cure All Ailments",
            63 => "Attack",
            64 => "Attack All",
            _ => "???",
        }
    }

    /// Get scope name for this item's scope ID.
    pub fn scope_name(&self) -> &'static str {
        match self.scope_id {
            0 => "None",
            1 => "Single PC",
            2 => "All PCs",
            3 => "Single Enemy",
            4 => "All Enemies",
            _ => "???",
        }
    }

    /// Get element name for this item's element ID.
    pub fn element_name(&self) -> &'static str {
        crate::lookups::element_name(self.element_id)
    }

    /// Read a single usable item entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let occasion_flags = OccasionFlags(cursor.read_u8()?);
        let effect_id = cursor.read_i8()?;
        let scope_id = cursor.read_u8()?;
        let consume_percent = cursor.read_i8()?;
        let sell_percent = cursor.read_i8()?;
        let order1 = cursor.read_i8()?;
        let order2 = cursor.read_i8()?;
        
        // EU has extra padding here
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let buy_price = cursor.read_u16_be()?;
        let _pad1 = cursor.read_u8()?;
        let _pad2 = cursor.read_u8()?;
        let effect_base = cursor.read_i16_be()?;
        let element_id = cursor.read_i8()?;
        let type_id = cursor.read_i8()?;
        let state_id = cursor.read_i16_be()?;
        let state_miss = cursor.read_i16_be()?;
        
        Ok(Self {
            id,
            name,
            occasion_flags,
            effect_id,
            scope_id,
            consume_percent,
            sell_percent,
            order1,
            order2,
            buy_price,
            effect_base,
            element_id,
            type_id,
            state_id,
            state_miss,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all usable item entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut items = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::USABLE_ITEM;
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
            Region::Eu => Self::ENTRY_SIZE + 1,
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Write a single usable item entry to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 17)?;
        writer.write_u8(self.occasion_flags.0)?;
        writer.write_i8(self.effect_id)?;
        writer.write_u8(self.scope_id)?;
        writer.write_i8(self.consume_percent)?;
        writer.write_i8(self.sell_percent)?;
        writer.write_i8(self.order1)?;
        writer.write_i8(self.order2)?;
        
        if version.region == Region::Eu {
            writer.write_u8(0)?;
        }
        
        writer.write_u16_be(self.buy_price)?;
        writer.write_u8(0)?; // pad1
        writer.write_u8(0)?; // pad2
        writer.write_i16_be(self.effect_base)?;
        writer.write_i8(self.element_id)?;
        writer.write_i8(self.type_id)?;
        writer.write_i16_be(self.state_id)?;
        writer.write_i16_be(self.state_miss)?;
        
        Ok(())
    }

    /// Write all usable item entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(items: &[Self], writer: &mut W, version: &GameVersion) -> Result<()> {
        for item in items {
            item.write_one(writer, version)?;
        }
        Ok(())
    }
}
