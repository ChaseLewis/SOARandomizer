//! CSV import functionality for reading data back from CSVs.

use std::io::Read;

use crate::entries::{
    Accessory, Armor, Weapon, CharacterSuperMove, Trait, CharacterFlags,
    UsableItem, SpecialItem, Character, CharacterMagic,
    Shop, TreasureChest, CrewMember, PlayableShip,
    ShipCannon, ShipAccessory, ShipItem, EnemyShip,
    EnemyMagic, EnemySuperMove, Swashbuckler, SpiritCurve, ExpBoost,
    SpiritLevel, OccasionFlags,
};
use crate::error::{Error, Result};

/// CSV importer for game data.
pub struct CsvImporter;

/// Validation result for an imported entry.
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self { valid: true, errors: Vec::new() }
    }
    
    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.valid = false;
        self.errors.push(msg.into());
    }
}

/// Helper to parse a value from CSV, with a default on failure.
fn parse_or_default<T: std::str::FromStr + Default>(s: &str) -> T {
    s.trim().parse().unwrap_or_default()
}

/// Helper to parse a hex value (with or without 0x prefix).
fn parse_hex(s: &str) -> u32 {
    let s = s.trim();
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    u32::from_str_radix(s, 16).unwrap_or(0)
}

/// Helper to parse a binary string (e.g., "0b111111").
fn parse_binary(s: &str) -> u8 {
    let s = s.trim();
    let s = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")).unwrap_or(s);
    u8::from_str_radix(s, 2).unwrap_or(0)
}

impl CsvImporter {
    /// Import accessories from CSV.
    pub fn import_accessories<R: Read>(reader: R) -> Result<Vec<Accessory>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut accessories = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let char_flags = parse_binary(record.get(2).unwrap_or("0"));
            let sell_percent: i8 = parse_or_default(record.get(9).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(10).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(11).unwrap_or("0"));
            let buy_price: u16 = parse_or_default(record.get(13).unwrap_or("0"));
            
            // Parse traits (4 traits, each with id, name, pad, value)
            let mut traits = [Trait::default(); 4];
            for i in 0..4 {
                let base = 14 + i * 4;
                traits[i].id = parse_or_default(record.get(base).unwrap_or("-1"));
                traits[i].value = parse_or_default(record.get(base + 3).unwrap_or("0"));
            }
            
            let desc_pos = parse_hex(record.get(30).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(31).unwrap_or("0"));
            let description = record.get(32).unwrap_or("").to_string();
            
            let accessory = Accessory {
                id,
                name,
                character_flags: CharacterFlags(char_flags),
                sell_percent,
                order1,
                order2,
                buy_price,
                traits,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            // Validate
            let validation = Self::validate_accessory(&accessory);
            if !validation.valid {
                return Err(Error::ValidationError(format!(
                    "Accessory ID {} validation failed: {}", 
                    id, 
                    validation.errors.join(", ")
                )));
            }
            
            accessories.push(accessory);
        }
        
