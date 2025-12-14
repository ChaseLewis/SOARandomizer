//! Game data entry types.
//!
//! Each entry type corresponds to a data structure in the game's executable.

mod accessory;
mod armor;
mod character;
mod character_magic;
mod character_super_move;
mod crew_member;
mod enemy;
mod enemy_encounter;
mod enemy_event;
mod enemy_magic;
mod enemy_ship;
mod enemy_super_move;
mod enemy_task;
mod exp_boost;
mod exp_curve;
mod magic_exp_curve;
mod playable_ship;
mod ship_accessory;
mod ship_cannon;
mod ship_item;
mod shop;
mod special_item;
mod spirit_curve;
mod swashbuckler;
mod traits;
mod treasure_chest;
mod usable_item;
mod weapon;
mod weapon_effect;

pub use accessory::Accessory;
pub use armor::{Armor, CharacterFlags};
pub use character::Character;
pub use character_magic::CharacterMagic;
pub use character_super_move::CharacterSuperMove;
pub use crew_member::CrewMember;
pub use enemy::{Enemy, EnemyItemDrop};
pub use enemy_encounter::{EnemyEncounter, EnemySlot, MAX_ENEMY_SLOTS};
pub use enemy_event::{
    EnemyEvent, EventCharacterSlot, EventEnemySlot, DEFEAT_CONDITIONS, ESCAPE_CONDITIONS,
    MAX_EVENT_CHARACTERS, MAX_EVENT_ENEMIES,
};
pub use enemy_magic::EnemyMagic;
pub use enemy_ship::{EnemyShip, ShipArmament, ShipItemDrop};
pub use enemy_super_move::{EnemySkillCategory, EnemySuperMove};
pub use enemy_task::EnemyTask;
pub use exp_boost::ExpBoost;
pub use exp_curve::ExpCurve;
pub use magic_exp_curve::MagicExpCurve;
pub use playable_ship::PlayableShip;
pub use ship_accessory::ShipAccessory;
pub use ship_cannon::ShipCannon;
pub use ship_item::ShipItem;
pub use shop::Shop;
pub use special_item::SpecialItem;
pub use spirit_curve::{SpiritCurve, SpiritLevel};
pub use swashbuckler::Swashbuckler;
pub use traits::{Trait, TraitId, TRAIT_NAMES};
pub use treasure_chest::TreasureChest;
pub use usable_item::{OccasionFlags, UsableItem};
pub use weapon::Weapon;
pub use weapon_effect::WeaponEffect;
