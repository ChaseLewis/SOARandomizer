//! Crew member entry type.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::offsets::id_ranges;
use crate::game::region::{GameVersion, Region};
use crate::io::BinaryReader;

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
    pub const ENTRY_SIZE: usize = 36;

    // Field offsets (name at 0-16 is NEVER written)
    const OFF_POSITION_ID: usize = 17;
    const OFF_TRAIT_ID: usize = 18;
    // 19 = pad
    const OFF_TRAIT_VALUE: usize = 20;
    const OFF_SHIP_EFFECT_ID: usize = 22;
    const OFF_SHIP_EFFECT_SP: usize = 23;
    const OFF_SHIP_EFFECT_TURNS: usize = 24;
    // 25-27 = pad
    const OFF_SHIP_EFFECT_BASE: usize = 28;
    const OFF_UNKNOWN: usize = 30;

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

    /// Patch a single crew member entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_POSITION_ID] = self.position_id as u8;
        buf[Self::OFF_TRAIT_ID] = self.trait_id as u8;
        buf[Self::OFF_TRAIT_VALUE..Self::OFF_TRAIT_VALUE + 2]
            .copy_from_slice(&self.trait_value.to_be_bytes());
        buf[Self::OFF_SHIP_EFFECT_ID] = self.ship_effect_id as u8;
        buf[Self::OFF_SHIP_EFFECT_SP] = self.ship_effect_sp as u8;
        buf[Self::OFF_SHIP_EFFECT_TURNS] = self.ship_effect_turns as u8;
        buf[Self::OFF_SHIP_EFFECT_BASE..Self::OFF_SHIP_EFFECT_BASE + 2]
            .copy_from_slice(&self.ship_effect_base.to_be_bytes());
        buf[Self::OFF_UNKNOWN..Self::OFF_UNKNOWN + 2].copy_from_slice(&self.unknown.to_be_bytes());
    }

    /// Patch all crew member entries into a buffer.
    pub fn patch_all(entries: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for e in entries {
            let idx = (e.id - id_ranges::CREW_MEMBER.start) as usize;
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
