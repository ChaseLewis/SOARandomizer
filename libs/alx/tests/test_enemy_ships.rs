//! Integration tests for EnemyShip entries.

mod common;

#[test]
fn test_read_enemy_ships() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let ships = game.read_enemy_ships().unwrap();

    // Should have 45 enemy ships (IDs 0-44)
    assert_eq!(ships.len(), 45, "Expected 45 enemy ships");

    // First enemy ship should be Valuan Warship
    let first = &ships[0];
    assert_eq!(first.id, 0);
    assert_eq!(first.name, "Valuan Warship");

    println!("✓ Read {} enemy ships", ships.len());
}

#[test]
fn test_enemy_ship_stats() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let ships = game.read_enemy_ships().unwrap();

    // Valuan Warship stats
    let warship = &ships[0];
    assert_eq!(warship.max_hp, 12000);
    assert_eq!(warship.will, 20);
    assert_eq!(warship.defense, 40);
    assert_eq!(warship.exp, 400);
    assert_eq!(warship.gold, 300);

    println!("✓ Enemy ship stats verified");
}

#[test]
fn test_enemy_ship_armaments() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let ships = game.read_enemy_ships().unwrap();

    // Valuan Warship has Main Cannon and Secondary Cannon
    let warship = &ships[0];
    assert_eq!(warship.armaments[0].type_id, 0); // Main Cannon
    assert_eq!(warship.armaments[0].attack, 120);
    assert_eq!(warship.armaments[1].type_id, 1); // Secondary Cannon
    assert_eq!(warship.armaments[1].attack, 95);
    assert_eq!(warship.armaments[2].type_id, -1); // None

    println!("✓ Enemy ship armaments verified");
}
