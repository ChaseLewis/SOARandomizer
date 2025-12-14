//! Roundtrip tests for binary patch compatibility.
//!
//! These tests verify that data can be read from the ISO, patched back,
//! and produce byte-identical output for numeric fields.

mod common;

use alx::entries::{
    Accessory, Armor, Character, CharacterMagic, CharacterSuperMove, CrewMember, PlayableShip,
    ShipAccessory, ShipCannon, ShipItem, Shop, SpecialItem, Swashbuckler, TreasureChest,
    UsableItem, Weapon,
};

// =============================================================================
// Patch Roundtrip Tests (read from ISO, patch back, compare)
// =============================================================================

macro_rules! patch_roundtrip_test {
    ($test_name:ident, $entry_type:ty, $data_field:ident) => {
        #[test]
        fn $test_name() {
            skip_if_no_iso!();
            let mut game = common::load_game();
            let version = game.version().clone();
            let offsets = game.offsets();
            let original = game
                .dol_slice(offsets.$data_field.clone())
                .unwrap()
                .to_vec();
            let crc_orig = common::crc32_checksum(&original);
            let entries = <$entry_type>::read_all_data(&original, &version).unwrap();
            let mut patched = original.clone();
            <$entry_type>::patch_all(&entries, &mut patched, &version);
            let crc_out = common::crc32_checksum(&patched);
            assert_eq!(
                crc_orig, crc_out,
                concat!(stringify!($entry_type), " patch roundtrip CRC32 mismatch")
            );
        }
    };
    // Version without version param for patch_all
    ($test_name:ident, $entry_type:ty, $data_field:ident, no_version) => {
        #[test]
        fn $test_name() {
            skip_if_no_iso!();
            let mut game = common::load_game();
            let version = game.version().clone();
            let offsets = game.offsets();
            let original = game
                .dol_slice(offsets.$data_field.clone())
                .unwrap()
                .to_vec();
            let crc_orig = common::crc32_checksum(&original);
            let entries = <$entry_type>::read_all_data(&original, &version).unwrap();
            let mut patched = original.clone();
            <$entry_type>::patch_all(&entries, &mut patched);
            let crc_out = common::crc32_checksum(&patched);
            assert_eq!(
                crc_orig, crc_out,
                concat!(stringify!($entry_type), " patch roundtrip CRC32 mismatch")
            );
        }
    };
}

// Entry types with patch_all(entries, buf, version)
patch_roundtrip_test!(test_accessory_roundtrip, Accessory, accessory_data);
patch_roundtrip_test!(test_armor_roundtrip, Armor, armor_data);
patch_roundtrip_test!(test_weapon_roundtrip, Weapon, weapon_data);
patch_roundtrip_test!(test_usable_item_roundtrip, UsableItem, usable_item_data);
patch_roundtrip_test!(test_special_item_roundtrip, SpecialItem, special_item_data);
patch_roundtrip_test!(test_ship_cannon_roundtrip, ShipCannon, ship_cannon_data);
patch_roundtrip_test!(
    test_character_magic_roundtrip,
    CharacterMagic,
    character_magic_data
);
patch_roundtrip_test!(
    test_character_super_move_roundtrip,
    CharacterSuperMove,
    character_super_move_data
);
patch_roundtrip_test!(test_crew_member_roundtrip, CrewMember, crew_member_data);
patch_roundtrip_test!(
    test_ship_accessory_roundtrip,
    ShipAccessory,
    ship_accessory_data
);
patch_roundtrip_test!(test_ship_item_roundtrip, ShipItem, ship_item_data);
patch_roundtrip_test!(test_swashbuckler_roundtrip, Swashbuckler, swashbuckler_data);

// Entry types with patch_all(entries, buf) - no version param
patch_roundtrip_test!(
    test_character_roundtrip,
    Character,
    character_data,
    no_version
);
patch_roundtrip_test!(
    test_playable_ship_roundtrip,
    PlayableShip,
    playable_ship_data,
    no_version
);
patch_roundtrip_test!(test_shop_roundtrip, Shop, shop_data, no_version);
patch_roundtrip_test!(
    test_treasure_chest_roundtrip,
    TreasureChest,
    treasure_chest_data,
    no_version
);
