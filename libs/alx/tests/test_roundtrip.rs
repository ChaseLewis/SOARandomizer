//! Roundtrip tests for binary read/write compatibility.
//!
//! These tests verify that data can be read from the ISO, written back,
//! and produce byte-identical output (for the data sections, not descriptions).

mod common;

use std::io::Cursor;
use alx::entries::{
    Accessory, Armor, Weapon, UsableItem, SpecialItem, Shop, TreasureChest, ShipCannon,
    Character, CharacterMagic, CharacterSuperMove, CrewMember, PlayableShip,
    ShipAccessory, ShipItem, EnemyShip, EnemyMagic, EnemySuperMove, Swashbuckler, SpiritCurve, ExpBoost,
};

/// Skip test if writable ISO is not available.
#[macro_use]
extern crate alx;

// =============================================================================
// Section Roundtrip Tests (read from ISO, write to buffer, compare)
// =============================================================================

#[test]
fn test_accessory_section_roundtrip() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    // Get the raw binary data from the DOL
    let offsets = game.offsets();
    let data_range = offsets.accessory_data.clone();
    let original_data = game.dol_slice(data_range).unwrap().to_vec();
    
    // Parse all accessories
    let accessories = Accessory::read_all_data(&original_data, &version).unwrap();
    
    // Write all back to a buffer
    let mut output = Cursor::new(Vec::new());
    Accessory::write_all_data(&accessories, &mut output, &version).unwrap();
    let written_data = output.into_inner();
    
    // Compare byte-for-byte
    assert_eq!(
        original_data.len(),
        written_data.len(),
        "Accessory section length mismatch"
    );
    
    // Find first difference if any
    for (i, (orig, written)) in original_data.iter().zip(written_data.iter()).enumerate() {
        if orig != written {
            let entry_idx = i / 40;
            let byte_in_entry = i % 40;
            panic!(
                "Byte mismatch at offset {} (entry {}, byte {}): original 0x{:02X} != written 0x{:02X}",
                i, entry_idx, byte_in_entry, orig, written
            );
        }
    }
    
    println!("✓ Accessory section roundtrip: {} bytes match", original_data.len());
}

#[test]
fn test_armor_section_roundtrip() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    // Get the raw binary data from the DOL
    let offsets = game.offsets();
    let data_range = offsets.armor_data.clone();
    let original_data = game.dol_slice(data_range).unwrap().to_vec();
    
    // Parse all armors
    let armors = Armor::read_all_data(&original_data, &version).unwrap();
    
    // Write all back to a buffer
    let mut output = Cursor::new(Vec::new());
    Armor::write_all_data(&armors, &mut output, &version).unwrap();
    let written_data = output.into_inner();
    
    // Compare byte-for-byte
    assert_eq!(
        original_data.len(),
        written_data.len(),
        "Armor section length mismatch"
    );
    
    for (i, (orig, written)) in original_data.iter().zip(written_data.iter()).enumerate() {
        if orig != written {
            let entry_idx = i / 40;
            let byte_in_entry = i % 40;
            panic!(
                "Byte mismatch at offset {} (entry {}, byte {}): original 0x{:02X} != written 0x{:02X}",
                i, entry_idx, byte_in_entry, orig, written
            );
        }
    }
    
    println!("✓ Armor section roundtrip: {} bytes match", original_data.len());
}

// =============================================================================
// CRC32 Checksum Helpers
// =============================================================================

#[test]
fn test_accessory_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    // Get original data and calculate CRC32
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.accessory_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    // Parse and write back
    let accessories = Accessory::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    Accessory::write_all_data(&accessories, &mut output, &version).unwrap();
    
    // Calculate CRC32 of written data
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(
        original_crc, written_crc,
        "CRC32 mismatch: original 0x{:08X} != written 0x{:08X}",
        original_crc, written_crc
    );
    
    println!("✓ Accessory CRC32 match: 0x{:08X}", original_crc);
}

#[test]
fn test_armor_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    // Get original data and calculate CRC32
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.armor_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    // Parse and write back
    let armors = Armor::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    Armor::write_all_data(&armors, &mut output, &version).unwrap();
    
    // Calculate CRC32 of written data
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(
        original_crc, written_crc,
        "CRC32 mismatch: original 0x{:08X} != written 0x{:08X}",
        original_crc, written_crc
    );
    
    println!("✓ Armor CRC32 match: 0x{:08X}", original_crc);
}

#[test]
fn test_weapon_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.weapon_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    let weapons = Weapon::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    Weapon::write_all_data(&weapons, &mut output, &version).unwrap();
    
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(original_crc, written_crc, "Weapon CRC32 mismatch");
    println!("✓ Weapon CRC32 match: 0x{:08X}", original_crc);
}

