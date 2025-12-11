//! Lookup tables for game data names and strings.
//!
//! These tables map IDs to human-readable names, matching the Ruby ALX vocabulary.

/// Get character name by ID.
pub fn character_name(id: i8) -> &'static str {
    match id {
        0 => "Vyse",
        1 => "Aika",
        2 => "Fina",
        3 => "Drachma",
        4 => "Enrique",
        5 => "Gilder",
        _ => "???",
    }
}

/// Get trait name by ID.
pub fn trait_name(id: i8) -> &'static str {
    match id {
        -1 => "None",
        0 => "Power",
        1 => "Will",
        2 => "Vigor",
        3 => "Agile",
        4 => "Quick",
        16 => "Attack",
        17 => "Defense",
        18 => "MagDef",
        19 => "Hit%",
        20 => "Dodge%",
        // Element traits (32-37)
        32 => "Green",
        33 => "Red",
        34 => "Purple",
        35 => "Blue",
        36 => "Yellow",
        37 => "Silver",
        // State traits (48-63)
        48 => "Poison",
        49 => "Unconscious",
        50 => "Stone",
        51 => "Sleep",
        52 => "Confusion",
        53 => "Silence",
        54 => "Fatigue",
        55 => "Revival",
        56 => "Weak",
        57 => "State 10",
        58 => "State 11",
        59 => "State 12",
        60 => "State 13",
        61 => "State 14",
        62 => "State 15",
        63 => "Danger",
        // Misc traits
        64 => "Block Magic",
        65 => "Block Attack",
        68 => "Reduce SP",
        73 => "Counter%",
        77 => "Recover SP",
        78 => "Regenerate",
        79 => "Block Neg States",
        80 => "PC 1st Strike%",
        81 => "PC Run%",
        82 => "EC 1st Strike%",
        83 => "EC Run%",
        84 => "Random Encounter%",
        _ => "???",
    }
}

/// Get element name by ID.
pub fn element_name(id: i8) -> &'static str {
    match id {
        -1 => "None",
        0 => "Green",
        1 => "Red",
        2 => "Purple",
        3 => "Blue",
        4 => "Yellow",
        5 => "Silver",
        6 => "Neutral",
        _ => "???",
    }
}

/// Get state name by ID.
pub fn state_name(id: i8) -> &'static str {
    match id {
        -1 => "None",
        0 => "Poison",
        1 => "Unconscious",
        2 => "Stone",
        3 => "Sleep",
        4 => "Confusion",
        5 => "Silence",
        6 => "Fatigue",
        7 => "Revival",
        8 => "Weak",
        _ => "???",
    }
}

/// Format character flags as binary string (e.g., "0b00111010").
pub fn format_character_flags(flags: u16) -> String {
    format!("0b{:08b}", flags & 0xFF)
}

