//! Integration tests for SpiritCurve entries.

mod common;

#[test]
fn test_read_spirit_curves() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let curves = game.read_spirit_curves().unwrap();

    // Should have 6 spirit curves (one per character)
    assert_eq!(curves.len(), 6, "Expected 6 spirit curves");

    // First curve should be Vyse
    let vyse = &curves[0];
    assert_eq!(vyse.id, 0);
    assert_eq!(vyse.character_name, "Vyse");
    assert_eq!(vyse.levels.len(), 99);

    println!("✓ Read {} spirit curves", curves.len());
}

#[test]
fn test_spirit_curve_values() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let curves = game.read_spirit_curves().unwrap();

    // Check Vyse's level 1 stats
    let vyse = &curves[0];
    assert_eq!(vyse.sp_at_level(1), Some(1));
    assert_eq!(vyse.max_sp_at_level(1), Some(4));

    // Check level 99 stats (should be higher)
    assert_eq!(vyse.sp_at_level(99), Some(20));
    assert_eq!(vyse.max_sp_at_level(99), Some(25));

    println!("✓ Spirit curve values verified");
}
