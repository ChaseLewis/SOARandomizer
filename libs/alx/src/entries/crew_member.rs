//! Crew member entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// A crew member in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewMember {
    /// Entry ID
    pub id: u32,
    /// Crew member name
    pub name: String,
    /// Position ID (0=Helmsman, 1=Engineer, 2=Gunner, etc.)
    pub position_id: i8,
    /// Trait ID (ship trait this crew member provides)
    pub trait_id: i8,
    /// Trait value
    pub trait_value: i16,
    /// Ship effect ID
    pub ship_effect_id: i8,
    /// Ship effect SP cost
    pub ship_effect_sp: i8,
    /// Ship effect duration in turns
    pub ship_effect_turns: i8,
    /// Ship effect base value
    pub ship_effect_base: i16,
    /// Unknown value
    pub unknown: i16,
    /// Description text
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
}

impl CrewMember {
    /// Size of one entry in bytes (US/JP).
    /// 17 + 1 + 1 + 1 + 2 + 1 + 1 + 1 + 1 + 1 + 1 + 2 + 2 + 4 = 36 bytes
    pub const ENTRY_SIZE: usize = 36;

    /// Get position name.
    pub fn position_name(&self) -> &'static str {
        match self.position_id {
            0 => "Helmsman",
            1 => "Engineer",
            2 => "Gunner",
            3 => "Artisan",
            4 => "Sailor",
            5 => "Cook",
            6 => "Merchant",
            7 => "Builder",
            8 => "Lookout",
            9 => "Jester",
            10 => "Delegate",
            _ => "???",
        }
    }

    /// Read a single crew member from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let position_id = cursor.read_i8()?;
        
        // EU has extra padding here
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let trait_id = cursor.read_i8()?;
        let _pad1 = cursor.read_u8()?;
        let trait_value = cursor.read_i16_be()?;
        let ship_effect_id = cursor.read_i8()?;
        let ship_effect_sp = cursor.read_i8()?;
        let ship_effect_turns = cursor.read_i8()?;
        let _pad2 = cursor.read_u8()?;
        let _pad3 = cursor.read_u8()?;
        let _pad4 = cursor.read_u8()?;
        let ship_effect_base = cursor.read_i16_be()?;
        let unknown = cursor.read_i16_be()?;
        let _pad5 = cursor.read_u8()?;
        let _pad6 = cursor.read_u8()?;
        let _pad7 = cursor.read_u8()?;
        let _pad8 = cursor.read_u8()?;
        
        Ok(Self {
            id,
            name,
            position_id,
            trait_id,
            trait_value,
            ship_effect_id,
            ship_effect_sp,
            ship_effect_turns,
            ship_effect_base,
            unknown,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all crew member entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::CREW_MEMBER;
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

    /// Write a single crew member to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 17)?;
        writer.write_i8(self.position_id)?;
        if version.region == Region::Eu { writer.write_u8(0)?; }
        writer.write_i8(self.trait_id)?;
        writer.write_u8(0)?;
        writer.write_i16_be(self.trait_value)?;
        writer.write_i8(self.ship_effect_id)?;
        writer.write_i8(self.ship_effect_sp)?;
        writer.write_i8(self.ship_effect_turns)?;
        writer.write_u8(0)?; writer.write_u8(0)?; writer.write_u8(0)?;
        writer.write_i16_be(self.ship_effect_base)?;
        writer.write_i16_be(self.unknown)?;
        writer.write_u8(0)?; writer.write_u8(0)?; writer.write_u8(0)?; writer.write_u8(0)?;
        Ok(())
    }

    /// Write all crew member entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(entries: &[Self], writer: &mut W, version: &GameVersion) -> Result<()> {
        for e in entries { e.write_one(writer, version)?; }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_names() {
        let mut cm = CrewMember {
            id: 0,
            name: String::new(),
            position_id: 0,
            trait_id: 0,
            trait_value: 0,
            ship_effect_id: 0,
            ship_effect_sp: 0,
            ship_effect_turns: 0,
            ship_effect_base: 0,
            unknown: 0,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        };
        
        assert_eq!(cm.position_name(), "Helmsman");
        cm.position_id = 1;
        assert_eq!(cm.position_name(), "Engineer");
        cm.position_id = 2;
        assert_eq!(cm.position_name(), "Gunner");
    }
}

