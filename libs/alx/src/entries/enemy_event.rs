//! Enemy event entry type.
//!
//! Enemy events define scripted battle scenarios from the EVP file (epevent.evp).
//! These are more complex than regular encounters, including character positions,
//! defeat/escape conditions, and BGM settings.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::io::BinaryReader;

/// Maximum number of character slots per event
pub const MAX_EVENT_CHARACTERS: usize = 4;

/// Maximum number of enemy slots per event
pub const MAX_EVENT_ENEMIES: usize = 7;

/// A character slot in an event (party member position)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventCharacterSlot {
    /// Character ID (-1 = none/empty slot)
    pub character_id: i8,
    /// X position on battlefield
    pub x: i8,
    /// Z position on battlefield
    pub z: i8,
    /// Character name (looked up, not stored in binary)
    #[serde(skip)]
    pub name: String,
}

impl EventCharacterSlot {
    /// Check if this slot is empty (no character)
    pub fn is_empty(&self) -> bool {
        self.character_id == -1
    }
}

/// An enemy slot in an event (enemy position)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnemySlot {
    /// Enemy ID (255 = none/empty slot)
    pub enemy_id: u8,
    /// X position on battlefield
    pub x: i8,
    /// Z position on battlefield
    pub z: i8,
    /// JP name (looked up, not stored in binary)
    #[serde(skip)]
    pub name_jp: String,
    /// US/EU name (looked up, not stored in binary)
    #[serde(skip)]
    pub name_us: String,
}

impl EventEnemySlot {
    /// Check if this slot is empty (no enemy)
    pub fn is_empty(&self) -> bool {
        self.enemy_id == 255
    }
}

/// Defeat condition names
pub const DEFEAT_CONDITIONS: [&str; 4] = [
    "Defeat All",   // 0
    "Defeat Boss",  // 1
    "Unknown (2)",  // 2
    "Unknown (3)",  // 3
];

/// Escape condition names
pub const ESCAPE_CONDITIONS: [&str; 3] = [
    "Can Escape",    // 0
    "Cannot Escape", // 1
    "Unknown (2)",   // 2
];

/// An enemy event defining a scripted battle scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyEvent {
    /// Entry ID (0-based within the EVP file)
    pub id: u32,
    /// Source file filter (which EVP file this event is from)
    pub filter: String,
    /// Magic EXP reward for this event
    pub magic_exp: u8,
    /// Character slots (up to 4 characters per event)
    pub characters: [EventCharacterSlot; MAX_EVENT_CHARACTERS],
    /// Enemy slots (up to 7 enemies per event)
    pub enemies: [EventEnemySlot; MAX_EVENT_ENEMIES],
    /// Initiative value (affects turn order)
    pub initiative: u8,
    /// Defeat condition ID
    pub defeat_cond_id: i8,
    /// Escape condition ID
    pub escape_cond_id: i8,
}

impl Default for EnemyEvent {
    fn default() -> Self {
        Self {
            id: 0,
            filter: String::new(),
            magic_exp: 0,
            characters: std::array::from_fn(|_| EventCharacterSlot {
                character_id: -1,
                ..Default::default()
            }),
            enemies: std::array::from_fn(|_| EventEnemySlot {
                enemy_id: 255,
                ..Default::default()
            }),
            initiative: 0,
            defeat_cond_id: 0,
            escape_cond_id: 0,
        }
    }
}

impl EnemyEvent {
    /// Size of one event entry in bytes.
    /// 1 (magic_exp) + 12 (4 chars × 3) + 21 (7 enemies × 3) + 1 (initiative) + 1 (defeat) + 1 (escape) = 37 bytes
    pub const ENTRY_SIZE: usize = 37;

