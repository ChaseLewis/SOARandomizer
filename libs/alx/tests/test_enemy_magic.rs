//! Integration tests for EnemyMagic entries.

mod common;

#[test]
fn test_read_enemy_magic() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let magic = game.read_enemy_magic().unwrap();
    
    // Should have 36 enemy magic spells (IDs 0-35)
    assert_eq!(magic.len(), 36, "Expected 36 enemy magic spells");
    
    // First spell should be Increm
    let first = &magic[0];
    assert_eq!(first.id, 0);
    assert_eq!(first.name, "Increm");
    
    println!("✓ Read {} enemy magic spells", magic.len());
}

#[test]
fn test_enemy_magic_stats() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let magic = game.read_enemy_magic().unwrap();
    
    // Check Increm stats
    let increm = &magic[0];
    assert_eq!(increm.category_id, 1); // Magic
    assert_eq!(increm.effect_id, 21);  // Incr Attack & Defense
    assert_eq!(increm.scope_id, 3);    // Single EC
    
    println!("✓ Enemy magic stats verified");
}

