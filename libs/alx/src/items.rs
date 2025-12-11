//! Item database for looking up item names and IDs.
//!
//! This module provides utilities for converting between item IDs and names.

use std::collections::HashMap;

use crate::entries::{
    Accessory, Armor, ShipAccessory, ShipCannon, ShipItem, SpecialItem, UsableItem, Weapon,
};

/// Item category based on ID range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemCategory {
    /// Weapon (0x00..0x50)
    Weapon,
    /// Armor (0x50..0xA0)
    Armor,
    /// Accessory (0xA0..0xF0)
    Accessory,
    /// Usable item (0xF0..0x140)
    UsableItem,
    /// Special item (0x140..0x190)
    SpecialItem,
    /// Ship cannon (0x190..0x1C0)
    ShipCannon,
    /// Ship accessory (0x1C0..0x1F0)
    ShipAccessory,
    /// Ship item (0x1F0..0x200)
    ShipItem,
    /// Gold (0x200+)
    Gold,
    /// Unknown
    Unknown,
}

impl ItemCategory {
    /// Get the category for an item ID.
    pub fn from_id(id: i32) -> Self {
        match id {
            0x00..=0x4F => ItemCategory::Weapon,
            0x50..=0x9F => ItemCategory::Armor,
            0xA0..=0xEF => ItemCategory::Accessory,
            0xF0..=0x13F => ItemCategory::UsableItem,
            0x140..=0x18F => ItemCategory::SpecialItem,
            0x190..=0x1BF => ItemCategory::ShipCannon,
            0x1C0..=0x1EF => ItemCategory::ShipAccessory,
            0x1F0..=0x1FF => ItemCategory::ShipItem,
            0x200.. => ItemCategory::Gold,
            _ => ItemCategory::Unknown,
        }
    }
}

/// Database of all items in the game, providing name lookups.
#[derive(Debug, Clone, Default)]
pub struct ItemDatabase {
    /// Map from item ID to item name
    id_to_name: HashMap<i32, String>,
    /// Map from item name (lowercase) to item ID
    name_to_id: HashMap<String, i32>,
}

impl ItemDatabase {
    /// Create a new empty item database.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the database from game data.
    pub fn from_game_data(
        weapons: &[Weapon],
        armors: &[Armor],
        accessories: &[Accessory],
        usable_items: &[UsableItem],
        special_items: &[SpecialItem],
        ship_cannons: &[ShipCannon],
        ship_accessories: &[ShipAccessory],
        ship_items: &[ShipItem],
    ) -> Self {
        let mut db = Self::new();

        for item in weapons {
            db.insert(item.id as i32, &item.name);
        }
        for item in armors {
            db.insert(item.id as i32, &item.name);
        }
        for item in accessories {
            db.insert(item.id as i32, &item.name);
        }
        for item in usable_items {
            db.insert(item.id as i32, &item.name);
        }
        for item in special_items {
            db.insert(item.id as i32, &item.name);
        }
        for item in ship_cannons {
            db.insert(item.id as i32, &item.name);
        }
        for item in ship_accessories {
            db.insert(item.id as i32, &item.name);
        }
        for item in ship_items {
            db.insert(item.id as i32, &item.name);
        }

        // Add Gold entry
        db.insert(0x200, "Gold");

        db
    }

    /// Insert an item into the database.
    pub fn insert(&mut self, id: i32, name: &str) {
        self.id_to_name.insert(id, name.to_string());
        self.name_to_id.insert(name.to_lowercase(), id);
    }

    /// Get the name for an item ID.
    pub fn get_name(&self, id: i32) -> Option<&str> {
        // Handle Gold specially
        if id >= 0x200 {
            return Some("Gold");
        }
        self.id_to_name.get(&id).map(|s| s.as_str())
    }

    /// Get the name for an item ID, or a default string if not found.
    pub fn name_or(&self, id: i32, default: &str) -> String {
        self.get_name(id).unwrap_or(default).to_string()
    }

