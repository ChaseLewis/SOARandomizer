//! Enemy task entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::io::BinaryReader;

/// An enemy AI task/action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyTask {
    /// Task slot ID (1-based)
    pub id: u32,
    /// Enemy ID this task belongs to
    pub enemy_id: u32,
    /// Source file filter
    pub filter: String,
    /// Type ID (-1=empty, 0=Branch, 1=Action)
    pub type_id: i16,
    /// Task ID (action or branch type)
    pub task_id: i16,
    /// Parameter ID
    pub param_id: i16,
}

impl EnemyTask {
    /// Size of one entry in bytes.
    pub const ENTRY_SIZE: usize = 6;

    /// Read a single enemy task from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, enemy_id: u32, filter: &str) -> Result<Self> {
        let type_id = cursor.read_i16_be()?;
        let task_id = cursor.read_i16_be()?;
        let param_id = cursor.read_i16_be()?;
        
        Ok(Self {
            id,
            enemy_id,
            filter: filter.to_string(),
            type_id,
            task_id,
            param_id,
        })
    }

    /// Get type name
    pub fn type_name(&self) -> &'static str {
        match self.type_id {
            -1 => "None",
            0 => "Branch",
            1 => "Action",
            _ => "???",
        }
    }

    /// Check if this task is empty
    pub fn is_empty(&self) -> bool {
        self.type_id == -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_size() {
        assert_eq!(EnemyTask::ENTRY_SIZE, 6);
    }
}

