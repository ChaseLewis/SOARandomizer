//! Treasure chest entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

/// Treasure chest entry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TreasureChest {
    /// Entry ID (corresponds to chest location)
    pub id: u32,
    /// Item ID contained in the chest (-1 means empty, 0x200+ means gold)
    pub item_id: i32,
    /// Item amount
    pub item_amount: i32,
}

impl TreasureChest {
    /// Size of one treasure chest entry in bytes (GC).
    pub const ENTRY_SIZE: usize = 8;
    
    // Field offsets
    const OFF_ITEM_ID: usize = 0;
    const OFF_ITEM_AMOUNT: usize = 4;

    /// Check if this chest contains gold.
    pub fn is_gold(&self) -> bool {
        self.item_id >= 0x200
    }

    /// Get the gold amount if this is a gold chest.
    /// The gold amount is calculated from item_id and item_amount.
    pub fn gold_amount(&self) -> Option<i32> {
        if self.is_gold() {
            // Gold ID 0x200 = 0, 0x201 = 1, etc.
            // The actual gold amount is in item_amount
            Some(self.item_amount)
        } else {
            None
        }
    }

    /// Get the item name for display.
    pub fn item_name(&self) -> &'static str {
        if self.item_id == -1 {
            "None"
        } else if self.item_id >= 0x200 {
            "Gold"
        } else {
            "Item" // Would need item database lookup for actual name
        }
    }

    /// Read a single treasure chest entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let item_id = cursor.read_i32_be()?;
        let item_amount = cursor.read_i32_be()?;
        
        Ok(Self {
            id,
            item_id,
            item_amount,
        })
    }

    /// Read all treasure chest entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut chests = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::TREASURE_CHEST;
        
        for id in id_range {
            if cursor.position() as usize + Self::ENTRY_SIZE > data.len() {
                break;
            }
            let chest = Self::read_one(&mut cursor, id, version)?;
            chests.push(chest);
        }
        
        Ok(chests)
    }

    /// Patch a single treasure chest entry in a mutable buffer.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        buf[Self::OFF_ITEM_ID..Self::OFF_ITEM_ID+4].copy_from_slice(&self.item_id.to_be_bytes());
        buf[Self::OFF_ITEM_AMOUNT..Self::OFF_ITEM_AMOUNT+4].copy_from_slice(&self.item_amount.to_be_bytes());
    }

    /// Patch all treasure chest entries into a buffer.
    pub fn patch_all(chests: &[Self], buf: &mut [u8]) {
        for chest in chests {
            let idx = (chest.id - id_ranges::TREASURE_CHEST.start) as usize;
            let start = idx * Self::ENTRY_SIZE;
            let end = start + Self::ENTRY_SIZE;
            if end <= buf.len() {
                chest.patch_entry(&mut buf[start..end]);
            }
        }
    }
}
