//! Shop entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

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
    /// 2 (id) + 2 (pad) + 4 (sot_pos) + 48*2 (items) = 104 bytes
    pub const ENTRY_SIZE: usize = 104;

    /// Get the number of items in this shop.
    pub fn item_count(&self) -> usize {
        self.item_ids.iter().filter(|&&id| id != -1).count()
    }

    /// Get the item IDs as a Vec (excluding empty slots).
    pub fn items(&self) -> Vec<i16> {
        self.item_ids.iter().filter(|&&id| id != -1).cloned().collect()
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

    /// Write a single shop entry to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, _version: &GameVersion) -> Result<()> {
        writer.write_u16_be(self.id)?;
        writer.write_i16_be(0)?; // padding
        writer.write_u32_be(self.sot_pos)?;
        
        // Write all 48 item slots
        for i in 0..48 {
            let item_id = self.item_ids.get(i).copied().unwrap_or(-1);
            writer.write_i16_be(item_id)?;
        }
        
        Ok(())
    }

    /// Write all shop entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(shops: &[Self], writer: &mut W, version: &GameVersion) -> Result<()> {
        for shop in shops {
            shop.write_one(writer, version)?;
        }
        Ok(())
    }
}
