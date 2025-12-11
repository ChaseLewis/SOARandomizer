//! Integration tests for EnemySuperMove entries.

mod common;

#[test]
fn test_read_enemy_super_moves() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let moves = game.read_enemy_super_moves().unwrap();
    
    // Should have 309 enemy super moves (IDs 0-308)
    assert_eq!(moves.len(), 309, "Expected 309 enemy super moves");
    
    // First move should be Volcanic Blast
    let first = &moves[0];
    assert_eq!(first.id, 0);
    assert_eq!(first.name, "Volcanic Blast");
    
    println!("✓ Read {} enemy super moves", moves.len());
}

#[test]
fn test_enemy_super_move_stats() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let moves = game.read_enemy_super_moves().unwrap();
    
    // Check Volcanic Blast stats
    let volcanic = &moves[0];
    assert_eq!(volcanic.category_id, 0);  // S-Move
    assert_eq!(volcanic.effect_id, 0);    // Damage
    assert_eq!(volcanic.scope_id, 2);     // All PCs
    assert_eq!(volcanic.element_id, 1);   // Red
    
    // Check the category name helper
    assert_eq!(volcanic.category_name(), "S-Move");
    assert_eq!(volcanic.element_name(), "Red");
    
    println!("✓ Enemy super move stats verified");
}

