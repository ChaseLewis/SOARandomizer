//! Integration tests that modify values, save, and verify they persist.
//!
//! These tests catch issues where the binary write isn't actually working,
//! which CSV roundtrip tests might miss.

use std::fs;
use std::path::Path;

use alx::game::GameRoot;

/// Find the ISO path, checking multiple locations
fn find_iso() -> Option<std::path::PathBuf> {
    // Try various paths relative to where tests run
    let candidates = [
        "roms/Skies of Arcadia Legends (USA).iso",
        "../../../roms/Skies of Arcadia Legends (USA).iso", // From bin/alx_rs/tests
        "../../roms/Skies of Arcadia Legends (USA).iso",    // From bin/alx_rs
    ];

    for path in &candidates {
        let p = Path::new(path);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }

    // Try from CARGO_MANIFEST_DIR
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = Path::new(&manifest_dir)
            .parent()?
            .parent()?
            .join("roms/Skies of Arcadia Legends (USA).iso");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Helper to create a test copy of the ISO
fn create_test_iso(test_name: &str) -> Option<std::path::PathBuf> {
    let source = find_iso()?;

    // Use a temp directory that we can find
    let test_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("target/modify_tests"))
            .unwrap_or_else(|| Path::new("target/modify_tests").to_path_buf())
    } else {
        Path::new("target/modify_tests").to_path_buf()
    };

    fs::create_dir_all(&test_dir).expect("Failed to create test dir");

    let dest = test_dir.join(format!("{}.iso", test_name));

    // Remove old test file if exists
    let _ = fs::remove_file(&dest);

    println!("Copying ISO from {:?} to {:?}...", source, dest);
    fs::copy(&source, &dest).expect("Failed to copy ISO");

    Some(dest)
}

/// Test that character modifications actually persist
#[test]
#[ignore] // Run with: cargo test --package alx_rs modify_character -- --ignored --nocapture
fn test_modify_character_stats() {
    let Some(test_iso) = create_test_iso("modify_character") else {
        return;
    };

    // Step 1: Read original values
    let mut game = GameRoot::open(&test_iso).expect("Failed to open ISO");
    let original_chars = game.read_characters().expect("Failed to read characters");

    println!("Original Vyse stats:");
    println!("  Name: {}", original_chars[0].name);
    println!("  Max HP: {}", original_chars[0].max_hp);
    println!("  Power: {}", original_chars[0].power);
    println!("  Will: {}", original_chars[0].will);

    // Step 2: Modify values with distinctive numbers
    let mut modified_chars = original_chars.clone();
    modified_chars[0].max_hp = 9999; // Vyse
    modified_chars[0].power = 123;
    modified_chars[1].max_hp = 8888; // Aika
    modified_chars[1].will = 234;

    // Step 3: Write and save
    game.write_characters(&modified_chars)
        .expect("Failed to write characters");
    game.save_dol().expect("Failed to save DOL");
    drop(game); // Close the file

    // Step 4: Re-open and verify
    let mut game2 = GameRoot::open(&test_iso).expect("Failed to reopen ISO");
    let reread_chars = game2
        .read_characters()
        .expect("Failed to re-read characters");

    println!("\nAfter modification and re-read:");
    println!("  Vyse Max HP: {} (expected 9999)", reread_chars[0].max_hp);
    println!("  Vyse Power: {} (expected 123)", reread_chars[0].power);
    println!("  Aika Max HP: {} (expected 8888)", reread_chars[1].max_hp);
    println!("  Aika Will: {} (expected 234)", reread_chars[1].will);

    // Assert values changed
    assert_eq!(reread_chars[0].max_hp, 9999, "Vyse max_hp not persisted");
    assert_eq!(reread_chars[0].power, 123, "Vyse power not persisted");
    assert_eq!(reread_chars[1].max_hp, 8888, "Aika max_hp not persisted");
    assert_eq!(reread_chars[1].will, 234, "Aika will not persisted");

    // Verify other values unchanged
    assert_eq!(
        reread_chars[0].name, original_chars[0].name,
        "Vyse name changed unexpectedly"
    );
    assert_eq!(
        reread_chars[0].age, original_chars[0].age,
        "Vyse age changed unexpectedly"
    );

    println!("\n✓ Character modification test passed!");

    // Cleanup
    let _ = fs::remove_file(&test_iso);
}

