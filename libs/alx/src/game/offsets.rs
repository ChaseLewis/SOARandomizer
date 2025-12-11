//! Version-specific data offsets in the executable.
//!
//! These offsets are extracted from the original ALX dscrptr.rb configuration.
//! They specify where various game data structures are located in Start.dol.

use std::ops::Range;

use super::region::{GameVersion, Region};
use crate::error::Result;

/// Data offsets for a specific game version.
#[derive(Debug, Clone)]
pub struct Offsets {
    /// Accessory data range in Start.dol
    pub accessory_data: Range<usize>,
    /// Accessory description range in Start.dol
    pub accessory_dscr: Range<usize>,

    /// Armor data range in Start.dol
    pub armor_data: Range<usize>,
    /// Armor description range in Start.dol
    pub armor_dscr: Range<usize>,

    /// Weapon data range in Start.dol
    pub weapon_data: Range<usize>,
    /// Weapon description range in Start.dol
    pub weapon_dscr: Range<usize>,

    /// Weapon effect data range in Start.dol
    pub weapon_effect_data: Range<usize>,

    /// Usable item data range in Start.dol
    pub usable_item_data: Range<usize>,
    /// Usable item description range in Start.dol
    pub usable_item_dscr: Range<usize>,

    /// Special item data range in Start.dol
    pub special_item_data: Range<usize>,
    /// Special item description range in Start.dol
    pub special_item_dscr: Range<usize>,

    /// Character data range in Start.dol
    pub character_data: Range<usize>,

    /// Character magic data range in Start.dol
    pub character_magic_data: Range<usize>,
    /// Character magic description range in Start.dol
    pub character_magic_dscr: Range<usize>,

    /// Character super move data range in Start.dol
    pub character_super_move_data: Range<usize>,
    /// Character super move description range in Start.dol
    pub character_super_move_dscr: Range<usize>,

    /// Enemy magic data range in Start.dol
    pub enemy_magic_data: Range<usize>,

    /// Enemy super move data range in Start.dol
    pub enemy_super_move_data: Range<usize>,

    /// Enemy ship data range in Start.dol
    pub enemy_ship_data: Range<usize>,

    /// Playable ship data range in Start.dol
    pub playable_ship_data: Range<usize>,

    /// Ship cannon data range in Start.dol
    pub ship_cannon_data: Range<usize>,
    /// Ship cannon description range in Start.dol
    pub ship_cannon_dscr: Range<usize>,

    /// Ship accessory data range in Start.dol
    pub ship_accessory_data: Range<usize>,
    /// Ship accessory description range in Start.dol
    pub ship_accessory_dscr: Range<usize>,

    /// Ship item data range in Start.dol
    pub ship_item_data: Range<usize>,
    /// Ship item description range in Start.dol
    pub ship_item_dscr: Range<usize>,

    /// Crew member data range in Start.dol
    pub crew_member_data: Range<usize>,
    /// Crew member description range in Start.dol
    pub crew_member_dscr: Range<usize>,

    /// Shop data range in Start.dol
    pub shop_data: Range<usize>,
    /// Shop description range in Start.dol
    pub shop_dscr: Range<usize>,

    /// Swashbuckler data range in Start.dol
    pub swashbuckler_data: Range<usize>,

    /// Treasure chest data range in Start.dol
    pub treasure_chest_data: Range<usize>,

    /// Spirit curve data range in Start.dol
    pub spirit_curve_data: Range<usize>,

    /// EXP boost data range in Start.dol (GC only)
    pub exp_boost_data: Option<Range<usize>>,

    /// Level data file path (for EXP curves)
    pub level_file: &'static str,
    /// EXP curve data range within level file
    pub exp_curve_data: Range<usize>,
    /// Magic EXP curve data range within level file
    pub magic_exp_curve_data: Range<usize>,
}

