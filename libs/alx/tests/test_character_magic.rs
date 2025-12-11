//! Integration tests for character magic reading.

mod common;

#[test]
fn test_read_character_magic() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let magics = game.read_character_magic().unwrap();

    // Should have 36 spells (0x00..0x24)
    assert_eq!(magics.len(), 36, "Expected 36 character magics");

    println!("Character magics loaded: {}", magics.len());
    for m in magics.iter().take(5) {
        println!(
            "  {} (ID {}): {} element, SP {}",
            m.name,
            m.id,
            m.element_name(),
            m.effect_sp
        );
    }
}

#[test]
fn test_magic_increm() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let magics = game.read_character_magic().unwrap();

    let increm = &magics[0];
    assert_eq!(increm.id, 0);
    assert_eq!(increm.name, "Increm");

    // Element: Red
    assert_eq!(increm.element_id, 1);
    assert_eq!(increm.element_name(), "Red");

    // Effect: Incr Attack & Defense
    assert_eq!(increm.effect_id, 21);

    // SP cost
    assert_eq!(increm.effect_sp, 4);

    // Scope: Single PC
    assert_eq!(increm.scope_id, 1);

    println!("✓ Increm spell matches reference!");
}

#[test]
fn test_magic_elements() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let magics = game.read_character_magic().unwrap();

    // Count spells by element
    let mut counts = [0; 7];
    for magic in &magics {
        if magic.element_id >= 0 && magic.element_id < 7 {
            counts[magic.element_id as usize] += 1;
        }
    }

    println!("Spells per element:");
    let names = [
        "Green", "Red", "Purple", "Blue", "Yellow", "Silver", "Neutral",
    ];
    for (i, name) in names.iter().enumerate() {
        if counts[i] > 0 {
            println!("  {}: {}", name, counts[i]);
        }
    }
}
