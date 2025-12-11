//! Integration tests for usable item reading.

mod common;

#[test]
fn test_read_usable_items() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let items = game.read_usable_items().unwrap();

    // Should have 80 usable items (0xF0..0x140)
    assert_eq!(items.len(), 80, "Expected 80 usable items");

    let first = &items[0];
    assert_eq!(first.id, 240); // 0xF0
    println!("First usable item: ID={}, Name='{}'", first.id, first.name);

    // Verify it's Sacri Crystal
    assert_eq!(first.name, "Sacri Crystal");
}

#[test]
fn test_usable_item_sacri_crystal() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let items = game.read_usable_items().unwrap();

    let sacri = &items[0];
    assert_eq!(sacri.id, 240);
    assert_eq!(sacri.name, "Sacri Crystal");

    // Occasion flags: 0b0110 = Menu + Battle
    assert!(sacri.occasion_flags.can_use_menu());
    assert!(sacri.occasion_flags.can_use_battle());
    assert!(!sacri.occasion_flags.can_use_ship());

    // Effect: Recover HP
    assert_eq!(sacri.effect_id, 31);
    assert_eq!(sacri.effect_name(), "Recover HP");

    // Scope: Single PC
    assert_eq!(sacri.scope_id, 1);
    assert_eq!(sacri.scope_name(), "Single PC");

    // Stats
    assert_eq!(sacri.consume_percent, 100);
    assert_eq!(sacri.sell_percent, 50);
    assert_eq!(sacri.buy_price, 20);
    assert_eq!(sacri.effect_base, 500);

    println!("✓ Sacri Crystal matches expected data!");
}

#[test]
fn test_usable_item_effects() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let items = game.read_usable_items().unwrap();

    // Count items by effect
    let mut hp_recover = 0;
    let mut mp_recover = 0;
    let mut attack = 0;

    for item in &items {
        match item.effect_id {
            31 | 32 => hp_recover += 1,
            48 | 49 => mp_recover += 1,
            63 | 64 => attack += 1,
            _ => {}
        }
    }

    println!("HP Recovery items: {}", hp_recover);
    println!("MP Recovery items: {}", mp_recover);
    println!("Attack items: {}", attack);

    assert!(hp_recover > 0, "Should have HP recovery items");
    assert!(mp_recover > 0, "Should have MP recovery items");
}
