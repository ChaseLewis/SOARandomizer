//! ENP (Enemy Parameters) and EVP (Enemy Event) file parsers.

use crate::entries::{Enemy, EnemyTask};
use crate::error::{Error, Result};
use crate::game::region::GameVersion;
use crate::io::BinaryReader;
use std::io::Cursor;

/// ENP file signature
const FILE_SIG: [u8; 4] = [0x00, 0x00, 0xff, 0xff];

/// Maximum number of enemy slots in ENP header (GC)
const MAX_ENEMIES: usize = 84;

/// Maximum number of tasks per enemy (GC)
const MAX_TASKS: usize = 64;

/// Maximum number of enemy slots in EVP header (GC)
const EVP_MAX_ENEMIES: usize = 200;

/// Maximum number of events in EVP (GC)
const EVP_MAX_EVENTS: usize = 250;

/// Size of an EnemyEvent entry (estimated)
const EVP_EVENT_SIZE: usize = 20;

/// A node in the ENP header pointing to enemy data
#[derive(Debug, Clone)]
struct EnemyNode {
    id: u32,
    pos: usize,
}

/// Parsed data from an ENP file
#[derive(Debug, Clone, Default)]
pub struct EnpData {
    /// All enemies parsed from the file
    pub enemies: Vec<Enemy>,
    /// All enemy tasks parsed from the file  
    pub tasks: Vec<EnemyTask>,
}

/// Parse an ENP file from raw bytes.
pub fn parse_enp(data: &[u8], filename: &str, version: &GameVersion) -> Result<EnpData> {
    let mut result = EnpData::default();

    if data.len() < 8 {
        return Ok(result);
    }

    let mut cursor = Cursor::new(data);

    // Check for segment header
    let mut sig = [0u8; 4];
    for i in 0..4 {
        sig[i] = cursor.read_u8()?;
    }

    if sig == FILE_SIG {
        // Multi-segment file
        let num_segments = cursor.read_i16_be()? as usize;
        let check = cursor.read_i16_be()?;
        if check != -1 {
            return Err(Error::ParseError {
                offset: 4,
                message: "ENP segments corrupted".to_string(),
            });
        }

        // Read segment info
        let mut segments = Vec::new();
        for _ in 0..num_segments {
            let seg_name = cursor.read_string_fixed(20)?;
            let seg_pos = cursor.read_i32_be()? as usize;
            let seg_size = cursor.read_i32_be()? as usize;
            let _check = cursor.read_i32_be()?;

            // Convert .bin extension to .enp for GC
            let seg_name = seg_name.replace(".bin", ".enp");
            segments.push((seg_name, seg_pos, seg_size));
        }

        // Parse each segment
        for (seg_name, seg_pos, seg_size) in segments {
            if seg_pos + seg_size > data.len() {
                continue;
            }
            let segment_data = &data[seg_pos..seg_pos + seg_size];
            parse_enp_segment(segment_data, &seg_name, version, &mut result)?;
        }
    } else {
        // Single segment file - reset and parse
        parse_enp_segment(data, filename, version, &mut result)?;
    }

    Ok(result)
}

/// Parse a single ENP segment
fn parse_enp_segment(
    data: &[u8],
    filename: &str,
    version: &GameVersion,
    result: &mut EnpData,
) -> Result<()> {
    if data.len() < 8 {
        return Ok(());
    }

    let mut cursor = Cursor::new(data);

    // Read header - array of (enemy_id: i32, position: i32) pairs
    // Read until we hit invalid entry or run out of data
    let mut nodes = Vec::new();
    let header_max = MAX_ENEMIES.min(data.len() / 8);

    for _ in 0..header_max {
        if cursor.position() as usize + 8 > data.len() {
            break;
        }
        let id = cursor.read_i32_be()?;
        let pos = cursor.read_i32_be()?;

        // Valid entry: non-negative ID and position within bounds
        if id >= 0 && pos >= 0 && (pos as usize) < data.len() {
            nodes.push(EnemyNode {
                id: id as u32,
                pos: pos as usize,
            });
        } else if id < 0 {
            // Negative ID marks end of header (usually 0xFFFFFFFF)
            break;
        }
    }

    if nodes.is_empty() {
        return Ok(());
    }

    // Read each enemy from their positions
    for node in &nodes {
        if node.pos + Enemy::ENTRY_SIZE > data.len() {
            continue;
        }

        let mut cursor = Cursor::new(&data[node.pos..]);

        // Read enemy data
        match Enemy::read_one(&mut cursor, node.id, filename, version) {
            Ok(enemy) => {
                result.enemies.push(enemy);

                // Read tasks until we hit an empty one or EOF mark
                let mut task_id = 1u32;
                let remaining = data.len() - node.pos - cursor.position() as usize;
                let max_tasks_possible = remaining / EnemyTask::ENTRY_SIZE;

                for _ in 0..max_tasks_possible.min(MAX_TASKS) {
                    if cursor.position() as usize + EnemyTask::ENTRY_SIZE > data.len() - node.pos {
                        break;
                    }

                    let task = EnemyTask::read_one(&mut cursor, task_id, node.id, filename)?;

                    // Check for EOF mark (-1 in type_id indicates end or we hit actual EOF mark)
                    if task.type_id == -1 && task.task_id == -1 {
                        // Check if this might be the EOF mark (0xFFFF)
                        break;
                    }

                    if task.type_id != -1 {
                        result.tasks.push(task);
                    }

                    task_id += 1;
                }
            }
            Err(_) => {
                // Skip this enemy if we can't read it
                continue;
            }
        }
    }

    Ok(())
}