impl Offsets {
    /// Get offsets for a specific game version.
    pub fn for_version(version: &GameVersion) -> Result<Self> {
        match version.region {
            Region::Us => Ok(Self::gc_us()),
            Region::Jp => Ok(Self::gc_jp()),
            Region::Eu => Ok(Self::gc_eu()),
        }
    }

    /// Offsets for GC-US-GEA (2002-12-19 Final US Build).
    pub fn gc_us() -> Self {
        Self {
            // Items
            accessory_data: 0x2c3e10..0x2c4a90,
            accessory_dscr: 0x2ca880..0x2cbc88,
            armor_data: 0x2c3190..0x2c3e10,
            armor_dscr: 0x2c9714..0x2ca880,
            weapon_data: 0x2c2790..0x2c3190,
            weapon_dscr: 0x2c7d9c..0x2c9714,
            weapon_effect_data: 0x2c4a90..0x2c4c34,
            usable_item_data: 0x2c4c34..0x2c5774,
            usable_item_dscr: 0x2cbc88..0x2cd4ec,
            special_item_data: 0x2c5774..0x2c5e54,
            special_item_dscr: 0x2cd4ec..0x2ce220,

            // Characters
            character_data: 0x2c1860..0x2c1bf0,
            character_magic_data: 0x2c1bf0..0x2c22b0,
            character_magic_dscr: 0x2c6668..0x2c73e0,
            character_super_move_data: 0x2c22b0..0x2c2790,
            character_super_move_dscr: 0x2c73e0..0x2c7d9c,

            // Enemies
            enemy_magic_data: 0x2aa440..0x2aa950,
            enemy_super_move_data: 0x2aa950..0x2ad4c4,

            // Ships
            enemy_ship_data: 0x2d3934..0x2d4e4c,
            playable_ship_data: 0x2d3740..0x2d3934,
            ship_cannon_data: 0x2d4e4c..0x2d53ec,
            ship_cannon_dscr: 0x2ce220..0x2ceef8,
            ship_accessory_data: 0x2d53ec..0x2d5a2c,
            ship_accessory_dscr: 0x2ceef8..0x2cfbf0,
            ship_item_data: 0x2d5a2c..0x2d5e64,
            ship_item_dscr: 0x2cfbf0..0x2d05c4,

            // Crew & Misc
            crew_member_data: 0x2d5e64..0x2d617c,
            crew_member_dscr: 0x2d0ef4..0x2d1600,
            shop_data: 0x2e90a0..0x2ea218,
            shop_dscr: 0x2b6554..0x2b6730,
            swashbuckler_data: 0x2c5e54..0x2c6184,
            treasure_chest_data: 0x2d29e8..0x2d2da0,
            spirit_curve_data: 0x2c6184..0x2c6628,
            exp_boost_data: Some(0x2d1638..0x2d168c),

            // Level file (contains EXP curves)
            level_file: "battle/first.lmt",
            exp_curve_data: 0x0..0x948,
            magic_exp_curve_data: 0x948..0xaf8,
        }
    }

