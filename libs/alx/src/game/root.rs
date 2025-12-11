//! Game root - main entry point for interacting with a game ISO.

use std::path::Path;

use super::offsets::Offsets;
use super::region::GameVersion;
use crate::entries::{Accessory, Armor, Weapon, WeaponEffect, UsableItem, SpecialItem, Character, CharacterMagic, CharacterSuperMove, CrewMember, PlayableShip, ShipCannon, ShipAccessory, ShipItem, EnemyShip, EnemyMagic, EnemySuperMove, Enemy, EnemyTask, Swashbuckler, SpiritCurve, ExpBoost, Shop, TreasureChest};
use crate::io::{parse_enp, parse_evp, parse_dat_file, decompress_aklz};
use crate::items::ItemDatabase;
use crate::error::{Error, Result};
use crate::io::{IsoFile, read_description_strings};

/// Main interface for working with a Skies of Arcadia Legends ISO.
pub struct GameRoot {
    iso: IsoFile,
    version: GameVersion,
    offsets: Offsets,
    /// Cached Start.dol data
    dol_data: Option<Vec<u8>>,
}

impl GameRoot {
    /// Open a game ISO and detect its version.
    pub fn open(path: &Path) -> Result<Self> {
        let mut iso = IsoFile::open(path)?;
        
        // Read game ID to detect version
        let game_id = iso.read_game_id()?;
        let version = GameVersion::from_game_id(&game_id)
            .ok_or_else(|| Error::InvalidIso(format!(
                "Not a Skies of Arcadia Legends ISO: {}",
                game_id
            )))?;
        
        let offsets = Offsets::for_version(&version)?;
        
        Ok(Self {
            iso,
            version,
            offsets,
            dol_data: None,
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
    pub fn list_iso_files_matching(&mut self, pattern: &str) -> Result<Vec<crate::io::IsoFileEntry>> {
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
                    range.start, range.end, dol.len()
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
                message: format!("Range {:x}..{:x} exceeds DOL size {:x}", range.start, range.end, dol.len()),
            });
        }
        if range.len() != data.len() {
            return Err(Error::ValidationError(format!(
                "Data length {} does not match range length {}",
                data.len(), range.len()
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

    // ========================================================================
    // Write methods for each entry type
    // ========================================================================

    /// Write accessories to the DOL.
    pub fn write_accessories(&mut self, accessories: &[Accessory]) -> Result<()> {
        let data_range = self.offsets.accessory_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        Accessory::write_all_data(accessories, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write armors to the DOL.
    pub fn write_armors(&mut self, armors: &[Armor]) -> Result<()> {
        let data_range = self.offsets.armor_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        Armor::write_all_data(armors, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write weapons to the DOL.
    pub fn write_weapons(&mut self, weapons: &[Weapon]) -> Result<()> {
        let data_range = self.offsets.weapon_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        Weapon::write_all_data(weapons, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write usable items to the DOL.
    pub fn write_usable_items(&mut self, items: &[UsableItem]) -> Result<()> {
        let data_range = self.offsets.usable_item_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        UsableItem::write_all_data(items, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write special items to the DOL.
    pub fn write_special_items(&mut self, items: &[SpecialItem]) -> Result<()> {
        let data_range = self.offsets.special_item_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        SpecialItem::write_all_data(items, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write characters to the DOL.
    pub fn write_characters(&mut self, characters: &[Character]) -> Result<()> {
        let data_range = self.offsets.character_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        Character::write_all_data(characters, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write character magic to the DOL.
    pub fn write_character_magic(&mut self, magic: &[CharacterMagic]) -> Result<()> {
        let data_range = self.offsets.character_magic_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        CharacterMagic::write_all_data(magic, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write character super moves to the DOL.
    pub fn write_character_super_moves(&mut self, moves: &[CharacterSuperMove]) -> Result<()> {
        let data_range = self.offsets.character_super_move_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        CharacterSuperMove::write_all_data(moves, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write shops to the DOL.
    pub fn write_shops(&mut self, shops: &[Shop]) -> Result<()> {
        let data_range = self.offsets.shop_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        Shop::write_all_data(shops, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write treasure chests to the DOL.
    pub fn write_treasure_chests(&mut self, chests: &[TreasureChest]) -> Result<()> {
        let data_range = self.offsets.treasure_chest_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        TreasureChest::write_all_data(chests, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write crew members to the DOL.
    pub fn write_crew_members(&mut self, members: &[CrewMember]) -> Result<()> {
        let data_range = self.offsets.crew_member_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        CrewMember::write_all_data(members, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write playable ships to the DOL.
    pub fn write_playable_ships(&mut self, ships: &[PlayableShip]) -> Result<()> {
        let data_range = self.offsets.playable_ship_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        PlayableShip::write_all_data(ships, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write ship cannons to the DOL.
    pub fn write_ship_cannons(&mut self, cannons: &[ShipCannon]) -> Result<()> {
        let data_range = self.offsets.ship_cannon_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        ShipCannon::write_all_data(cannons, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write ship accessories to the DOL.
    pub fn write_ship_accessories(&mut self, accessories: &[ShipAccessory]) -> Result<()> {
        let data_range = self.offsets.ship_accessory_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        ShipAccessory::write_all_data(accessories, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write ship items to the DOL.
    pub fn write_ship_items(&mut self, items: &[ShipItem]) -> Result<()> {
        let data_range = self.offsets.ship_item_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        ShipItem::write_all_data(items, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write enemy ships to the DOL.
    pub fn write_enemy_ships(&mut self, ships: &[EnemyShip]) -> Result<()> {
        let data_range = self.offsets.enemy_ship_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        EnemyShip::write_all_data(ships, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write enemy magic to the DOL.
    pub fn write_enemy_magic(&mut self, magic: &[EnemyMagic]) -> Result<()> {
        let data_range = self.offsets.enemy_magic_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        EnemyMagic::write_all_data(magic, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write enemy super moves to the DOL.
    pub fn write_enemy_super_moves(&mut self, moves: &[EnemySuperMove]) -> Result<()> {
        let data_range = self.offsets.enemy_super_move_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        EnemySuperMove::write_all_data(moves, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write swashbucklers to the DOL.
    pub fn write_swashbucklers(&mut self, swashbucklers: &[Swashbuckler]) -> Result<()> {
        let data_range = self.offsets.swashbuckler_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        Swashbuckler::write_all_data(swashbucklers, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write spirit curves to the DOL.
    pub fn write_spirit_curves(&mut self, curves: &[SpiritCurve]) -> Result<()> {
        let data_range = self.offsets.spirit_curve_data.clone();
        let mut buffer = std::io::Cursor::new(Vec::new());
        SpiritCurve::write_all_data(curves, &mut buffer, &self.version)?;
        self.write_to_dol(data_range, buffer.get_ref())
    }

    /// Write exp boosts to the DOL.
    pub fn write_exp_boosts(&mut self, boosts: &[ExpBoost]) -> Result<()> {
        if let Some(data_range) = self.offsets.exp_boost_data.clone() {
            let mut buffer = std::io::Cursor::new(Vec::new());
            ExpBoost::write_all_data(boosts, &mut buffer, &self.version)?;
            self.write_to_dol(data_range, buffer.get_ref())
        } else {
            Ok(())
        }
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
                4  // 4-byte alignment for US/JP
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
            let descriptions = read_description_strings(
                &dscr_data, 
                dscr_range.start, 
                armors.len(), 
                4
            )?;
            
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
            let descriptions = read_description_strings(
                &dscr_data, 
                dscr_range.start, 
                weapons.len(), 
                4
            )?;
            
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
            let descriptions = read_description_strings(
                &dscr_data, 
                dscr_range.start, 
                items.len(), 
                4
            )?;
            
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
            let descriptions = read_description_strings(
                &dscr_data, 
                dscr_range.start, 
                items.len(), 
                4
            )?;
            
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
            let descriptions = read_description_strings(
                &dscr_data, 
                dscr_range.start, 
                magics.len(), 
                4
            )?;
            
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
            let descriptions = read_description_strings(
                &dscr_data, 
                dscr_range.start, 
                shops.len(), 
                4
            )?;
            
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
            let descriptions = read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;
            
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
            let descriptions = read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;
            
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
            let descriptions = read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;
            
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
            let descriptions = read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;
            
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
            let descriptions = read_description_strings(&dscr_data, dscr_range.start, entries.len(), 4)?;
            
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
                
                let filename = entry.path.file_name()
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
            
            let filename = entry.path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "*".to_string());
            
            let parsed = parse_enp(&data, &filename, &self.version)?;
            raw_enemies.extend(parsed.enemies);
            all_tasks.extend(parsed.tasks);
        }
        
        // 3. Read EC/EB DAT files - battle init enemies
        let ec_files = self.iso.list_files_matching("ecinit");
        let eb_files = self.iso.list_files_matching("ebinit");
        
        for files_result in [ec_files, eb_files] {
            if let Ok(files) = files_result {
                for entry in &files {
                    if entry.path.to_string_lossy().ends_with(".dat") {
                        let raw_data = self.iso.read_file_direct(entry)?;
                        let data = decompress_aklz(&raw_data)?;
                        
                        let filename = entry.path.file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "*".to_string());
                        
                        let parsed = parse_dat_file(&data, &filename, &self.version)?;
                        raw_enemies.extend(parsed.enemies);
                        all_tasks.extend(parsed.tasks);
                    }
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
                e.max_hp, e.exp, e.gold, e.attack, e.defense, e.mag_def,
                e.quick, e.agile, e.level, e.counter, e.danger,
                e.element_id, e.width, e.depth, e.will, e.vigor, e.hit, e.name_jp
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
                if cmp != std::cmp::Ordering::Equal { return cmp; }
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
}

#[cfg(test)]
mod tests {
    // Integration tests would go here
}

