//! Game root - main entry point for interacting with a game ISO.

use std::path::Path;

use super::offsets::Offsets;
use super::region::GameVersion;
use crate::entries::{
    Accessory, Armor, Character, CharacterMagic, CharacterSuperMove, CrewMember, Enemy,
    EnemyEncounter, EnemyMagic, EnemyShip, EnemySuperMove, EnemyTask, ExpBoost, ExpCurve,
    MagicExpCurve, PlayableShip, ShipAccessory, ShipCannon, ShipItem, Shop, SpecialItem,
    SpiritCurve, Swashbuckler, TreasureChest, UsableItem, Weapon, WeaponEffect,
};
use crate::error::{Error, Result};
use crate::io::{
    compress_aklz, decompress_aklz, is_aklz, parse_dat_file, parse_enp, parse_evp,
    patch_enp_encounters,
};
use crate::io::{read_description_strings, IsoFile};
use crate::items::ItemDatabase;

/// Main interface for working with a Skies of Arcadia Legends ISO.
pub struct GameRoot {
    iso: IsoFile,
    version: GameVersion,
    offsets: Offsets,
    /// Cached Start.dol data
    dol_data: Option<Vec<u8>>,
    /// Cached level file data (for EXP curves)
    level_data: Option<Vec<u8>>,
}

impl GameRoot {
    /// Open a game ISO and detect its version.
    pub fn open(path: &Path) -> Result<Self> {
        let mut iso = IsoFile::open(path)?;

        // Read game ID to detect version
        let game_id = iso.read_game_id()?;
        let version = GameVersion::from_game_id(&game_id).ok_or_else(|| {
            Error::InvalidIso(format!("Not a Skies of Arcadia Legends ISO: {}", game_id))
        })?;

        let offsets = Offsets::for_version(&version)?;

        Ok(Self {
            iso,
            version,
            offsets,
            dol_data: None,
            level_data: None,
        })
    }

    /// Get the detected game version.
    pub fn version(&self) -> &GameVersion {
        &self.version
    }

    /// Get the data offsets for this version.
    pub fn offsets(&self) -> &Offsets {
        &self.offsets
    }

    /// List files in the ISO matching a pattern.
    pub fn list_iso_files_matching(
        &mut self,
        pattern: &str,
    ) -> Result<Vec<crate::io::IsoFileEntry>> {
        self.iso.list_files_matching(pattern)
    }

    /// Read a file from the ISO by its entry.
    pub fn read_file_direct(&mut self, entry: &crate::io::IsoFileEntry) -> Result<Vec<u8>> {
        self.iso.read_file_direct(entry)
    }

    /// Get a reference to the ISO file.
    pub fn iso(&self) -> &IsoFile {
        &self.iso
    }

    /// Get a mutable reference to the ISO file.
    pub fn iso_mut(&mut self) -> &mut IsoFile {
        &mut self.iso
    }

    /// Load the Start.dol executable into memory.
    /// This is cached for subsequent reads.
    pub fn load_dol(&mut self) -> Result<&[u8]> {
        if self.dol_data.is_none() {
            // gc_fst uses "Start.dol" as a special path (not the filesystem path)
            let dol_path = Path::new("Start.dol");
            let data = self.iso.read_file(dol_path)?;
            self.dol_data = Some(data);
        }
        Ok(self.dol_data.as_ref().unwrap())
    }

    /// Get a slice of the DOL data at the given range.
    pub fn dol_slice(&mut self, range: std::ops::Range<usize>) -> Result<&[u8]> {
        let dol = self.load_dol()?;
        if range.end > dol.len() {
            return Err(Error::ParseError {
                offset: range.start,
                message: format!(
                    "Range {:x}..{:x} exceeds DOL size {:x}",
                    range.start,
                    range.end,
                    dol.len()
                ),
            });
        }
        Ok(&dol[range])
    }

    /// Load the DOL data mutably (for writing).
    fn load_dol_mut(&mut self) -> Result<&mut Vec<u8>> {
        if self.dol_data.is_none() {
            let dol_path = Path::new("Start.dol");
            let data = self.iso.read_file(dol_path)?;
            self.dol_data = Some(data);
        }
        Ok(self.dol_data.as_mut().unwrap())
    }

    /// Write bytes to a range in the DOL data.
    pub fn write_to_dol(&mut self, range: std::ops::Range<usize>, data: &[u8]) -> Result<()> {
        let dol = self.load_dol_mut()?;
        if range.end > dol.len() {
            return Err(Error::ParseError {
                offset: range.start,
                message: format!(
                    "Range {:x}..{:x} exceeds DOL size {:x}",
                    range.start,
                    range.end,
                    dol.len()
                ),
            });
        }
        if range.len() != data.len() {
            return Err(Error::ValidationError(format!(
                "Data length {} does not match range length {}",
                data.len(),
                range.len()
            )));
        }
        dol[range].copy_from_slice(data);
        Ok(())
    }

    /// Save the modified DOL back to the ISO.
    pub fn save_dol(&mut self) -> Result<()> {
        if let Some(ref dol_data) = self.dol_data {
            let dol_path = Path::new("Start.dol");
            self.iso.write_file(dol_path, dol_data)?;
        }
        Ok(())
    }

