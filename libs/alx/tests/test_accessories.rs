//! Integration tests for accessory reading.

mod common;

#[test]
fn test_read_accessories() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let accessories = game.read_accessories().unwrap();

    // Should have 80 accessories (0xA0..0xF0)
    assert_eq!(accessories.len(), 80, "Expected 80 accessories");

    // Check first accessory (ID 160 = 0xA0 = "Gemstone Ring")
    let first = &accessories[0];
    assert_eq!(first.id, 160);
    println!("First accessory: ID={}, Name='{}'", first.id, first.name);

    assert!(!first.name.is_empty(), "Accessory name should not be empty");

    // Check trait values from reference CSV
    // Gemstone Ring has: Trait 1 ID=18 (MagDef), Value=21
    println!("First accessory traits: {:?}", first.traits);
}

#[test]
fn test_accessory_gemstone_ring() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let accessories = game.read_accessories().unwrap();

    // Compare with known values from reference CSV
    // Row 1: 160,Gemstone Ring,0b00111010,...,18,MagDef,0,21,...
    let gemstone_ring = &accessories[0];
    assert_eq!(gemstone_ring.id, 160);
    assert_eq!(gemstone_ring.name, "Gemstone Ring");

    // Character flags: 0b00111010 = V,A,F,E can equip
    assert!(gemstone_ring.character_flags.can_equip_vyse());
    assert!(gemstone_ring.character_flags.can_equip_aika());
    assert!(gemstone_ring.character_flags.can_equip_fina());
    assert!(!gemstone_ring.character_flags.can_equip_drachma());
    assert!(gemstone_ring.character_flags.can_equip_enrique());
    assert!(!gemstone_ring.character_flags.can_equip_gilder());

    // Sell percent = 50
    assert_eq!(gemstone_ring.sell_percent, 50);

    // Buy price = 150
    assert_eq!(gemstone_ring.buy_price, 150);

    // Trait 1: ID=18 (MagDef), Value=21
    assert_eq!(gemstone_ring.traits[0].id, 18);
    assert_eq!(gemstone_ring.traits[0].value, 21);

    // Trait 2-4: ID=-1 (None)
    assert_eq!(gemstone_ring.traits[1].id, -1);
    assert_eq!(gemstone_ring.traits[2].id, -1);
    assert_eq!(gemstone_ring.traits[3].id, -1);

    println!("✓ Gemstone Ring matches reference data!");
}

#[test]
fn test_accessory_last_entry() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let accessories = game.read_accessories().unwrap();

    // Last accessory should be ID 239 (0xEF)
    let last = accessories.last().unwrap();
    assert_eq!(last.id, 239);

    // It's a dummy entry in the reference data
    println!("Last accessory: ID={}, Name='{}'", last.id, last.name);
}
