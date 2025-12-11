//! Integration tests for treasure chest reading.

mod common;

#[test]
fn test_read_treasure_chests() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let chests = game.read_treasure_chests().unwrap();
    
    // Should have 119 chests (0x00..0x77)
    assert_eq!(chests.len(), 119, "Expected 119 treasure chests");
    
    println!("Treasure chests loaded: {}", chests.len());
    for chest in chests.iter().take(5) {
        if chest.is_gold() {
            println!("  Chest {}: {} gold", chest.id, chest.item_amount);
        } else {
            println!("  Chest {}: Item {} x{}", chest.id, chest.item_id, chest.item_amount);
        }
    }
}

#[test]
fn test_treasure_chest_first() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let chests = game.read_treasure_chests().unwrap();
    
    // First chest: Pirate Isle Village, Sacri Crystal x3
    let first = &chests[0];
    assert_eq!(first.id, 0);
    assert_eq!(first.item_id, 240); // Sacri Crystal
    assert_eq!(first.item_amount, 3);
    assert!(!first.is_gold());
    
    println!("✓ First treasure chest matches reference!");
}

#[test]
fn test_treasure_chest_gold() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let chests = game.read_treasure_chests().unwrap();
    
    // Find gold chests
    let gold_chests: Vec<_> = chests.iter().filter(|c| c.is_gold()).collect();
    
    println!("Gold chests: {}", gold_chests.len());
    for chest in gold_chests.iter().take(5) {
        println!("  Chest {}: {} gold", chest.id, chest.item_amount);
    }
    
    // Should have some gold chests
    assert!(!gold_chests.is_empty(), "Should have gold chests");
    
    // Check chest 3 which should be 150 gold
    let chest3 = &chests[3];
    assert!(chest3.is_gold(), "Chest 3 should be gold");
    assert_eq!(chest3.item_amount, 150, "Chest 3 should have 150 gold");
}

#[test]
fn test_treasure_chest_moonberry() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let chests = game.read_treasure_chests().unwrap();
    
    // Count Moonberry chests (item ID 258)
    let moonberry_chests: Vec<_> = chests.iter()
        .filter(|c| c.item_id == 258)
        .collect();
    
    println!("Moonberry chests: {}", moonberry_chests.len());
    
    // Moonberries are important collectibles
    assert!(!moonberry_chests.is_empty(), "Should have Moonberry chests");
}