    /// Load the level file (contains EXP curves) into memory.
    /// This is cached for subsequent reads.
    pub fn load_level_file(&mut self) -> Result<&[u8]> {
        if self.level_data.is_none() {
            let level_path = Path::new(self.offsets.level_file);
            let data = self.iso.read_file(level_path)?;
            self.level_data = Some(data);
        }
        Ok(self.level_data.as_ref().unwrap())
    }

    /// Get a slice of the level file data at the given range.
    pub fn level_slice(&mut self, range: std::ops::Range<usize>) -> Result<&[u8]> {
        let level = self.load_level_file()?;
        if range.end > level.len() {
            return Err(Error::ParseError {
                offset: range.start,
                message: format!(
                    "Range {:x}..{:x} exceeds level file size {:x}",
                    range.start,
                    range.end,
                    level.len()
                ),
            });
        }
        Ok(&level[range])
    }

    /// Load the level file data mutably (for writing).
    fn load_level_mut(&mut self) -> Result<&mut Vec<u8>> {
        if self.level_data.is_none() {
            let level_path = Path::new(self.offsets.level_file);
            let data = self.iso.read_file(level_path)?;
            self.level_data = Some(data);
        }
        Ok(self.level_data.as_mut().unwrap())
    }

    /// Write bytes to a range in the level file data.
    pub fn write_to_level(&mut self, range: std::ops::Range<usize>, data: &[u8]) -> Result<()> {
        let level = self.load_level_mut()?;
        if range.end > level.len() {
            return Err(Error::ParseError {
                offset: range.start,
                message: format!(
                    "Range {:x}..{:x} exceeds level file size {:x}",
                    range.start,
                    range.end,
                    level.len()
                ),
            });
        }
        if range.len() != data.len() {
            return Err(Error::ValidationError(format!(
                "Data length {} does not match range length {}",
                data.len(),
                range.len()
            )));
        }
        level[range].copy_from_slice(data);
        Ok(())
    }

    /// Save the modified level file back to the ISO.
    pub fn save_level(&mut self) -> Result<()> {
        if let Some(ref level_data) = self.level_data {
            let level_path = Path::new(self.offsets.level_file);
            self.iso.write_file(level_path, level_data)?;
        }
        Ok(())
    }

    // ========================================================================
    // Write methods for each entry type
    // ========================================================================

