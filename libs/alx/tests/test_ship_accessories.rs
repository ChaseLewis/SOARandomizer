//! Integration tests for ShipAccessory entries.

mod common;

#[test]
fn test_read_ship_accessories() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let accessories = game.read_ship_accessories().unwrap();
    
    // Should have 40 accessories (IDs 440-479)
    assert_eq!(accessories.len(), 40, "Expected 40 ship accessories");
    
    // First accessory should be Rogue Figure
    let first = &accessories[0];
    assert_eq!(first.id, 440);
    assert_eq!(first.name, "Rogue Figure");
    
    println!("✓ Read {} ship accessories", accessories.len());
}

#[test]
fn test_ship_accessory_traits() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let accessories = game.read_ship_accessories().unwrap();
    
    // Rogue Figure has MagDef trait
    let rogue = &accessories[0];
    assert_eq!(rogue.traits[0].id, 3); // MagDef
    assert_eq!(rogue.traits[0].value, 20);
    assert_eq!(rogue.traits[1].id, -1); // None
    
    println!("✓ Ship accessory traits verified");
}

#[test]
fn test_ship_accessory_equip_flags() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let accessories = game.read_ship_accessories().unwrap();
    
    // Rogue Figure has ship_flags 0b00111110 (L1=X, L2=X, D1=X, D2=X, D3=X)
    let rogue = &accessories[0];
    assert_eq!(rogue.ship_flags, 0b00111110);
    assert!(rogue.can_equip_ship(0)); // L1
    assert!(rogue.can_equip_ship(1)); // L2
    assert!(rogue.can_equip_ship(2)); // D1
    assert!(rogue.can_equip_ship(3)); // D2
    assert!(rogue.can_equip_ship(4)); // D3
    
    println!("✓ Ship accessory equip flags verified");
}

