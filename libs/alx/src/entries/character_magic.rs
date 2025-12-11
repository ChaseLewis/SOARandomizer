//! Character magic (spells) and super moves entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

/// Character magic/spell entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterMagic {
    /// Entry ID
    pub id: u32,
    /// Spell name
    pub name: String,
    /// Element ID
    pub element_id: i8,
    /// Sort order
    pub order: i16,
    /// Occasion flags (Menu/Battle/Ship)
    pub occasion_flags: u8,
    /// Effect ID
    pub effect_id: i8,
    /// Scope ID
    pub scope_id: u8,
    /// Category ID
    pub category_id: i8,
    /// Effect speed
    pub effect_speed: i8,
    /// SP cost
    pub effect_sp: i8,
    /// Effect base value
    pub effect_base: i16,
    /// Damage type ID
    pub type_id: i8,
    /// State ID (for status effects)
    pub state_id: i8,
    /// State miss percentage
    pub state_miss: i8,
    /// Ship occasion ID
    pub ship_occasion_id: i8,
    /// Ship effect ID
    pub ship_effect_id: i16,
    /// Ship effect SP cost
    pub ship_effect_sp: i8,
    /// Ship effect turns
    pub ship_effect_turns: i8,
    /// Ship effect base value
    pub ship_effect_base: i16,
    /// Unknown value
    pub unknown: i8,
    /// Description text
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
    /// Ship description text
    pub ship_description: String,
    /// Ship description position in DOL
    pub ship_description_pos: u32,
    /// Ship description size
    pub ship_description_size: u32,
}

impl Default for CharacterMagic {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            element_id: 0,
            order: -1,
            occasion_flags: 0,
            effect_id: -1,
            scope_id: 0,
            category_id: 0,
            effect_speed: -1,
            effect_sp: -1,
            effect_base: 0,
            type_id: 0,
            state_id: 0,
            state_miss: 0,
            ship_occasion_id: 0,
            ship_effect_id: -1,
            ship_effect_sp: -1,
            ship_effect_turns: -1,
            ship_effect_base: 0,
            unknown: -1,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
            ship_description: String::new(),
            ship_description_pos: 0,
            ship_description_size: 0,
        }
    }
}

impl CharacterMagic {
    /// Size of one magic entry in bytes (JP/US).
    /// 17 (name) + 1 (element) + 2 (order) + 1 (occasion) + 1 (effect_id) + 1 (scope) + 
    /// 1 (category) + 1 (speed) + 1 (sp) + 2 (pad) + 2 (base) + 1 (type) + 1 (state_id) + 
    /// 1 (state_miss) + 3 (pad) + 1 (ship_occ) + 1 (pad) + 2 (ship_eff_id) + 1 (ship_sp) + 
    /// 1 (ship_turns) + 2 (ship_base) + 1 (unknown) + 3 (pad) = 48 bytes
    pub const ENTRY_SIZE: usize = 48;
    
    // Field offsets (name at 0-16 is NEVER written)
    const OFF_ELEMENT_ID: usize = 17;
    const OFF_ORDER: usize = 18;
    const OFF_OCCASION: usize = 20;
    const OFF_EFFECT_ID: usize = 21;
    const OFF_SCOPE_ID: usize = 22;
    const OFF_CATEGORY_ID: usize = 23;
    const OFF_EFFECT_SPEED: usize = 24;
    const OFF_EFFECT_SP: usize = 25;
    // 26-27 = pad
    const OFF_EFFECT_BASE: usize = 28;
    const OFF_TYPE_ID: usize = 30;
    const OFF_STATE_ID: usize = 31;
    const OFF_STATE_MISS: usize = 32;
    // 33-35 = pad
    const OFF_SHIP_OCC_ID: usize = 36;
    // 37 = pad
    const OFF_SHIP_EFFECT_ID: usize = 38;
    const OFF_SHIP_EFFECT_SP: usize = 40;
    const OFF_SHIP_EFFECT_TURNS: usize = 41;
    const OFF_SHIP_EFFECT_BASE: usize = 42;
    const OFF_UNKNOWN: usize = 44;
    // 45-47 = pad