    /// Get the name for an item ID, with "None" for -1 and "???" for unknown.
    pub fn name_or_default(&self, id: i32) -> String {
        if id == -1 {
            "None".to_string()
        } else if id >= 0x200 {
            "Gold".to_string()
        } else {
            self.id_to_name
                .get(&id)
                .cloned()
                .unwrap_or_else(|| "???".to_string())
        }
    }

    /// Get the ID for an item name (case-insensitive).
    pub fn get_id(&self, name: &str) -> Option<i32> {
        self.name_to_id.get(&name.to_lowercase()).copied()
    }

    /// Get the ID for an item name, or a default if not found.
    pub fn id_or(&self, name: &str, default: i32) -> i32 {
        self.get_id(name).unwrap_or(default)
    }

    /// Get the category for an item ID.
    pub fn category(&self, id: i32) -> ItemCategory {
        ItemCategory::from_id(id)
    }

    /// Check if an ID represents gold.
    pub fn is_gold(&self, id: i32) -> bool {
        id >= 0x200
    }

    /// Get the number of items in the database.
    pub fn len(&self) -> usize {
        self.id_to_name.len()
    }

    /// Check if the database is empty.
    pub fn is_empty(&self) -> bool {
        self.id_to_name.is_empty()
    }

    /// Iterate over all (id, name) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&i32, &String)> {
        self.id_to_name.iter()
    }
}

/// Format an item for display (ID + name).
pub fn format_item(id: i32, db: &ItemDatabase) -> String {
    let name = db.name_or_default(id);
    if id == -1 {
        "None".to_string()
    } else if id >= 0x200 {
        "Gold".to_string()
    } else {
        format!("{} (ID {})", name, id)
    }
}

/// Format an item with amount for display.
pub fn format_item_with_amount(id: i32, amount: i32, db: &ItemDatabase) -> String {
    if id == -1 {
        "None".to_string()
    } else if id >= 0x200 {
        format!("{} Gold", amount)
    } else {
        let name = db.name_or_default(id);
        if amount == 1 {
            name.to_string()
        } else {
            format!("{} x{}", name, amount)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_category() {
        assert_eq!(ItemCategory::from_id(0), ItemCategory::Weapon);
        assert_eq!(ItemCategory::from_id(79), ItemCategory::Weapon);
        assert_eq!(ItemCategory::from_id(80), ItemCategory::Armor);
        assert_eq!(ItemCategory::from_id(160), ItemCategory::Accessory);
        assert_eq!(ItemCategory::from_id(240), ItemCategory::UsableItem);
        assert_eq!(ItemCategory::from_id(320), ItemCategory::SpecialItem);
        assert_eq!(ItemCategory::from_id(512), ItemCategory::Gold);
        assert_eq!(ItemCategory::from_id(513), ItemCategory::Gold);
    }

    #[test]
    fn test_item_database() {
        let mut db = ItemDatabase::new();
        db.insert(0, "Cutlass");
        db.insert(80, "Vyse's Uniform");
        db.insert(240, "Sacri Crystal");

        assert_eq!(db.get_name(0), Some("Cutlass"));
        assert_eq!(db.get_id("Cutlass"), Some(0));
        assert_eq!(db.get_id("cutlass"), Some(0)); // case-insensitive
        assert_eq!(db.get_id("CUTLASS"), Some(0));

        assert_eq!(db.name_or_default(-1), "None");
        assert_eq!(db.name_or_default(100), "???"); // Unknown item ID
        assert_eq!(db.name_or_default(512), "Gold"); // Gold IDs are 0x200+
    }

    #[test]
    fn test_format_item() {
        let mut db = ItemDatabase::new();
        db.insert(240, "Sacri Crystal");

        assert_eq!(format_item(-1, &db), "None");
        assert_eq!(format_item(512, &db), "Gold");
        assert_eq!(format_item(240, &db), "Sacri Crystal (ID 240)");

        assert_eq!(format_item_with_amount(-1, 0, &db), "None");
        assert_eq!(format_item_with_amount(512, 150, &db), "150 Gold");
        assert_eq!(format_item_with_amount(240, 1, &db), "Sacri Crystal");
        assert_eq!(format_item_with_amount(240, 3, &db), "Sacri Crystal x3");
    }
}
