//! Integration tests for PlayableShip entries.

mod common;

#[test]
fn test_read_playable_ships() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let ships = game.read_playable_ships().unwrap();

    // Should have 5 ships (IDs 0-4)
    assert_eq!(ships.len(), 5, "Expected 5 playable ships");

    // First ship should be Little Jack
    let first = &ships[0];
    assert_eq!(first.id, 0);
    assert_eq!(first.name, "Little Jack");
    assert_eq!(first.max_hp, 10000);

    println!("✓ Read {} playable ships", ships.len());
}

#[test]
fn test_playable_ship_stats() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let ships = game.read_playable_ships().unwrap();

    // Little Jack stats
    let little_jack = &ships[0];
    assert_eq!(little_jack.max_sp, 9);
    assert_eq!(little_jack.sp, 2);
    assert_eq!(little_jack.defense, 20);
    assert_eq!(little_jack.mag_def, 50);
    assert_eq!(little_jack.quick, 55);
    assert_eq!(little_jack.dodge, 10);

    // Delphinus (ID 2) should have higher stats
    let delphinus = &ships[2];
    assert_eq!(delphinus.name, "Delphinus");
    assert_eq!(delphinus.max_hp, 36000);
    assert_eq!(delphinus.defense, 50);

    println!("✓ Ship stats verified");
}

#[test]
fn test_playable_ship_equipment() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let ships = game.read_playable_ships().unwrap();

    // Little Jack (ID 0) has Main Cannon (400) equipped
    let little_jack = &ships[0];
    assert_eq!(
        little_jack.cannon_ids[0], 400,
        "Cannon 1 should be Main Cannon"
    );
    assert_eq!(
        little_jack.cannon_ids[3], 401,
        "Cannon 4 should be Standard Cannon"
    );
    assert_eq!(
        little_jack.cannon_ids[4], 430,
        "Cannon 5 should be Harpoon Cannon"
    );

    // Little Jack accessories should be -1 (none)
    assert_eq!(little_jack.accessory_ids[0], -1);

    println!("✓ Ship equipment verified");
}