    /// Write accessories to the DOL.
    pub fn write_accessories(&mut self, accessories: &[Accessory]) -> Result<()> {
        let data_range = self.offsets.accessory_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        Accessory::patch_all(accessories, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write armors to the DOL (patch approach).
    pub fn write_armors(&mut self, armors: &[Armor]) -> Result<()> {
        let data_range = self.offsets.armor_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        Armor::patch_all(armors, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write weapons to the DOL (patch approach).
    pub fn write_weapons(&mut self, weapons: &[Weapon]) -> Result<()> {
        let data_range = self.offsets.weapon_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        Weapon::patch_all(weapons, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write usable items to the DOL (patch approach).
    pub fn write_usable_items(&mut self, items: &[UsableItem]) -> Result<()> {
        let data_range = self.offsets.usable_item_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        UsableItem::patch_all(items, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write special items to the DOL (patch approach).
    pub fn write_special_items(&mut self, items: &[SpecialItem]) -> Result<()> {
        let data_range = self.offsets.special_item_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        SpecialItem::patch_all(items, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write characters to the DOL.
    pub fn write_characters(&mut self, characters: &[Character]) -> Result<()> {
        let data_range = self.offsets.character_data.clone();
        // Read original section, patch only numeric fields, write back
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        Character::patch_all(characters, &mut buffer);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write character magic to the DOL (patch approach).
    pub fn write_character_magic(&mut self, magic: &[CharacterMagic]) -> Result<()> {
        let data_range = self.offsets.character_magic_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        CharacterMagic::patch_all(magic, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write character super moves to the DOL (patch approach).
    pub fn write_character_super_moves(&mut self, moves: &[CharacterSuperMove]) -> Result<()> {
        let data_range = self.offsets.character_super_move_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        CharacterSuperMove::patch_all(moves, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write shops to the DOL (patch approach).
    pub fn write_shops(&mut self, shops: &[Shop]) -> Result<()> {
        let data_range = self.offsets.shop_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        Shop::patch_all(shops, &mut buffer);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write treasure chests to the DOL (patch approach).
    pub fn write_treasure_chests(&mut self, chests: &[TreasureChest]) -> Result<()> {
        let data_range = self.offsets.treasure_chest_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        TreasureChest::patch_all(chests, &mut buffer);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write crew members to the DOL (patch approach).
    pub fn write_crew_members(&mut self, members: &[CrewMember]) -> Result<()> {
        let data_range = self.offsets.crew_member_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        CrewMember::patch_all(members, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write playable ships to the DOL (patch approach).
    pub fn write_playable_ships(&mut self, ships: &[PlayableShip]) -> Result<()> {
        let data_range = self.offsets.playable_ship_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        PlayableShip::patch_all(ships, &mut buffer);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write ship cannons to the DOL (patch approach).
    pub fn write_ship_cannons(&mut self, cannons: &[ShipCannon]) -> Result<()> {
        let data_range = self.offsets.ship_cannon_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        ShipCannon::patch_all(cannons, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write ship accessories to the DOL (patch approach).
    pub fn write_ship_accessories(&mut self, accessories: &[ShipAccessory]) -> Result<()> {
        let data_range = self.offsets.ship_accessory_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        ShipAccessory::patch_all(accessories, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write ship items to the DOL (patch approach).
    pub fn write_ship_items(&mut self, items: &[ShipItem]) -> Result<()> {
        let data_range = self.offsets.ship_item_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        ShipItem::patch_all(items, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write enemy ships to the DOL (patch approach).
    pub fn write_enemy_ships(&mut self, ships: &[EnemyShip]) -> Result<()> {
        let data_range = self.offsets.enemy_ship_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        EnemyShip::patch_all(ships, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write enemy magic to the DOL (patch approach).
    pub fn write_enemy_magic(&mut self, magic: &[EnemyMagic]) -> Result<()> {
        let data_range = self.offsets.enemy_magic_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        EnemyMagic::patch_all(magic, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write enemy super moves to the DOL (patch approach).
    pub fn write_enemy_super_moves(&mut self, moves: &[EnemySuperMove]) -> Result<()> {
        let data_range = self.offsets.enemy_super_move_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        EnemySuperMove::patch_all(moves, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write swashbucklers to the DOL (patch approach).
    pub fn write_swashbucklers(&mut self, swashbucklers: &[Swashbuckler]) -> Result<()> {
        let data_range = self.offsets.swashbuckler_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        Swashbuckler::patch_all(swashbucklers, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write spirit curves to the DOL (patch approach).
    pub fn write_spirit_curves(&mut self, curves: &[SpiritCurve]) -> Result<()> {
        let data_range = self.offsets.spirit_curve_data.clone();
        let dol = self
            .dol_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
        let mut buffer = dol[data_range.clone()].to_vec();
        SpiritCurve::patch_all(curves, &mut buffer, &self.version);
        self.write_to_dol(data_range, &buffer)
    }

    /// Write exp boosts to the DOL (patch approach).
    pub fn write_exp_boosts(&mut self, boosts: &[ExpBoost]) -> Result<()> {
        if let Some(data_range) = self.offsets.exp_boost_data.clone() {
            let dol = self
                .dol_data
                .as_ref()
                .ok_or_else(|| crate::error::Error::InvalidIso("DOL not loaded".into()))?;
            let mut buffer = dol[data_range.clone()].to_vec();
            ExpBoost::patch_all(boosts, &mut buffer, &self.version);
            self.write_to_dol(data_range, &buffer)
        } else {
            Ok(())
        }
    }

    /// Write EXP curves to the level file (patch approach).
    pub fn write_exp_curves(&mut self, curves: &[ExpCurve]) -> Result<()> {
        let data_range = self.offsets.exp_curve_data.clone();
        let level = self
            .level_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("Level file not loaded".into()))?;
        let mut buffer = level[data_range.clone()].to_vec();
        ExpCurve::patch_all(curves, &mut buffer);
        self.write_to_level(data_range, &buffer)
    }

    /// Write Magic EXP curves to the level file (patch approach).
    pub fn write_magic_exp_curves(&mut self, curves: &[MagicExpCurve]) -> Result<()> {
        let data_range = self.offsets.magic_exp_curve_data.clone();
        let level = self
            .level_data
            .as_ref()
            .ok_or_else(|| crate::error::Error::InvalidIso("Level file not loaded".into()))?;
        let mut buffer = level[data_range.clone()].to_vec();
        MagicExpCurve::patch_all(curves, &mut buffer);
        self.write_to_level(data_range, &buffer)
    }

    // ========================================================================
    // Read methods for each entry type
    // ========================================================================

    /// Read all accessories from the game.
    pub fn read_accessories(&mut self) -> Result<Vec<Accessory>> {
        let data_range = self.offsets.accessory_data.clone();
        let dscr_range = self.offsets.accessory_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut accessories = Accessory::read_all_data(&data, &self.version)?;

        // Read descriptions
        if dscr_range.start < dscr_range.end {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions = read_description_strings(
                &dscr_data,
                dscr_range.start,
                accessories.len(),
                4, // 4-byte alignment for US/JP
            )?;

            for (acc, (pos, size, text)) in accessories.iter_mut().zip(descriptions) {
                acc.description_pos = pos;
                acc.description_size = size;
                acc.description = text;
            }
        }

        Ok(accessories)
    }

    /// Read all armors from the game.
    pub fn read_armors(&mut self) -> Result<Vec<Armor>> {
        let data_range = self.offsets.armor_data.clone();
        let dscr_range = self.offsets.armor_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut armors = Armor::read_all_data(&data, &self.version)?;

        // Read descriptions
        if dscr_range.start < dscr_range.end {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, armors.len(), 4)?;

            for (armor, (pos, size, text)) in armors.iter_mut().zip(descriptions) {
                armor.description_pos = pos;
                armor.description_size = size;
                armor.description = text;
            }
        }

        Ok(armors)
    }

    /// Read all weapons from the game.
    pub fn read_weapons(&mut self) -> Result<Vec<Weapon>> {
        let data_range = self.offsets.weapon_data.clone();
        let dscr_range = self.offsets.weapon_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut weapons = Weapon::read_all_data(&data, &self.version)?;

        // Read descriptions
        if dscr_range.start < dscr_range.end {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, weapons.len(), 4)?;

            for (weapon, (pos, size, text)) in weapons.iter_mut().zip(descriptions) {
                weapon.description_pos = pos;
                weapon.description_size = size;
                weapon.description = text;
            }
        }

        Ok(weapons)
    }

    /// Read all weapon effects from the game.
    pub fn read_weapon_effects(&mut self) -> Result<Vec<WeaponEffect>> {
        let data_range = self.offsets.weapon_effect_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        WeaponEffect::read_all_data(&data, &self.version)
    }

    /// Read all usable items from the game.
    pub fn read_usable_items(&mut self) -> Result<Vec<UsableItem>> {
        let data_range = self.offsets.usable_item_data.clone();
        let dscr_range = self.offsets.usable_item_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut items = UsableItem::read_all_data(&data, &self.version)?;

        // Read descriptions
        if dscr_range.start < dscr_range.end {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, items.len(), 4)?;

            for (item, (pos, size, text)) in items.iter_mut().zip(descriptions) {
                item.description_pos = pos;
                item.description_size = size;
                item.description = text;
            }
        }

        Ok(items)
    }

    /// Read all special items from the game.
    pub fn read_special_items(&mut self) -> Result<Vec<SpecialItem>> {
        let data_range = self.offsets.special_item_data.clone();
        let dscr_range = self.offsets.special_item_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut items = SpecialItem::read_all_data(&data, &self.version)?;

        // Read descriptions
        if dscr_range.start < dscr_range.end {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, items.len(), 4)?;

            for (item, (pos, size, text)) in items.iter_mut().zip(descriptions) {
                item.description_pos = pos;
                item.description_size = size;
                item.description = text;
            }
        }

        Ok(items)
    }

    /// Read all playable characters from the game.
    pub fn read_characters(&mut self) -> Result<Vec<Character>> {
        let data_range = self.offsets.character_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        Character::read_all_data(&data, &self.version)
    }

    /// Read all character magic (spells) from the game.
    pub fn read_character_magic(&mut self) -> Result<Vec<CharacterMagic>> {
        let data_range = self.offsets.character_magic_data.clone();
        let dscr_range = self.offsets.character_magic_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut magics = CharacterMagic::read_all_data(&data, &self.version)?;

        // Read descriptions
        if dscr_range.start < dscr_range.end {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, magics.len(), 4)?;

            for (magic, (pos, size, text)) in magics.iter_mut().zip(descriptions) {
                magic.description_pos = pos;
                magic.description_size = size;
                magic.description = text;
            }
        }

        Ok(magics)
    }

    /// Read all shops from the game.
    pub fn read_shops(&mut self) -> Result<Vec<Shop>> {
        let data_range = self.offsets.shop_data.clone();
        let dscr_range = self.offsets.shop_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut shops = Shop::read_all_data(&data, &self.version)?;

        // Read descriptions
        if dscr_range.start < dscr_range.end {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, shops.len(), 4)?;

            for (shop, (pos, size, text)) in shops.iter_mut().zip(descriptions) {
                shop.description_pos = pos;
                shop.description_size = size;
                shop.description = text;
            }
        }

        Ok(shops)
    }

    /// Read all treasure chests from the game.
    pub fn read_treasure_chests(&mut self) -> Result<Vec<TreasureChest>> {
        let data_range = self.offsets.treasure_chest_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        TreasureChest::read_all_data(&data, &self.version)
    }

    /// Build an item database from all item types in the game.
    /// This allows looking up item names by ID and vice versa.
    pub fn build_item_database(&mut self) -> Result<ItemDatabase> {
        let weapons = self.read_weapons()?;
        let armors = self.read_armors()?;
        let accessories = self.read_accessories()?;
        let usable_items = self.read_usable_items()?;
        let special_items = self.read_special_items()?;
        let ship_cannons = self.read_ship_cannons()?;
        let ship_accessories = self.read_ship_accessories()?;
        let ship_items = self.read_ship_items()?;

        Ok(ItemDatabase::from_game_data(
            &weapons,
            &armors,
            &accessories,
            &usable_items,
            &special_items,
            &ship_cannons,
            &ship_accessories,
            &ship_items,
        ))
    }

    /// Read all character super moves (S-Moves) from the game.
    pub fn read_character_super_moves(&mut self) -> Result<Vec<CharacterSuperMove>> {
        let data_range = self.offsets.character_super_move_data.clone();
        let dscr_range = self.offsets.character_super_move_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut entries = CharacterSuperMove::read_all_data(&data, &self.version)?;

        // Read descriptions if range is valid
        if !dscr_range.is_empty() {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;

            for (entry, (pos, size, text)) in entries.iter_mut().zip(descriptions.iter()) {
                entry.description_pos = *pos;
                entry.description_size = *size;
                entry.description = text.clone();
            }
        }

        Ok(entries)
    }

    /// Read all crew members from the game.
    pub fn read_crew_members(&mut self) -> Result<Vec<CrewMember>> {
        let data_range = self.offsets.crew_member_data.clone();
        let dscr_range = self.offsets.crew_member_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut entries = CrewMember::read_all_data(&data, &self.version)?;

        // Read descriptions if range is valid
        if !dscr_range.is_empty() {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;

            for (entry, (pos, size, text)) in entries.iter_mut().zip(descriptions.iter()) {
                entry.description_pos = *pos;
                entry.description_size = *size;
                entry.description = text.clone();
            }
        }

        Ok(entries)
    }

    /// Read all playable ships from the game.
    pub fn read_playable_ships(&mut self) -> Result<Vec<PlayableShip>> {
        let data_range = self.offsets.playable_ship_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        PlayableShip::read_all_data(&data, &self.version)
    }

    /// Read all ship cannons from the game.
    pub fn read_ship_cannons(&mut self) -> Result<Vec<ShipCannon>> {
        let data_range = self.offsets.ship_cannon_data.clone();
        let dscr_range = self.offsets.ship_cannon_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut entries = ShipCannon::read_all_data(&data, &self.version)?;

        // Read descriptions if range is valid
        if !dscr_range.is_empty() {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;

            for (entry, (pos, size, text)) in entries.iter_mut().zip(descriptions.iter()) {
                entry.description_pos = *pos;
                entry.description_size = *size;
                entry.description = text.clone();
            }
        }

        Ok(entries)
    }

    /// Read all ship accessories from the game.
    pub fn read_ship_accessories(&mut self) -> Result<Vec<ShipAccessory>> {
        let data_range = self.offsets.ship_accessory_data.clone();
        let dscr_range = self.offsets.ship_accessory_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut entries = ShipAccessory::read_all_data(&data, &self.version)?;

        // Read descriptions if range is valid
        if !dscr_range.is_empty() {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;

            for (entry, (pos, size, text)) in entries.iter_mut().zip(descriptions.iter()) {
                entry.description_pos = *pos;
                entry.description_size = *size;
                entry.description = text.clone();
            }
        }

        Ok(entries)
    }

    /// Read all ship items from the game.
    pub fn read_ship_items(&mut self) -> Result<Vec<ShipItem>> {
        let data_range = self.offsets.ship_item_data.clone();
        let dscr_range = self.offsets.ship_item_dscr.clone();

        let data = self.dol_slice(data_range)?.to_vec();
        let mut entries = ShipItem::read_all_data(&data, &self.version)?;

        // Read descriptions if range is valid
        if !dscr_range.is_empty() {
            let dscr_data = self.dol_slice(dscr_range.clone())?.to_vec();
            let descriptions =
                read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;

            for (entry, (pos, size, text)) in entries.iter_mut().zip(descriptions.iter()) {
                entry.description_pos = *pos;
                entry.description_size = *size;
                entry.description = text.clone();
            }
        }

        Ok(entries)
    }

    /// Read all enemy ships from the game.
    pub fn read_enemy_ships(&mut self) -> Result<Vec<EnemyShip>> {
        let data_range = self.offsets.enemy_ship_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        EnemyShip::read_all_data(&data, &self.version)
    }

    /// Read all enemy magic spells from the game.
    pub fn read_enemy_magic(&mut self) -> Result<Vec<EnemyMagic>> {
        let data_range = self.offsets.enemy_magic_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        EnemyMagic::read_all_data(&data, &self.version)
    }

    /// Read all enemy super moves from the game.
    pub fn read_enemy_super_moves(&mut self) -> Result<Vec<EnemySuperMove>> {
        let data_range = self.offsets.enemy_super_move_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        EnemySuperMove::read_all_data(&data, &self.version)
    }

    /// Read all swashbuckler ratings from the game.
    pub fn read_swashbucklers(&mut self) -> Result<Vec<Swashbuckler>> {
        let data_range = self.offsets.swashbuckler_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        Swashbuckler::read_all_data(&data, &self.version)
    }

    /// Read all spirit curves from the game.
    pub fn read_spirit_curves(&mut self) -> Result<Vec<SpiritCurve>> {
        let data_range = self.offsets.spirit_curve_data.clone();
        let data = self.dol_slice(data_range)?.to_vec();
        SpiritCurve::read_all_data(&data, &self.version)
    }

    /// Read all exp boosts from the game.
    pub fn read_exp_boosts(&mut self) -> Result<Vec<ExpBoost>> {
        if let Some(data_range) = self.offsets.exp_boost_data.clone() {
            let data = self.dol_slice(data_range)?.to_vec();
            ExpBoost::read_all_data(&data, &self.version)
        } else {
            Ok(Vec::new())
        }
    }

    /// Read all EXP curves from the level file.
    pub fn read_exp_curves(&mut self) -> Result<Vec<ExpCurve>> {
        let data_range = self.offsets.exp_curve_data.clone();
        let data = self.level_slice(data_range)?.to_vec();
        ExpCurve::read_all_data(&data, &self.version)
    }

    /// Read all Magic EXP curves from the level file.
    pub fn read_magic_exp_curves(&mut self) -> Result<Vec<MagicExpCurve>> {
        let data_range = self.offsets.magic_exp_curve_data.clone();
        let data = self.level_slice(data_range)?.to_vec();
        MagicExpCurve::read_all_data(&data, &self.version)
    }

    /// Read all enemies from ENP, EVP, and DAT files in the ISO.
    /// Returns enemies and their tasks.
    ///
    /// Enemy handling (matching Ruby ALX behavior):
    /// - Collect all enemies from all files
    /// - Post-process to handle duplicates:
    ///   - Group by (ID, stats)
    ///   - If group has 2+ entries from different files: one becomes `*`, keep one file-specific
    ///   - Unique entries keep their original filter
    pub fn read_enemies(&mut self) -> Result<(Vec<Enemy>, Vec<EnemyTask>)> {
        let mut raw_enemies: Vec<Enemy> = Vec::new();
        let mut all_tasks: Vec<EnemyTask> = Vec::new();

        // 1. Read EVP file (epevent.evp) - scripted battle events
        if let Ok(evp_files) = self.iso.list_files_matching("epevent.evp") {
            for entry in &evp_files {
                let raw_data = self.iso.read_file_direct(entry)?;
                let data = decompress_aklz(&raw_data)?;

                let filename = entry
                    .path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "epevent.evp".to_string());

                let parsed = parse_evp(&data, &filename, &self.version)?;
                raw_enemies.extend(parsed.enemies);
                all_tasks.extend(parsed.tasks);
            }
        }

        // 2. Read ENP files (*_ep.enp) - field encounters
        let enp_files = self.iso.list_files_matching("_ep.enp")?;

        for entry in &enp_files {
            let raw_data = self.iso.read_file_direct(entry)?;
            let data = decompress_aklz(&raw_data)?;

            let filename = entry
                .path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "*".to_string());

            let parsed = parse_enp(&data, &filename, &self.version)?;
            raw_enemies.extend(parsed.enemies);
            all_tasks.extend(parsed.tasks);
        }

        // 3. Read EC/EB DAT files - battle init enemies
        let ec_files = self.iso.list_files_matching("ecinit");
        let eb_files = self.iso.list_files_matching("ebinit");

        for files in [ec_files, eb_files].into_iter().flatten() {
            for entry in &files {
                if entry.path.to_string_lossy().ends_with(".dat") {
                    let raw_data = self.iso.read_file_direct(entry)?;
                    let data = decompress_aklz(&raw_data)?;

                    let filename = entry
                        .path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "*".to_string());

                    let parsed = parse_dat_file(&data, &filename, &self.version)?;
                    raw_enemies.extend(parsed.enemies);
                    all_tasks.extend(parsed.tasks);
                }
            }
        }

        // Post-process: deduplicate enemies (matching Ruby ALX behavior)
        // 1. Enemies with IDENTICAL stats merge (files combined)
        // 2. Enemies with DIFFERENT stats stay separate
        // 3. After sorting, the entry with most files from ENP/EVP gets `*` filter
        use std::collections::HashMap;

        // Helper to compute a stats key for comparison
        // Two enemies with same stats_key are considered identical
        // Use a string key to avoid tuple size limits
        fn stats_key(e: &Enemy) -> String {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                e.max_hp,
                e.exp,
                e.gold,
                e.attack,
                e.defense,
                e.mag_def,
                e.quick,
                e.agile,
                e.level,
                e.counter,
                e.danger,
                e.element_id,
                e.width,
                e.depth,
                e.will,
                e.vigor,
                e.hit,
                e.name_jp
            )
        }

        // Helper to get order (ENP=0, EVP=1, DAT=2)
        fn file_order(filter: &str) -> u8 {
            if filter == "*" || filter.ends_with(".enp") {
                0
            } else if filter.ends_with(".evp") {
                1
            } else {
                2 // DAT files
            }
        }

        // Group by (ID, stats_key) - enemies with same ID and stats merge
        let mut merged: HashMap<(u32, String), Enemy> = HashMap::new();

        // Track which (id, stats) combinations appeared in multiple files
        use std::collections::HashSet;
        let mut multi_file: HashSet<(u32, String)> = HashSet::new();

        for enemy in raw_enemies {
            let key = (enemy.id, stats_key(&enemy));

            if let Some(existing) = merged.get_mut(&key) {
                // Same ID and stats - mark as appearing in multiple files
                multi_file.insert(key.clone());

                // Keep the filter with lower order (ENP < EVP < DAT)
                let existing_order = file_order(&existing.filter);
                let new_order = file_order(&enemy.filter);

                if new_order < existing_order {
                    // New file has better priority - switch to it
                    existing.filter = enemy.filter;
                }
            } else {
                merged.insert(key, enemy);
            }
        }

        // Now process merged enemies - apply filter summarization
        let mut all_enemies: Vec<Enemy> = Vec::new();

        // Group by ID to apply filter rules
        let mut by_id: HashMap<u32, Vec<Enemy>> = HashMap::new();
        for (_, enemy) in merged {
            by_id.entry(enemy.id).or_default().push(enemy);
        }

        for (_id, mut enemies) in by_id {
            if enemies.len() == 1 {
                // Single stat variant for this ID
                let mut enemy = enemies.remove(0);
                let key = (enemy.id, stats_key(&enemy));

                // If this enemy appeared in multiple files, mark as global
                if multi_file.contains(&key) {
                    let order = file_order(&enemy.filter);
                    if order <= 2 {
                        enemy.filter = "*".to_string();
                    }
                }
                all_enemies.push(enemy);
                continue;
            }

            // Multiple stat variants for this ID
            // Sort by: order (asc - ENP/EVP first), then filter name
            enemies.sort_by(|a, b| {
                let a_order = file_order(&a.filter);
                let b_order = file_order(&b.filter);
                let cmp = a_order.cmp(&b_order);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
                a.filter.cmp(&b.filter)
            });

            // First enemy (lowest order = ENP/EVP) becomes global
            let mut first = enemies.remove(0);
            let first_order = file_order(&first.filter);
            let first_key = (first.id, stats_key(&first));

            // If first entry is from ENP/EVP (order <= 1) OR appeared in multiple files, mark as global
            if first_order <= 1 || multi_file.contains(&first_key) {
                first.filter = "*".to_string();
            }
            all_enemies.push(first);

            // Remaining entries become file-specific
            for enemy in enemies {
                all_enemies.push(enemy);
            }
        }

        Ok((all_enemies, all_tasks))
    }

    /// Read all enemy encounters from ENP files in the ISO.
    ///
    /// Enemy encounters define battle formations - which enemies appear together
    /// in a given battle, along with initiative and magic exp values.
    pub fn read_enemy_encounters(&mut self) -> Result<Vec<EnemyEncounter>> {
        let mut all_encounters: Vec<EnemyEncounter> = Vec::new();

        // Read ENP files (*_ep.enp) - field encounters
        let enp_files = self.iso.list_files_matching("_ep.enp")?;

        for entry in &enp_files {
            let raw_data = self.iso.read_file_direct(entry)?;
            let data = decompress_aklz(&raw_data)?;

            let filename = entry
                .path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "*".to_string());

            let parsed = parse_enp(&data, &filename, &self.version)?;
            all_encounters.extend(parsed.encounters);
        }

        // Sort by filter (filename) then by ID
        all_encounters.sort_by(|a, b| {
            let filter_cmp = a.filter.cmp(&b.filter);
            if filter_cmp != std::cmp::Ordering::Equal {
                return filter_cmp;
            }
            a.id.cmp(&b.id)
        });

        Ok(all_encounters)
    }

    /// Write enemy encounters back to ENP files in the ISO.
    ///
    /// This groups encounters by their filter (filename) and writes them
    /// back to the corresponding ENP files.
    ///
    /// If the original file was AKLZ compressed, the output will also be compressed.
    pub fn write_enemy_encounters(&mut self, encounters: &[EnemyEncounter]) -> Result<()> {
        use std::collections::HashMap;

        // Group encounters by filter (filename)
        let mut by_file: HashMap<String, Vec<&EnemyEncounter>> = HashMap::new();
        for enc in encounters {
            by_file.entry(enc.filter.clone()).or_default().push(enc);
        }

        // Process each file
        for (filename, file_encounters) in &by_file {
            // Sort encounters by ID to ensure correct order
            let mut sorted_encounters: Vec<EnemyEncounter> =
                file_encounters.iter().map(|e| (*e).clone()).collect();
            sorted_encounters.sort_by_key(|e| e.id);

            // Find the ENP file in the ISO
            let pattern = filename.replace(".enp", "");
            let matching_files = self.iso.list_files_matching(&pattern)?;

            for entry in matching_files {
                // Check if this is the right file
                let entry_filename = entry
                    .path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                if entry_filename != *filename {
                    continue;
                }

                // Read the original file
                let raw_data = self.iso.read_file_direct(&entry)?;
                let was_compressed = is_aklz(&raw_data);
                let data = decompress_aklz(&raw_data)?;

                // Patch encounters
                let patched = patch_enp_encounters(&data, &sorted_encounters);

                // Re-compress if original was compressed
                let output = if was_compressed {
                    compress_aklz(&patched)
                } else {
                    patched
                };

                self.iso.write_file(&entry.path, &output)?;
            }
        }

        Ok(())
    }

    /// Build a GlobalEnemyDatabase from all ENP files in the game.
    /// This stores ALL enemy variants (multiple entries per name with different stats).
    /// Use this as a fallback when an enemy isn't found in a file-specific database.
    pub fn build_global_enemy_database(&mut self) -> Result<crate::io::GlobalEnemyDatabase> {
        use crate::io::{decompress_aklz, GlobalEnemyDatabase};
        use crate::lookups::enemy_names_map;

        let mut db = GlobalEnemyDatabase::new();
        let enemy_names = enemy_names_map();

        // Read ENP files (*_ep.enp) - field encounters
        let enp_files = self.iso.list_files_matching("_ep.enp")?;

        for entry in &enp_files {
            let raw_data = self.iso.read_file_direct(entry)?;
            let data = decompress_aklz(&raw_data)?;

            // Parse header to find enemy positions
            if data.len() < 8 {
                continue;
            }

            // Check for multi-segment signature
            if data[0..4] == [0x00, 0x00, 0xff, 0xff] {
                continue;
            }

            const MAX_ENEMIES: usize = 84;
            let mut enemies: Vec<(u32, String, usize)> = Vec::new();

            // Read header entries
            for i in 0..MAX_ENEMIES {
                let offset = i * 8;
                if offset + 8 > data.len() {
                    break;
                }

                let enemy_id = i32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                let position = i32::from_be_bytes([
                    data[offset + 4],
                    data[offset + 5],
                    data[offset + 6],
                    data[offset + 7],
                ]);

                if enemy_id >= 0 && position > 0 && (position as usize) < data.len() {
                    let name = enemy_names
                        .get(&(enemy_id as u32))
                        .cloned()
                        .unwrap_or_else(|| format!("Enemy_{}", enemy_id));
                    enemies.push((enemy_id as u32, name, position as usize));
                }
            }

            // Sort by position to find boundaries
            enemies.sort_by_key(|(_, _, pos)| *pos);

            // Extract raw enemy data - add ALL variants to global database
            for i in 0..enemies.len() {
                let (id, name, pos) = &enemies[i];
                let end = if i + 1 < enemies.len() {
                    enemies[i + 1].2
                } else {
                    data.len()
                };

                if *pos < data.len() && end <= data.len() {
                    let raw = data[*pos..end].to_vec();
                    db.add(name.clone(), *id, raw);
                }
            }
        }

        Ok(db)
    }

    /// Build an EnemyDatabase from a specific ENP file.
    /// This extracts raw enemy data from that file for use in rebuilding it.
    pub fn build_enemy_database_for_file(
        &mut self,
        filename: &str,
    ) -> Result<crate::io::EnemyDatabase> {
        use crate::io::{decompress_aklz, EnemyDatabase};
        use crate::lookups::enemy_names_map;

        let mut db = EnemyDatabase::new();
        let enemy_names = enemy_names_map();

        // Find and read the specific ENP file
        let matching = self.iso.list_files_matching(filename)?;

        for entry in &matching {
            let entry_name = entry
                .path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            if entry_name != filename {
                continue;
            }

            let raw_data = self.iso.read_file_direct(entry)?;
            let data = decompress_aklz(&raw_data)?;

            // Parse header to find enemy positions
            if data.len() < 8 {
                continue;
            }

            // Check for multi-segment signature
            if data[0..4] == [0x00, 0x00, 0xff, 0xff] {
                continue;
            }

            const MAX_ENEMIES: usize = 84;
            let mut enemies: Vec<(u32, String, usize)> = Vec::new();

            // Read header entries
            for i in 0..MAX_ENEMIES {
                let offset = i * 8;
                if offset + 8 > data.len() {
                    break;
                }

                let enemy_id = i32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                let position = i32::from_be_bytes([
                    data[offset + 4],
                    data[offset + 5],
                    data[offset + 6],
                    data[offset + 7],
                ]);

                if enemy_id >= 0 && position > 0 && (position as usize) < data.len() {
                    let name = enemy_names
                        .get(&(enemy_id as u32))
                        .cloned()
                        .unwrap_or_else(|| format!("Enemy_{}", enemy_id));
                    enemies.push((enemy_id as u32, name, position as usize));
                }
            }

            // Sort by position to find boundaries
            enemies.sort_by_key(|(_, _, pos)| *pos);

            // Extract raw enemy data
            for i in 0..enemies.len() {
                let (id, name, pos) = &enemies[i];
                let end = if i + 1 < enemies.len() {
                    enemies[i + 1].2
                } else {
                    data.len()
                };

                if *pos < data.len() && end <= data.len() {
                    let raw = data[*pos..end].to_vec();
                    db.add(name.clone(), *id, raw);
                }
            }

            return Ok(db);
        }

        Err(Error::FileNotFound {
            path: std::path::PathBuf::from(filename),
        })
    }

    /// Write an ENP file back to the ISO.
    /// Compresses with AKLZ if the original was compressed.
    pub fn write_enp_file(&mut self, filename: &str, data: &[u8]) -> Result<()> {
        use crate::io::{compress_aklz, is_aklz};

        // Find the file
        let matching = self.iso.list_files_matching(filename)?;

        for entry in &matching {
            let entry_name = entry
                .path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            if entry_name == filename {
                // Check if original was compressed
                let raw_data = self.iso.read_file_direct(entry)?;
                let was_compressed = is_aklz(&raw_data);

                // Compress if original was compressed
                let output = if was_compressed {
                    compress_aklz(data)
                } else {
                    data.to_vec()
                };

                self.iso.write_file(&entry.path, &output)?;
                return Ok(());
            }
        }

        Err(Error::FileNotFound {
            path: std::path::PathBuf::from(filename),
        })
    }

    /// Read the raw (potentially compressed) bytes of an ENP file from the ISO.
    pub fn read_enp_file_raw(&mut self, filename: &str) -> Result<Vec<u8>> {
        // Find the file
        let matching = self.iso.list_files_matching(filename)?;

        for entry in &matching {
            let entry_name = entry
                .path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            if entry_name == filename {
                return self.iso.read_file_direct(entry);
            }
        }

        Err(Error::FileNotFound {
            path: std::path::PathBuf::from(filename),
        })
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here
}