        Ok(accessories)
    }
    
    /// Validate an accessory entry.
    fn validate_accessory(acc: &Accessory) -> ValidationResult {
        let mut result = ValidationResult::ok();
        
        // Name must fit in 17 bytes when encoded
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(&acc.name);
        if encoded.len() > 17 {
            result.add_error(format!("Name too long: {} bytes (max 17)", encoded.len()));
        }
        
        // Trait IDs are valid from -1 (none) to ~100 (game has many traits)
        for (i, t) in acc.traits.iter().enumerate() {
            if t.id < -1 {
                result.add_error(format!("Trait {} ID out of range: {}", i + 1, t.id));
            }
        }
        
        result
    }
    
    /// Import armors from CSV.
    pub fn import_armors<R: Read>(reader: R) -> Result<Vec<Armor>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut armors = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let char_flags = parse_binary(record.get(2).unwrap_or("0"));
            let sell_percent: i8 = parse_or_default(record.get(9).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(10).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(11).unwrap_or("0"));
            let buy_price: u16 = parse_or_default(record.get(13).unwrap_or("0"));
            
            let mut traits = [Trait::default(); 4];
            for i in 0..4 {
                let base = 14 + i * 4;
                traits[i].id = parse_or_default(record.get(base).unwrap_or("-1"));
                traits[i].value = parse_or_default(record.get(base + 3).unwrap_or("0"));
            }
            
            let desc_pos = parse_hex(record.get(30).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(31).unwrap_or("0"));
            let description = record.get(32).unwrap_or("").to_string();
            
            let armor = Armor {
                id,
                name,
                character_flags: CharacterFlags(char_flags),
                sell_percent,
                order1,
                order2,
                buy_price,
                traits,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            let validation = Self::validate_armor(&armor);
            if !validation.valid {
                return Err(Error::ValidationError(format!(
                    "Armor ID {} validation failed: {}", 
                    id, 
                    validation.errors.join(", ")
                )));
            }
            
            armors.push(armor);
        }
        
        Ok(armors)
    }
    
    fn validate_armor(armor: &Armor) -> ValidationResult {
        let mut result = ValidationResult::ok();
        
        if armor.name.len() > 17 {
            result.add_error(format!("Name too long: {} chars (max 17)", armor.name.len()));
        }
        
        if armor.buy_price > 65000 {
            result.add_error(format!("Buy price too high: {}", armor.buy_price));
        }
        
        result
    }
    
    /// Import weapons from CSV.
    pub fn import_weapons<R: Read>(reader: R) -> Result<Vec<Weapon>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut weapons = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let character_id: i8 = parse_or_default(record.get(2).unwrap_or("-1"));
            let sell_percent: i8 = parse_or_default(record.get(4).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(5).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(6).unwrap_or("0"));
            let effect_id: i8 = parse_or_default(record.get(7).unwrap_or("-1"));
            let buy_price: u16 = parse_or_default(record.get(9).unwrap_or("0"));
            let attack: i16 = parse_or_default(record.get(10).unwrap_or("0"));
            let hit_percent: i16 = parse_or_default(record.get(11).unwrap_or("0"));
            
            let trait_id: i8 = parse_or_default(record.get(12).unwrap_or("-1"));
            let trait_value: i16 = parse_or_default(record.get(15).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(16).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(17).unwrap_or("0"));
            let description = record.get(18).unwrap_or("").to_string();
            
            let weapon = Weapon {
                id,
                name,
                character_id,
                sell_percent,
                order1,
                order2,
                effect_id,
                buy_price,
                attack,
                hit_percent,
                trait_data: Trait { id: trait_id, value: trait_value },
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            let validation = Self::validate_weapon(&weapon);
            if !validation.valid {
                return Err(Error::ValidationError(format!(
                    "Weapon ID {} validation failed: {}", 
                    id, 
                    validation.errors.join(", ")
                )));
            }
            
            weapons.push(weapon);
        }
        
        Ok(weapons)
    }
    
    fn validate_weapon(weapon: &Weapon) -> ValidationResult {
        let mut result = ValidationResult::ok();
        
        if weapon.name.len() > 17 {
            result.add_error(format!("Name too long: {} chars (max 17)", weapon.name.len()));
        }
        
        if weapon.character_id < -1 || weapon.character_id > 5 {
            result.add_error(format!("Invalid character ID: {}", weapon.character_id));
        }
        
        result
    }
    
    /// Import usable items from CSV.
    pub fn import_usable_items<R: Read>(reader: R) -> Result<Vec<UsableItem>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut items = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let occasion_flags = parse_binary(record.get(2).unwrap_or("0"));
            let effect_id: i8 = parse_or_default(record.get(4).unwrap_or("-1"));
            let scope_id: u8 = parse_or_default(record.get(6).unwrap_or("0"));
            let consume_percent: i8 = parse_or_default(record.get(8).unwrap_or("100"));
            let sell_percent: i8 = parse_or_default(record.get(9).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(10).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(11).unwrap_or("0"));
            let buy_price: u16 = parse_or_default(record.get(12).unwrap_or("0"));
            let effect_base: i16 = parse_or_default(record.get(15).unwrap_or("0"));
            let element_id: i8 = parse_or_default(record.get(16).unwrap_or("-1"));
            let type_id: i8 = parse_or_default(record.get(18).unwrap_or("0"));
            let state_id: i16 = parse_or_default(record.get(20).unwrap_or("0"));
            let state_miss: i16 = parse_or_default(record.get(22).unwrap_or("0"));
            
            // Description fields at end
            let desc_pos = parse_hex(record.get(23).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(24).unwrap_or("0"));
            let description = record.get(25).unwrap_or("").to_string();
            
            let item = UsableItem {
                id,
                name,
                occasion_flags: OccasionFlags(occasion_flags),
                effect_id,
                scope_id,
                consume_percent,
                sell_percent,
                order1,
                order2,
                buy_price,
                effect_base,
                element_id,
                type_id,
                state_id,
                state_miss,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            items.push(item);
        }
        
        Ok(items)
    }
    
    /// Import special items from CSV.
    pub fn import_special_items<R: Read>(reader: R) -> Result<Vec<SpecialItem>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut items = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let sell_percent: i8 = parse_or_default(record.get(2).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(3).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(4).unwrap_or("0"));
            let buy_price: u16 = parse_or_default(record.get(6).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(8).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(9).unwrap_or("0"));
            let description = record.get(10).unwrap_or("").to_string();
            
            let item = SpecialItem {
                id,
                name,
                sell_percent,
                order1,
                order2,
                buy_price,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            items.push(item);
        }
        
        Ok(items)
    }
    
    /// Import characters from CSV.
    pub fn import_characters<R: Read>(reader: R) -> Result<Vec<Character>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut characters = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let age: i8 = parse_or_default(record.get(2).unwrap_or("0"));
            let gender_id: i8 = parse_or_default(record.get(3).unwrap_or("0"));
            let width: i8 = parse_or_default(record.get(5).unwrap_or("0"));
            let depth: i8 = parse_or_default(record.get(6).unwrap_or("0"));
            let max_mp: i8 = parse_or_default(record.get(7).unwrap_or("0"));
            let element_id: i8 = parse_or_default(record.get(8).unwrap_or("0"));
            let weapon_id: u16 = parse_or_default(record.get(11).unwrap_or("0"));
            let armor_id: u16 = parse_or_default(record.get(13).unwrap_or("0"));
            let accessory_id: u16 = parse_or_default(record.get(15).unwrap_or("0"));
            let movement_flags: i16 = parse_or_default(record.get(17).unwrap_or("0"));
            let hp: i16 = parse_or_default(record.get(24).unwrap_or("0"));
            let max_hp: i16 = parse_or_default(record.get(25).unwrap_or("0"));
            let max_hp_growth: i16 = parse_or_default(record.get(26).unwrap_or("0"));
            let sp: i16 = parse_or_default(record.get(27).unwrap_or("0"));
            let max_sp: i16 = parse_or_default(record.get(28).unwrap_or("0"));
            let counter_percent: i16 = parse_or_default(record.get(29).unwrap_or("0"));
            let exp: u32 = parse_or_default(record.get(31).unwrap_or("0"));
            let max_mp_growth: f32 = parse_or_default(record.get(32).unwrap_or("0"));
            let unknown1: f32 = parse_or_default(record.get(33).unwrap_or("0"));
            
            // Parse element resistances (6 values)
            let mut element_resistances = [0i16; 6];
            for i in 0..6 {
                element_resistances[i] = parse_or_default(record.get(34 + i).unwrap_or("0"));
            }
            
            // Parse state resistances (15 values)
            let mut state_resistances = [0i16; 15];
            for i in 0..15 {
                state_resistances[i] = parse_or_default(record.get(40 + i).unwrap_or("0"));
            }
            
            let danger: i16 = parse_or_default(record.get(55).unwrap_or("0"));
            let power: i16 = parse_or_default(record.get(56).unwrap_or("0"));
            let will: i16 = parse_or_default(record.get(57).unwrap_or("0"));
            let vigor: i16 = parse_or_default(record.get(58).unwrap_or("0"));
            let agile: i16 = parse_or_default(record.get(59).unwrap_or("0"));
            let quick: i16 = parse_or_default(record.get(60).unwrap_or("0"));
            
            let power_growth: f32 = parse_or_default(record.get(62).unwrap_or("0"));
            let will_growth: f32 = parse_or_default(record.get(63).unwrap_or("0"));
            let vigor_growth: f32 = parse_or_default(record.get(64).unwrap_or("0"));
            let agile_growth: f32 = parse_or_default(record.get(65).unwrap_or("0"));
            let quick_growth: f32 = parse_or_default(record.get(66).unwrap_or("0"));
            
            // Magic EXP (6 values)
            let mut magic_exp = [0i32; 6];
            for i in 0..6 {
                magic_exp[i] = parse_or_default(record.get(67 + i).unwrap_or("0"));
            }
            
            let character = Character {
                id,
                name,
                age,
                gender_id,
                width,
                depth,
                max_mp,
                element_id,
                weapon_id,
                armor_id,
                accessory_id,
                movement_flags,
                hp,
                max_hp,
                max_hp_growth,
                sp,
                max_sp,
                counter_percent,
                exp,
                max_mp_growth,
                unknown1,
                element_resistances,
                state_resistances,
                danger,
                power,
                will,
                vigor,
                agile,
                quick,
                power_growth,
                will_growth,
                vigor_growth,
                agile_growth,
                quick_growth,
                magic_exp,
            };
            
            characters.push(character);
        }
        
        Ok(characters)
    }
    
    /// Import character magic from CSV.
    pub fn import_character_magic<R: Read>(reader: R) -> Result<Vec<CharacterMagic>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut magic = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let element_id: i8 = parse_or_default(record.get(2).unwrap_or("0"));
            let order: i16 = parse_or_default(record.get(4).unwrap_or("0"));
            let occasion_flags: u8 = parse_binary(record.get(5).unwrap_or("0"));
            let effect_id: i8 = parse_or_default(record.get(7).unwrap_or("-1"));
            let scope_id: u8 = parse_or_default(record.get(9).unwrap_or("0"));
            let category_id: i8 = parse_or_default(record.get(11).unwrap_or("0"));
            let effect_speed: i8 = parse_or_default(record.get(13).unwrap_or("0"));
            let effect_sp: i8 = parse_or_default(record.get(14).unwrap_or("0"));
            let effect_base: i16 = parse_or_default(record.get(17).unwrap_or("0"));
            let type_id: i8 = parse_or_default(record.get(18).unwrap_or("0"));
            let state_id: i8 = parse_or_default(record.get(20).unwrap_or("0"));
            let state_miss: i8 = parse_or_default(record.get(22).unwrap_or("0"));
            let ship_occasion_id: i8 = parse_or_default(record.get(26).unwrap_or("0"));
            let ship_effect_id: i16 = parse_or_default(record.get(28).unwrap_or("0"));
            let ship_effect_sp: i8 = parse_or_default(record.get(30).unwrap_or("0"));
            let ship_effect_turns: i8 = parse_or_default(record.get(31).unwrap_or("0"));
            let ship_effect_base: i16 = parse_or_default(record.get(32).unwrap_or("0"));
            let unknown: i8 = parse_or_default(record.get(33).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(37).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(38).unwrap_or("0"));
            let description = record.get(39).unwrap_or("").to_string();
            
            let entry = CharacterMagic {
                id,
                name,
                element_id,
                order,
                occasion_flags,
                effect_id,
                scope_id,
                category_id,
                effect_speed,
                effect_sp,
                effect_base,
                type_id,
                state_id,
                state_miss,
                ship_occasion_id,
                ship_effect_id,
                ship_effect_sp,
                ship_effect_turns,
                ship_effect_base,
                unknown,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
                ship_description: String::new(),
                ship_description_pos: 0,
                ship_description_size: 0,
            };
            
            magic.push(entry);
        }
        
        Ok(magic)
    }
    
    /// Import character super moves from CSV.
    pub fn import_character_super_moves<R: Read>(reader: R) -> Result<Vec<CharacterSuperMove>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut moves = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let element_id: i8 = parse_or_default(record.get(2).unwrap_or("0"));
            let order: i16 = parse_or_default(record.get(4).unwrap_or("0"));
            let occasion_flags: u8 = parse_binary(record.get(5).unwrap_or("0"));
            let effect_id: i8 = parse_or_default(record.get(7).unwrap_or("-1"));
            let scope_id: u8 = parse_or_default(record.get(9).unwrap_or("0"));
            let category_id: i8 = parse_or_default(record.get(11).unwrap_or("0"));
            let effect_speed: i8 = parse_or_default(record.get(13).unwrap_or("0"));
            let effect_sp: i8 = parse_or_default(record.get(14).unwrap_or("0"));
            let effect_base: i16 = parse_or_default(record.get(17).unwrap_or("0"));
            let type_id: i8 = parse_or_default(record.get(18).unwrap_or("0"));
            let state_id: i8 = parse_or_default(record.get(20).unwrap_or("0"));
            let state_miss: i8 = parse_or_default(record.get(22).unwrap_or("0"));
            let ship_occasion_id: i8 = parse_or_default(record.get(26).unwrap_or("0"));
            let ship_effect_id: i16 = parse_or_default(record.get(28).unwrap_or("0"));
            let ship_effect_sp: i8 = parse_or_default(record.get(30).unwrap_or("0"));
            let ship_effect_turns: i8 = parse_or_default(record.get(31).unwrap_or("0"));
            let ship_effect_base: i16 = parse_or_default(record.get(32).unwrap_or("0"));
            let unknown: i8 = parse_or_default(record.get(33).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(37).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(38).unwrap_or("0"));
            let description = record.get(39).unwrap_or("").to_string();
            
            let entry = CharacterSuperMove {
                id,
                name,
                element_id,
                order,
                occasion_flags,
                effect_id,
                scope_id,
                category_id,
                effect_speed,
                effect_sp,
                effect_base,
                type_id,
                state_id,
                state_miss,
                ship_occasion_id,
                ship_effect_id,
                ship_effect_sp,
                ship_effect_turns,
                ship_effect_base,
                unknown,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            moves.push(entry);
        }
        
        Ok(moves)
    }
    
    /// Import shops from CSV.
    pub fn import_shops<R: Read>(reader: R) -> Result<Vec<Shop>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut shops = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u16 = parse_or_default(record.get(0).unwrap_or("0"));
            let sot_pos = parse_hex(record.get(2).unwrap_or("0"));
            
            let mut item_ids = Vec::with_capacity(48);
            for i in 0..48 {
                let col = 3 + i * 2;
                if let Some(val) = record.get(col) {
                    let id: i16 = parse_or_default(val);
                    item_ids.push(id);
                } else {
                    item_ids.push(-1);
                }
            }
            
            let desc_pos = parse_hex(record.get(67).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(68).unwrap_or("0"));
            let description = record.get(69).unwrap_or("").to_string();
            
            let shop = Shop {
                id,
                sot_pos,
                item_ids,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            shops.push(shop);
        }
        
        Ok(shops)
    }
    
    /// Import treasure chests from CSV.
    pub fn import_treasure_chests<R: Read>(reader: R) -> Result<Vec<TreasureChest>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut chests = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let item_id: i32 = parse_or_default(record.get(1).unwrap_or("0"));
            let item_amount: i32 = parse_or_default(record.get(3).unwrap_or("0"));
            
            let chest = TreasureChest {
                id,
                item_id,
                item_amount,
            };
            
            chests.push(chest);
        }
        
        Ok(chests)
    }
    
    /// Import crew members from CSV.
    pub fn import_crew_members<R: Read>(reader: R) -> Result<Vec<CrewMember>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut members = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let position_id: i8 = parse_or_default(record.get(2).unwrap_or("0"));
            let trait_id: i8 = parse_or_default(record.get(5).unwrap_or("-1"));
            let trait_value: i16 = parse_or_default(record.get(8).unwrap_or("0"));
            let ship_effect_id: i8 = parse_or_default(record.get(9).unwrap_or("0"));
            let ship_effect_sp: i8 = parse_or_default(record.get(11).unwrap_or("0"));
            let ship_effect_turns: i8 = parse_or_default(record.get(12).unwrap_or("0"));
            let ship_effect_base: i16 = parse_or_default(record.get(16).unwrap_or("0"));
            let unknown: i16 = parse_or_default(record.get(17).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(22).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(23).unwrap_or("0"));
            let description = record.get(24).unwrap_or("").to_string();
            
            let member = CrewMember {
                id,
                name,
                position_id,
                trait_id,
                trait_value,
                ship_effect_id,
                ship_effect_sp,
                ship_effect_turns,
                ship_effect_base,
                unknown,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            members.push(member);
        }
        
        Ok(members)
    }
    
    /// Import playable ships from CSV.
    pub fn import_playable_ships<R: Read>(reader: R) -> Result<Vec<PlayableShip>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut ships = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let max_hp: u32 = parse_or_default(record.get(2).unwrap_or("0"));
            let max_sp: i16 = parse_or_default(record.get(3).unwrap_or("0"));
            let sp: i16 = parse_or_default(record.get(4).unwrap_or("0"));
            let defense: i16 = parse_or_default(record.get(5).unwrap_or("0"));
            let mag_def: i16 = parse_or_default(record.get(6).unwrap_or("0"));
            let quick: i16 = parse_or_default(record.get(7).unwrap_or("0"));
            let dodge: i16 = parse_or_default(record.get(8).unwrap_or("0"));
            
            let mut elements = [0i16; 6];
            for i in 0..6 {
                elements[i] = parse_or_default(record.get(9 + i).unwrap_or("0"));
            }
            
            let mut cannon_ids = [0i16; 5];
            for i in 0..5 {
                cannon_ids[i] = parse_or_default(record.get(15 + i * 2).unwrap_or("0"));
            }
            
            let mut accessory_ids = [0i16; 3];
            for i in 0..3 {
                accessory_ids[i] = parse_or_default(record.get(23 + i * 2).unwrap_or("0"));
            }
            
            let value: u32 = parse_or_default(record.get(29).unwrap_or("0"));
            
            let max_hp_growth: i32 = parse_or_default(record.get(32).unwrap_or("0"));
            let max_sp_growth: i16 = parse_or_default(record.get(33).unwrap_or("0"));
            let sp_growth: i16 = parse_or_default(record.get(34).unwrap_or("0"));
            let defense_growth: i16 = parse_or_default(record.get(35).unwrap_or("0"));
            let mag_def_growth: i16 = parse_or_default(record.get(36).unwrap_or("0"));
            let quick_growth: i16 = parse_or_default(record.get(37).unwrap_or("0"));
            let dodge_growth: i16 = parse_or_default(record.get(38).unwrap_or("0"));
            
            let ship = PlayableShip {
                id,
                name,
                max_hp,
                max_sp,
                sp,
                defense,
                mag_def,
                quick,
                dodge,
                elements,
                cannon_ids,
                accessory_ids,
                value,
                max_hp_growth,
                max_sp_growth,
                sp_growth,
                defense_growth,
                mag_def_growth,
                quick_growth,
                dodge_growth,
            };
            
            ships.push(ship);
        }
        
        Ok(ships)
    }
    
    /// Import ship cannons from CSV.
    pub fn import_ship_cannons<R: Read>(reader: R) -> Result<Vec<ShipCannon>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut cannons = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let ship_flags: u8 = parse_binary(record.get(2).unwrap_or("0"));
            let type_id: i8 = parse_or_default(record.get(8).unwrap_or("0"));
            let element_id: i8 = parse_or_default(record.get(10).unwrap_or("-1"));
            let attack: i16 = parse_or_default(record.get(12).unwrap_or("0"));
            let hit: u16 = parse_or_default(record.get(13).unwrap_or("0"));
            let limit: i8 = parse_or_default(record.get(14).unwrap_or("0"));
            let sp: i8 = parse_or_default(record.get(15).unwrap_or("0"));
            let trait_id: i8 = parse_or_default(record.get(16).unwrap_or("-1"));
            let trait_value: i16 = parse_or_default(record.get(19).unwrap_or("0"));
            let buy_price: u16 = parse_or_default(record.get(20).unwrap_or("0"));
            let sell_percent: i8 = parse_or_default(record.get(21).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(22).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(23).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(25).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(26).unwrap_or("0"));
            let description = record.get(27).unwrap_or("").to_string();
            
            let cannon = ShipCannon {
                id,
                name,
                ship_flags,
                type_id,
                element_id,
                attack,
                hit,
                limit,
                sp,
                trait_id,
                trait_value,
                buy_price,
                sell_percent,
                order1,
                order2,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            cannons.push(cannon);
        }
        
        Ok(cannons)
    }
    
    /// Import ship accessories from CSV.
    pub fn import_ship_accessories<R: Read>(reader: R) -> Result<Vec<ShipAccessory>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut accessories = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let ship_flags: u8 = parse_binary(record.get(2).unwrap_or("0"));
            
            let mut traits = [Trait::default(); 4];
            for i in 0..4 {
                let base = 8 + i * 4;
                traits[i].id = parse_or_default(record.get(base).unwrap_or("-1"));
                traits[i].value = parse_or_default(record.get(base + 3).unwrap_or("0"));
            }
            
            let buy_price: u16 = parse_or_default(record.get(24).unwrap_or("0"));
            let sell_percent: i8 = parse_or_default(record.get(25).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(26).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(27).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(29).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(30).unwrap_or("0"));
            let description = record.get(31).unwrap_or("").to_string();
            
            let accessory = ShipAccessory {
                id,
                name,
                ship_flags,
                traits,
                buy_price,
                sell_percent,
                order1,
                order2,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            accessories.push(accessory);
        }
        
        Ok(accessories)
    }
    
    /// Import ship items from CSV.
    pub fn import_ship_items<R: Read>(reader: R) -> Result<Vec<ShipItem>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut items = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let occasion_flags: u8 = parse_binary(record.get(2).unwrap_or("0"));
            let ship_effect_id: i8 = parse_or_default(record.get(4).unwrap_or("0"));
            let ship_effect_turns: i8 = parse_or_default(record.get(6).unwrap_or("0"));
            let consume: i8 = parse_or_default(record.get(7).unwrap_or("0"));
            let buy_price: u16 = parse_or_default(record.get(9).unwrap_or("0"));
            let sell_percent: i8 = parse_or_default(record.get(10).unwrap_or("0"));
            let order1: i8 = parse_or_default(record.get(11).unwrap_or("0"));
            let order2: i8 = parse_or_default(record.get(12).unwrap_or("0"));
            let ship_effect_base: i16 = parse_or_default(record.get(15).unwrap_or("0"));
            let element_id: i8 = parse_or_default(record.get(16).unwrap_or("-1"));
            let unknown1: i8 = parse_or_default(record.get(18).unwrap_or("0"));
            let unknown2: i16 = parse_or_default(record.get(19).unwrap_or("0"));
            let hit: i16 = parse_or_default(record.get(20).unwrap_or("0"));
            
            let desc_pos = parse_hex(record.get(21).unwrap_or("0"));
            let desc_size: u32 = parse_or_default(record.get(22).unwrap_or("0"));
            let description = record.get(23).unwrap_or("").to_string();
            
            let item = ShipItem {
                id,
                name,
                occasion_flags,
                ship_effect_id,
                ship_effect_turns,
                consume,
                buy_price,
                sell_percent,
                order1,
                order2,
                ship_effect_base,
                element_id,
                unknown1,
                unknown2,
                hit,
                description_pos: desc_pos,
                description_size: desc_size,
                description,
            };
            
            items.push(item);
        }
        
        Ok(items)
    }
    
    /// Import enemy ships from CSV.
    pub fn import_enemy_ships<R: Read>(reader: R) -> Result<Vec<EnemyShip>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut ships = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let max_hp: i32 = parse_or_default(record.get(2).unwrap_or("0"));
            let will: i16 = parse_or_default(record.get(3).unwrap_or("0"));
            let defense: i16 = parse_or_default(record.get(4).unwrap_or("0"));
            let mag_def: i16 = parse_or_default(record.get(5).unwrap_or("0"));
            let quick: i16 = parse_or_default(record.get(6).unwrap_or("0"));
            let agile: i16 = parse_or_default(record.get(7).unwrap_or("0"));
            let dodge: i16 = parse_or_default(record.get(8).unwrap_or("0"));
            
            let mut elements = [0i16; 6];
            for i in 0..6 {
                elements[i] = parse_or_default(record.get(9 + i).unwrap_or("0"));
            }
            
            // Skip armaments for now - complex nested structure
            // Parse exp, gold
            let exp: i32 = parse_or_default(record.get(45).unwrap_or("0"));
            let gold: i32 = parse_or_default(record.get(46).unwrap_or("0"));
            
            let ship = EnemyShip {
                id,
                name,
                max_hp,
                will,
                defense,
                mag_def,
                quick,
                agile,
                dodge,
                elements,
                armaments: Default::default(), // Will use existing data
                exp,
                gold,
                item_drops: Default::default(), // Will use existing data
            };
            
            ships.push(ship);
        }
        
        Ok(ships)
    }
    
    /// Import enemy magic from CSV.
    pub fn import_enemy_magic<R: Read>(reader: R) -> Result<Vec<EnemyMagic>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut magic = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let category_id: i8 = parse_or_default(record.get(2).unwrap_or("0"));
            let effect_id: i8 = parse_or_default(record.get(4).unwrap_or("-1"));
            let scope_id: u8 = parse_or_default(record.get(6).unwrap_or("0"));
            let effect_param_id: u16 = parse_or_default(record.get(8).unwrap_or("0"));
            let effect_base: u16 = parse_or_default(record.get(9).unwrap_or("0"));
            let element_id: i8 = parse_or_default(record.get(10).unwrap_or("-1"));
            let type_id: i8 = parse_or_default(record.get(12).unwrap_or("0"));
            let state_infliction_id: i8 = parse_or_default(record.get(14).unwrap_or("0"));
            let state_resistance_id: i8 = parse_or_default(record.get(16).unwrap_or("0"));
            let state_id: i8 = parse_or_default(record.get(18).unwrap_or("0"));
            let state_miss: i8 = parse_or_default(record.get(20).unwrap_or("0"));
            
            let entry = EnemyMagic {
                id,
                name,
                category_id,
                effect_id,
                scope_id,
                effect_param_id,
                effect_base,
                element_id,
                type_id,
                state_infliction_id,
                state_resistance_id,
                state_id,
                state_miss,
            };
            
            magic.push(entry);
        }
        
        Ok(magic)
    }
    
    /// Import enemy super moves from CSV.
    pub fn import_enemy_super_moves<R: Read>(reader: R) -> Result<Vec<EnemySuperMove>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut moves = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let category_id: i8 = parse_or_default(record.get(2).unwrap_or("0"));
            let effect_id: i8 = parse_or_default(record.get(4).unwrap_or("-1"));
            let scope_id: u8 = parse_or_default(record.get(6).unwrap_or("0"));
            let effect_param_id: u16 = parse_or_default(record.get(8).unwrap_or("0"));
            let effect_base: u16 = parse_or_default(record.get(9).unwrap_or("0"));
            let element_id: i8 = parse_or_default(record.get(10).unwrap_or("-1"));
            let type_id: i8 = parse_or_default(record.get(12).unwrap_or("0"));
            let state_infliction_id: i8 = parse_or_default(record.get(14).unwrap_or("0"));
            let state_resistance_id: i8 = parse_or_default(record.get(16).unwrap_or("0"));
            let state_id: i8 = parse_or_default(record.get(18).unwrap_or("0"));
            let state_miss: i8 = parse_or_default(record.get(20).unwrap_or("0"));
            
            let entry = EnemySuperMove {
                id,
                name,
                category_id,
                effect_id,
                scope_id,
                effect_param_id,
                effect_base,
                element_id,
                type_id,
                state_infliction_id,
                state_resistance_id,
                state_id,
                state_miss,
            };
            
            moves.push(entry);
        }
        
        Ok(moves)
    }
    
    /// Import swashbucklers from CSV.
    pub fn import_swashbucklers<R: Read>(reader: R) -> Result<Vec<Swashbuckler>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut swashbucklers = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let name = record.get(1).unwrap_or("").to_string();
            let rating: u8 = parse_or_default(record.get(2).unwrap_or("0"));
            let regular_attack: i16 = parse_or_default(record.get(3).unwrap_or("0"));
            let super_move_attack: i16 = parse_or_default(record.get(4).unwrap_or("0"));
            let dodge: i16 = parse_or_default(record.get(5).unwrap_or("0"));
            let run: i16 = parse_or_default(record.get(6).unwrap_or("0"));
            
            let entry = Swashbuckler {
                id,
                name,
                rating,
                regular_attack,
                super_move_attack,
                dodge,
                run,
            };
            
            swashbucklers.push(entry);
        }
        
        Ok(swashbucklers)
    }
    
    /// Import spirit curves from CSV.
    pub fn import_spirit_curves<R: Read>(reader: R) -> Result<Vec<SpiritCurve>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut curves = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            // Character name is at column 1
            let character_name = record.get(1).unwrap_or("").to_string();
            
            // Levels start at column 2
            let mut levels = Vec::with_capacity(99);
            for i in 0..99 {
                let base = 2 + i * 2;
                let sp = parse_or_default(record.get(base).unwrap_or("0"));
                let max_sp = parse_or_default(record.get(base + 1).unwrap_or("0"));
                levels.push(SpiritLevel { sp, max_sp });
            }
            
            let entry = SpiritCurve {
                id,
                character_name,
                levels,
            };
            
            curves.push(entry);
        }
        
        Ok(curves)
    }
    
    /// Import exp boosts from CSV.
    pub fn import_exp_boosts<R: Read>(reader: R) -> Result<Vec<ExpBoost>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut boosts = Vec::new();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            let character_name = record.get(1).unwrap_or("").to_string();
            let exp: u32 = parse_or_default(record.get(2).unwrap_or("0"));
            let green_exp: u32 = parse_or_default(record.get(3).unwrap_or("0"));
            let red_exp: u32 = parse_or_default(record.get(4).unwrap_or("0"));
            let purple_exp: u32 = parse_or_default(record.get(5).unwrap_or("0"));
            let blue_exp: u32 = parse_or_default(record.get(6).unwrap_or("0"));
            let yellow_exp: u32 = parse_or_default(record.get(7).unwrap_or("0"));
            let silver_exp: u32 = parse_or_default(record.get(8).unwrap_or("0"));
            
            let entry = ExpBoost {
                id,
                character_name,
                exp,
                green_exp,
                red_exp,
                purple_exp,
                blue_exp,
                yellow_exp,
                silver_exp,
            };
            
            boosts.push(entry);
        }
        
        Ok(boosts)
    }
}