#[test]
fn test_usable_item_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.usable_item_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    let items = UsableItem::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    UsableItem::write_all_data(&items, &mut output, &version).unwrap();
    
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(original_crc, written_crc, "UsableItem CRC32 mismatch");
    println!("✓ UsableItem CRC32 match: 0x{:08X}", original_crc);
}

#[test]
fn test_special_item_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.special_item_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    let items = SpecialItem::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    SpecialItem::write_all_data(&items, &mut output, &version).unwrap();
    
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(original_crc, written_crc, "SpecialItem CRC32 mismatch");
    println!("✓ SpecialItem CRC32 match: 0x{:08X}", original_crc);
}

#[test]
fn test_shop_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.shop_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    let shops = Shop::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    Shop::write_all_data(&shops, &mut output, &version).unwrap();
    
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(original_crc, written_crc, "Shop CRC32 mismatch");
    println!("✓ Shop CRC32 match: 0x{:08X}", original_crc);
}

#[test]
fn test_treasure_chest_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.treasure_chest_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    let chests = TreasureChest::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    TreasureChest::write_all_data(&chests, &mut output, &version).unwrap();
    
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(original_crc, written_crc, "TreasureChest CRC32 mismatch");
    println!("✓ TreasureChest CRC32 match: 0x{:08X}", original_crc);
}

#[test]
fn test_ship_cannon_section_crc32() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    let offsets = game.offsets();
    let original_data = game.dol_slice(offsets.ship_cannon_data.clone()).unwrap().to_vec();
    let original_crc = common::crc32_checksum(&original_data);
    
    let cannons = ShipCannon::read_all_data(&original_data, &version).unwrap();
    let mut output = Cursor::new(Vec::new());
    ShipCannon::write_all_data(&cannons, &mut output, &version).unwrap();
    
    let written_crc = common::crc32_checksum(&output.into_inner());
    
    assert_eq!(original_crc, written_crc, "ShipCannon CRC32 mismatch");
    println!("✓ ShipCannon CRC32 match: 0x{:08X}", original_crc);
}

// Additional entry type roundtrip tests

macro_rules! roundtrip_test {
    ($test_name:ident, $entry_type:ty, $data_field:ident) => {
        #[test]
        fn $test_name() {
            skip_if_no_iso!();
            let mut game = common::load_game();
            let version = game.version().clone();
            let offsets = game.offsets();
            let original = game.dol_slice(offsets.$data_field.clone()).unwrap().to_vec();
            let crc_orig = common::crc32_checksum(&original);
            let entries = <$entry_type>::read_all_data(&original, &version).unwrap();
            let mut output = Cursor::new(Vec::new());
            <$entry_type>::write_all_data(&entries, &mut output, &version).unwrap();
            let crc_out = common::crc32_checksum(&output.into_inner());
            assert_eq!(crc_orig, crc_out, concat!(stringify!($entry_type), " CRC32 mismatch"));
        }
    };
    // Version for Option<Range<usize>> fields
    ($test_name:ident, $entry_type:ty, $data_field:ident, optional) => {
        #[test]
        fn $test_name() {
            skip_if_no_iso!();
            let mut game = common::load_game();
            let version = game.version().clone();
            let offsets = game.offsets();
            let range = offsets.$data_field.clone().expect(concat!("No range for ", stringify!($data_field)));
            let original = game.dol_slice(range).unwrap().to_vec();
            let crc_orig = common::crc32_checksum(&original);
            let entries = <$entry_type>::read_all_data(&original, &version).unwrap();
            let mut output = Cursor::new(Vec::new());
            <$entry_type>::write_all_data(&entries, &mut output, &version).unwrap();
            let crc_out = common::crc32_checksum(&output.into_inner());
            assert_eq!(crc_orig, crc_out, concat!(stringify!($entry_type), " CRC32 mismatch"));
        }
    };
}

roundtrip_test!(test_character_roundtrip, Character, character_data);
roundtrip_test!(test_character_magic_roundtrip, CharacterMagic, character_magic_data);
roundtrip_test!(test_character_super_move_roundtrip, CharacterSuperMove, character_super_move_data);
roundtrip_test!(test_crew_member_roundtrip, CrewMember, crew_member_data);
roundtrip_test!(test_playable_ship_roundtrip, PlayableShip, playable_ship_data);
roundtrip_test!(test_ship_accessory_roundtrip, ShipAccessory, ship_accessory_data);
roundtrip_test!(test_ship_item_roundtrip, ShipItem, ship_item_data);
roundtrip_test!(test_enemy_ship_roundtrip, EnemyShip, enemy_ship_data);
roundtrip_test!(test_enemy_magic_roundtrip, EnemyMagic, enemy_magic_data);
roundtrip_test!(test_enemy_super_move_roundtrip, EnemySuperMove, enemy_super_move_data);
roundtrip_test!(test_swashbuckler_roundtrip, Swashbuckler, swashbuckler_data);
roundtrip_test!(test_spirit_curve_roundtrip, SpiritCurve, spirit_curve_data);
roundtrip_test!(test_exp_boost_roundtrip, ExpBoost, exp_boost_data, optional);

