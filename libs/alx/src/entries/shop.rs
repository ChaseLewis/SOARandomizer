//! Shop entry type.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::game::offsets::id_ranges;
use crate::game::region::GameVersion;
use crate::io::BinaryReader;

/// Shop entry with up to 48 item slots.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Shop {
    /// Entry ID
    pub id: u16,
    /// SOT (String Offset Table) position
    pub sot_pos: u32,
    /// Shop name/description
    pub description: String,
    /// Description position in DOL
    pub description_pos: u32,
    /// Description size
    pub description_size: u32,
    /// Item IDs (up to 48 slots, -1 means empty)
    pub item_ids: Vec<i16>,
}

impl Shop {
    /// Size of one shop entry in bytes.
    pub const ENTRY_SIZE: usize = 104;

    // Field offsets
    // 0-1 = id, 2-3 = pad, 4-7 = sot_pos (don't change these)
    const OFF_ITEMS: usize = 8; // 48 items * 2 bytes each

    /// Get the number of items in this shop.
    pub fn item_count(&self) -> usize {
        self.item_ids.iter().filter(|&&id| id != -1).count()
    }

    /// Get the item IDs as a Vec (excluding empty slots).
    pub fn items(&self) -> Vec<i16> {
        self.item_ids
            .iter()
            .filter(|&&id| id != -1)
            .cloned()
            .collect()
    }

    /// Read a single shop entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, _version: &GameVersion) -> Result<Self> {
        let id = cursor.read_u16_be()?;
        let _pad = cursor.read_i16_be()?;
        let sot_pos = cursor.read_u32_be()?;

        let mut item_ids = Vec::with_capacity(48);
        for _ in 0..48 {
            item_ids.push(cursor.read_i16_be()?);
        }

        Ok(Self {
            id,
            sot_pos,
            description: String::new(),
            description_pos: 0,
            description_size: 0,
            item_ids,
        })
    }

    /// Read all shop entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut shops = Vec::new();
        let mut cursor = Cursor::new(data);

        let id_range = id_ranges::SHOP;

        for _ in id_range {
            if cursor.position() as usize + Self::ENTRY_SIZE > data.len() {
                break;
            }
            let shop = Self::read_one(&mut cursor, version)?;
            shops.push(shop);
        }

        Ok(shops)
    }

    /// Patch a single shop entry in a mutable buffer.
    /// Only patches item IDs - id, pad, sot_pos are untouched.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        for i in 0..48 {
            let item_id = self.item_ids.get(i).copied().unwrap_or(-1);
            let off = Self::OFF_ITEMS + i * 2;
            buf[off..off + 2].copy_from_slice(&item_id.to_be_bytes());
        }
    }

    /// Patch all shop entries into a buffer.
    pub fn patch_all(shops: &[Self], buf: &mut [u8]) {
        for (idx, shop) in shops.iter().enumerate() {
            let start = idx * Self::ENTRY_SIZE;
            let end = start + Self::ENTRY_SIZE;
            if end <= buf.len() {
                shop.patch_entry(&mut buf[start..end]);
            }
        }
    }
}
