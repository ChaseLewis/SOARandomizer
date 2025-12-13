//! ENP file builder - constructs ENP files from JSON definitions.

use crate::entries::Enemy;
use crate::error::{Error, Result};
use crate::io::enp_dump::{EnemyDefinition, EnpDefinition};
use crate::io::BinaryWriter;
use crate::items::ItemDatabase;
use std::collections::HashMap;
use std::io::Cursor;

/// Element name to ID mapping
const ELEMENTS: [&str; 6] = ["Green", "Red", "Purple", "Blue", "Yellow", "Silver"];

/// Maximum header entries in an ENP file
const MAX_HEADER_ENTRIES: usize = 84;

/// Maximum encounters in an ENP file
const MAX_ENCOUNTERS: usize = 32;

/// Size of encounter entry
const ENCOUNTER_SIZE: usize = 10;

/// Raw enemy data including stats and AI tasks
#[derive(Debug, Clone)]
pub struct RawEnemyData {
    /// Enemy ID
    pub id: u32,
    /// US name for lookup
    pub name: String,
    /// Level (for matching variants)
    pub level: i16,
    /// Raw bytes of enemy data (stats + AI tasks)
    pub data: Vec<u8>,
    /// Just the stats portion (first 136 bytes)
    pub stats_size: usize,
}

impl RawEnemyData {
    /// Extract level from raw data bytes (offset 92-93, big-endian i16)
    fn level_from_data(data: &[u8]) -> i16 {
        if data.len() >= 94 {
            i16::from_be_bytes([data[92], data[93]])
        } else {
            0
        }
    }
}

/// Enemy database for looking up raw enemy data by name.
/// Stores a single entry per name (for file-specific lookups).
#[derive(Debug, Clone, Default)]
pub struct EnemyDatabase {
    /// Map from US enemy name to raw data
    enemies: HashMap<String, RawEnemyData>,
}

impl EnemyDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an enemy to the database
    pub fn add(&mut self, name: String, id: u32, data: Vec<u8>) {
        let level = RawEnemyData::level_from_data(&data);
        self.enemies.insert(
            name.clone(),
            RawEnemyData {
                id,
                name,
                level,
                data,
                stats_size: Enemy::ENTRY_SIZE,
            },
        );
    }

    /// Look up an enemy by US name
    pub fn get(&self, name: &str) -> Option<&RawEnemyData> {
        self.enemies.get(name)
    }

    /// Get all enemy names
    pub fn names(&self) -> Vec<&str> {
        self.enemies.keys().map(|s| s.as_str()).collect()
    }

    /// Number of enemies in database
    pub fn len(&self) -> usize {
        self.enemies.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.enemies.is_empty()
    }
}

/// Global enemy database that stores ALL enemy variants.
/// Multiple enemies can have the same name but different stats/levels.
#[derive(Debug, Clone, Default)]
pub struct GlobalEnemyDatabase {
    /// Map from US enemy name to list of variants
    enemies: HashMap<String, Vec<RawEnemyData>>,
}

impl GlobalEnemyDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an enemy variant to the database
    pub fn add(&mut self, name: String, id: u32, data: Vec<u8>) {
        let level = RawEnemyData::level_from_data(&data);
        let entry = RawEnemyData {
            id,
            name: name.clone(),
            level,
            data,
            stats_size: Enemy::ENTRY_SIZE,
        };
        self.enemies.entry(name).or_default().push(entry);
    }

    /// Look up an enemy by name, returning the variant with the closest level
    pub fn get_closest(&self, name: &str, target_level: i16) -> Option<&RawEnemyData> {
        let variants = self.enemies.get(name)?;
        if variants.is_empty() {
            return None;
        }

        // Find the variant with the closest level
        variants
            .iter()
            .min_by_key(|e| (e.level - target_level).abs())
    }

    /// Look up any enemy by name (returns the first variant)
    pub fn get_any(&self, name: &str) -> Option<&RawEnemyData> {
        self.enemies.get(name)?.first()
    }

    /// Get all enemy names
    pub fn names(&self) -> Vec<&str> {
        self.enemies.keys().map(|s| s.as_str()).collect()
    }

    /// Total number of enemy variants
    pub fn total_variants(&self) -> usize {
        self.enemies.values().map(|v| v.len()).sum()
    }

    /// Number of unique enemy names
    pub fn len(&self) -> usize {
        self.enemies.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.enemies.is_empty()
    }
}