/// Get effect name by ID (full vocabulary).
pub fn effect_name(id: i16) -> &'static str {
    match id {
        -1 | 255 => "None",
        0 => "Damage",
        1 => "Guard",
        2 => "Run",
        3 => "Poison",
        4 => "Poison + Damage",
        5 => "Unconscious",
        6 => "Unconscious + Damage A",
        7 => "Stone",
        8 => "Stone + Damage",
        9 => "Sleep",
        10 => "Sleep by 100%",
        11 => "Confusion",
        12 => "Silence",
        13 => "Silence by 100%",
        14 => "Fatigue",
        15 => "Poison + Sleep (Undef)",
        16 => "Rem Pos States",
        17 => "Rem Pos States + Damage",
        18 => "Incr Attack (Undef)",
        19 => "Incr Defense (Undef)",
        20 => "Incr Quick",
        21 => "Incr Attack & Defense",
        22 => "Incr All Attr (Undef)",
        23 => "Decr Attack (Undef)",
        24 => "Decr Defense (Undef)",
        25 => "Decr Quick (Undef)",
        26 => "Decr All Attr",
        27 => "Revive With HP of 50%",
        28 => "Revive With HP of 100%",
        29 => "Rem Neg States A",
        30 => "Rem Neg States B",
        31 => "Recover HP",
        32 => "Recover HP of 100%",
        33 => "Regenerate",
        35 => "Recover SP",
        36 => "Recover SP + Incr Counter% (Undef)",
        37 => "Counter% to 100%",
        38 => "Counter% to 100% + Guard",
        39 => "Block Magic",
        40 => "Invulnerable (Undef)",
        41 => "Recover SP by 200% + Guard",
        42 => "Block Neg States",
        43 => "Call EC",
        44 => "Absorb HP",
        45 => "Reduce SP (Undef)",
        46 => "Counter% to 100% + Block Attack",
        47 => "Recover HP & MP + Rem Neg States",
        48 => "Recover MP",
        49 => "Recover MP of 100%",
        50 => "Recover HP & MP by 100% A",
        51 => "Recover HP & MP by 100% B",
        52 => "Learn S-Move",
        53 => "Evolve Cupil",
        54 => "Grow Power",
        55 => "Grow Will",
        56 => "Grow Vigor",
        57 => "Grow Agile",
        58 => "Grow Quick",
        59 => "Grow MAXHP",
        60 => "Grow MAXMP",
        61 => "Unconscious by 100%",
        62 => "Defeat User + Damage",
        63 => "Regenerate + Sleep",
        64 => "Call EC + Defeat All ECs",
        65 => "Crew Special A",
        66 => "Crew Special B",
        67 => "Invulnerable",
        68 => "Unconscious + Damage B",
        69 => "Sleep + Damage",
        70 => "Confusion + Damage",
        71 => "Silence + Damage",
        72 => "Fatigue + Damage",
        73 => "Weak + Damage",
        74 => "Control PC",
        75 => "Evolve Cupil Now",
        76 => "Reset Cupil",
        77 => "Recover MP + Invulnerable",
        78 => "Regenerate + Incr Atk, Def & Qck",
        100 => "Recover HP",
        101 => "Recover HP by 100%",
        102 => "Recover SP",
        103 => "Recover SP by 100%",
        104 => "Incr Quick",
        105 => "Incr Attack + Defense",
        106 => "Weak by 100%",
        107 => "Damage",
        108 => "Incr All Attr",
        109 => "Block Attack",
        110 => "Block Magic",
        111 => "Incr First Strike%",
        112 => "Incr Critical%",
        113 => "Consume SP by 50%",
        114 => "Recover SP by 200%",
        115 => "Grow All Attr",
        116 => "Silence by 100%",
        117 => "Recover MP",
        _ => "???",
    }
}

/// Get scope name by ID (full vocabulary).
pub fn scope_name(id: u8) -> &'static str {
    match id {
        0 => "None",
        1 => "Single PC",
        2 => "All PCs",
        3 => "Single EC",
        4 => "All ECs",
        5 => "Self",
        6 => "All PCs & ECs",
        32 => "EC Square S3",
        33 => "EC Square S5",
        34 => "EC Square S7",
        36 => "EC Ray S1",
        37 | 39 => "EC Ray S3",
        38 => "EC Ray S5",
        43 => "EC Triangle 60°",
        96 => "PC Square S3",
        97 => "PC Square S5",
        98 => "PC Square S7",
        100 => "PC Ray S1",
        101 => "PC Ray S3",
        102 => "PC Ray S5",
        103 | 107 => "PC Triangle 60°",
        200 | 201 => "PCs Nearby S1",
        _ => "???",
    }
}

/// Get ship cannon type name by ID.
pub fn cannon_type_name(id: i8) -> &'static str {
    match id {
        -1 => "None",
        0 => "Main Cannon",
        1 => "Sec Cannon",
        2 => "Torpedo",
        3 => "Special",
        _ => "???",
    }
}

/// Get ship trait name by ID (different from character traits).
pub fn ship_trait_name(id: i8) -> &'static str {
    match id {
        -1 => "None",
        2 => "Defense",
        3 => "MagDef",
        4 => "Quick",
        6 => "Dodge%",
        7 => "Value",
        48 => "Main Cannon Atk",
        64 => "Sec Cannon Atk",
        81 => "Torpedo Hit%",
        96 => "Special Attack",
        _ => "???",
    }
}

/// Get type name by ID (for usable items, character magic, etc.).
pub fn type_name(id: i8) -> &'static str {
    match id {
        -1 => "None",
        0 => "Physical",
        1 => "Magical",
        _ => "???",
    }
}