    /// Offsets for GC-JP-GEA (2002-11-12 Final JP Build).
    pub fn gc_jp() -> Self {
        Self {
            // Items
            accessory_data: 0x2c3308..0x2c3f88,
            accessory_dscr: 0x2ca370..0x2cba54,
            armor_data: 0x2c2688..0x2c3308,
            armor_dscr: 0x2c8ddc..0x2ca370,
            weapon_data: 0x2c1c88..0x2c2688,
            weapon_dscr: 0x2c72c0..0x2c8ddc,
            weapon_effect_data: 0x2c3f88..0x2c412c,
            usable_item_data: 0x2c412c..0x2c4c6c,
            usable_item_dscr: 0x2cba54..0x2cd644,
            special_item_data: 0x2c4c6c..0x2c534c,
            special_item_dscr: 0x2cd644..0x2ce2b4,

            // Characters
            character_data: 0x2c0d58..0x2c10e8,
            character_magic_data: 0x2c10e8..0x2c17a8,
            character_magic_dscr: 0x2c5b60..0x2c68d4,
            character_super_move_data: 0x2c17a8..0x2c1c88,
            character_super_move_dscr: 0x2c68d4..0x2c72c0,

            // Enemies
            enemy_magic_data: 0x2a9ee8..0x2aa3f8,
            enemy_super_move_data: 0x2aa3f8..0x2acf6c,

            // Ships
            enemy_ship_data: 0x2d3574..0x2d4a8c,
            playable_ship_data: 0x2d3380..0x2d3574,
            ship_cannon_data: 0x2d4a8c..0x2d502c,
            ship_cannon_dscr: 0x2ce2b4..0x2cef30,
            ship_accessory_data: 0x2d502c..0x2d566c,
            ship_accessory_dscr: 0x2cef30..0x2cfc58,
            ship_item_data: 0x2d566c..0x2d5aa4,
            ship_item_dscr: 0x2cfc58..0x2d058c,

            // Crew & Misc
            crew_member_data: 0x2d5aa4..0x2d5dbc,
            crew_member_dscr: 0x2d0dcc..0x2d15dc,
            shop_data: 0x2e8d08..0x2e9e80,
            shop_dscr: 0x2b6158..0x2b6344,
            swashbuckler_data: 0x2c534c..0x2c567c,
            treasure_chest_data: 0x2d26d0..0x2d2a88,
            spirit_curve_data: 0x2c567c..0x2c5b20,
            exp_boost_data: Some(0x2d1614..0x2d1668),

            // Level file (contains EXP curves)
            level_file: "battle/first.lmt",
            exp_curve_data: 0x0..0x948,
            magic_exp_curve_data: 0x948..0xaf8,
        }
    }

    /// Offsets for GC-EU-GEA (2003-03-05 Final EU Build).
    pub fn gc_eu() -> Self {
        Self {
            // Items
            accessory_data: 0x2f3a68..0x2f4328,
            accessory_dscr: 0..0, // EU uses SOT files
            armor_data: 0x2f31a8..0x2f3a68,
            armor_dscr: 0..0,
            weapon_data: 0x2f2b68..0x2f31a8,
            weapon_dscr: 0..0,
            weapon_effect_data: 0x2c40a8..0x2c424c,
            usable_item_data: 0x2f4328..0x2f4aa8,
            usable_item_dscr: 0..0,
            special_item_data: 0x2f4aa8..0x2f4e68,
            special_item_dscr: 0..0,

            // Characters
            character_data: 0x2c2ff0..0x2c3380,
            character_magic_data: 0x2f22b0..0x2f27c0,
            character_magic_dscr: 0..0,
            character_super_move_data: 0x2f27c0..0x2f2b68,
            character_super_move_dscr: 0..0,

            // Enemies
            enemy_magic_data: 0x2d9398..0x2d9668,
            enemy_super_move_data: 0x2d9668..0x2dae8c,

            // Ships
            enemy_ship_data: 0x2f6d14..0x2f7f5c,
            playable_ship_data: 0x2f6b70..0x2f6d14,
            ship_cannon_data: 0x2f7f5c..0x2f831c,
            ship_cannon_dscr: 0..0,
            ship_accessory_data: 0x2f831c..0x2f877c,
            ship_accessory_dscr: 0..0,
            ship_item_data: 0x2f877c..0x2f8a4c,
            ship_item_dscr: 0..0,

            // Crew & Misc
            crew_member_data: 0x2f8a4c..0x2f8c5c,
            crew_member_dscr: 0..0,
            shop_data: 0x2e7dd4..0x2e8f4c,
            shop_dscr: 0..0,
            swashbuckler_data: 0x2f4e68..0x2f4fe8,
            treasure_chest_data: 0x2d1610..0x2d19c8,
            spirit_curve_data: 0x2c4aa0..0x2c4f44,
            exp_boost_data: Some(0x2cffd0..0x2d0024),

            // Level file (contains EXP curves)
            level_file: "battle/first.lmt",
            exp_curve_data: 0x0..0x948,
            magic_exp_curve_data: 0x948..0xaf8,
        }
    }
}

