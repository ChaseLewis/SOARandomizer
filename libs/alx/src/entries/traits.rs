//! Trait system for equipment effects.

use serde::{Deserialize, Serialize};

/// Trait ID used for equipment effects.
pub type TraitId = i8;

/// A trait (stat modifier) on equipment.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Trait {
    /// Trait ID (-1 = None)
    pub id: TraitId,
    /// Trait value (modifier amount)
    pub value: i16,
}

impl Trait {
    /// Check if this is an empty/none trait.
    pub fn is_none(&self) -> bool {
        self.id < 0
    }

    /// Get the trait name for the given ID.
    pub fn name(&self) -> &'static str {
        trait_name(self.id)
    }
}

/// Get the name for a trait ID.
pub fn trait_name(id: TraitId) -> &'static str {
    if id < 0 || id as usize >= TRAIT_NAMES.len() {
        "None"
    } else {
        TRAIT_NAMES[id as usize]
    }
}

/// Trait names indexed by trait ID.
/// Extracted from the original ALX voc.rb vocabulary.
pub const TRAIT_NAMES: &[&str] = &[
    "MaxHP",           // 0
    "Will",            // 1
    "Vigor",           // 2
    "Agile",           // 3
    "Quick",           // 4
    "Unknown 5",       // 5
    "Unknown 6",       // 6
    "Unknown 7",       // 7
    "Unknown 8",       // 8
    "Unknown 9",       // 9
    "Unknown 10",      // 10
    "Unknown 11",      // 11
    "Unknown 12",      // 12
    "Unknown 13",      // 13
    "Unknown 14",      // 14
    "Unknown 15",      // 15
    "Attack",          // 16
    "Defense",         // 17
    "MagDef",          // 18
    "Hit%",            // 19
    "Dodge%",          // 20
    "Unknown 21",      // 21
    "Unknown 22",      // 22
    "Unknown 23",      // 23
    "Unknown 24",      // 24
    "Unknown 25",      // 25
    "Unknown 26",      // 26
    "Unknown 27",      // 27
    "Unknown 28",      // 28
    "Unknown 29",      // 29
    "Unknown 30",      // 30
    "Unknown 31",      // 31
    "Green",           // 32
    "Red",             // 33
    "Purple",          // 34
    "Blue",            // 35
    "Yellow",          // 36
    "Silver",          // 37
    "Unknown 38",      // 38
    "Unknown 39",      // 39
    "Unknown 40",      // 40
    "Unknown 41",      // 41
    "Unknown 42",      // 42
    "Unknown 43",      // 43
    "Unknown 44",      // 44
    "Unknown 45",      // 45
    "Unknown 46",      // 46
    "Unknown 47",      // 47
    "Poison",          // 48
    "Unconscious",     // 49
    "Stone",           // 50
    "Sleep",           // 51
    "Confusion",       // 52
    "Silence",         // 53
    "Fatigue",         // 54
    "Unknown 55",      // 55
    "Weak",            // 56
    "Unknown 57",      // 57
    "Unknown 58",      // 58
    "Unknown 59",      // 59
    "Unknown 60",      // 60
    "Unknown 61",      // 61
    "Unknown 62",      // 62
    "Unknown 63",      // 63
    "Unknown 64",      // 64
    "Block Attack",    // 65
    "Unknown 66",      // 66
    "Unknown 67",      // 67
    "Reduce SP",       // 68
    "Unknown 69",      // 69
    "Unknown 70",      // 70
    "Unknown 71",      // 71
    "Unknown 72",      // 72
    "Counter%",        // 73
    "Unknown 74",      // 74
    "Unknown 75",      // 75
    "Unknown 76",      // 76
    "Recover SP",      // 77
    "Unknown 78",      // 78
    "Block Neg States", // 79
    "PC 1st Strike%",  // 80
    "PC Run%",         // 81
    "EC 1st Strike%",  // 82
    "EC Run%",         // 83
    "Random Encounter%", // 84
];

