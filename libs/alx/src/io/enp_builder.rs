//! ENP file builder - constructs ENP files from JSON definitions.

use crate::entries::Enemy;
use crate::error::{Error, Result};
use crate::io::enp_dump::EnpDefinition;
use crate::io::BinaryWriter;
use std::collections::HashMap;
use std::io::Cursor;

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
    /// Raw bytes of enemy data (stats + AI tasks)
    pub data: Vec<u8>,
    /// Just the stats portion (first 136 bytes)
    pub stats_size: usize,
}

/// Enemy database for looking up raw enemy data by name
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
        self.enemies.insert(
            name.clone(),
            RawEnemyData {
                id,
                name,
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

/// Build an ENP file from a definition and enemy database
pub fn build_enp(def: &EnpDefinition, db: &EnemyDatabase) -> Result<Vec<u8>> {
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

    // Look up all enemies and collect their raw data
    let mut enemy_data: Vec<(u32, &RawEnemyData)> = Vec::new();
    let mut name_to_local_id: HashMap<String, u8> = HashMap::new();

    for (local_id, enemy_def) in def.enemies.iter().enumerate() {
        let raw = db.get(&enemy_def.name).ok_or_else(|| Error::ParseError {
            offset: 0,
            message: format!("Enemy not found in database: {}", enemy_def.name),
        })?;

        enemy_data.push((raw.id, raw));
        name_to_local_id.insert(enemy_def.name.clone(), local_id as u8);
    }

    // Calculate positions
    let header_size = MAX_HEADER_ENTRIES * 8;
    let encounters_size = def.encounters.len() * ENCOUNTER_SIZE;
    let enemies_start = header_size + encounters_size;

    // Calculate enemy positions
    let mut enemy_positions: Vec<usize> = Vec::new();
    let mut current_pos = enemies_start;

    for (_, raw) in &enemy_data {
        enemy_positions.push(current_pos);
        current_pos += raw.data.len();
    }

    let total_size = current_pos;

    // Build the ENP file
    let mut result = vec![0u8; total_size];
    let mut cursor = Cursor::new(&mut result[..]);

    // Write header (84 entries)
    for i in 0..MAX_HEADER_ENTRIES {
        if i < enemy_data.len() {
            let (enemy_id, _) = enemy_data[i];
            let position = enemy_positions[i] as i32;
            cursor.write_i32_be(enemy_id as i32)?;
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

        // Enemy slots (8 slots)
        for slot in 0..8 {
            if slot < encounter.enemies.len() {
                let enemy_name = &encounter.enemies[slot];
                let local_id = name_to_local_id.get(enemy_name).ok_or_else(|| {
                    Error::ParseError {
                        offset: cursor.position() as usize,
                        message: format!(
                            "Encounter references unknown enemy: {}",
                            enemy_name
                        ),
                    }
                })?;
                cursor.write_u8(*local_id)?;
            } else {
                cursor.write_u8(255)?; // Empty slot
            }
        }
    }

    // Write enemy data
    for (i, (_, raw)) in enemy_data.iter().enumerate() {
        let pos = enemy_positions[i];
        result[pos..pos + raw.data.len()].copy_from_slice(&raw.data);
    }

    Ok(result)
}

/// Extract raw enemy data from a parsed ENP/DAT file
/// Returns a map of enemy name -> raw bytes
pub fn extract_enemy_data(
    data: &[u8],
    enemies: &[(u32, String, usize)], // (id, name, position)
) -> HashMap<String, Vec<u8>> {
    let mut result = HashMap::new();

    // Sort enemies by position to find boundaries
    let mut sorted: Vec<_> = enemies.iter().collect();
    sorted.sort_by_key(|(_, _, pos)| *pos);

    for i in 0..sorted.len() {
        let (_id, name, pos) = sorted[i];
        let end = if i + 1 < sorted.len() {
            sorted[i + 1].2
        } else {
            data.len()
        };

        if *pos < data.len() && end <= data.len() {
            result.insert(name.clone(), data[*pos..end].to_vec());
        }
    }

    result
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