/// ID ranges for various entry types.
pub mod id_ranges {
    use std::ops::Range;

    /// Accessory ID range (0xA0..0xF0 = 160..240)
    pub const ACCESSORY: Range<u32> = 0xa0..0xf0;

    /// Armor ID range (0x50..0xA0 = 80..160)
    pub const ARMOR: Range<u32> = 0x50..0xa0;

    /// Weapon ID range (0x00..0x50 = 0..80)
    pub const WEAPON: Range<u32> = 0x00..0x50;

    /// Weapon effect ID range
    pub const WEAPON_EFFECT: Range<u32> = 0x00..0x15;

    /// Usable item ID range (0xF0..0x140 = 240..320)
    pub const USABLE_ITEM: Range<u32> = 0xf0..0x140;

    /// Special item ID range (0x140..0x190 = 320..400)
    pub const SPECIAL_ITEM: Range<u32> = 0x140..0x190;

    /// Character ID range (0..6)
    pub const CHARACTER: Range<u32> = 0x00..0x06;

    /// Character magic ID range (0..36)
    pub const CHARACTER_MAGIC: Range<u32> = 0x00..0x24;

    /// Character super move ID range (36..62)
    pub const CHARACTER_SUPER_MOVE: Range<u32> = 0x24..0x3e;

    /// Enemy magic ID range (same as character magic)
    pub const ENEMY_MAGIC: Range<u32> = 0x00..0x24;

    /// Enemy super move ID range (GC version: 0..309)
    pub const ENEMY_SUPER_MOVE_GC: Range<u32> = 0x00..0x135;

    /// Enemy ship ID range
    pub const ENEMY_SHIP: Range<u32> = 0x00..0x2d;

    /// Playable ship ID range
    pub const PLAYABLE_SHIP: Range<u32> = 0x00..0x05;

    /// Ship cannon ID range
    pub const SHIP_CANNON: Range<u32> = 0x190..0x1b8;

    /// Ship accessory ID range
    pub const SHIP_ACCESSORY: Range<u32> = 0x1b8..0x1e0;

    /// Ship item ID range
    pub const SHIP_ITEM: Range<u32> = 0x1e0..0x1fe;

    /// Crew member ID range
    pub const CREW_MEMBER: Range<u32> = 0x00..0x16;

    /// Shop ID range
    pub const SHOP: Range<u32> = 0x00..0x2b;

    /// Swashbuckler rating ID range (GC: 0..24)
    pub const SWASHBUCKLER_GC: Range<u32> = 0x00..0x18;

    /// Treasure chest ID range
    pub const TREASURE_CHEST: Range<u32> = 0x00..0x77;

    /// Spirit curve ID range
    pub const SPIRIT_CURVE: Range<u32> = 0x00..0x06;

    /// EXP curve ID range
    pub const EXP_CURVE: Range<u32> = 0x00..0x06;

    /// Magic EXP curve ID range
    pub const MAGIC_EXP_CURVE: Range<u32> = 0x00..0x06;

    /// EXP boost ID range
    pub const EXP_BOOST: Range<u32> = 0x03..0x06;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_us_accessory_data_size() {
        let offsets = Offsets::gc_us();
        let size = offsets.accessory_data.end - offsets.accessory_data.start;
        let count = id_ranges::ACCESSORY.end - id_ranges::ACCESSORY.start;
        // Each accessory entry is a fixed size
        assert!(size > 0);
        assert_eq!(count, 80); // 0xF0 - 0xA0 = 80 accessories
    }

    #[test]
    fn test_gc_us_armor_data_size() {
        let offsets = Offsets::gc_us();
        let size = offsets.armor_data.end - offsets.armor_data.start;
        let count = id_ranges::ARMOR.end - id_ranges::ARMOR.start;
        assert!(size > 0);
        assert_eq!(count, 80); // 0xA0 - 0x50 = 80 armors
    }
}
