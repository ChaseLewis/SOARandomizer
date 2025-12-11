//! Enemy magic entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};
use crate::lookups::{EFFECT_NAMES, ELEMENT_NAMES, STATE_NAMES};

/// An enemy magic spell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyMagic {
    /// Entry ID
    pub id: u32,
    /// Spell name
    pub name: String,
    /// Category ID (always 1=Magic for this type)
    pub category_id: i8,
    /// Effect ID
    pub effect_id: i8,
    /// Scope ID
    pub scope_id: u8,
    /// Effect parameter ID
    pub effect_param_id: u16,
    /// Effect base value
    pub effect_base: u16,
    /// Element ID
    pub element_id: i8,
    /// Type ID
    pub type_id: i8,
    /// State infliction trait ID
    pub state_infliction_id: i8,
    /// State resistance trait ID
    pub state_resistance_id: i8,
    /// State ID
    pub state_id: i8,
    /// State miss chance percentage
    pub state_miss: i8,
}

impl EnemyMagic {
    /// Size of one entry in bytes (US/JP).
    pub const ENTRY_SIZE_US_JP: usize = 36;
    
    /// Size of one entry in bytes (EU).
    pub const ENTRY_SIZE_EU: usize = 34;

    /// Read a single enemy magic from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        
        // JP/US have 4 padding bytes after name
        if version.region != Region::Eu {
            for _ in 0..4 {
                let _pad = cursor.read_u8()?;
            }
        }
        
        let category_id = cursor.read_i8()?;
        let effect_id = cursor.read_i8()?;
        let scope_id = cursor.read_u8()?;
        
        // EU has 1 padding byte after scope
        if version.region == Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let effect_param_id = cursor.read_u16_be()?;
        let effect_base = cursor.read_u16_be()?;
        let element_id = cursor.read_i8()?;
        let type_id = cursor.read_i8()?;
        let state_infliction_id = cursor.read_i8()?;
        let state_resistance_id = cursor.read_i8()?;
        let state_id = cursor.read_i8()?;
        let state_miss = cursor.read_i8()?;
        
        // 2 padding bytes at end
        let _pad = cursor.read_u8()?;
        let _pad = cursor.read_u8()?;
        
        Ok(Self {
            id,
            name,
            category_id,
            effect_id,
            scope_id,
            effect_param_id,
            effect_base,
            element_id,
            type_id,
            state_infliction_id,
            state_resistance_id,
            state_id,
            state_miss,
        })
    }

    /// Read all enemy magic entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut entries = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::ENEMY_MAGIC;
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
            Region::Eu => Self::ENTRY_SIZE_EU,
            _ => Self::ENTRY_SIZE_US_JP,
        }
    }

    /// Get effect name
    pub fn effect_name(&self) -> &'static str {
        EFFECT_NAMES.get(self.effect_id.into())
    }

    /// Get element name
    pub fn element_name(&self) -> &'static str {
        ELEMENT_NAMES.get(self.element_id)
    }

    /// Get state name
    pub fn state_name(&self) -> &'static str {
        STATE_NAMES.get(self.state_id)
    }

    /// Write a single enemy magic to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 17)?;
        if version.region != Region::Eu { for _ in 0..4 { writer.write_u8(0)?; } }
        writer.write_i8(self.category_id)?;
        writer.write_i8(self.effect_id)?;
        writer.write_u8(self.scope_id)?;
        if version.region == Region::Eu { writer.write_u8(0)?; }
        writer.write_u16_be(self.effect_param_id)?;
        writer.write_u16_be(self.effect_base)?;
        writer.write_i8(self.element_id)?;
        writer.write_i8(self.type_id)?;
        writer.write_i8(self.state_infliction_id)?;
        writer.write_i8(self.state_resistance_id)?;
        writer.write_i8(self.state_id)?;
        writer.write_i8(self.state_miss)?;
        writer.write_u8(0)?; writer.write_u8(0)?;
        Ok(())
    }

    /// Write all enemy magic entries to binary data.
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
        assert_eq!(EnemyMagic::ENTRY_SIZE_US_JP, 36);
    }
}

