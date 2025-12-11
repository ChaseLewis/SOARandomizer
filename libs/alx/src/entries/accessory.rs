//! Accessory entry type.
//!
//! Accessories use the same structure as armors.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use super::armor::{Armor, CharacterFlags};
use super::traits::Trait;
use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

/// Accessory entry (uses same structure as Armor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accessory {
    /// Entry ID
    pub id: u32,
    /// Entry name (US region)
    pub name: String,
    /// Character equipment flags
    pub character_flags: CharacterFlags,
    /// Sell percentage
    pub sell_percent: i8,
    /// Sort order 1
    pub order1: i8,
    /// Sort order 2
    pub order2: i8,
    /// Buy price
    pub buy_price: u16,
    /// Equipment traits (up to 4)
    pub traits: [Trait; 4],
    /// Description text
    pub description: String,
    /// Description position in DOL (for reference)
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
}

impl Default for Accessory {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            character_flags: CharacterFlags::default(),
            sell_percent: 0,
            order1: -1,
            order2: -1,
            buy_price: 0,
            traits: Default::default(),
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        }
    }
}

impl Accessory {
    /// Size of one accessory entry in bytes (same as armor).
    pub const ENTRY_SIZE: usize = Armor::ENTRY_SIZE;
    
    // Field offsets (name at 0-16 is NEVER written)
    const OFF_CHAR_FLAGS: usize = 17;
    const OFF_SELL_PERCENT: usize = 18;
    const OFF_ORDER1: usize = 19;
    const OFF_ORDER2: usize = 20;
    // 21 = pad
    const OFF_BUY_PRICE: usize = 22;
    const OFF_TRAITS: usize = 24; // Each trait: 1 id + 1 pad + 2 value = 4 bytes

    /// Read a single accessory entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(17)?;
        let character_flags = CharacterFlags(cursor.read_u8()?);
        let sell_percent = cursor.read_i8()?;
        let order1 = cursor.read_i8()?;
        let order2 = cursor.read_i8()?;
        
        // Padding byte (JP/US only)
        if version.region != Region::Eu {
            let _pad = cursor.read_u8()?;
        }
        
        let buy_price = cursor.read_u16_be()?;
        
        // Read 4 traits
        let mut traits = [Trait::default(), Trait::default(), Trait::default(), Trait::default()];
        for trait_slot in &mut traits {
            trait_slot.id = cursor.read_i8()?;
            let _pad = cursor.read_u8()?; // padding
            trait_slot.value = cursor.read_i16_be()?;
        }
        
        // EU has extra padding
        if version.region == Region::Eu {
            let _pad1 = cursor.read_u8()?;
            let _pad2 = cursor.read_u8()?;
        }
        
        Ok(Self {
            id,
            name,
            character_flags,
            sell_percent,
            order1,
            order2,
            buy_price,
            traits,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
        })
    }

    /// Read all accessory entries from binary data (without descriptions).
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut accessories = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::ACCESSORY;
        let entry_size = Self::entry_size_for_version(version);
        
        for id in id_range {
            if cursor.position() as usize + entry_size > data.len() {
                break;
            }
            let accessory = Self::read_one(&mut cursor, id, version)?;
            accessories.push(accessory);
        }
        
        Ok(accessories)
    }

    /// Get entry size for a specific version.
    fn entry_size_for_version(version: &GameVersion) -> usize {
        match version.region {
            Region::Eu => Self::ENTRY_SIZE + 1,
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Patch a single accessory entry in a mutable buffer.
    /// Only writes numeric fields - strings and padding are untouched.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_CHAR_FLAGS] = self.character_flags.0;
        buf[Self::OFF_SELL_PERCENT] = self.sell_percent as u8;
        buf[Self::OFF_ORDER1] = self.order1 as u8;
        buf[Self::OFF_ORDER2] = self.order2 as u8;
        buf[Self::OFF_BUY_PRICE..Self::OFF_BUY_PRICE+2].copy_from_slice(&self.buy_price.to_be_bytes());
        
        // Traits: each 4 bytes (id=1, pad=1, value=2)
        for (i, t) in self.traits.iter().enumerate() {
            let off = Self::OFF_TRAITS + i * 4;
            buf[off] = t.id as u8;
            // Skip pad at off+1
            buf[off+2..off+4].copy_from_slice(&t.value.to_be_bytes());
        }
    }

    /// Patch all accessory entries into a buffer.
    pub fn patch_all(accessories: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for acc in accessories {
            let idx = (acc.id - id_ranges::ACCESSORY.start) as usize;
            let start = idx * entry_size;
            let end = start + entry_size;
            if end <= buf.len() {
                acc.patch_entry(&mut buf[start..end]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_accessory_binary_roundtrip() {
        // Sample accessory binary data (40 bytes for US/JP)
        let original: Vec<u8> = vec![
            // Name: "Test Ring" (17 bytes with padding)
            b'T', b'e', b's', b't', b' ', b'R', b'i', b'n', b'g', 0, 0, 0, 0, 0, 0, 0, 0,
            0x3F, // character_flags (all can equip)
            50,   // sell_percent
            10,   // order1
            20,   // order2
            0,    // padding
            0x00, 0x64, // buy_price = 100 (big-endian)
            // Trait 1: id=0 (Power), pad, value=5
            0, 0, 0x00, 0x05,
            // Trait 2: id=-1 (None), pad, value=0
            0xFF, 0, 0x00, 0x00,
            // Trait 3: id=-1 (None), pad, value=0
            0xFF, 0, 0x00, 0x00,
            // Trait 4: id=-1 (None), pad, value=0
            0xFF, 0, 0x00, 0x00,
        ];

        let version = GameVersion::new(
            crate::game::region::Platform::GameCube,
            Region::Us,
            "GEAS8P".to_string(),
        );

        // Read
        let mut cursor = Cursor::new(original.as_slice());
        let accessory = Accessory::read_one(&mut cursor, 160, &version).unwrap();

        // Verify parsed values
        assert_eq!(accessory.name, "Test Ring");
        assert_eq!(accessory.sell_percent, 50);
        assert_eq!(accessory.buy_price, 100);
        assert_eq!(accessory.traits[0].id, 0);
        assert_eq!(accessory.traits[0].value, 5);

        // Patch back to a copy of original
        let mut output = original.clone();
        accessory.patch_entry(&mut output);

        // Numeric fields should match, string (name) unchanged
        assert_eq!(original, output, "Patch round-trip failed");
    }
}

