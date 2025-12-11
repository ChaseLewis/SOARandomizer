//! Integration tests for weapon reading.

mod common;

#[test]
fn test_read_weapons() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let weapons = game.read_weapons().unwrap();
    
    // Should have 80 weapons (0x00..0x50)
    assert_eq!(weapons.len(), 80, "Expected 80 weapons");
    
    let first = &weapons[0];
    assert_eq!(first.id, 0);
    println!("First weapon: ID={}, Name='{}'", first.id, first.name);
}

#[test]
fn test_weapon_cutlass() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let weapons = game.read_weapons().unwrap();
    
    // First weapon should be Cutlass (ID 0)
    let cutlass = &weapons[0];
    assert_eq!(cutlass.id, 0);
    assert_eq!(cutlass.name, "Cutlass");
    
    // Cutlass is Vyse's weapon (character_id = 0)
    assert_eq!(cutlass.character_id, 0);
    assert_eq!(cutlass.character_name(), "Vyse");
    
    // Check stats from reference CSV
    assert_eq!(cutlass.sell_percent, 50);
    assert_eq!(cutlass.buy_price, 70);
    assert_eq!(cutlass.attack, 20);
    assert_eq!(cutlass.hit_percent, 90);
    
    println!("✓ Cutlass matches reference data!");
    println!("  Attack: {}, Hit%: {}", cutlass.attack, cutlass.hit_percent);
}

#[test]
fn test_weapon_characters() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let weapons = game.read_weapons().unwrap();
    
    // Count weapons per character
    let mut counts = [0; 6];
    for weapon in &weapons {
        if weapon.character_id >= 0 && weapon.character_id < 6 {
            counts[weapon.character_id as usize] += 1;
        }
    }
    
    println!("Weapons per character:");
    println!("  Vyse: {}", counts[0]);
    println!("  Aika: {}", counts[1]);
    println!("  Fina: {}", counts[2]);
    println!("  Drachma: {}", counts[3]);
    println!("  Enrique: {}", counts[4]);
    println!("  Gilder: {}", counts[5]);
    
    // Each character should have some weapons
    for (i, &count) in counts.iter().enumerate() {
        assert!(count > 0, "Character {} should have weapons", i);
    }
}

#[test]
fn test_weapon_effects() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let weapons = game.read_weapons().unwrap();
    
    // Find weapons with effects
    let weapons_with_effects: Vec<_> = weapons.iter()
        .filter(|w| w.effect_id >= 0)
        .collect();
    
    println!("Weapons with effects: {}", weapons_with_effects.len());
    
    for weapon in weapons_with_effects.iter().take(5) {
        println!("  {} (ID {}): effect_id={}", 
            weapon.name, weapon.id, weapon.effect_id);
    }
    
    // Should have some weapons with effects
    assert!(!weapons_with_effects.is_empty(), "Some weapons should have effects");
}