/// Patch raw enemy data with values from an EnemyDefinition
/// This updates stats and item drops while preserving AI tasks
fn patch_enemy_data(raw: &[u8], def: &EnemyDefinition, item_db: &ItemDatabase) -> Vec<u8> {
    let mut data = raw.to_vec();

    if data.len() < Enemy::ENTRY_SIZE {
        return data; // Not enough data to patch
    }

    // Parse element name to ID
    let element_id = ELEMENTS
        .iter()
        .position(|&e| e.eq_ignore_ascii_case(&def.stats.element))
        .unwrap_or(0) as i8;

    // Patch element_id at offset 23
    data[23] = element_id as u8;

    // Patch counter at offset 28-29
    write_i16_be(&mut data, 28, def.stats.counter);

    // Patch exp at offset 30-31
    write_u16_be(&mut data, 30, def.stats.exp);

    // Patch gold at offset 32-33
    write_u16_be(&mut data, 32, def.stats.gold);

    // Patch max_hp at offset 36-39
    write_i32_be(&mut data, 36, def.stats.max_hp);

    // Patch level at offset 92-93
    write_i16_be(&mut data, 92, def.stats.level);

    // Patch will at offset 94-95
    write_i16_be(&mut data, 94, def.stats.will);

    // Patch vigor at offset 96-97
    write_i16_be(&mut data, 96, def.stats.vigor);

    // Patch agile at offset 98-99
    write_i16_be(&mut data, 98, def.stats.agile);

    // Patch quick at offset 100-101
    write_i16_be(&mut data, 100, def.stats.quick);

    // Patch attack at offset 102-103
    write_i16_be(&mut data, 102, def.stats.attack);

    // Patch defense at offset 104-105
    write_i16_be(&mut data, 104, def.stats.defense);

    // Patch mag_def at offset 106-107
    write_i16_be(&mut data, 106, def.stats.mag_def);

    // Patch hit at offset 108-109
    write_i16_be(&mut data, 108, def.stats.hit);

    // Patch dodge at offset 110-111
    write_i16_be(&mut data, 110, def.stats.dodge);

    // Patch item drops at offset 114-137 (4 drops, 6 bytes each)
    for i in 0..4 {
        let offset = 114 + i * 6;
        if i < def.item_drops.len() {
            let drop = &def.item_drops[i];
            write_i16_be(&mut data, offset, drop.probability);
            write_i16_be(&mut data, offset + 2, drop.amount);

            // Look up item ID from name
            let item_id = if drop.item.eq_ignore_ascii_case("None") {
                -1i16
            } else if drop.item.eq_ignore_ascii_case("Gold") {
                0x200i16 // Gold ID
            } else {
                item_db.get_id(&drop.item).unwrap_or(-1) as i16
            };
            write_i16_be(&mut data, offset + 4, item_id);
        } else {
            // Empty drop slot
            write_i16_be(&mut data, offset, -1);
            write_i16_be(&mut data, offset + 2, -1);
            write_i16_be(&mut data, offset + 4, -1);
        }
    }

    data
}

/// Write an i16 in big-endian at the specified offset
fn write_i16_be(data: &mut [u8], offset: usize, value: i16) {
    let bytes = value.to_be_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
}

/// Write a u16 in big-endian at the specified offset
fn write_u16_be(data: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_be_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
}

/// Write an i32 in big-endian at the specified offset
fn write_i32_be(data: &mut [u8], offset: usize, value: i32) {
    let bytes = value.to_be_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
    data[offset + 2] = bytes[2];
    data[offset + 3] = bytes[3];
}

