//! Game data entry types.
//!
//! Each entry type corresponds to a data structure in the game's executable.

mod accessory;
mod armor;
mod weapon;
mod weapon_effect;
mod traits;
mod usable_item;
mod special_item;
mod character;
mod character_magic;
mod character_super_move;
mod crew_member;
mod playable_ship;
mod ship_cannon;
mod ship_accessory;
mod ship_item;
mod enemy_ship;
mod enemy;
mod enemy_task;
mod enemy_magic;
mod enemy_super_move;
mod swashbuckler;
mod spirit_curve;
mod exp_boost;
mod shop;
mod treasure_chest;

pub use accessory::Accessory;
pub use armor::{Armor, CharacterFlags};
pub use weapon::Weapon;
pub use weapon_effect::WeaponEffect;
pub use traits::{Trait, TraitId, TRAIT_NAMES};
pub use usable_item::{UsableItem, OccasionFlags};
pub use special_item::SpecialItem;
pub use character::Character;
pub use character_magic::CharacterMagic;
pub use character_super_move::CharacterSuperMove;
pub use crew_member::CrewMember;
pub use playable_ship::PlayableShip;
pub use ship_cannon::ShipCannon;
pub use ship_accessory::ShipAccessory;
pub use ship_item::ShipItem;
pub use enemy_ship::{EnemyShip, ShipArmament, ShipItemDrop};
pub use enemy::{Enemy, EnemyItemDrop};
pub use enemy_task::EnemyTask;
pub use enemy_magic::EnemyMagic;
pub use enemy_super_move::{EnemySuperMove, EnemySkillCategory};
pub use swashbuckler::Swashbuckler;
pub use spirit_curve::{SpiritCurve, SpiritLevel};
pub use exp_boost::ExpBoost;
pub use shop::Shop;
pub use treasure_chest::TreasureChest;