    /// Create a new empty event
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a single event from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, filter: &str) -> Result<Self> {
        let magic_exp = cursor.read_u8()?;

        // Read 4 character slots (id, x, z each)
        let mut characters: [EventCharacterSlot; MAX_EVENT_CHARACTERS] =
            std::array::from_fn(|_| EventCharacterSlot::default());
        for slot in &mut characters {
            slot.character_id = cursor.read_i8()?;
            slot.x = cursor.read_i8()?;
            slot.z = cursor.read_i8()?;
        }

        // Read 7 enemy slots (id, x, z each)
        let mut enemies: [EventEnemySlot; MAX_EVENT_ENEMIES] =
            std::array::from_fn(|_| EventEnemySlot::default());
        for slot in &mut enemies {
            slot.enemy_id = cursor.read_u8()?;
            slot.x = cursor.read_i8()?;
            slot.z = cursor.read_i8()?;
        }

        let initiative = cursor.read_u8()?;
        let defeat_cond_id = cursor.read_i8()?;
        let escape_cond_id = cursor.read_i8()?;

        Ok(Self {
            id,
            filter: filter.to_string(),
            magic_exp,
            characters,
            enemies,
            initiative,
            defeat_cond_id,
            escape_cond_id,
        })
    }

    /// Write this event to binary data.
    pub fn write_to(&self, buffer: &mut [u8]) {
        if buffer.len() < Self::ENTRY_SIZE {
            return;
        }

        let mut offset = 0;

        buffer[offset] = self.magic_exp;
        offset += 1;

        // Write character slots
        for slot in &self.characters {
            buffer[offset] = slot.character_id as u8;
            buffer[offset + 1] = slot.x as u8;
            buffer[offset + 2] = slot.z as u8;
            offset += 3;
        }

        // Write enemy slots
        for slot in &self.enemies {
            buffer[offset] = slot.enemy_id;
            buffer[offset + 1] = slot.x as u8;
            buffer[offset + 2] = slot.z as u8;
            offset += 3;
        }

        buffer[offset] = self.initiative;
        buffer[offset + 1] = self.defeat_cond_id as u8;
        buffer[offset + 2] = self.escape_cond_id as u8;
    }

    /// Check if this is an empty/default event
    pub fn is_empty(&self) -> bool {
        self.magic_exp == 0
            && self.initiative == 0
            && self.defeat_cond_id == 0
            && self.escape_cond_id == 0
            && self.characters.iter().all(|c| c.is_empty())
            && self.enemies.iter().all(|e| e.is_empty())
    }

    /// Get defeat condition name
    pub fn defeat_cond_name(&self) -> &'static str {
        let idx = self.defeat_cond_id as usize;
        if idx < DEFEAT_CONDITIONS.len() {
            DEFEAT_CONDITIONS[idx]
        } else {
            "Unknown"
        }
    }

    /// Get escape condition name
    pub fn escape_cond_name(&self) -> &'static str {
        let idx = self.escape_cond_id as usize;
        if idx < ESCAPE_CONDITIONS.len() {
            ESCAPE_CONDITIONS[idx]
        } else {
            "Unknown"
        }
    }

    /// Update enemy names from a lookup function.
    pub fn update_enemy_names<F>(&mut self, lookup: F)
    where
        F: Fn(u8) -> (String, String),
    {
        for slot in &mut self.enemies {
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

    /// Update character names from a lookup function.
    pub fn update_character_names<F>(&mut self, lookup: F)
    where
        F: Fn(i8) -> String,
    {
        for slot in &mut self.characters {
            if slot.is_empty() {
                slot.name = "None".to_string();
            } else {
                slot.name = lookup(slot.character_id);
            }
        }
    }

    /// Count the number of active (non-empty) enemy slots.
    pub fn enemy_count(&self) -> usize {
        self.enemies.iter().filter(|s| !s.is_empty()).count()
    }

    /// Count the number of active (non-empty) character slots.
    pub fn character_count(&self) -> usize {
        self.characters.iter().filter(|s| !s.is_empty()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        // 1 + 12 + 21 + 3 = 37
        assert_eq!(EnemyEvent::ENTRY_SIZE, 37);
    }

    #[test]
    fn test_read_write_roundtrip() {
        let mut original = EnemyEvent::default();
        original.id = 5;
        original.filter = "epevent.evp".to_string();
        original.magic_exp = 10;
        original.initiative = 50;
        original.defeat_cond_id = 1;
        original.escape_cond_id = 0;

        // Set some characters
        original.characters[0].character_id = 0; // Vyse
        original.characters[0].x = 10;
        original.characters[0].z = 20;

        // Set some enemies
        original.enemies[0].enemy_id = 42;
        original.enemies[0].x = -5;
        original.enemies[0].z = 15;

        // Write to buffer
        let mut buffer = [0u8; EnemyEvent::ENTRY_SIZE];
        original.write_to(&mut buffer);

        // Read back
        let mut cursor = Cursor::new(buffer.as_slice());
        let read_back = EnemyEvent::read_one(&mut cursor, original.id, &original.filter).unwrap();

        assert_eq!(read_back.magic_exp, original.magic_exp);
        assert_eq!(read_back.initiative, original.initiative);
        assert_eq!(read_back.defeat_cond_id, original.defeat_cond_id);
        assert_eq!(read_back.escape_cond_id, original.escape_cond_id);

        assert_eq!(
            read_back.characters[0].character_id,
            original.characters[0].character_id
        );
        assert_eq!(read_back.characters[0].x, original.characters[0].x);
        assert_eq!(read_back.characters[0].z, original.characters[0].z);

        assert_eq!(read_back.enemies[0].enemy_id, original.enemies[0].enemy_id);
        assert_eq!(read_back.enemies[0].x, original.enemies[0].x);
        assert_eq!(read_back.enemies[0].z, original.enemies[0].z);
    }

    #[test]
    fn test_is_empty() {
        let empty = EnemyEvent::default();
        assert!(empty.is_empty());

        let mut non_empty = EnemyEvent::default();
        non_empty.enemies[0].enemy_id = 1;
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_counts() {
        let mut event = EnemyEvent::default();
        event.characters[0].character_id = 0;
        event.characters[1].character_id = 1;
        event.enemies[0].enemy_id = 10;
        event.enemies[1].enemy_id = 20;
        event.enemies[2].enemy_id = 30;

        assert_eq!(event.character_count(), 2);
        assert_eq!(event.enemy_count(), 3);
    }
}
