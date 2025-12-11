//! Integration tests for ExpBoost entries.

mod common;

#[test]
fn test_read_exp_boosts() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let boosts = game.read_exp_boosts().unwrap();
    
    // Should have 3 exp boosts (Drachma, Enrique, Gilder)
    assert_eq!(boosts.len(), 3, "Expected 3 exp boosts");
    
    // First boost should be Drachma
    let drachma = &boosts[0];
    assert_eq!(drachma.id, 3);
    assert_eq!(drachma.character_name, "Drachma");
    
    println!("✓ Read {} exp boosts", boosts.len());
}

#[test]
fn test_exp_boost_values() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let boosts = game.read_exp_boosts().unwrap();
    
    // Check Drachma's starting EXP
    let drachma = &boosts[0];
    assert_eq!(drachma.exp, 150000);
    assert_eq!(drachma.green_exp, 120);
    assert_eq!(drachma.blue_exp, 630);
    
    // Check Enrique
    let enrique = &boosts[1];
    assert_eq!(enrique.character_name, "Enrique");
    assert_eq!(enrique.exp, 30000);
    
    println!("✓ Exp boost values verified");
}

