//! Integration tests for special item reading.

mod common;

#[test]
fn test_read_special_items() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let items = game.read_special_items().unwrap();
    
    // Should have 80 special items (0x140..0x190)
    assert_eq!(items.len(), 80, "Expected 80 special items");
    
    let first = &items[0];
    assert_eq!(first.id, 320); // 0x140
    println!("First special item: ID={}, Name='{}'", first.id, first.name);
    
    // Verify it's Green Crystal
    assert_eq!(first.name, "Green Crystal");
}

#[test]
fn test_special_item_moon_crystals() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let items = game.read_special_items().unwrap();
    
    // First 5 items should be Moon Crystals (ID 325+ may be JP text)
    let expected = [
        (320, "Green Crystal"),
        (321, "Red Crystal"),
        (322, "Purple Crystal"),
        (323, "Blue Crystal"),
        (324, "Yellow Crystal"),
    ];
    
    for (i, (expected_id, expected_name)) in expected.iter().enumerate() {
        assert_eq!(items[i].id, *expected_id, "Item {} ID mismatch", i);
        assert_eq!(items[i].name, *expected_name, "Item {} name mismatch", i);
    }
    
    println!("✓ All Moon Crystals found!");
}

#[test]
fn test_special_item_descriptions() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let items = game.read_special_items().unwrap();
    
    // Green Crystal should have a description about the Green Gigas
    let green = &items[0];
    assert!(green.description.contains("Green") || green.description.contains("life"),
        "Green Crystal description should mention 'Green' or 'life': {:?}", green.description);
    
    println!("Green Crystal description: {}", green.description);
}