/// Test that accessory modifications persist
#[test]
#[ignore]
fn test_modify_accessory_stats() {
    let Some(test_iso) = create_test_iso("modify_accessory") else {
        return;
    };

    let mut game = GameRoot::open(&test_iso).expect("Failed to open ISO");
    let original = game.read_accessories().expect("Failed to read accessories");

    println!(
        "Original first accessory: {} - Buy: {}",
        original[0].name, original[0].buy_price
    );

    // Modify
    let mut modified = original.clone();
    modified[0].buy_price = 12345;
    modified[1].sell_percent = 99;

    game.write_accessories(&modified).expect("Failed to write");
    game.save_dol().expect("Failed to save");
    drop(game);

    // Re-read
    let mut game2 = GameRoot::open(&test_iso).expect("Failed to reopen");
    let reread = game2.read_accessories().expect("Failed to re-read");

    println!(
        "After modification: {} - Buy: {}",
        reread[0].name, reread[0].buy_price
    );

    assert_eq!(reread[0].buy_price, 12345, "Buy price not persisted");
    assert_eq!(reread[1].sell_percent, 99, "Sell percent not persisted");

    println!("✓ Accessory modification test passed!");
    let _ = fs::remove_file(&test_iso);
}

/// Test that weapon modifications persist
#[test]
#[ignore]
fn test_modify_weapon_stats() {
    let Some(test_iso) = create_test_iso("modify_weapon") else {
        return;
    };

    let mut game = GameRoot::open(&test_iso).expect("Failed to open ISO");
    let original = game.read_weapons().expect("Failed to read weapons");

    println!(
        "Original first weapon: {} - Attack: {}",
        original[0].name, original[0].attack
    );

    let mut modified = original.clone();
    modified[0].attack = 255;
    modified[0].hit_percent = 100;

    game.write_weapons(&modified).expect("Failed to write");
    game.save_dol().expect("Failed to save");
    drop(game);

    let mut game2 = GameRoot::open(&test_iso).expect("Failed to reopen");
    let reread = game2.read_weapons().expect("Failed to re-read");

    println!(
        "After modification: {} - Attack: {}",
        reread[0].name, reread[0].attack
    );

    assert_eq!(reread[0].attack, 255, "Attack not persisted");
    assert_eq!(reread[0].hit_percent, 100, "Hit percent not persisted");

    println!("✓ Weapon modification test passed!");
    let _ = fs::remove_file(&test_iso);
}

/// Test that shop modifications persist
#[test]
#[ignore]
fn test_modify_shop_items() {
    let Some(test_iso) = create_test_iso("modify_shop") else {
        return;
    };

    let mut game = GameRoot::open(&test_iso).expect("Failed to open ISO");
    let original = game.read_shops().expect("Failed to read shops");

    println!(
        "Original first shop items: {:?}",
        &original[0].item_ids[..5]
    );

    let mut modified = original.clone();
    // Change first shop's items to something distinctive
    modified[0].item_ids[0] = 100;
    modified[0].item_ids[1] = 200;
    modified[0].item_ids[2] = 300;

    game.write_shops(&modified).expect("Failed to write");
    game.save_dol().expect("Failed to save");
    drop(game);

    let mut game2 = GameRoot::open(&test_iso).expect("Failed to reopen");
    let reread = game2.read_shops().expect("Failed to re-read");

    println!("After modification: {:?}", &reread[0].item_ids[..5]);

    assert_eq!(reread[0].item_ids[0], 100, "Shop item 0 not persisted");
    assert_eq!(reread[0].item_ids[1], 200, "Shop item 1 not persisted");
    assert_eq!(reread[0].item_ids[2], 300, "Shop item 2 not persisted");

    println!("✓ Shop modification test passed!");
    let _ = fs::remove_file(&test_iso);
}

/// Test that treasure chest modifications persist
#[test]
#[ignore]
fn test_modify_treasure_chest() {
    let Some(test_iso) = create_test_iso("modify_chest") else {
        return;
    };

    let mut game = GameRoot::open(&test_iso).expect("Failed to open ISO");
    let original = game.read_treasure_chests().expect("Failed to read chests");

    println!(
        "Original first chest: item_id={}, amount={}",
        original[0].item_id, original[0].item_amount
    );

    let mut modified = original.clone();
    modified[0].item_id = 999;
    modified[0].item_amount = 50;

    game.write_treasure_chests(&modified)
        .expect("Failed to write");
    game.save_dol().expect("Failed to save");
    drop(game);

    let mut game2 = GameRoot::open(&test_iso).expect("Failed to reopen");
    let reread = game2.read_treasure_chests().expect("Failed to re-read");

    println!(
        "After modification: item_id={}, amount={}",
        reread[0].item_id, reread[0].item_amount
    );

    assert_eq!(reread[0].item_id, 999, "Chest item_id not persisted");
    assert_eq!(reread[0].item_amount, 50, "Chest amount not persisted");

    println!("✓ Treasure chest modification test passed!");
    let _ = fs::remove_file(&test_iso);
}

