//! Integration tests for character reading.

mod common;

#[test]
fn test_read_characters() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let characters = game.read_characters().unwrap();

    // Should have 6 playable characters
    assert_eq!(characters.len(), 6, "Expected 6 characters");

    println!("Characters loaded:");
    for c in &characters {
        println!(
            "  {} (ID {}): Age {}, {}",
            c.name,
            c.id,
            c.age,
            c.gender_name()
        );
    }
}

#[test]
fn test_character_vyse() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let characters = game.read_characters().unwrap();

    let vyse = &characters[0];
    assert_eq!(vyse.id, 0);
    assert_eq!(vyse.name, "Vyse");
    assert_eq!(vyse.age, 17);
    assert_eq!(vyse.gender_id, 0);
    assert_eq!(vyse.gender_name(), "Male");

    // Element: Red
    assert_eq!(vyse.element_id, 1);
    assert_eq!(vyse.element_name(), "Red");

    // Stats from reference
    assert_eq!(vyse.hp, 420);
    assert_eq!(vyse.max_hp, 420);
    assert_eq!(vyse.max_hp_growth, 84);
    assert_eq!(vyse.power, 23);
    assert_eq!(vyse.will, 16);
    assert_eq!(vyse.vigor, 19);
    assert_eq!(vyse.agile, 11);
    assert_eq!(vyse.quick, 22);

    // Equipment
    assert_eq!(vyse.weapon_id, 0); // Cutlass
    assert_eq!(vyse.armor_id, 80); // Vyse's Uniform
    assert_eq!(vyse.accessory_id, 204); // Skyseer Goggles

    println!("✓ Vyse data matches reference!");
}

#[test]
fn test_all_character_names() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let characters = game.read_characters().unwrap();

    let expected_names = ["Vyse", "Aika", "Fina", "Drachma", "Enrique", "Gilder"];

    for (c, expected) in characters.iter().zip(expected_names.iter()) {
        assert_eq!(&c.name, expected, "Character {} name mismatch", c.id);
    }

    println!("✓ All character names match!");
}
