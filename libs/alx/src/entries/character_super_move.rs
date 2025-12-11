//! Character Super Move (S-Move) entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// A character super move (S-Move) in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSuperMove {
    /// Entry ID
    pub id: u32,
    /// Name of the S-Move (US localized)
    pub name: String,
    /// Description text
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
    
    // --- Binary fields ---
    
    /// Element ID (0=Green, 1=Red, 2=Purple, 3=Blue, 4=Yellow, 5=Silver, 6=Neutral)
    pub element_id: i8,
    /// Order in menu
    pub order: i16,
    /// Occasion flags (bit 2=Menu, bit 1=Battle, bit 0=Ship)
    pub occasion_flags: u8,
    /// Effect ID
    pub effect_id: i8,
    /// Scope ID (targeting)
    pub scope_id: u8,
    /// Category ID (which character can use)
    pub category_id: i8,
    /// Effect speed
    pub effect_speed: i8,
    /// SP cost
    pub effect_sp: i8,
    /// Base effect value
    pub effect_base: i16,
    /// Type ID (Physical=0, Magical=1)
    pub type_id: i8,
    /// State ID (status effect inflicted)
    pub state_id: i8,
    /// Chance that state misses (%)
    pub state_miss: i8,
    /// Ship occasion ID
    pub ship_occasion_id: i8,
    /// Ship effect ID
    pub ship_effect_id: i16,
    /// Ship effect SP cost
    pub ship_effect_sp: i8,
    /// Ship effect duration in turns
    pub ship_effect_turns: i8,
    /// Ship effect base value
    pub ship_effect_base: i16,
    /// Unknown value
    pub unknown: i8,
}

impl CharacterSuperMove {
    /// Size of one entry in bytes (JP/US).
    /// 17 + 1 + 2 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 1 + 1 + 1 + 3 + 1 + 1 + 2 + 1 + 1 + 2 + 1 + 3 = 44 bytes
    pub const ENTRY_SIZE: usize = 44;

    /// Read a single CharacterSuperMove from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let element_id = cursor.read_i8()?;
        
        // EU has extra padding byte
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
            description: String::new(),
            description_pos: 0,
            description_size: 0,
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
        })
    }

    /// Read all CharacterSuperMove entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::CHARACTER_SUPER_MOVE;
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

    /// Write a single CharacterSuperMove to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 17)?;
        writer.write_i8(self.element_id)?;
        if version.region == Region::Eu { writer.write_u8(0)?; }
        writer.write_i16_be(self.order)?;
        writer.write_u8(self.occasion_flags)?;
        writer.write_i8(self.effect_id)?;
        writer.write_u8(self.scope_id)?;
        writer.write_i8(self.category_id)?;
        writer.write_i8(self.effect_speed)?;
        writer.write_i8(self.effect_sp)?;
        writer.write_u8(0)?; writer.write_u8(0)?;
        writer.write_i16_be(self.effect_base)?;
        writer.write_i8(self.type_id)?;
        writer.write_i8(self.state_id)?;
        writer.write_i8(self.state_miss)?;
        writer.write_u8(0)?; writer.write_u8(0)?; writer.write_u8(0)?;
        writer.write_i8(self.ship_occasion_id)?;
        writer.write_u8(0)?;
        writer.write_i16_be(self.ship_effect_id)?;
        writer.write_i8(self.ship_effect_sp)?;
        writer.write_i8(self.ship_effect_turns)?;
        writer.write_i16_be(self.ship_effect_base)?;
        writer.write_i8(self.unknown)?;
        writer.write_u8(0)?; writer.write_u8(0)?; writer.write_u8(0)?;
        Ok(())
    }

    /// Write all CharacterSuperMove entries to binary data.
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
        assert_eq!(CharacterSuperMove::ENTRY_SIZE, 44);
    }

    #[test]
    fn test_occasion_flags() {
        let mut sm = CharacterSuperMove {
            id: 36,
            name: String::new(),
            description: String::new(),
            description_pos: 0,
            description_size: 0,
            element_id: 0,
            order: 0,
            occasion_flags: 0b0010,
            effect_id: 0,
            scope_id: 0,
            category_id: 0,
            effect_speed: 0,
            effect_sp: 0,
            effect_base: 0,
            type_id: 0,
            state_id: 0,
            state_miss: 0,
            ship_occasion_id: 0,
            ship_effect_id: 0,
            ship_effect_sp: 0,
            ship_effect_turns: 0,
            ship_effect_base: 0,
            unknown: 0,
        };
        
        assert!(!sm.usable_in_menu());
        assert!(sm.usable_in_battle());
        assert!(!sm.usable_on_ship());
        
        sm.occasion_flags = 0b0111;
        assert!(sm.usable_in_menu());
        assert!(sm.usable_in_battle());
        assert!(sm.usable_on_ship());
    }
}

