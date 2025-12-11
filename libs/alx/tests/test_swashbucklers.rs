//! Integration tests for Swashbuckler entries.

mod common;

#[test]
fn test_read_swashbucklers() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let ratings = game.read_swashbucklers().unwrap();
    
    // Should have 24 swashbuckler ratings (IDs 0-23)
    assert_eq!(ratings.len(), 24, "Expected 24 swashbuckler ratings");
    
    // First rating should be "Vyse the Ninny"
    let first = &ratings[0];
    assert_eq!(first.id, 0);
    assert_eq!(first.name, "Vyse the Ninny");
    assert_eq!(first.rating, 5);
    
    println!("✓ Read {} swashbuckler ratings", ratings.len());
}

#[test]
fn test_swashbuckler_ratings() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let ratings = game.read_swashbucklers().unwrap();
    
    // Check Vyse the Legend (highest rating)
    let legend = ratings.iter().find(|r| r.name.contains("Legend"));
    assert!(legend.is_some(), "Should have 'Vyse the Legend' rating");
    
    // Check rating modifiers
    let ninny = &ratings[0];
    assert_eq!(ninny.dodge, 50, "Ninny should have +50 dodge");
    assert_eq!(ninny.run, 200, "Ninny should have +200 run");
    
    println!("✓ Swashbuckler ratings verified");
}

