//! Integration tests for ShipCannon entries.

mod common;

#[test]
fn test_read_ship_cannons() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let cannons = game.read_ship_cannons().unwrap();

    // Should have 40 cannons (IDs 400-439)
    assert_eq!(cannons.len(), 40, "Expected 40 ship cannons");

    // First cannon should be Main Cannon
    let first = &cannons[0];
    assert_eq!(first.id, 400);
    assert_eq!(first.name, "Main Cannon");

    println!("✓ Read {} ship cannons", cannons.len());
}

#[test]
fn test_ship_cannon_stats() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let cannons = game.read_ship_cannons().unwrap();

    // Main Cannon stats
    let main = &cannons[0];
    assert_eq!(main.attack, 35);
    assert_eq!(main.hit, 80);
    assert_eq!(main.sp, 4);
    assert_eq!(main.buy_price, 450);

    // Standard Cannon (ID 401)
    let standard = &cannons[1];
    assert_eq!(standard.name, "Standard Cannon");
    assert_eq!(standard.attack, 40);

    println!("✓ Ship cannon stats verified");
}

#[test]
fn test_ship_cannon_equip_flags() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let cannons = game.read_ship_cannons().unwrap();

    // Main Cannon has ship_flags 0b00110000 (L1=X, L2=X)
    let main = &cannons[0];
    assert_eq!(main.ship_flags, 0b00110000);
    assert!(main.can_equip_ship(0)); // L1
    assert!(main.can_equip_ship(1)); // L2
    assert!(!main.can_equip_ship(2)); // D1

    println!("✓ Ship cannon equip flags verified");
}
