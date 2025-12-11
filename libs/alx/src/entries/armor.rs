//! Armor entry type (base for armor and accessories).

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use super::traits::Trait;
use crate::error::Result;
use crate::game::region::{GameVersion, Region};
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

/// Character flags for equipment.
/// Bit 5 = Vyse, Bit 4 = Aika, Bit 3 = Fina, Bit 2 = Drachma, Bit 1 = Enrique, Bit 0 = Gilder
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterFlags(pub u8);

impl CharacterFlags {
    /// Character flag bits: V=0x20, A=0x10, F=0x08, D=0x04, E=0x02, G=0x01
    pub const VYSE: u8 = 0x20;
    pub const AIKA: u8 = 0x10;
    pub const FINA: u8 = 0x08;
    pub const DRACHMA: u8 = 0x04;
    pub const ENRIQUE: u8 = 0x02;
    pub const GILDER: u8 = 0x01;

    pub fn can_equip_vyse(&self) -> bool { self.0 & Self::VYSE != 0 }
    pub fn can_equip_aika(&self) -> bool { self.0 & Self::AIKA != 0 }
    pub fn can_equip_fina(&self) -> bool { self.0 & Self::FINA != 0 }
    pub fn can_equip_drachma(&self) -> bool { self.0 & Self::DRACHMA != 0 }
    pub fn can_equip_enrique(&self) -> bool { self.0 & Self::ENRIQUE != 0 }
    pub fn can_equip_gilder(&self) -> bool { self.0 & Self::GILDER != 0 }

    /// Format as binary string for CSV output (e.g., "0b00111010")
    pub fn as_binary_string(&self) -> String {
        format!("0b{:08b}", self.0)
    }

    /// Format character flags as column values ("X" or "")
    pub fn as_columns(&self) -> [&'static str; 6] {
        [
            if self.can_equip_vyse() { "X" } else { "" },
            if self.can_equip_aika() { "X" } else { "" },
            if self.can_equip_fina() { "X" } else { "" },
            if self.can_equip_drachma() { "X" } else { "" },
            if self.can_equip_enrique() { "X" } else { "" },
            if self.can_equip_gilder() { "X" } else { "" },
        ]
    }
}

/// Armor entry (also used as base for Accessory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Armor {
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

impl Default for Armor {
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

impl Armor {
    /// Size of one armor entry in bytes.
    /// Name (17) + flags (1) + sell (1) + order1 (1) + order2 (1) + pad (1) + buy (2) + traits (4 * 4) = 40 bytes
    pub const ENTRY_SIZE: usize = 40;
    
    // Field offsets (name at 0-16 is NEVER written)
    const OFF_CHAR_FLAGS: usize = 17;
    const OFF_SELL_PERCENT: usize = 18;
    const OFF_ORDER1: usize = 19;
    const OFF_ORDER2: usize = 20;
    // 21 = pad
    const OFF_BUY_PRICE: usize = 22;
    const OFF_TRAITS: usize = 24; // 4 traits, each 4 bytes

    /// Read a single armor entry from binary data.
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

    /// Read all armor entries from binary data (without descriptions).
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut armors = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::ARMOR;
        let entry_size = Self::entry_size_for_version(version);
        
        for id in id_range {
            if cursor.position() as usize + entry_size > data.len() {
                break;
            }
            let armor = Self::read_one(&mut cursor, id, version)?;
            armors.push(armor);
        }
        
        Ok(armors)
    }

    /// Get entry size for a specific version.
    fn entry_size_for_version(version: &GameVersion) -> usize {
        match version.region {
            Region::Eu => Self::ENTRY_SIZE + 1, // EU has extra padding
            _ => Self::ENTRY_SIZE,
        }
    }

    /// Patch a single armor entry in a mutable buffer.
    /// Only writes numeric fields - strings and padding are untouched.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_CHAR_FLAGS] = self.character_flags.0;
        buf[Self::OFF_SELL_PERCENT] = self.sell_percent as u8;
        buf[Self::OFF_ORDER1] = self.order1 as u8;
        buf[Self::OFF_ORDER2] = self.order2 as u8;
        buf[Self::OFF_BUY_PRICE..Self::OFF_BUY_PRICE+2].copy_from_slice(&self.buy_price.to_be_bytes());
        
        for (i, t) in self.traits.iter().enumerate() {
            let off = Self::OFF_TRAITS + i * 4;
            buf[off] = t.id as u8;
            buf[off+2..off+4].copy_from_slice(&t.value.to_be_bytes());
        }
    }

    /// Patch all armor entries into a buffer.
    pub fn patch_all(armors: &[Self], buf: &mut [u8], version: &GameVersion) {
        let entry_size = Self::entry_size_for_version(version);
        for a in armors {
            let idx = (a.id - id_ranges::ARMOR.start) as usize;
            let start = idx * entry_size;
            let end = start + entry_size;
            if end <= buf.len() {
                a.patch_entry(&mut buf[start..end]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_character_flags() {
        // 0b00111010: V=1, A=1, F=1, D=0, E=1, G=0
        // Bit layout: 0x20=Vyse, 0x10=Aika, 0x08=Fina, 0x04=Drachma, 0x02=Enrique, 0x01=Gilder
        let flags = CharacterFlags(0b00111010);
        assert!(flags.can_equip_vyse());     // bit 5 (0x20) is set
        assert!(flags.can_equip_aika());     // bit 4 (0x10) is set
        assert!(flags.can_equip_fina());     // bit 3 (0x08) is set
        assert!(!flags.can_equip_drachma()); // bit 2 (0x04) is NOT set
        assert!(flags.can_equip_enrique());  // bit 1 (0x02) is set
        assert!(!flags.can_equip_gilder());  // bit 0 (0x01) is NOT set
        
        assert_eq!(flags.as_binary_string(), "0b00111010");
    }

    #[test]
    fn test_armor_binary_roundtrip() {
        // Sample armor binary data (40 bytes for US/JP)
        let original: Vec<u8> = vec![
            // Name: "Vyse's Uniform" (17 bytes with padding)
            b'V', b'y', b's', b'e', b'\'', b's', b' ', b'U', b'n', b'i', b'f', b'o', b'r', b'm', 0, 0, 0,
            0x20, // character_flags (Vyse only)
            50,   // sell_percent
            1,    // order1
            2,    // order2
            0,    // padding
            0x00, 0xC8, // buy_price = 200 (big-endian)
            // Trait 1: id=17 (Defense), pad, value=10
            17, 0, 0x00, 0x0A,
            // Trait 2-4: None
            0xFF, 0, 0x00, 0x00,
            0xFF, 0, 0x00, 0x00,
            0xFF, 0, 0x00, 0x00,
        ];

        let version = GameVersion::new(
            crate::game::region::Platform::GameCube,
            Region::Us,
            "GEAS8P".to_string(),
        );

        // Read
        let mut cursor = Cursor::new(original.as_slice());
        let armor = Armor::read_one(&mut cursor, 80, &version).unwrap();

        // Verify parsed values
        assert_eq!(armor.name, "Vyse's Uniform");
        assert!(armor.character_flags.can_equip_vyse());
        assert!(!armor.character_flags.can_equip_aika());
        assert_eq!(armor.buy_price, 200);
        assert_eq!(armor.traits[0].id, 17);
        assert_eq!(armor.traits[0].value, 10);

        // Patch back to copy
        let mut patched = original.clone();
        armor.patch_entry(&mut patched);

        // Compare byte-for-byte (numeric fields match)
        assert_eq!(original, patched, "Binary patch round-trip failed");
    }
}

