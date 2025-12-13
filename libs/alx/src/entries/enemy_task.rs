//! Enemy task entry type.

use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::Result;
use crate::io::BinaryReader;
use crate::lookups::{action_name, action_param_name, branch_name, branch_param_name, task_type_name};

/// An enemy AI task/action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyTask {
    /// Task slot ID (1-based)
    pub id: u32,
    /// Enemy ID this task belongs to
    pub enemy_id: u32,
    /// Source file filter (can be "*" for main definition or specific filename)
    pub filter: String,
    /// Enemy JP name (looked up)
    pub enemy_name_jp: String,
    /// Enemy US name (looked up)
    pub enemy_name_us: String,
    /// Type ID (-1=empty, 0=Branch, 1=Action)
    pub type_id: i16,
    /// Task ID (action or branch type)
    pub task_id: i16,
    /// Task name (looked up based on type and task_id)
    pub task_name: String,
    /// Parameter ID
    pub param_id: i16,
    /// Parameter name (looked up based on type and param_id)
    pub param_name: String,
}

impl EnemyTask {
    /// Size of one entry in bytes.
    pub const ENTRY_SIZE: usize = 6;

    /// Read a single enemy task from binary data.
    pub fn read_one(
        cursor: &mut Cursor<&[u8]>,
        id: u32,
        enemy_id: u32,
        filter: &str,
    ) -> Result<Self> {
        let type_id = cursor.read_i16_be()?;
        let task_id = cursor.read_i16_be()?;
        let param_id = cursor.read_i16_be()?;

        Ok(Self {
            id,
            enemy_id,
            filter: filter.to_string(),
            enemy_name_jp: String::new(),
            enemy_name_us: String::new(),
            type_id,
            task_id,
            task_name: String::new(),
            param_id,
            param_name: String::new(),
        })
    }

    /// Populate lookup names based on type_id, task_id, param_id.
    /// For action tasks with task_id < 550, the task_name should be looked up
    /// from magic (500-549) or super moves (0-499) separately.
    pub fn populate_names(&mut self) {
        match self.type_id {
            0 => {
                // Branch
                self.task_name = branch_name(self.task_id).to_string();
                self.param_name = branch_param_name(self.param_id);
            }
            1 => {
                // Action
                let action = action_name(self.task_id);
                if action.is_empty() {
                    // Will be populated from magic/super moves later
                    self.task_name = "???".to_string();
                } else {
                    self.task_name = action.to_string();
                }
                self.param_name = action_param_name(self.param_id).to_string();
            }
            _ => {
                self.task_name = "None".to_string();
                self.param_name = "None".to_string();
            }
        }
    }

    /// Get type name
    pub fn type_name(&self) -> &'static str {
        task_type_name(self.type_id)
    }

    /// Check if this task is empty
    pub fn is_empty(&self) -> bool {
        self.type_id == -1
    }

    /// Check if this task matches another for aggregation purposes.
    /// Two tasks match if they have the same type_id, task_id, and param_id.
    pub fn matches_content(&self, other: &Self) -> bool {
        self.id == other.id
            && self.type_id == other.type_id
            && self.task_id == other.task_id
            && self.param_id == other.param_id
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
