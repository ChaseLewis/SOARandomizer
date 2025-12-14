//! ENP and EVP file structure dumper for debugging and analysis.

use crate::entries::{Enemy, EnemyEvent};
use crate::error::Result;
use crate::game::region::GameVersion;
use crate::io::BinaryReader;
use crate::items::ItemDatabase;
use crate::lookups::enemy_names_map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

// ============================================================================
// Simplified Editable Schema
// ============================================================================

/// Simplified ENP definition for editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnpDefinition {
    /// Source filename
    pub filename: String,
    /// Enemies in this file
    pub enemies: Vec<EnemyDefinition>,
    /// Battle encounters
    pub encounters: Vec<EncounterDefinition>,
}

/// Simplified enemy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyDefinition {
    /// US name (used as reference key)
    pub name: String,
    /// Japanese name
    pub name_jp: String,
    /// Combat stats
    pub stats: EnemyStatsDef,
    /// Item drops
    pub item_drops: Vec<ItemDropDef>,
}

/// Enemy stats for editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyStatsDef {
    pub level: i16,
    pub max_hp: i32,
    pub attack: i16,
    pub defense: i16,
    pub mag_def: i16,
    pub will: i16,
    pub vigor: i16,
    pub agile: i16,
    pub quick: i16,
    pub hit: i16,
    pub dodge: i16,
    pub exp: u16,
    pub gold: u16,
    pub counter: i16,
    pub element: String,
}

/// Item drop definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDropDef {
    pub probability: i16,
    pub item: String,
    pub amount: i16,
}

/// Encounter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterDefinition {
    pub initiative: u8,
    pub magic_exp: u8,
    /// Enemy names in this encounter (max 8)
    pub enemies: Vec<String>,
}

// ============================================================================
// Full Debug Schema (for analysis)
// ============================================================================