    /// Get element name.
    pub fn element_name(&self) -> &'static str {
        crate::lookups::element_name(self.element_id)
    }

    /// Get effect name.
    pub fn effect_name(&self) -> &'static str {
        match self.effect_id {
            -1 => "None",
            11 => "Confusion",
            12 => "Silence",
            21 => "Incr Attack & Defense",
            31 => "Recover HP",
            32 => "Recover HP of 100%",
            48 => "Recover MP",
            _ => "???",
        }
    }

    /// Get scope name.
    pub fn scope_name(&self) -> &'static str {
        match self.scope_id {
            0 => "None",
            1 => "Single PC",
            2 => "All PCs",
            3 => "Single EC",
            4 => "All Enemies",
            _ => "???",
        }
    }

    /// Read a single magic entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let element_id = cursor.read_i8()?;
        
        // EU has extra padding here
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let order = cursor.read_i16_be()?;
        let occasion_flags = cursor.read_u8()?;
        let effect_id = cursor.read_i8()?;
        let scope_id = cursor.read_u8()?;
        let category_id = cursor.read_i8()?;
        let effect_speed = cursor.read_i8()?;
        let effect_sp = cursor.read_i8()?;
        let _pad1 = cursor.read_u8()?;
        let _pad2 = cursor.read_u8()?;
        let effect_base = cursor.read_i16_be()?;
        let type_id = cursor.read_i8()?;
        let state_id = cursor.read_i8()?;
        let state_miss = cursor.read_i8()?;
        let _pad3 = cursor.read_u8()?;
        let _pad4 = cursor.read_u8()?;
        let _pad5 = cursor.read_u8()?;
        let ship_occasion_id = cursor.read_i8()?;
        let _pad6 = cursor.read_u8()?;
        let ship_effect_id = cursor.read_i16_be()?;
        let ship_effect_sp = cursor.read_i8()?;
        let ship_effect_turns = cursor.read_i8()?;
        let ship_effect_base = cursor.read_i16_be()?;
        let unknown = cursor.read_i8()?;
        let _pad7 = cursor.read_u8()?;
        let _pad8 = cursor.read_u8()?;
        let _pad9 = cursor.read_u8()?;
        
        Ok(Self {
            id,
            name,
            element_id,
            order,
            occasion_flags,
            effect_id,
            scope_id,
            category_id,
            effect_speed,
            effect_sp,
            effect_base,
            type_id,
            state_id,
            state_miss,
            ship_occasion_id,
            ship_effect_id,
            ship_effect_sp,
            ship_effect_turns,
            ship_effect_base,
            unknown,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
            ship_description: String::new(),
            ship_description_pos: 0,
            ship_description_size: 0,
        })
    }

    /// Read all magic entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut magics = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::CHARACTER_MAGIC;
        let entry_size = Self::entry_size_for_version(version);
        
        for id in id_range {
            if cursor.position() as usize + entry_size > data.len() {
                break;
            }
            let magic = Self::read_one(&mut cursor, id, version)?;
            magics.push(magic);
        }
        
        Ok(magics)
    }

    fn entry_size_for_version(version: &GameVersion) -> usize {
        match version.region {
            Region::Eu => Self::ENTRY_SIZE + 1,
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Patch a single magic entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_ELEMENT_ID] = self.element_id as u8;
        buf[Self::OFF_ORDER..Self::OFF_ORDER+2].copy_from_slice(&self.order.to_be_bytes());
        buf[Self::OFF_OCCASION] = self.occasion_flags;
        buf[Self::OFF_EFFECT_ID] = self.effect_id as u8;
        buf[Self::OFF_SCOPE_ID] = self.scope_id;
        buf[Self::OFF_CATEGORY_ID] = self.category_id as u8;
        buf[Self::OFF_EFFECT_SPEED] = self.effect_speed as u8;
        buf[Self::OFF_EFFECT_SP] = self.effect_sp as u8;
        buf[Self::OFF_EFFECT_BASE..Self::OFF_EFFECT_BASE+2].copy_from_slice(&self.effect_base.to_be_bytes());
        buf[Self::OFF_TYPE_ID] = self.type_id as u8;
        buf[Self::OFF_STATE_ID] = self.state_id as u8;
        buf[Self::OFF_STATE_MISS] = self.state_miss as u8;
        buf[Self::OFF_SHIP_OCC_ID] = self.ship_occasion_id as u8;
        buf[Self::OFF_SHIP_EFFECT_ID..Self::OFF_SHIP_EFFECT_ID+2].copy_from_slice(&self.ship_effect_id.to_be_bytes());
        buf[Self::OFF_SHIP_EFFECT_SP] = self.ship_effect_sp as u8;
        buf[Self::OFF_SHIP_EFFECT_TURNS] = self.ship_effect_turns as u8;
        buf[Self::OFF_SHIP_EFFECT_BASE..Self::OFF_SHIP_EFFECT_BASE+2].copy_from_slice(&self.ship_effect_base.to_be_bytes());
        buf[Self::OFF_UNKNOWN] = self.unknown as u8;
    }

    /// Patch all magic entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for e in entries {
            let idx = (e.id - id_ranges::CHARACTER_MAGIC.start) as usize;
            let start = idx * entry_size;
            let end = start + entry_size;
            if end <= buf.len() {
                e.patch_entry(&mut buf[start..end]);
            }
        }
    }
}
