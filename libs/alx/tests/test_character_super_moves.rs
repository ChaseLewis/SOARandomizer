//! Integration tests for CharacterSuperMove entries.

mod common;

#[test]
fn test_read_character_super_moves() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let super_moves = game.read_character_super_moves().unwrap();
    
    // Should have 26 S-Moves (IDs 36-61)
    assert_eq!(super_moves.len(), 26, "Expected 26 S-Moves");
    
    // First S-Move should be Cutlass Fury (Vyse)
    let first = &super_moves[0];
    assert_eq!(first.id, 36);
    assert_eq!(first.name, "Cutlass Fury");
    assert_eq!(first.element_id, 6); // Neutral
    assert_eq!(first.category_id, 0); // Vyse
    
    println!("✓ Read {} S-Moves", super_moves.len());
}

#[test]
fn test_character_super_move_stats() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let super_moves = game.read_character_super_moves().unwrap();
    
    // Cutlass Fury (ID 36)
    let cutlass = &super_moves[0];
    assert_eq!(cutlass.effect_sp, 7, "Cutlass Fury costs 7 SP");
    assert_eq!(cutlass.effect_id, 0, "Cutlass Fury is Damage effect");
    assert_eq!(cutlass.scope_id, 3, "Cutlass Fury targets Single EC");
    assert!(cutlass.usable_in_battle(), "Cutlass Fury usable in battle");
    assert!(!cutlass.usable_in_menu(), "Cutlass Fury not usable in menu");
    
    // Rain of Swords (ID 37)
    let rain = &super_moves[1];
    assert_eq!(rain.name, "Rain of Swords");
    assert_eq!(rain.effect_sp, 14, "Rain of Swords costs 14 SP");
    assert_eq!(rain.scope_id, 4, "Rain of Swords targets All ECs");
    
    println!("✓ S-Move stats verified");
}

#[test]
fn test_character_super_move_descriptions() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let super_moves = game.read_character_super_moves().unwrap();
    
    // Check that descriptions are loaded
    let cutlass = &super_moves[0];
    assert!(!cutlass.description.is_empty(), "Cutlass Fury should have description");
    assert!(cutlass.description.contains("Vyse"), "Description should mention Vyse");
    
    println!("✓ S-Move descriptions loaded");
    println!("  Cutlass Fury: {:?}", cutlass.description);
}