/// Parse a DAT file (ecinit*.dat or ebinit*.dat) from raw bytes.
/// DAT files contain a single enemy with their ID derived from the filename.
pub fn parse_dat_file(data: &[u8], filename: &str, version: &GameVersion) -> Result<EnpData> {
    let mut result = EnpData::default();

    if data.len() < Enemy::ENTRY_SIZE {
        return Ok(result);
    }

    // Extract ID from filename: ecinit001.dat -> 1, ebinit001.dat -> 1 + 0x80 = 129
    let id = extract_dat_id(filename);
    if id.is_none() {
        return Ok(result);
    }
    let id = id.unwrap();

    let mut cursor = Cursor::new(data);

    // Read the single enemy
    match Enemy::read_one(&mut cursor, id, filename, version) {
        Ok(enemy) => {
            result.enemies.push(enemy);

            // Read tasks
            let mut task_id = 1u32;
            let remaining = data.len().saturating_sub(cursor.position() as usize);
            let max_tasks_possible = remaining / EnemyTask::ENTRY_SIZE;

            for _ in 0..max_tasks_possible.min(MAX_TASKS) {
                if cursor.position() as usize + EnemyTask::ENTRY_SIZE > data.len() {
                    break;
                }

                let task = EnemyTask::read_one(&mut cursor, task_id, id, filename)?;

                if task.type_id == -1 && task.task_id == -1 {
                    break;
                }

                if task.type_id != -1 {
                    result.tasks.push(task);
                }

                task_id += 1;
            }
        }
        Err(_) => {}
    }

    Ok(result)
}

/// Extract enemy ID from DAT filename.
/// ecinit001.dat -> 1 (regular enemy)
/// ebinit001.dat -> 129 (boss, 1 + 0x80)
fn extract_dat_id(filename: &str) -> Option<u32> {
    let lower = filename.to_lowercase();
    let is_boss = lower.starts_with("ebinit");

    // Find the 3-digit number in the filename
    let digits: String = lower
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(3)
        .collect();

    if digits.len() == 3 {
        let num: u32 = digits.parse().ok()?;
        if is_boss {
            Some(num + 0x80)
        } else {
            Some(num)
        }
    } else {
        None
    }
}

/// Parse an EVP (enemy event) file from raw bytes.
/// EVP files have a different structure: header + events + enemies
pub fn parse_evp(data: &[u8], filename: &str, version: &GameVersion) -> Result<EnpData> {
    let mut result = EnpData::default();

    // EVP header: 200 entries * 8 bytes = 1600 bytes
    let header_size = EVP_MAX_ENEMIES * 8;
    // EVP events: 250 events * ~20 bytes = 5000 bytes
    let events_size = EVP_MAX_EVENTS * EVP_EVENT_SIZE;
    let enemies_start = header_size + events_size;

    if data.len() < enemies_start {
        return Ok(result);
    }

    let mut cursor = Cursor::new(data);

    // Read header - array of (enemy_id: i32, position: i32) pairs
    let mut nodes = Vec::new();
    for _ in 0..EVP_MAX_ENEMIES {
        let id = cursor.read_i32_be()?;
        let pos = cursor.read_i32_be()?;

        // Valid entry: non-negative ID and positive position
        if id >= 0 && pos > 0 && (pos as usize) < data.len() {
            nodes.push(EnemyNode {
                id: id as u32,
                pos: pos as usize,
            });
        }
    }

    if nodes.is_empty() {
        return Ok(result);
    }

    // Skip events section (we don't need it for enemies)
    // Events start at header_size and end at enemies_start

    // Read each enemy from their positions
    for node in &nodes {
        if node.pos + Enemy::ENTRY_SIZE > data.len() {
            continue;
        }

        let mut cursor = Cursor::new(&data[node.pos..]);

        // Read enemy data
        match Enemy::read_one(&mut cursor, node.id, filename, version) {
            Ok(enemy) => {
                result.enemies.push(enemy);

                // Read tasks
                let mut task_id = 1u32;
                let remaining = data
                    .len()
                    .saturating_sub(node.pos + cursor.position() as usize);
                let max_tasks_possible = remaining / EnemyTask::ENTRY_SIZE;

                for _ in 0..max_tasks_possible.min(MAX_TASKS) {
                    let task_start = node.pos + cursor.position() as usize;
                    if task_start + EnemyTask::ENTRY_SIZE > data.len() {
                        break;
                    }

                    let task = EnemyTask::read_one(&mut cursor, task_id, node.id, filename)?;

                    if task.type_id == -1 && task.task_id == -1 {
                        break;
                    }

                    if task.type_id != -1 {
                        result.tasks.push(task);
                    }

                    task_id += 1;
                }
            }
            Err(_) => continue,
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_constants() {
        assert_eq!(super::MAX_ENEMIES, 84);
        assert_eq!(super::MAX_TASKS, 64);
        assert_eq!(super::EVP_MAX_ENEMIES, 200);
        assert_eq!(super::EVP_MAX_EVENTS, 250);
    }
}
