//! Weapon effect entry type.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::io::{BinaryReader, BinaryWriter};
use crate::lookups::STATE_NAMES;

/// A weapon effect in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponEffect {
    /// Entry ID
    pub id: u32,
    /// Japanese name
    pub name_jp: String,
    /// Effect ID
    pub effect_id: i8,
    /// State ID
    pub state_id: i8,
    /// State miss percentage
    pub state_miss: i8,
}

impl WeaponEffect {
    /// Size of one entry in bytes.
    /// 17 (name) + 1 (effect_id) + 1 (state_id) + 1 (state_miss) = 20 bytes
    pub const ENTRY_SIZE: usize = 20;

    /// Get the description string for this effect.
    /// Format: "{State Name} by {100 - state_miss}%" or "None"
    pub fn description(&self) -> String {
        if self.effect_id > 0 && self.state_id != -1 {
            let hit = 100 - self.state_miss as i32;
            let state_name = STATE_NAMES.get(self.state_id);
            format!("{} by {}%", state_name, hit)
        } else {
            "None".to_string()
        }
    }

    /// Read a single weapon effect from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let name_jp = cursor.read_string_fixed(17)?;
        let effect_id = cursor.read_i8()?;
        let state_id = cursor.read_i8()?;
        let state_miss = cursor.read_i8()?;

        Ok(Self {
            id,
            name_jp,
            effect_id,
            state_id,
            state_miss,
        })
    }

    /// Read all weapon effect entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        let mut id = 0u32;

        while cursor.position() as usize + Self::ENTRY_SIZE <= data.len() {
            let entry = Self::read_one(&mut cursor, id, version)?;
            entries.push(entry);
            id += 1;
        }

        Ok(entries)
    }

    /// Write a single weapon effect to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, _version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name_jp, 17)?;
        writer.write_i8(self.effect_id)?;
        writer.write_i8(self.state_id)?;
        writer.write_i8(self.state_miss)?;
        Ok(())
    }

    /// Write all weapon effect entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(
        entries: &[Self],
        writer: &mut W,
        version: &GameVersion,
    ) -> Result<()> {
        for e in entries {
            e.write_one(writer, version)?;
        }
        Ok(())
    }
}