/// Complete dump of an ENP file structure (for debugging)
#[derive(Debug, Clone, Serialize)]
pub struct EnpDump {
    pub filename: String,
    pub file_size: usize,
    pub header: Vec<HeaderEntry>,
    pub encounters: Vec<EncounterDump>,
    pub enemies: Vec<EnemyDump>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeaderEntry {
    pub enemy_id: i32,
    pub name: String,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EncounterDump {
    pub initiative: u8,
    pub magic_exp: u8,
    pub enemies: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnemyDump {
    pub id: u32,
    pub name: String,
    pub name_jp: String,
    pub stats: EnemyStatsDef,
    pub item_drops: Vec<ItemDropDef>,
    /// Number of AI tasks (tasks are looked up from base game)
    pub task_count: usize,
}

// ============================================================================
// Constants
// ============================================================================

const MAX_ENEMIES: usize = 84;
const ELEMENTS: [&str; 6] = ["Green", "Red", "Purple", "Blue", "Yellow", "Silver"];

// ============================================================================
// Export Functions
// ============================================================================

/// Dump an ENP file to a simplified editable format
pub fn dump_enp_editable(
    data: &[u8],
    filename: &str,
    version: &GameVersion,
    item_db: &ItemDatabase,
) -> Result<EnpDefinition> {
    let dump = dump_enp(data, filename, version, item_db)?;

    Ok(EnpDefinition {
        filename: dump.filename,
        enemies: dump
            .enemies
            .into_iter()
            .map(|e| EnemyDefinition {
                name: e.name,
                name_jp: e.name_jp,
                stats: e.stats,
                item_drops: e.item_drops,
            })
            .collect(),
        encounters: dump
            .encounters
            .into_iter()
            .map(|e| EncounterDefinition {
                initiative: e.initiative,
                magic_exp: e.magic_exp,
                enemies: e.enemies,
            })
            .collect(),
    })
}

/// Dump an ENP file to a full debug format
pub fn dump_enp(
    data: &[u8],
    filename: &str,
    version: &GameVersion,
    item_db: &ItemDatabase,
) -> Result<EnpDump> {
    let enemy_names = enemy_names_map();

    let mut dump = EnpDump {
        filename: filename.to_string(),
        file_size: data.len(),
        header: Vec::new(),
        encounters: Vec::new(),
        enemies: Vec::new(),
    };

    if data.len() < 8 {
        return Ok(dump);
    }

    let mut cursor = Cursor::new(data);

    // Check for multi-segment header
    let mut sig = [0u8; 4];
    for i in 0..4 {
        sig[i] = cursor.read_u8()?;
    }

    if sig == [0x00, 0x00, 0xff, 0xff] {
        return Ok(dump);
    }

    cursor.set_position(0);

    // Read header entries
    let header_entries = MAX_ENEMIES.min(data.len() / 8);
    let mut valid_positions: Vec<(u32, usize)> = Vec::new();
    let mut id_to_name: HashMap<u8, String> = HashMap::new();

    for _ in 0..header_entries {
        if cursor.position() as usize + 8 > data.len() {
            break;
        }

        let enemy_id = cursor.read_i32_be()?;
        let position = cursor.read_i32_be()?;

        if enemy_id >= 0 && position > 0 && (position as usize) < data.len() {
            let name = enemy_names
                .get(&(enemy_id as u32))
                .cloned()
                .unwrap_or_else(|| format!("Enemy_{}", enemy_id));

            dump.header.push(HeaderEntry {
                enemy_id,
                name: name.clone(),
                position,
            });
            valid_positions.push((enemy_id as u32, position as usize));
            id_to_name.insert(enemy_id as u8, name);
        }
    }

    // Calculate encounter region
    let header_size = header_entries * 8;
    let first_enemy_pos = valid_positions
        .iter()
        .map(|(_, pos)| *pos)
        .min()
        .unwrap_or(data.len());

    // Read encounters
    if first_enemy_pos > header_size {
        let encounter_space = first_enemy_pos - header_size;
        let num_encounters = encounter_space / 10;

        for i in 0..num_encounters {
            let offset = header_size + i * 10;
            if offset + 10 > data.len() {
                break;
            }

            let mut enemies = Vec::new();
            for slot_idx in 0..8 {
                let enemy_id = data[offset + 2 + slot_idx];
                if enemy_id != 255 {
                    let name = id_to_name
                        .get(&enemy_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Enemy_{}", enemy_id));
                    enemies.push(name);
                }
            }

            dump.encounters.push(EncounterDump {
                initiative: data[offset],
                magic_exp: data[offset + 1],
                enemies,
            });
        }
    }

    // Read enemy data
    for (enemy_id, position) in &valid_positions {
        let enemy_id = *enemy_id;
        let position = *position;

        if position + Enemy::ENTRY_SIZE > data.len() {
            continue;
        }

        let enemy_data = &data[position..];
        let mut cursor = Cursor::new(enemy_data);

        match Enemy::read_one(&mut cursor, enemy_id, filename, version) {
            Ok(enemy) => {
                let next_pos = valid_positions
                    .iter()
                    .filter(|(_, p)| *p > position)
                    .map(|(_, p)| *p)
                    .min()
                    .unwrap_or(data.len());

                let name = enemy_names
                    .get(&enemy_id)
                    .cloned()
                    .unwrap_or_else(|| enemy.name_jp.clone());

                // Count tasks
                let task_start = position + Enemy::ENTRY_SIZE;
                let mut task_count = 0;
                let mut task_pos = task_start;

                while task_pos + 6 <= next_pos && task_pos + 6 <= data.len() {
                    let type_id = i16::from_be_bytes([data[task_pos], data[task_pos + 1]]);
                    let task_id = i16::from_be_bytes([data[task_pos + 2], data[task_pos + 3]]);

                    if type_id == -1 && task_id == -1 {
                        break;
                    }

                    task_count += 1;
                    task_pos += 6;

                    if task_count > 64 {
                        break;
                    }
                }

                // Build item drops with names
                let item_drops: Vec<ItemDropDef> = enemy
                    .item_drops
                    .iter()
                    .filter(|d| d.item_id >= 0)
                    .map(|d| ItemDropDef {
                        probability: d.probability,
                        item: item_db.name_or_default(d.item_id as i32),
                        amount: d.amount,
                    })
                    .collect();

                let element = ELEMENTS
                    .get(enemy.element_id as usize)
                    .unwrap_or(&"Unknown")
                    .to_string();

                dump.enemies.push(EnemyDump {
                    id: enemy_id,
                    name,
                    name_jp: enemy.name_jp,
                    stats: EnemyStatsDef {
                        level: enemy.level,
                        max_hp: enemy.max_hp,
                        attack: enemy.attack,
                        defense: enemy.defense,
                        mag_def: enemy.mag_def,
                        will: enemy.will,
                        vigor: enemy.vigor,
                        agile: enemy.agile,
                        quick: enemy.quick,
                        hit: enemy.hit,
                        dodge: enemy.dodge,
                        exp: enemy.exp,
                        gold: enemy.gold,
                        counter: enemy.counter,
                        element,
                    },
                    item_drops,
                    task_count,
                });
            }
            Err(_) => continue,
        }
    }

    Ok(dump)
}

// ============================================================================
// EVP File Schema (Event Parameters)
// ============================================================================

/// Maximum number of enemies in EVP header (GC)
const EVP_MAX_ENEMIES: usize = 200;

/// Maximum number of events in EVP (GC)
const EVP_MAX_EVENTS: usize = 250;

/// Simplified EVP definition for editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvpDefinition {
    /// Source filename
    pub filename: String,
    /// Enemies in this file (with stats)
    pub enemies: Vec<EnemyDefinition>,
    /// Battle events (scripted encounters)
    pub events: Vec<EventDefinition>,
}

/// Event definition for editing (scripted battle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    /// Event ID
    pub id: u32,
    /// Magic EXP reward
    pub magic_exp: u8,
    /// Initiative value
    pub initiative: u8,
    /// Defeat condition (0 = Defeat All, 1 = Defeat Boss, etc.)
    pub defeat_condition: String,
    /// Escape condition (0 = Can Escape, 1 = Cannot Escape)
    pub escape_condition: String,
    /// Character slots (party member positions)
    pub characters: Vec<EventCharacterDef>,
    /// Enemy slots (enemy positions)
    pub enemies: Vec<EventEnemyDef>,
}

/// Character slot in an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCharacterDef {
    /// Character name (Vyse, Aika, etc.) or "None"
    pub name: String,
    /// X position
    pub x: i8,
    /// Z position
    pub z: i8,
}

/// Enemy slot in an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnemyDef {
    /// Enemy name (US)
    pub name: String,
    /// X position
    pub x: i8,
    /// Z position
    pub z: i8,
}

