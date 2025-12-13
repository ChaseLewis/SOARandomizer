//! Enemy encounter entry type.
//!
//! Enemy encounters define battle formations - which enemies appear together
//! in a given battle, along with initiative and magic exp values.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::io::BinaryReader;

/// Maximum number of enemy slots per encounter
pub const MAX_ENEMY_SLOTS: usize = 8;

/// Represents an enemy slot in an encounter (enemy ID or 255 for none)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnemySlot {
    /// Enemy ID (255 = none/empty slot)
    pub enemy_id: u8,
    /// JP name (looked up, not stored in binary)
    #[serde(skip)]
    pub name_jp: String,
    /// US/EU name (looked up, not stored in binary)
    #[serde(skip)]
    pub name_us: String,
}

impl EnemySlot {
    /// Check if this slot is empty (no enemy)
    pub fn is_empty(&self) -> bool {
        self.enemy_id == 255
    }
}

/// An enemy encounter defining a battle formation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyEncounter {
    /// Entry ID (0-based within the ENP segment)
    pub id: u32,
    /// Source file filter (which ENP file this encounter is from)
    pub filter: String,
    /// Initiative value (affects turn order)
    pub initiative: u8,
    /// Magic EXP reward for this encounter
    pub magic_exp: u8,
    /// Enemy slots (up to 8 enemies per encounter)
    pub enemy_slots: [EnemySlot; MAX_ENEMY_SLOTS],
}

impl Default for EnemyEncounter {
    fn default() -> Self {
        Self {
            id: 0,
            filter: String::new(),
            initiative: 0,
            magic_exp: 0,
            enemy_slots: std::array::from_fn(|_| EnemySlot::default()),
        }
    }
}

impl EnemyEncounter {
    /// Size of one encounter entry in bytes.
    /// 1 (initiative) + 1 (magic_exp) + 8 (enemy_ids) = 10 bytes
    pub const ENTRY_SIZE: usize = 10;

    /// Create a new empty encounter
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a single encounter from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, filter: &str) -> Result<Self> {
        let initiative = cursor.read_u8()?;
        let magic_exp = cursor.read_u8()?;

        let mut enemy_slots: [EnemySlot; MAX_ENEMY_SLOTS] =
            std::array::from_fn(|_| EnemySlot::default());
        for slot in &mut enemy_slots {
            slot.enemy_id = cursor.read_u8()?;
        }

        Ok(Self {
            id,
            filter: filter.to_string(),
            initiative,
            magic_exp,
            enemy_slots,
        })
    }

    /// Write this encounter to binary data.
    pub fn write_to(&self, buffer: &mut [u8]) {
        if buffer.len() < Self::ENTRY_SIZE {
            return;
        }

        buffer[0] = self.initiative;
        buffer[1] = self.magic_exp;

        for (i, slot) in self.enemy_slots.iter().enumerate() {
            buffer[2 + i] = slot.enemy_id;
        }
    }

    /// Patch an encounter in a buffer at a specific offset.
    pub fn patch_at(&self, buffer: &mut [u8], offset: usize) {
        if offset + Self::ENTRY_SIZE <= buffer.len() {
            self.write_to(&mut buffer[offset..offset + Self::ENTRY_SIZE]);
        }
    }

    /// Update enemy names from a lookup function.
    pub fn update_names<F>(&mut self, lookup: F)
    where
        F: Fn(u8) -> (String, String),
    {
        for slot in &mut self.enemy_slots {
            if slot.is_empty() {
                slot.name_jp = "None".to_string();
                slot.name_us = "None".to_string();
            } else {
                let (jp, us) = lookup(slot.enemy_id);
                slot.name_jp = jp;
                slot.name_us = us;
            }
        }
    }

    /// Count the number of active (non-empty) enemy slots.
    pub fn enemy_count(&self) -> usize {
        self.enemy_slots.iter().filter(|s| !s.is_empty()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        assert_eq!(EnemyEncounter::ENTRY_SIZE, 10);
    }

    #[test]
    fn test_read_write_roundtrip() {
        let original = EnemyEncounter {
            id: 5,
            filter: "test_ep.enp".to_string(),
            initiative: 102,
            magic_exp: 2,
            enemy_slots: [
                EnemySlot {
                    enemy_id: 42,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
                EnemySlot {
                    enemy_id: 42,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
                EnemySlot {
                    enemy_id: 255,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
                EnemySlot {
                    enemy_id: 255,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
                EnemySlot {
                    enemy_id: 255,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
                EnemySlot {
                    enemy_id: 255,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
                EnemySlot {
                    enemy_id: 255,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
                EnemySlot {
                    enemy_id: 255,
                    name_jp: String::new(),
                    name_us: String::new(),
                },
            ],
        };

        // Write to buffer
        let mut buffer = [0u8; EnemyEncounter::ENTRY_SIZE];
        original.write_to(&mut buffer);

        // Read back
        let mut cursor = Cursor::new(buffer.as_slice());
        let read_back =
            EnemyEncounter::read_one(&mut cursor, original.id, &original.filter).unwrap();

        assert_eq!(read_back.initiative, original.initiative);
        assert_eq!(read_back.magic_exp, original.magic_exp);
        for i in 0..MAX_ENEMY_SLOTS {
            assert_eq!(
                read_back.enemy_slots[i].enemy_id,
                original.enemy_slots[i].enemy_id
            );
        }
    }

    #[test]
    fn test_enemy_count() {
        let enc = EnemyEncounter {
            enemy_slots: [
                EnemySlot {
                    enemy_id: 1,
                    ..Default::default()
                },
                EnemySlot {
                    enemy_id: 2,
                    ..Default::default()
                },
                EnemySlot {
                    enemy_id: 255,
                    ..Default::default()
                },
                EnemySlot {
                    enemy_id: 3,
                    ..Default::default()
                },
                EnemySlot {
                    enemy_id: 255,
                    ..Default::default()
                },
                EnemySlot {
                    enemy_id: 255,
                    ..Default::default()
                },
                EnemySlot {
                    enemy_id: 255,
                    ..Default::default()
                },
                EnemySlot {
                    enemy_id: 255,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(enc.enemy_count(), 3);
    }

    #[test]
    fn test_slot_is_empty() {
        let empty = EnemySlot {
            enemy_id: 255,
            ..Default::default()
        };
        let filled = EnemySlot {
            enemy_id: 42,
            ..Default::default()
        };

        assert!(empty.is_empty());
        assert!(!filled.is_empty());
    }
}