// =============================================================================
// Full ISO Identity Test
// =============================================================================

/// Test that writing all entries back produces identical DOL data.
/// This verifies the complete roundtrip without actually modifying the ISO.
#[test]
fn test_full_dol_identity() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let version = game.version().clone();
    let offsets = game.offsets().clone();
    
    // Read all sections and calculate original CRCs
    let mut sections: Vec<(&str, std::ops::Range<usize>, u32)> = Vec::new();
    
    // Collect all data ranges and their CRCs
    let ranges = vec![
        ("accessory", offsets.accessory_data.clone()),
        ("armor", offsets.armor_data.clone()),
        ("weapon", offsets.weapon_data.clone()),
        ("usable_item", offsets.usable_item_data.clone()),
        ("special_item", offsets.special_item_data.clone()),
        ("shop", offsets.shop_data.clone()),
        ("treasure_chest", offsets.treasure_chest_data.clone()),
        ("ship_cannon", offsets.ship_cannon_data.clone()),
        ("character", offsets.character_data.clone()),
        ("character_magic", offsets.character_magic_data.clone()),
        ("character_super_move", offsets.character_super_move_data.clone()),
        ("crew_member", offsets.crew_member_data.clone()),
        ("playable_ship", offsets.playable_ship_data.clone()),
        ("ship_accessory", offsets.ship_accessory_data.clone()),
        ("ship_item", offsets.ship_item_data.clone()),
        ("enemy_ship", offsets.enemy_ship_data.clone()),
        ("enemy_magic", offsets.enemy_magic_data.clone()),
        ("enemy_super_move", offsets.enemy_super_move_data.clone()),
        ("swashbuckler", offsets.swashbuckler_data.clone()),
        ("spirit_curve", offsets.spirit_curve_data.clone()),
    ];
    
    for (name, range) in ranges {
        let data = game.dol_slice(range.clone()).unwrap().to_vec();
        let crc = common::crc32_checksum(&data);
        sections.push((name, range, crc));
    }
    
    // Now parse and rewrite each section, verifying CRC matches
    let mut all_match = true;
    let mut failures = Vec::new();
    
    for (name, range, orig_crc) in &sections {
        let data = game.dol_slice(range.clone()).unwrap().to_vec();
        
        let written = match *name {
            "accessory" => {
                let entries = Accessory::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                Accessory::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "armor" => {
                let entries = Armor::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                Armor::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "weapon" => {
                let entries = Weapon::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                Weapon::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "usable_item" => {
                let entries = UsableItem::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                UsableItem::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "special_item" => {
                let entries = SpecialItem::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                SpecialItem::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "shop" => {
                let entries = Shop::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                Shop::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "treasure_chest" => {
                let entries = TreasureChest::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                TreasureChest::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "ship_cannon" => {
                let entries = ShipCannon::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                ShipCannon::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "character" => {
                let entries = Character::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                Character::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "character_magic" => {
                let entries = CharacterMagic::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                CharacterMagic::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "character_super_move" => {
                let entries = CharacterSuperMove::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                CharacterSuperMove::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "crew_member" => {
                let entries = CrewMember::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                CrewMember::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "playable_ship" => {
                let entries = PlayableShip::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                PlayableShip::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "ship_accessory" => {
                let entries = ShipAccessory::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                ShipAccessory::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "ship_item" => {
                let entries = ShipItem::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                ShipItem::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "enemy_ship" => {
                let entries = EnemyShip::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                EnemyShip::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "enemy_magic" => {
                let entries = EnemyMagic::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                EnemyMagic::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "enemy_super_move" => {
                let entries = EnemySuperMove::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                EnemySuperMove::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "swashbuckler" => {
                let entries = Swashbuckler::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                Swashbuckler::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            "spirit_curve" => {
                let entries = SpiritCurve::read_all_data(&data, &version).unwrap();
                let mut out = Cursor::new(Vec::new());
                SpiritCurve::write_all_data(&entries, &mut out, &version).unwrap();
                out.into_inner()
            }
            _ => continue,
        };
        
        let new_crc = common::crc32_checksum(&written);
        if new_crc != *orig_crc {
            all_match = false;
            failures.push(format!("{}: 0x{:08X} != 0x{:08X}", name, new_crc, orig_crc));
        }
    }
    
    if !all_match {
        panic!("CRC mismatches:\n{}", failures.join("\n"));
    }
    
    println!("✓ Full DOL identity test passed: {} sections verified", sections.len());
}

