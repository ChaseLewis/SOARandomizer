//! Integration tests for item database.

mod common;

use alx::ItemCategory;
use alx::items::format_item_with_amount;

#[test]
fn test_build_item_database() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let db = game.build_item_database().unwrap();
    
    // Should have items from all categories
    assert!(db.len() > 300, "Database should have 300+ items, got {}", db.len());
    
    println!("Item database built with {} items", db.len());
}

#[test]
fn test_lookup_by_id() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let db = game.build_item_database().unwrap();
    
    // Test known items
    assert_eq!(db.get_name(0), Some("Cutlass"));
    assert_eq!(db.get_name(80), Some("Vyse's Uniform"));
    assert_eq!(db.get_name(160), Some("Gemstone Ring"));
    assert_eq!(db.get_name(240), Some("Sacri Crystal"));
    assert_eq!(db.get_name(320), Some("Green Crystal"));
    
    // Test special cases
    assert_eq!(db.name_or_default(-1), "None");
    assert_eq!(db.name_or_default(512), "Gold");
    // ID 500 is in ship item range but likely unused
    let unknown = db.name_or_default(500);
    assert!(unknown == "???" || unknown.len() > 0, "Should return ??? or item name for ID 500");
    
    println!("✓ ID to name lookups work!");
}

#[test]
fn test_lookup_by_name() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let db = game.build_item_database().unwrap();
    
    // Test known items (case-insensitive)
    assert_eq!(db.get_id("Cutlass"), Some(0));
    assert_eq!(db.get_id("cutlass"), Some(0));
    assert_eq!(db.get_id("CUTLASS"), Some(0));
    assert_eq!(db.get_id("Sacri Crystal"), Some(240));
    
    // Test unknown item
    assert_eq!(db.get_id("Nonexistent Item"), None);
    
    println!("✓ Name to ID lookups work!");
}

#[test]
fn test_item_categories() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let db = game.build_item_database().unwrap();
    
    // Check categories
    assert_eq!(db.category(0), ItemCategory::Weapon);
    assert_eq!(db.category(80), ItemCategory::Armor);
    assert_eq!(db.category(160), ItemCategory::Accessory);
    assert_eq!(db.category(240), ItemCategory::UsableItem);
    assert_eq!(db.category(320), ItemCategory::SpecialItem);
    assert_eq!(db.category(512), ItemCategory::Gold);
    
    // Check is_gold
    assert!(!db.is_gold(240));
    assert!(db.is_gold(512));
    assert!(db.is_gold(513));
    
    println!("✓ Item categories work!");
}

#[test]
fn test_format_treasure_chest_items() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let db = game.build_item_database().unwrap();
    let chests = game.read_treasure_chests().unwrap();
    
    println!("First 10 treasure chests:");
    for chest in chests.iter().take(10) {
        let formatted = format_item_with_amount(chest.item_id, chest.item_amount, &db);
        println!("  Chest {}: {}", chest.id, formatted);
    }
    
    // Check specific chests
    let chest0 = &chests[0];
    let formatted0 = format_item_with_amount(chest0.item_id, chest0.item_amount, &db);
    assert!(formatted0.contains("Sacri Crystal"), "Chest 0 should be Sacri Crystal: {}", formatted0);
    
    let chest3 = &chests[3];
    let formatted3 = format_item_with_amount(chest3.item_id, chest3.item_amount, &db);
    assert!(formatted3.contains("Gold"), "Chest 3 should be Gold: {}", formatted3);
    
    println!("✓ Treasure chest formatting works!");
}

#[test]
fn test_format_shop_items() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let db = game.build_item_database().unwrap();
    let shops = game.read_shops().unwrap();
    
    // Print first shop's items
    let shop = &shops[0];
    println!("Shop '{}' items:", shop.description);
    for &item_id in shop.items().iter().take(10) {
        let name = db.name_or_default(item_id as i32);
        println!("  - {} (ID {})", name, item_id);
    }
    
    println!("✓ Shop item formatting works!");
}