/// Defeat condition names
const DEFEAT_COND_NAMES: [&str; 4] = [
    "Must Not Lose",  // 0
    "May Try Again",  // 1
    "May Lose",       // 2
    "Unknown",        // 3+
];

/// Escape condition names
const ESCAPE_COND_NAMES: [&str; 3] = [
    "May Escape",       // 0
    "Must Not Escape",  // 1
    "Unknown",          // 2+
];

/// Character names by ID
const CHARACTER_NAMES: [&str; 6] = ["Vyse", "Aika", "Fina", "Drachma", "Gilder", "Enrique"];

/// Full dump of an EVP file structure (for debugging)
#[derive(Debug, Clone, Serialize)]
pub struct EvpDump {
    pub filename: String,
    pub file_size: usize,
    pub header: Vec<HeaderEntry>,
    pub events: Vec<EventDump>,
    pub enemies: Vec<EnemyDump>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventDump {
    pub id: u32,
    pub magic_exp: u8,
    pub initiative: u8,
    pub defeat_condition: String,
    pub escape_condition: String,
    pub characters: Vec<EventCharacterDef>,
    pub enemies: Vec<EventEnemyDef>,
}

// ============================================================================
// EVP Export Functions
// ============================================================================

/// Dump an EVP file to a simplified editable format
pub fn dump_evp_editable(
    data: &[u8],
    filename: &str,
    version: &GameVersion,
    item_db: &ItemDatabase,
) -> Result<EvpDefinition> {
    let dump = dump_evp(data, filename, version, item_db)?;

    Ok(EvpDefinition {
        filename: dump.filename,
        enemies: dump
            .enemies
            .into_iter()
            .map(|e| EnemyDefinition {
                name: e.name,
                name_jp: e.name_jp,
                stats: e.stats,
                item_drops: e.item_drops,
            })
            .collect(),
        events: dump
            .events
            .into_iter()
            .map(|e| EventDefinition {
                id: e.id,
                magic_exp: e.magic_exp,
                initiative: e.initiative,
                defeat_condition: e.defeat_condition,
                escape_condition: e.escape_condition,
                characters: e.characters,
                enemies: e.enemies,
            })
            .collect(),
    })
}

/// Dump an EVP file to a full debug format
pub fn dump_evp(
    data: &[u8],
    filename: &str,
    version: &GameVersion,
    item_db: &ItemDatabase,
) -> Result<EvpDump> {
    let enemy_names = enemy_names_map();

    let mut dump = EvpDump {
        filename: filename.to_string(),
        file_size: data.len(),
        header: Vec::new(),
        events: Vec::new(),
        enemies: Vec::new(),
    };

    // EVP header: 200 entries * 8 bytes = 1600 bytes
    let header_size = EVP_MAX_ENEMIES * 8;
    // EVP events: 250 events * 37 bytes = 9250 bytes
    let events_size = EVP_MAX_EVENTS * EnemyEvent::ENTRY_SIZE;
    let enemies_start = header_size + events_size;

    if data.len() < enemies_start {
        return Ok(dump);
    }

    let mut cursor = Cursor::new(data);

    // Read header entries
    let mut valid_positions: Vec<(u32, usize)> = Vec::new();
    let mut id_to_name: HashMap<u8, String> = HashMap::new();

    for _ in 0..EVP_MAX_ENEMIES {
        if cursor.position() as usize + 8 > data.len() {
            break;
        }

        let enemy_id = cursor.read_i32_be()?;
        let position = cursor.read_i32_be()?;

        if enemy_id >= 0 && position > 0 && (position as usize) < data.len() {
            let name = enemy_names
                .get(&(enemy_id as u32))
                .cloned()
                .unwrap_or_else(|| format!("Enemy_{}", enemy_id));

            dump.header.push(HeaderEntry {
                enemy_id,
                name: name.clone(),
                position,
            });
            valid_positions.push((enemy_id as u32, position as usize));
            id_to_name.insert(enemy_id as u8, name);
        }
    }

    // Read events section
    cursor.set_position(header_size as u64);

    for event_id in 0..EVP_MAX_EVENTS {
        if cursor.position() as usize + EnemyEvent::ENTRY_SIZE > data.len() {
            break;
        }

        match EnemyEvent::read_one(&mut cursor, event_id as u32, filename) {
            Ok(event) => {
                // Skip empty events
                if event.is_empty() {
                    continue;
                }

                // Build character list
                let characters: Vec<EventCharacterDef> = event
                    .characters
                    .iter()
                    .filter(|c| !c.is_empty())
                    .map(|c| {
                        let name = if c.character_id >= 0
                            && (c.character_id as usize) < CHARACTER_NAMES.len()
                        {
                            CHARACTER_NAMES[c.character_id as usize].to_string()
                        } else {
                            format!("Character_{}", c.character_id)
                        };
                        EventCharacterDef {
                            name,
                            x: c.x,
                            z: c.z,
                        }
                    })
                    .collect();

                // Build enemy list
                let enemies: Vec<EventEnemyDef> = event
                    .enemies
                    .iter()
                    .filter(|e| !e.is_empty())
                    .map(|e| {
                        let name = id_to_name
                            .get(&e.enemy_id)
                            .cloned()
                            .unwrap_or_else(|| format!("Enemy_{}", e.enemy_id));
                        EventEnemyDef {
                            name,
                            x: e.x,
                            z: e.z,
                        }
                    })
                    .collect();

                // Get condition names
                let defeat_idx = event.defeat_cond_id as usize;
                let defeat_condition = if defeat_idx < DEFEAT_COND_NAMES.len() - 1 {
                    DEFEAT_COND_NAMES[defeat_idx].to_string()
                } else {
                    format!("Unknown ({})", event.defeat_cond_id)
                };

                let escape_idx = event.escape_cond_id as usize;
                let escape_condition = if escape_idx < ESCAPE_COND_NAMES.len() - 1 {
                    ESCAPE_COND_NAMES[escape_idx].to_string()
                } else {
                    format!("Unknown ({})", event.escape_cond_id)
                };

                dump.events.push(EventDump {
                    id: event_id as u32,
                    magic_exp: event.magic_exp,
                    initiative: event.initiative,
                    defeat_condition,
                    escape_condition,
                    characters,
                    enemies,
                });
            }
            Err(_) => break,
        }
    }

    // Read enemy data
    for (enemy_id, position) in &valid_positions {
        let enemy_id = *enemy_id;
        let position = *position;

        if position + Enemy::ENTRY_SIZE > data.len() {
            continue;
        }

        let enemy_data = &data[position..];
        let mut cursor = Cursor::new(enemy_data);

        match Enemy::read_one(&mut cursor, enemy_id, filename, version) {
            Ok(enemy) => {
                let next_pos = valid_positions
                    .iter()
                    .filter(|(_, p)| *p > position)
                    .map(|(_, p)| *p)
                    .min()
                    .unwrap_or(data.len());

                let name = enemy_names
                    .get(&enemy_id)
                    .cloned()
                    .unwrap_or_else(|| enemy.name_jp.clone());

                // Count tasks
                let task_start = position + Enemy::ENTRY_SIZE;
                let mut task_count = 0;
                let mut task_pos = task_start;

                while task_pos + 6 <= next_pos && task_pos + 6 <= data.len() {
                    let type_id = i16::from_be_bytes([data[task_pos], data[task_pos + 1]]);
                    let task_id = i16::from_be_bytes([data[task_pos + 2], data[task_pos + 3]]);

                    if type_id == -1 && task_id == -1 {
                        break;
                    }

                    task_count += 1;
                    task_pos += 6;

                    if task_count > 64 {
                        break;
                    }
                }

                // Build item drops with names
                let item_drops: Vec<ItemDropDef> = enemy
                    .item_drops
                    .iter()
                    .filter(|d| d.item_id >= 0)
                    .map(|d| ItemDropDef {
                        probability: d.probability,
                        item: item_db.name_or_default(d.item_id as i32),
                        amount: d.amount,
                    })
                    .collect();

                let element = ELEMENTS
                    .get(enemy.element_id as usize)
                    .unwrap_or(&"Unknown")
                    .to_string();

                dump.enemies.push(EnemyDump {
                    id: enemy_id,
                    name,
                    name_jp: enemy.name_jp,
                    stats: EnemyStatsDef {
                        level: enemy.level,
                        max_hp: enemy.max_hp,
                        attack: enemy.attack,
                        defense: enemy.defense,
                        mag_def: enemy.mag_def,
                        will: enemy.will,
                        vigor: enemy.vigor,
                        agile: enemy.agile,
                        quick: enemy.quick,
                        hit: enemy.hit,
                        dodge: enemy.dodge,
                        exp: enemy.exp,
                        gold: enemy.gold,
                        counter: enemy.counter,
                        element,
                    },
                    item_drops,
                    task_count,
                });
            }
            Err(_) => continue,
        }
    }

    Ok(dump)
}