/// Get category name by ID (character skill category).
pub fn category_name(id: i8) -> &'static str {
    match id {
        0 => "Vyse",
        1 => "Aika",
        2 => "Fina",
        3 => "Drachma",
        4 => "Enrique",
        5 => "Gilder",
        _ => "???",
    }
}

/// Get ship occasion name by ID.
pub fn ship_occasion_name(id: i8) -> &'static str {
    match id {
        -1 => "None",
        0 => "Magic Cannon",
        1 => "Always",
        _ => "???",
    }
}

/// Effect name lookup table.
pub struct EffectNames;

impl EffectNames {
    pub fn get(&self, id: i16) -> &'static str {
        effect_name(id)
    }
}

/// Scope name lookup table.
pub struct ScopeNames;

impl ScopeNames {
    pub fn get(&self, id: u8) -> &'static str {
        scope_name(id)
    }
}

/// Ship occasion name lookup table.
pub struct ShipOccasionNames;

impl ShipOccasionNames {
    pub fn get(&self, id: i8) -> &'static str {
        ship_occasion_name(id)
    }
}

/// State name lookup for CSV output.
pub const STATE_NAMES: StateNames = StateNames;

/// Element name lookup for CSV output.
pub const ELEMENT_NAMES: ElementNames = ElementNames;

/// Effect name lookup for CSV output.
pub const EFFECT_NAMES: EffectNames = EffectNames;

/// Scope name lookup for CSV output.
pub const SCOPE_NAMES: ScopeNames = ScopeNames;

/// Ship occasion lookup for CSV output.
pub const SHIP_OCCASION_NAMES: ShipOccasionNames = ShipOccasionNames;

/// State names lookup table.
pub struct StateNames;

impl StateNames {
    pub fn get(&self, id: i8) -> &'static str {
        state_name(id)
    }
}

/// Element names lookup table.
pub struct ElementNames;

impl ElementNames {
    pub fn get(&self, id: i8) -> &'static str {
        element_name(id)
    }
}

/// Get character flag marker ("X" or "") for a specific character.
pub fn character_flag_marker(flags: u16, character_id: u8) -> &'static str {
    // Bits are: V=5, A=4, F=3, D=2, E=1, G=0 (in the lower 6 bits)
    // The Ruby code uses (0x20 >> _id) for character index _id (0-5)
    let bit = 0x20 >> character_id;
    if (flags & bit) != 0 { "X" } else { "" }
}