/// Build an ENP file from a definition and enemy databases, applying edits from the definition.
///
/// - `db`: File-specific database (enemies from the original ENP file)
/// - `global_db`: Optional global database with all enemies from all files (fallback)
///
/// For each enemy in the definition:
/// 1. Try to find it in the file-specific database
/// 2. If not found, try the global database (matching by closest level)
pub fn build_enp(
    def: &EnpDefinition,
    db: &EnemyDatabase,
    global_db: Option<&GlobalEnemyDatabase>,
    item_db: &ItemDatabase,
) -> Result<Vec<u8>> {
    // Validate enemy count
    if def.enemies.len() > MAX_HEADER_ENTRIES {
        return Err(Error::ParseError {
            offset: 0,
            message: format!(
                "Too many enemies: {} (max {})",
                def.enemies.len(),
                MAX_HEADER_ENTRIES
            ),
        });
    }

    // Validate encounter count
    if def.encounters.len() > MAX_ENCOUNTERS {
        return Err(Error::ParseError {
            offset: 0,
            message: format!(
                "Too many encounters: {} (max {})",
                def.encounters.len(),
                MAX_ENCOUNTERS
            ),
        });
    }

    // Look up all enemies and collect their patched data
    let mut enemy_data: Vec<(u32, Vec<u8>)> = Vec::new();
    // Map from enemy name to GLOBAL enemy ID (for encounter references)
    let mut name_to_global_id: HashMap<String, u8> = HashMap::new();

    for enemy_def in def.enemies.iter() {
        // First try file-specific database
        let raw = if let Some(r) = db.get(&enemy_def.name) {
            r
        } else if let Some(gdb) = global_db {
            // Fallback to global database, matching by closest level
            gdb.get_closest(&enemy_def.name, enemy_def.stats.level)
                .ok_or_else(|| Error::ParseError {
                    offset: 0,
                    message: format!("Enemy not found in any database: {}", enemy_def.name),
                })?
        } else {
            return Err(Error::ParseError {
                offset: 0,
                message: format!("Enemy not found in database: {}", enemy_def.name),
            });
        };

        // Apply patches from the definition
        let patched = patch_enemy_data(&raw.data, enemy_def, item_db);
        enemy_data.push((raw.id, patched));
        // Store the GLOBAL enemy ID for encounter references
        name_to_global_id.insert(enemy_def.name.clone(), raw.id as u8);
    }

    // Calculate positions
    let header_size = MAX_HEADER_ENTRIES * 8;
    let encounters_size = def.encounters.len() * ENCOUNTER_SIZE;
    let enemies_start = header_size + encounters_size;

    // Calculate enemy positions
    let mut enemy_positions: Vec<usize> = Vec::new();
    let mut current_pos = enemies_start;

    for (_, data) in &enemy_data {
        enemy_positions.push(current_pos);
        current_pos += data.len();
    }

    let total_size = current_pos;

    // Build the ENP file
    let mut result = vec![0u8; total_size];
    let mut cursor = Cursor::new(&mut result[..]);

    // Write header (84 entries)
    for i in 0..MAX_HEADER_ENTRIES {
        if i < enemy_data.len() {
            let (enemy_id, _) = &enemy_data[i];
            let position = enemy_positions[i] as i32;
            cursor.write_i32_be(*enemy_id as i32)?;
            cursor.write_i32_be(position)?;
        } else {
            // Empty slot
            cursor.write_i32_be(-1)?;
            cursor.write_i32_be(-1)?;
        }
    }

    // Write encounters
    for encounter in &def.encounters {
        // Initiative
        cursor.write_u8(encounter.initiative)?;
        // Magic EXP
        cursor.write_u8(encounter.magic_exp)?;

        // Enemy slots (8 slots) - uses GLOBAL enemy IDs
        for slot in 0..8 {
            if slot < encounter.enemies.len() {
                let enemy_name = &encounter.enemies[slot];
                let global_id =
                    name_to_global_id
                        .get(enemy_name)
                        .ok_or_else(|| Error::ParseError {
                            offset: cursor.position() as usize,
                            message: format!("Encounter references unknown enemy: {}", enemy_name),
                        })?;
                cursor.write_u8(*global_id)?;
            } else {
                cursor.write_u8(255)?; // Empty slot
            }
        }
    }

    // Write enemy data
    for (i, (_, data)) in enemy_data.iter().enumerate() {
        let pos = enemy_positions[i];
        result[pos..pos + data.len()].copy_from_slice(data);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_database() {
        let mut db = EnemyDatabase::new();
        db.add("Soldier".to_string(), 0, vec![0u8; 200]);
        db.add("Guard".to_string(), 1, vec![0u8; 200]);

        assert_eq!(db.len(), 2);
        assert!(db.get("Soldier").is_some());
        assert!(db.get("Unknown").is_none());
    }
}
