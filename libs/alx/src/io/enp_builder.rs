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

/// Bake multiple ENP segment files into a single multi-segment ENP file.
///
/// This is used for files like `a099a_ep.enp` which combines `a099a_01ep.enp` through
/// `a099a_13ep.enp` into a single baked file.
///
/// # Arguments
/// * `segments` - Vec of (segment_name, segment_data) pairs. Names should be like "a099a_01ep.enp"
///
/// # Returns
/// The baked multi-segment ENP file as raw bytes (uncompressed)
pub fn bake_enp_segments(segments: &[(&str, &[u8])]) -> Result<Vec<u8>> {
    if segments.is_empty() {
        return Err(Error::ParseError {
            offset: 0,
            message: "No segments to bake".to_string(),
        });
    }

    // Calculate header size: 8 bytes header + 32 bytes per segment
    let header_size = 8 + (segments.len() * 32);

    // Calculate total size and positions
    let mut current_pos = header_size;
    let mut segment_info: Vec<(String, usize, usize)> = Vec::new();

    for (name, data) in segments {
        // Convert .enp to .bin for the stored name (game expects .bin extension)
        let stored_name = name.replace(".enp", ".bin");
        segment_info.push((stored_name, current_pos, data.len()));
        current_pos += data.len();
    }

    let total_size = current_pos;
    let mut output = vec![0u8; total_size];

    // Write header signature: 00 00 FF FF
    output[0] = 0x00;
    output[1] = 0x00;
    output[2] = 0xFF;
    output[3] = 0xFF;

    // Write number of segments (i16 BE)
    let num_segments = segments.len() as i16;
    output[4] = (num_segments >> 8) as u8;
    output[5] = num_segments as u8;

    // Write check value: FF FF (-1 as i16 BE)
    output[6] = 0xFF;
    output[7] = 0xFF;

    // Write segment entries (32 bytes each)
    let mut offset = 8;
    for (stored_name, pos, size) in &segment_info {
        // Write name (20 bytes, null-padded)
        let name_bytes = stored_name.as_bytes();
        let copy_len = name_bytes.len().min(20);
        output[offset..offset + copy_len].copy_from_slice(&name_bytes[..copy_len]);
        // Rest is already zeroed
        offset += 20;

        // Write position (i32 BE)
        let pos_bytes = (*pos as i32).to_be_bytes();
        output[offset..offset + 4].copy_from_slice(&pos_bytes);
        offset += 4;

        // Write size (i32 BE)
        let size_bytes = (*size as i32).to_be_bytes();
        output[offset..offset + 4].copy_from_slice(&size_bytes);
        offset += 4;

        // Write check value (i32 BE, use 0)
        output[offset..offset + 4].copy_from_slice(&[0, 0, 0, 0]);
        offset += 4;
    }

    // Write segment data
    for ((_, data), (_, pos, _)) in segments.iter().zip(segment_info.iter()) {
        output[*pos..*pos + data.len()].copy_from_slice(data);
    }

    Ok(output)
}

/// List of segment filenames for the a099a baked file
pub const A099A_SEGMENTS: [&str; 13] = [
    "a099a_01ep.enp",
    "a099a_02ep.enp",
    "a099a_03ep.enp",
    "a099a_04ep.enp",
    "a099a_05ep.enp",
    "a099a_06ep.enp",
    "a099a_07ep.enp",
    "a099a_08ep.enp",
    "a099a_09ep.enp",
    "a099a_10ep.enp",
    "a099a_11ep.enp",
    "a099a_12ep.enp",
    "a099a_13ep.enp",
];

/// The baked filename for a099a
pub const A099A_BAKED_FILENAME: &str = "a099a_ep.enp";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bake_enp_segments() {
        // Create dummy segment data
        let seg1 = vec![1u8; 100];
        let seg2 = vec![2u8; 200];
        let seg3 = vec![3u8; 150];

        let segments: Vec<(&str, &[u8])> = vec![
            ("a099a_01ep.enp", &seg1),
            ("a099a_02ep.enp", &seg2),
            ("a099a_03ep.enp", &seg3),
        ];

        let baked = bake_enp_segments(&segments).unwrap();

        // Check header signature
        assert_eq!(&baked[0..4], &[0x00, 0x00, 0xFF, 0xFF]);

        // Check segment count (3)
        assert_eq!(&baked[4..6], &[0x00, 0x03]);

        // Check header check value
        assert_eq!(&baked[6..8], &[0xFF, 0xFF]);

        // Header size = 8 + 3*32 = 104 bytes
        let header_size = 104;

        // Check first segment name starts at offset 8
        let name1 = String::from_utf8_lossy(&baked[8..28]);
        assert!(name1.starts_with("a099a_01ep.bin"));

        // Check segment data is at the right positions
        assert_eq!(&baked[header_size..header_size + 100], &seg1[..]);
        assert_eq!(&baked[header_size + 100..header_size + 300], &seg2[..]);
        assert_eq!(&baked[header_size + 300..header_size + 450], &seg3[..]);

        println!("Baked file size: {} bytes", baked.len());
    }

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