/// Get enemy name by ID.
/// Returns the US/English name for an enemy, or "???" if unknown.
pub fn enemy_name(id: u32) -> &'static str {
    match id {
        0 => "Soldier",
        1 => "Guard",
        2 => "Seeker",
        3 => "Marocca",
        4 => "Grouder",
        7 => "Spell Warden",
        8 => "Flestik",
        9 => "Basallish",
        10 => "Mind Stealer",
        11 => "Walrenk",
        12 => "Varkris",
        13 => "Azbeth",
        14 => "Zivilyn Bane",
        15 => "Ghastling",
        16 => "Florast",
        17 => "Tsirat",
        18 => "Thorkryn",
        19 => "Scorfly",
        20 => "Centralk",
        21 => "Feralisk",
        22 => "Ferlith",
        23 => "Crylhound",
        24 => "Magma Tiki",
        25 => "Salamander",
        26 => "Baroo",
        27 => "Dung Fly",
        28 => "Death's Head",
        29 => "Looper",
        30 => "Loopalon",
        31 => "Tsurak",
        32 => "Enforcer",
        33 => "Serpantis",
        34 => "Dralnog",
        35 => "Que'lak",
        36 => "Pinalisk",
        37 => "Crylbeast",
        38 => "Slothstra",
        39 => "Polraxis",
        40 => "Roseln",
        41 => "Elooper",
        42 => "Grapor",
        43 => "Sphyrus",
        44 => "Iridzu",
        45 => "Flyst",
        46 => "Medulizk",
        47 => "Lucich",
        48 => "Digger",
        49 => "Flat Fiend",
        50 => "Burocca",
        51 => "Drogerp",
        52 => "Nadnarb",
        53 => "Nairad",
        54 => "Kanezl",
        55 => "Mantoid",
        56 => "Sorcerer",
        57 => "Slithar",
        58 => "Golooper",
        59 => "Alusphere",
        60 => "Soldier",
        61 => "Delzool",
        62 => "Defender",
        63 => "Ghrost",
        64 => "Graver",
        65 => "Shrilp",
        66 => "Cerosik",
        67 => "Carnilak",
        68 => "Valgand",
        69 => "Berserker",
        70 => "Lurgel Tank",
        71 => "Tenkou",
        72 => "Patrol Guard",
        73 => "Mine Patrol",
        74 => "Frost Worm",
        75 => "Destroyer",
        76 => "Marauder",
        77 => "Guardian",
        78 => "Telsor",
        79 => "Dorntak",
        80 => "Linark",
        81 => "Dolthstra",
        82 => "Dracolurg",
        83 => "Garagor",
        84 => "Kilite",
        85 => "Imezl",
        86 => "Lurker",
        87 => "Jellikra",
        88 => "Scorpon",
        89 => "Arclooper",
        90 => "Tsorok",
        91 => "Sentry",
        92 => "Langry",
        93 => "Durel Beetle",
        94 => "Totelm",
        95 => "Thryllak",
        96 => "Stonebeak",
        97 => "Razorbeak",
        98 => "Stalk Fiend",
        99 => "Kantor",
        100 => "Shadow",
        101 => "Assassin",
        102 => "Officer",
        103 => "Mage Warden",
        104 => "Shock Troop",
        105 => "Red Guard",
        106 => "Elite Guard",
        107 => "Jynnus",
        108 => "Hydra Elite",
        109 => "Yulooper",
        110 | 118 => "Hunter",
        111..=117 => "Zivilyn Bane",
        119 => "Delvax",
        120 => "Hopril",
        121 => "Elcian",
        128 => "Antonio",
        129 => "Sentinel",
        130 => "Bleigock",
        131 => "Executioner",
        132 | 151 => "Galcian",
        133 => "Royal Guard",
        134 => "Rokwyrm",
        136 => "Antonio 2",
        137 => "Rik'talish",
        138 => "Sinistra",
        139 => "Destra",
        140 | 152 | 153 | 157 => "Ramirez",
        141 | 149 => "Vigoro",
        142 => "Dralkor Tank",
        144 => "Tortigar",
        145 => "Jao",
        146 => "Mao",
        147 => "Muraji",
        148 => "Veltarn",
        150 => "Dracoslyth",
        154 => "Eliminator",
        155 => "Gordo",
        156 => "Mad Chef",
        158..=160 => "Piastol",
        161 => "Death Hound",
        _ => "???",
    }
}

/// Build a HashMap of enemy IDs to names.
pub fn enemy_names_map() -> std::collections::HashMap<u32, String> {
    let mut map = std::collections::HashMap::new();
    // Add all known enemies
    for id in 0..=200 {
        let name = enemy_name(id);
        if name != "???" {
            map.insert(id, name.to_string());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_names() {
        assert_eq!(character_name(0), "Vyse");
        assert_eq!(character_name(5), "Gilder");
        assert_eq!(character_name(99), "???");
    }

    #[test]
    fn test_trait_names() {
        assert_eq!(trait_name(-1), "None");
        assert_eq!(trait_name(18), "MagDef");
        assert_eq!(trait_name(36), "Yellow");
    }

    #[test]
    fn test_character_flags() {
        // 0b00111010 = V,A,F,E can equip
        let flags = 0b00111010u16;
        assert_eq!(format_character_flags(flags), "0b00111010");
        assert_eq!(character_flag_marker(flags, 0), "X"); // Vyse
        assert_eq!(character_flag_marker(flags, 1), "X"); // Aika
        assert_eq!(character_flag_marker(flags, 2), "X"); // Fina
        assert_eq!(character_flag_marker(flags, 3), "");  // Drachma
        assert_eq!(character_flag_marker(flags, 4), "X"); // Enrique
        assert_eq!(character_flag_marker(flags, 5), "");  // Gilder
    }
}

