//! Integration tests for ShipItem entries.

mod common;

#[test]
fn test_read_ship_items() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let items = game.read_ship_items().unwrap();

    // Should have 30 ship items (IDs 480-509)
    assert_eq!(items.len(), 30, "Expected 30 ship items");

    // First item should be Bomb
    let first = &items[0];
    assert_eq!(first.id, 480);
    assert_eq!(first.name, "Bomb");

    println!("✓ Read {} ship items", items.len());
}

#[test]
fn test_ship_item_stats() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let items = game.read_ship_items().unwrap();

    // Bomb stats
    let bomb = &items[0];
    assert_eq!(bomb.buy_price, 100);
    assert_eq!(bomb.consume, 100);
    assert_eq!(bomb.ship_effect_base, 80);
    assert!(bomb.usable_on_ship(), "Bomb should be usable on ship");
    assert!(
        !bomb.usable_in_battle(),
        "Bomb should not be usable in regular battle"
    );

    println!("✓ Ship item stats verified");
}
