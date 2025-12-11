//! Enemy entry type.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::io::BinaryReader;
use crate::lookups::{EFFECT_NAMES, ELEMENT_NAMES, STATE_NAMES};

/// An item drop from an enemy.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EnemyItemDrop {
    /// Drop probability
    pub probability: i16,
    /// Amount of item
    pub amount: i16,
    /// Item ID (-1 = none)
    pub item_id: i16,
}

/// An enemy in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    /// Entry ID
    pub id: u32,
    /// Source file filter (which ENP/DAT file this enemy appears in)
    pub filter: String,
    /// Japanese name
    pub name_jp: String,
    /// US/EU name (looked up from vocabulary)
    pub name: String,
    /// Width
    pub width: i8,
    /// Depth
    pub depth: i8,
    /// Element ID
    pub element_id: i8,
    /// Movement flags
    pub movement_flags: i16,
    /// Counter attack percentage
    pub counter: i16,
    /// Experience reward
    pub exp: u16,
    /// Gold reward
    pub gold: u16,
    /// Max HP
    pub max_hp: i32,
    /// Unknown float value
    pub unknown_float: f32,
    /// Elemental resistances (Green, Red, Purple, Blue, Yellow, Silver)
    pub elements: [i16; 6],
    /// State resistances (15 states)
    pub states: [i16; 15],
    /// Danger rating
    pub danger: i16,
    /// Attack effect ID
    pub effect_id: i8,
    /// State infliction ID
    pub state_id: i8,
    /// State miss percentage
    pub state_miss: i8,
    /// Level
    pub level: i16,
    /// Will stat
    pub will: i16,
    /// Vigor stat
    pub vigor: i16,
    /// Agile stat
    pub agile: i16,
    /// Quick stat
    pub quick: i16,
    /// Attack stat
    pub attack: i16,
    /// Defense stat
    pub defense: i16,
    /// Magic Defense stat
    pub mag_def: i16,
    /// Hit percentage
    pub hit: i16,
    /// Dodge percentage
    pub dodge: i16,
    /// Item drops (up to 4)
    pub item_drops: [EnemyItemDrop; 4],
}

impl Enemy {
    /// Size of one entry in bytes.
    /// 21 + 1 + 1 + 1 + 2 + 2 + 2 + 2 + 2 + 2 + 4 + 4 + 12 + 30 + 2 + 1 + 1 + 1 + 1 + 20 + 2 + 24 = 136 bytes
    pub const ENTRY_SIZE: usize = 136;

    /// Read a single enemy from binary data.
    pub fn read_one(
        cursor: &mut Cursor<&[u8]>,
        id: u32,
        filter: &str,
        _version: &GameVersion,
    ) -> Result<Self> {
        let name_jp = cursor.read_string_fixed(21)?;
        let width = cursor.read_i8()?;
        let depth = cursor.read_i8()?;
        let element_id = cursor.read_i8()?;
        let _pad1 = cursor.read_i8()?;
        let _pad2 = cursor.read_i8()?;
        let movement_flags = cursor.read_i16_be()?;
        let counter = cursor.read_i16_be()?;
        let exp = cursor.read_u16_be()?;
        let gold = cursor.read_u16_be()?;
        let _pad3 = cursor.read_i8()?;
        let _pad4 = cursor.read_i8()?;
        let max_hp = cursor.read_i32_be()?;
        let unknown_float = cursor.read_f32_be()?;

        let mut elements = [0i16; 6];
        for i in 0..6 {
            elements[i] = cursor.read_i16_be()?;
        }

        let mut states = [0i16; 15];
        for i in 0..15 {
            states[i] = cursor.read_i16_be()?;
        }

        let danger = cursor.read_i16_be()?;
        let effect_id = cursor.read_i8()?;
        let state_id = cursor.read_i8()?;
        let state_miss = cursor.read_i8()?;
        let _pad5 = cursor.read_i8()?;
        let level = cursor.read_i16_be()?;
        let will = cursor.read_i16_be()?;
        let vigor = cursor.read_i16_be()?;
        let agile = cursor.read_i16_be()?;
        let quick = cursor.read_i16_be()?;
        let attack = cursor.read_i16_be()?;
        let defense = cursor.read_i16_be()?;
        let mag_def = cursor.read_i16_be()?;
        let hit = cursor.read_i16_be()?;
        let dodge = cursor.read_i16_be()?;
        let _pad6 = cursor.read_i8()?;
        let _pad7 = cursor.read_i8()?;

        let mut item_drops = [EnemyItemDrop::default(); 4];
        for i in 0..4 {
            item_drops[i] = EnemyItemDrop {
                probability: cursor.read_i16_be()?,
                amount: cursor.read_i16_be()?,
                item_id: cursor.read_i16_be()?,
            };
        }

        Ok(Self {
            id,
            filter: filter.to_string(),
            name_jp,
            name: String::new(), // Will be looked up later
            width,
            depth,
            element_id,
            movement_flags,
            counter,
            exp,
            gold,
            max_hp,
            unknown_float,
            elements,
            states,
            danger,
            effect_id,
            state_id,
            state_miss,
            level,
            will,
            vigor,
            agile,
            quick,
            attack,
            defense,
            mag_def,
            hit,
            dodge,
            item_drops,
        })
    }

    /// Get element name
    pub fn element_name(&self) -> &'static str {
        ELEMENT_NAMES.get(self.element_id)
    }

    /// Get effect name
    pub fn effect_name(&self) -> &'static str {
        EFFECT_NAMES.get(self.effect_id.into())
    }

    /// Get state name
    pub fn state_name(&self) -> &'static str {
        STATE_NAMES.get(self.state_id)
    }

    /// Check movement flag: May Dodge
    pub fn may_dodge(&self) -> bool {
        (self.movement_flags & 0x800) != 0
    }
    /// Check movement flag: Unknown Damage
    pub fn unk_damage(&self) -> bool {
        (self.movement_flags & 0x400) != 0
    }
    /// Check movement flag: Unknown Ranged
    pub fn unk_ranged(&self) -> bool {
        (self.movement_flags & 0x200) != 0
    }
    /// Check movement flag: Unknown Melee
    pub fn unk_melee(&self) -> bool {
        (self.movement_flags & 0x100) != 0
    }
    /// Check movement flag: Ranged Attack
    pub fn ranged_atk(&self) -> bool {
        (self.movement_flags & 0x080) != 0
    }
    /// Check movement flag: Melee Attack
    pub fn melee_atk(&self) -> bool {
        (self.movement_flags & 0x040) != 0
    }
    /// Check movement flag: Ranged Only
    pub fn ranged_only(&self) -> bool {
        (self.movement_flags & 0x020) != 0
    }
    /// Check movement flag: Take Cover
    pub fn take_cover(&self) -> bool {
        (self.movement_flags & 0x010) != 0
    }
    /// Check movement flag: In Air
    pub fn in_air(&self) -> bool {
        (self.movement_flags & 0x008) != 0
    }
    /// Check movement flag: On Ground
    pub fn on_ground(&self) -> bool {
        (self.movement_flags & 0x004) != 0
    }
    /// Check movement flag: Reserved
    pub fn reserved(&self) -> bool {
        (self.movement_flags & 0x002) != 0
    }
    /// Check movement flag: May Move
    pub fn may_move(&self) -> bool {
        (self.movement_flags & 0x001) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        assert_eq!(Enemy::ENTRY_SIZE, 136);
    }
}
