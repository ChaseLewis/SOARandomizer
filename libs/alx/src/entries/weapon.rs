//! Weapon entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use super::traits::Trait;
use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

/// Weapon entry.
/// 
/// Structure (JP/US, 32 bytes):
/// - Name: 17 bytes (Shift-JIS)
/// - Character ID: i8 (which character can equip)
/// - Sell%: i8
/// - Order 1: i8
/// - Order 2: i8
/// - Effect ID: i8
/// - Buy: u16
/// - Attack: i16
/// - Hit%: i16
/// - Trait ID: i8
/// - Padding: i8
/// - Trait Value: i16
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    /// Entry ID
    pub id: u32,
    /// Entry name (US region)
    pub name: String,
    /// Character ID that can equip (0=Vyse, 1=Aika, 2=Fina, etc.)
    pub character_id: i8,
    /// Sell percentage
    pub sell_percent: i8,
    /// Sort order 1
    pub order1: i8,
    /// Sort order 2
    pub order2: i8,
    /// Weapon effect ID (-1 = none)
    pub effect_id: i8,
    /// Buy price
    pub buy_price: u16,
    /// Attack power
    pub attack: i16,
    /// Hit percentage
    pub hit_percent: i16,
    /// Equipment trait
    pub trait_data: Trait,
    /// Description text
    pub description: String,
    /// Description position in DOL (for reference)
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            character_id: 0,
            sell_percent: 0,
            order1: -1,
            order2: -1,
            effect_id: -1,
            buy_price: 0,
            attack: 0,
            hit_percent: 0,
            trait_data: Trait::default(),
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        }
    }
}

impl Weapon {
    /// Size of one weapon entry in bytes (JP/US).
    /// 17 + 1 + 1 + 1 + 1 + 1 + 2 + 2 + 2 + 1 + 1 + 2 = 32 bytes
    pub const ENTRY_SIZE: usize = 32;
    
    // Field offsets (name at 0-16 is NEVER written)
    const OFF_CHAR_ID: usize = 17;
    const OFF_SELL_PERCENT: usize = 18;
    const OFF_ORDER1: usize = 19;
    const OFF_ORDER2: usize = 20;
    const OFF_EFFECT_ID: usize = 21;
    const OFF_BUY_PRICE: usize = 22;
    const OFF_ATTACK: usize = 24;
    const OFF_HIT_PERCENT: usize = 26;
    const OFF_TRAIT_ID: usize = 28;
    // 29 = pad
    const OFF_TRAIT_VALUE: usize = 30;

    /// Get the character name for this weapon's character ID.
    pub fn character_name(&self) -> &'static str {
        match self.character_id {
            0 => "Vyse",
            1 => "Aika",
            2 => "Fina",
            3 => "Drachma",
            4 => "Enrique",
            5 => "Gilder",
            _ => "Unknown",
        }
    }

    /// Get the effect name for this weapon's effect ID.
    pub fn effect_name(&self) -> &'static str {
        if self.effect_id < 0 {
            "None"
        } else {
            // Effect names would need to be looked up from weapon_effect data
            "Effect"
        }
    }

    /// Read a single weapon entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let character_id = cursor.read_i8()?;
        let sell_percent = cursor.read_i8()?;
        let order1 = cursor.read_i8()?;
        let order2 = cursor.read_i8()?;
        let effect_id = cursor.read_i8()?;
        
        // EU has extra padding here
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let buy_price = cursor.read_u16_be()?;
        let attack = cursor.read_i16_be()?;
        let hit_percent = cursor.read_i16_be()?;
        
        // Single trait
        let trait_id = cursor.read_i8()?;
        let _pad = cursor.read_u8()?;
        let trait_value = cursor.read_i16_be()?;
        
        Ok(Self {
            id,
            name,
            character_id,
            sell_percent,
            order1,
            order2,
            effect_id,
            buy_price,
            attack,
            hit_percent,
            trait_data: Trait { id: trait_id, value: trait_value },
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all weapon entries from binary data (without descriptions).
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut weapons = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::WEAPON;
        let entry_size = Self::entry_size_for_version(version);
        
        for id in id_range {
            if cursor.position() as usize + entry_size > data.len() {
                break;
            }
            let weapon = Self::read_one(&mut cursor, id, version)?;
            weapons.push(weapon);
        }
        
        Ok(weapons)
    }

    /// Get entry size for a specific version.
    fn entry_size_for_version(version: &GameVersion) -> usize {
        match version.region {
            Region::Eu => Self::ENTRY_SIZE + 1, // EU has extra padding
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Patch a single weapon entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_CHAR_ID] = self.character_id as u8;
        buf[Self::OFF_SELL_PERCENT] = self.sell_percent as u8;
        buf[Self::OFF_ORDER1] = self.order1 as u8;
        buf[Self::OFF_ORDER2] = self.order2 as u8;
        buf[Self::OFF_EFFECT_ID] = self.effect_id as u8;
        buf[Self::OFF_BUY_PRICE..Self::OFF_BUY_PRICE+2].copy_from_slice(&self.buy_price.to_be_bytes());
        buf[Self::OFF_ATTACK..Self::OFF_ATTACK+2].copy_from_slice(&self.attack.to_be_bytes());
        buf[Self::OFF_HIT_PERCENT..Self::OFF_HIT_PERCENT+2].copy_from_slice(&self.hit_percent.to_be_bytes());
        buf[Self::OFF_TRAIT_ID] = self.trait_data.id as u8;
        buf[Self::OFF_TRAIT_VALUE..Self::OFF_TRAIT_VALUE+2].copy_from_slice(&self.trait_data.value.to_be_bytes());
    }

    /// Patch all weapon entries into a buffer.
    pub fn patch_all(weapons: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for w in weapons {
            let idx = (w.id - id_ranges::WEAPON.start) as usize;
            let start = idx * entry_size;
            let end = start + entry_size;
            if end <= buf.len() {
                w.patch_entry(&mut buf[start..end]);
            }
        }
    }
}
