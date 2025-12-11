//! Integration tests for shop reading.

mod common;

#[test]
fn test_read_shops() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let shops = game.read_shops().unwrap();
    
    // Should have 43 shops (0x00..0x2B)
    assert_eq!(shops.len(), 43, "Expected 43 shops");
    
    println!("Shops loaded: {}", shops.len());
    for shop in shops.iter().take(5) {
        println!("  Shop {}: '{}' ({} items)", 
            shop.id, shop.description, shop.item_count());
    }
}

#[test]
fn test_shop_zacks_weapons() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let shops = game.read_shops().unwrap();
    
    // First shop should be "Zack's Weapons"
    let zacks = &shops[0];
    assert_eq!(zacks.id, 0);
    assert_eq!(zacks.description, "Zack's Weapons");
    
    // Should have some items
    let items = zacks.items();
    assert!(!items.is_empty(), "Shop should have items");
    
    println!("Zack's Weapons items: {:?}", items);
    
    // First item should be Pirate Cutlass (ID 1)
    assert_eq!(items[0], 1, "First item should be Pirate Cutlass");
    
    println!("✓ Zack's Weapons shop matches reference!");
}

#[test]
fn test_shop_item_counts() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let shops = game.read_shops().unwrap();
    
    // Count shops by item count
    let mut empty = 0;
    let mut small = 0;  // 1-10 items
    let mut medium = 0; // 11-20 items
    let mut large = 0;  // 21+ items
    
    for shop in &shops {
        match shop.item_count() {
            0 => empty += 1,
            1..=10 => small += 1,
            11..=20 => medium += 1,
            _ => large += 1,
        }
    }
    
    println!("Shop sizes:");
    println!("  Empty: {}", empty);
    println!("  Small (1-10): {}", small);
    println!("  Medium (11-20): {}", medium);
    println!("  Large (21+): {}", large);
}