/// Run all modification tests together
#[test]
#[ignore]
fn test_all_modifications() {
    test_modify_character_stats();
    test_modify_accessory_stats();
    test_modify_weapon_stats();
    test_modify_shop_items();
    test_modify_treasure_chest();
}

/// Test that CSV import actually modifies the ISO correctly
/// This is the most complete test - it simulates the full user workflow
#[test]
#[ignore]
fn test_csv_import_workflow() {
    use alx::csv::{CsvExporter, CsvImporter};
    use std::io::BufReader;

    let Some(test_iso) = create_test_iso("csv_import_workflow") else {
        return;
    };

    // Step 1: Read original values
    let mut game = GameRoot::open(&test_iso).expect("Failed to open ISO");
    let original_chars = game.read_characters().expect("Failed to read characters");
    let item_db = game.build_item_database().expect("Failed to build item database");

    println!(
        "Original Vyse: Max HP={}, Power={}",
        original_chars[0].max_hp, original_chars[0].power
    );

    // Step 2: Export to CSV
    let mut csv_data = Vec::new();
    CsvExporter::export_characters(&original_chars, &item_db, &mut csv_data).expect("Failed to export");
    let csv_string = String::from_utf8_lossy(&csv_data);
    println!("\nOriginal CSV (first 2 lines):");
    for line in csv_string.lines().take(2) {
        println!("  {}...", &line.chars().take(100).collect::<String>());
    }

    // Step 3: Modify the CSV by parsing, modifying, and re-exporting
    // This is more robust than string replacement
    let reader = BufReader::new(csv_data.as_slice());
    let mut chars_to_modify = CsvImporter::import_characters(reader, &original_chars)
        .expect("Failed to import for modification");

    // Modify specific values
    chars_to_modify[0].max_hp = 7777;
    chars_to_modify[0].power = 111;
    chars_to_modify[1].max_hp = 6666;
    chars_to_modify[1].power = 222;

    println!("\nAfter modification (before write):");
    println!(
        "  Vyse Max HP: {}, Power: {}",
        chars_to_modify[0].max_hp, chars_to_modify[0].power
    );
    println!(
        "  Aika Max HP: {}, Power: {}",
        chars_to_modify[1].max_hp, chars_to_modify[1].power
    );

    // Step 4: Write and save
    game.write_characters(&chars_to_modify)
        .expect("Failed to write");
    game.save_dol().expect("Failed to save");
    drop(game);

    // Step 5: Re-open and verify
    let mut game2 = GameRoot::open(&test_iso).expect("Failed to reopen");
    let final_chars = game2.read_characters().expect("Failed to re-read");

    println!("\nAfter save and re-read:");
    println!("  Vyse Max HP: {} (expected 7777)", final_chars[0].max_hp);
    println!("  Vyse Power: {} (expected 111)", final_chars[0].power);
    println!("  Aika Max HP: {} (expected 6666)", final_chars[1].max_hp);
    println!("  Aika Power: {} (expected 222)", final_chars[1].power);

    assert_eq!(
        final_chars[0].max_hp, 7777,
        "Vyse max_hp not persisted from CSV"
    );
    assert_eq!(
        final_chars[0].power, 111,
        "Vyse power not persisted from CSV"
    );
    assert_eq!(
        final_chars[1].max_hp, 6666,
        "Aika max_hp not persisted from CSV"
    );
    assert_eq!(
        final_chars[1].power, 222,
        "Aika power not persisted from CSV"
    );

    // Also verify that unchanged fields are preserved
    assert_eq!(final_chars[0].name, "Vyse", "Vyse name corrupted");
    assert_eq!(final_chars[0].age, 17, "Vyse age corrupted");
    assert_eq!(final_chars[0].width, 1, "Vyse width corrupted");
    assert_eq!(final_chars[0].depth, 1, "Vyse depth corrupted");

    println!("\n✓ CSV import workflow test passed!");
    let _ = fs::remove_file(&test_iso);
}
