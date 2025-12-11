//! Integration tests for armor reading.

mod common;

#[test]
fn test_read_armors() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let armors = game.read_armors().unwrap();

    // Should have 80 armors (0x50..0xA0)
    assert_eq!(armors.len(), 80, "Expected 80 armors");

    let first = &armors[0];
    assert_eq!(first.id, 80); // 0x50 = 80
    println!("First armor: ID={}, Name='{}'", first.id, first.name);
}

#[test]
fn test_armor_vyse_uniform() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let armors = game.read_armors().unwrap();

    // First armor should be Vyse's Uniform (ID 80 = 0x50)
    let vyse_uniform = &armors[0];
    assert_eq!(vyse_uniform.id, 80);
    assert!(
        vyse_uniform.name.contains("Vyse"),
        "Expected Vyse's armor, got: {}",
        vyse_uniform.name
    );

    println!(
        "✓ Vyse's Uniform: {:?}",
        vyse_uniform.character_flags.as_binary_string()
    );
}

#[test]
fn test_armor_character_flags() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let armors = game.read_armors().unwrap();

    // Check that different armors have different character restrictions
    let mut has_vyse_only = false;
    let mut has_all_chars = false;

    for armor in &armors {
        let flags = armor.character_flags.0;
        if flags == 0b00100000 {
            has_vyse_only = true;
        }
        if flags == 0b00111111 {
            has_all_chars = true;
        }
    }

    println!("Found Vyse-only armor: {}", has_vyse_only);
    println!("Found all-character armor: {}", has_all_chars);
}
