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
    /// Import usable items from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Occasion Flags, 3-5: [M/B/S],
    /// 6: Effect ID, 7: [Effect Name], 8: Scope ID, 9: [Scope Name],
    /// 10: Element ID, 11: [Element Name], 12: Sell%, 13: US Order 1, 14: US Order 2,
    /// 15: Pad 1, 16: Buy, 17: Effect Base, 18: Type ID, 19: [Type Name],
    /// 20: State ID, 21: [State Name], 22: State Miss%, 23-25: Pads,
    /// 26: [US Descr Pos], 27: [US Descr Size], 28: US Descr Str
    pub fn import_usable_items<R: Read>(reader: R, existing: &[UsableItem]) -> Result<Vec<UsableItem>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut items: Vec<UsableItem> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                item.name = record.get(1).unwrap_or("").to_string();
                item.occasion_flags = OccasionFlags(parse_binary(record.get(2).unwrap_or("0")));
                // Skip [M], [B], [S] at 3-5
                item.effect_id = parse_or_default(record.get(6).unwrap_or("-1"));
                // Skip [Effect Name] at 7
                item.scope_id = parse_or_default(record.get(8).unwrap_or("0"));
                // Skip [Scope Name] at 9
                item.element_id = parse_or_default(record.get(10).unwrap_or("-1"));
                // Skip [Element Name] at 11
                item.sell_percent = parse_or_default(record.get(12).unwrap_or("0"));
                item.order1 = parse_or_default(record.get(13).unwrap_or("0"));
                item.order2 = parse_or_default(record.get(14).unwrap_or("0"));
                // Skip Pad 1 at 15
                item.buy_price = parse_or_default(record.get(16).unwrap_or("0"));
                item.effect_base = parse_or_default(record.get(17).unwrap_or("0"));
                item.type_id = parse_or_default(record.get(18).unwrap_or("0"));
                // Skip [Type Name] at 19
                item.state_id = parse_or_default(record.get(20).unwrap_or("0"));
                // Skip [State Name] at 21
                item.state_miss = parse_or_default(record.get(22).unwrap_or("0"));
                // Skip Pads at 23-25, description pos/size at 26-27
                item.description = record.get(28).unwrap_or("").to_string();
            }
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
    /// Import characters from CSV, merging with existing data.
    /// 
    /// The CSV only contains a subset of fields, so we need to start with
    /// existing data and update only the fields present in the CSV.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Age, 3: Gender, 4: Max MP, 5: Element ID,
    /// 6: Base MAXHP, 7: Base Power, 8: Base Will, 9: Base Vigor, 10: Base Agile, 11: Base Quick,
    /// 12: Weapon ID, 13: Armor ID, 14: Accessory ID,
    /// 15-20: Magic EXP (6 elements)
    pub fn import_characters<R: Read>(reader: R, existing: &[Character]) -> Result<Vec<Character>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut characters: Vec<Character> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            // Find existing character to update
            let char = characters.iter_mut().find(|c| c.id == id);
            if let Some(c) = char {
                // Update fields from CSV
                c.name = record.get(1).unwrap_or("").to_string();
                c.age = parse_or_default(record.get(2).unwrap_or("0"));
                // Gender is text like "Male"/"Female", need to convert
                let gender_str = record.get(3).unwrap_or("Male");
                c.gender_id = if gender_str.eq_ignore_ascii_case("Female") { 1 } else { 0 };
                c.max_mp = parse_or_default(record.get(4).unwrap_or("0"));
                c.element_id = parse_or_default(record.get(5).unwrap_or("0"));
                c.max_hp = parse_or_default(record.get(6).unwrap_or("0"));
                c.power = parse_or_default(record.get(7).unwrap_or("0"));
                c.will = parse_or_default(record.get(8).unwrap_or("0"));
                c.vigor = parse_or_default(record.get(9).unwrap_or("0"));
                c.agile = parse_or_default(record.get(10).unwrap_or("0"));
                c.quick = parse_or_default(record.get(11).unwrap_or("0"));
                c.weapon_id = parse_or_default(record.get(12).unwrap_or("0"));
                c.armor_id = parse_or_default(record.get(13).unwrap_or("0"));
                c.accessory_id = parse_or_default(record.get(14).unwrap_or("0"));
                
                // Magic EXP (6 elements)
                for i in 0..6 {
                    c.magic_exp[i] = parse_or_default(record.get(15 + i).unwrap_or("0"));
                }
            }
        }
        
        Ok(characters)
    }
    
    /// Import character magic from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Element ID, 3: [Element Name],
    /// 4: Order, 5: Effect ID, 6: [Effect Name], 7: Scope ID, 8: [Scope Name],
    /// 9: Effect SP, 10: Effect Base, 11: Type ID, 12: [Type Name],
    /// 13: State ID, 14: [State Name], 15: State Miss%,
    /// 16: [US Descr Pos], 17: [US Descr Size], 18: US Descr Str
    pub fn import_character_magic<R: Read>(reader: R, existing: &[CharacterMagic]) -> Result<Vec<CharacterMagic>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut magic: Vec<CharacterMagic> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            // Find existing entry to update
            if let Some(m) = magic.iter_mut().find(|m| m.id == id) {
                m.name = record.get(1).unwrap_or("").to_string();
                m.element_id = parse_or_default(record.get(2).unwrap_or("0"));
                // Skip [Element Name] at 3
                m.order = parse_or_default(record.get(4).unwrap_or("0"));
                m.effect_id = parse_or_default(record.get(5).unwrap_or("-1"));
                // Skip [Effect Name] at 6
                m.scope_id = parse_or_default(record.get(7).unwrap_or("0"));
                // Skip [Scope Name] at 8
                m.effect_sp = parse_or_default(record.get(9).unwrap_or("0"));
                m.effect_base = parse_or_default(record.get(10).unwrap_or("0"));
                m.type_id = parse_or_default(record.get(11).unwrap_or("0"));
                // Skip [Type Name] at 12
                m.state_id = parse_or_default(record.get(13).unwrap_or("0"));
                // Skip [State Name] at 14
                m.state_miss = parse_or_default(record.get(15).unwrap_or("0"));
                // Skip description pos/size (read-only), keep description
                m.description = record.get(18).unwrap_or("").to_string();
            }
        }
        
        Ok(magic)
    }
    
    /// Import character super moves from CSV.
    /// Import character super moves from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Element ID, 3: [Element Name], 4: Order,
    /// 5: Occasion Flags, 6-8: [M/B/S], 9: Effect ID, 10: [Effect Name],
    /// 11: Scope ID, 12: [Scope Name], 13: Category ID, 14: [Category Name],
    /// 15: Effect Speed, 16: Effect SP, 17-18: Pads, 19: Effect Base,
    /// 20: Type ID, 21: [Type Name], 22: State ID, 23: [State Name], 24: State Miss%,
    /// 25-27: Pads, 28: Ship Occ ID, 29: [Ship Occ Name], 30: Pad,
    /// 31: Ship Eff ID, 32: [Ship Eff Name], 33: Ship Eff SP, 34: Ship Eff Turns,
    /// 35: Ship Eff Base, 36: Unk, 37-39: Pads, 40-42: Description
    pub fn import_character_super_moves<R: Read>(reader: R, existing: &[CharacterSuperMove]) -> Result<Vec<CharacterSuperMove>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut moves: Vec<CharacterSuperMove> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(m) = moves.iter_mut().find(|m| m.id == id) {
                m.name = record.get(1).unwrap_or("").to_string();
                m.element_id = parse_or_default(record.get(2).unwrap_or("0"));
                m.order = parse_or_default(record.get(4).unwrap_or("0"));
                m.occasion_flags = parse_binary(record.get(5).unwrap_or("0"));
                m.effect_id = parse_or_default(record.get(9).unwrap_or("-1"));
                m.scope_id = parse_or_default(record.get(11).unwrap_or("0"));
                m.category_id = parse_or_default(record.get(13).unwrap_or("0"));
                m.effect_speed = parse_or_default(record.get(15).unwrap_or("0"));
                m.effect_sp = parse_or_default(record.get(16).unwrap_or("0"));
                m.effect_base = parse_or_default(record.get(19).unwrap_or("0"));
                m.type_id = parse_or_default(record.get(20).unwrap_or("0"));
                m.state_id = parse_or_default(record.get(22).unwrap_or("0"));
                m.state_miss = parse_or_default(record.get(24).unwrap_or("0"));
                m.ship_occasion_id = parse_or_default(record.get(28).unwrap_or("0"));
                m.ship_effect_id = parse_or_default(record.get(31).unwrap_or("0"));
                m.ship_effect_sp = parse_or_default(record.get(33).unwrap_or("0"));
                m.ship_effect_turns = parse_or_default(record.get(34).unwrap_or("0"));
                m.ship_effect_base = parse_or_default(record.get(35).unwrap_or("0"));
                m.unknown = parse_or_default(record.get(36).unwrap_or("0"));
                m.description = record.get(42).unwrap_or("").to_string();
            }
        }
        
        Ok(moves)
    }
    
    /// Import shops from CSV.
    /// Import shops from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1-48: Item IDs (48 items),
    /// 49: [US Descr Pos], 50: [US Descr Size], 51: US Descr Str
    pub fn import_shops<R: Read>(reader: R, existing: &[Shop]) -> Result<Vec<Shop>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut shops: Vec<Shop> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u16 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(shop) = shops.iter_mut().find(|s| s.id == id) {
                // Read item IDs from consecutive columns 1-48
                shop.item_ids.clear();
                for i in 0..48 {
                    let col = 1 + i;
                    if let Some(val) = record.get(col) {
                        shop.item_ids.push(parse_or_default(val));
                    } else {
                        shop.item_ids.push(-1);
                    }
                }
                // Skip desc pos/size, keep description
                shop.description = record.get(51).unwrap_or("").to_string();
            }
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
    /// Import crew members from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Position ID, 3: [Position Name],
    /// 4: Trait ID, 5: [Trait Name], 6: Trait Value,
    /// 7: Ship Eff ID, 8: Ship Eff SP, 9: Ship Eff Turns, 10: Ship Eff Base,
    /// 11: [US Descr Pos], 12: [US Descr Size], 13: US Descr Str
    pub fn import_crew_members<R: Read>(reader: R, existing: &[CrewMember]) -> Result<Vec<CrewMember>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut members: Vec<CrewMember> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(m) = members.iter_mut().find(|m| m.id == id) {
                m.name = record.get(1).unwrap_or("").to_string();
                m.position_id = parse_or_default(record.get(2).unwrap_or("0"));
                m.trait_id = parse_or_default(record.get(4).unwrap_or("-1"));
                m.trait_value = parse_or_default(record.get(6).unwrap_or("0"));
                m.ship_effect_id = parse_or_default(record.get(7).unwrap_or("0"));
                m.ship_effect_sp = parse_or_default(record.get(8).unwrap_or("0"));
                m.ship_effect_turns = parse_or_default(record.get(9).unwrap_or("0"));
                m.ship_effect_base = parse_or_default(record.get(10).unwrap_or("0"));
                m.description = record.get(13).unwrap_or("").to_string();
            }
        }
        
        Ok(members)
    }
    
    /// Import playable ships from CSV.
    /// Import playable ships from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: MAXHP, 3: MAXSP, 4: SP, 5: Defense,
    /// 6: MagDef, 7: Quick, 8: Dodge%, 9-14: Elements (6),
    /// 15-19: Cannon IDs (5), 20-22: Accessory IDs (3)
    pub fn import_playable_ships<R: Read>(reader: R, existing: &[PlayableShip]) -> Result<Vec<PlayableShip>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut ships: Vec<PlayableShip> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(ship) = ships.iter_mut().find(|s| s.id == id) {
                ship.name = record.get(1).unwrap_or("").to_string();
                ship.max_hp = parse_or_default(record.get(2).unwrap_or("0"));
                ship.max_sp = parse_or_default(record.get(3).unwrap_or("0"));
                ship.sp = parse_or_default(record.get(4).unwrap_or("0"));
                ship.defense = parse_or_default(record.get(5).unwrap_or("0"));
                ship.mag_def = parse_or_default(record.get(6).unwrap_or("0"));
                ship.quick = parse_or_default(record.get(7).unwrap_or("0"));
                ship.dodge = parse_or_default(record.get(8).unwrap_or("0"));
                
                for i in 0..6 {
                    ship.elements[i] = parse_or_default(record.get(9 + i).unwrap_or("0"));
                }
                
                for i in 0..5 {
                    ship.cannon_ids[i] = parse_or_default(record.get(15 + i).unwrap_or("-1"));
                }
                
                for i in 0..3 {
                    ship.accessory_ids[i] = parse_or_default(record.get(20 + i).unwrap_or("-1"));
                }
            }
        }
        
        Ok(ships)
    }
    
    
    /// Import ship cannons from CSV.
    /// Import ship cannons from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Ship Flags, 3-8: [Ship flag columns],
    /// 9: Type ID, 10: [Type Name], 11: Element ID, 12: [Element Name],
    /// 13: Attack, 14: Hit%, 15: Limit, 16: SP Cost,
    /// 17: Trait ID, 18: [Trait Name], 19: Trait Value,
    /// 20: Buy, 21: Sell%, 22: US Order, 23-25: Description
    pub fn import_ship_cannons<R: Read>(reader: R, existing: &[ShipCannon]) -> Result<Vec<ShipCannon>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut cannons: Vec<ShipCannon> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(c) = cannons.iter_mut().find(|c| c.id == id) {
                c.name = record.get(1).unwrap_or("").to_string();
                c.ship_flags = parse_binary(record.get(2).unwrap_or("0"));
                c.type_id = parse_or_default(record.get(9).unwrap_or("0"));
                c.element_id = parse_or_default(record.get(11).unwrap_or("-1"));
                c.attack = parse_or_default(record.get(13).unwrap_or("0"));
                c.hit = parse_or_default(record.get(14).unwrap_or("0"));
                c.limit = parse_or_default(record.get(15).unwrap_or("0"));
                c.sp = parse_or_default(record.get(16).unwrap_or("0"));
                c.trait_id = parse_or_default(record.get(17).unwrap_or("-1"));
                c.trait_value = parse_or_default(record.get(19).unwrap_or("0"));
                c.buy_price = parse_or_default(record.get(20).unwrap_or("0"));
                c.sell_percent = parse_or_default(record.get(21).unwrap_or("0"));
                c.order1 = parse_or_default(record.get(22).unwrap_or("0"));
                c.description = record.get(25).unwrap_or("").to_string();
            }
        }
        
        Ok(cannons)
    }
    
    /// Import ship accessories from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Ship Flags, 3-8: [Ship flag columns],
    /// 9: Trait 1 ID, 10: [Trait 1 Name], 11: Trait 1 Value,
    /// 12: Trait 2 ID, 13: [Trait 2 Name], 14: Trait 2 Value,
    /// 15: Trait 3 ID, 16: [Trait 3 Name], 17: Trait 3 Value,
    /// 18: Trait 4 ID, 19: [Trait 4 Name], 20: Trait 4 Value,
    /// 21: Buy, 22: Sell%, 23: US Order, 24-26: Description
    pub fn import_ship_accessories<R: Read>(reader: R, existing: &[ShipAccessory]) -> Result<Vec<ShipAccessory>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut accessories: Vec<ShipAccessory> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(acc) = accessories.iter_mut().find(|a| a.id == id) {
                acc.name = record.get(1).unwrap_or("").to_string();
                acc.ship_flags = parse_binary(record.get(2).unwrap_or("0"));
                
                // Traits: columns 9,11 / 12,14 / 15,17 / 18,20
                acc.traits[0].id = parse_or_default(record.get(9).unwrap_or("-1"));
                acc.traits[0].value = parse_or_default(record.get(11).unwrap_or("0"));
                acc.traits[1].id = parse_or_default(record.get(12).unwrap_or("-1"));
                acc.traits[1].value = parse_or_default(record.get(14).unwrap_or("0"));
                acc.traits[2].id = parse_or_default(record.get(15).unwrap_or("-1"));
                acc.traits[2].value = parse_or_default(record.get(17).unwrap_or("0"));
                acc.traits[3].id = parse_or_default(record.get(18).unwrap_or("-1"));
                acc.traits[3].value = parse_or_default(record.get(20).unwrap_or("0"));
                
                acc.buy_price = parse_or_default(record.get(21).unwrap_or("0"));
                acc.sell_percent = parse_or_default(record.get(22).unwrap_or("0"));
                acc.order1 = parse_or_default(record.get(23).unwrap_or("0"));
                acc.description = record.get(26).unwrap_or("").to_string();
            }
        }
        
        Ok(accessories)
    }
    
    /// Import ship items from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Occasion Flags, 3-5: [M/B/S],
    /// 6: Ship Eff ID, 7: Ship Eff Turns, 8: Consume%,
    /// 9: Buy, 10: Sell%, 11: US Order 1, 12: US Order 2,
    /// 13: Ship Eff Base, 14: Element ID, 15: [Element Name],
    /// 16: Unk 1, 17: Unk 2, 18: Hit%, 19-21: Description
    pub fn import_ship_items<R: Read>(reader: R, existing: &[ShipItem]) -> Result<Vec<ShipItem>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut items: Vec<ShipItem> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                item.name = record.get(1).unwrap_or("").to_string();
                item.occasion_flags = parse_binary(record.get(2).unwrap_or("0"));
                item.ship_effect_id = parse_or_default(record.get(6).unwrap_or("0"));
                item.ship_effect_turns = parse_or_default(record.get(7).unwrap_or("0"));
                item.consume = parse_or_default(record.get(8).unwrap_or("0"));
                item.buy_price = parse_or_default(record.get(9).unwrap_or("0"));
                item.sell_percent = parse_or_default(record.get(10).unwrap_or("0"));
                item.order1 = parse_or_default(record.get(11).unwrap_or("0"));
                item.order2 = parse_or_default(record.get(12).unwrap_or("0"));
                item.ship_effect_base = parse_or_default(record.get(13).unwrap_or("0"));
                item.element_id = parse_or_default(record.get(14).unwrap_or("-1"));
                item.unknown1 = parse_or_default(record.get(16).unwrap_or("0"));
                item.unknown2 = parse_or_default(record.get(17).unwrap_or("0"));
                item.hit = parse_or_default(record.get(18).unwrap_or("0"));
                item.description = record.get(21).unwrap_or("").to_string();
            }
        }
        
        Ok(items)
    }
    
    /// Import enemy ships from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: MAXHP, 3: Will, 4: Defense, 5: MagDef,
    /// 6: Quick, 7: Agile, 8: Dodge%, 9-14: Elements (6),
    /// 15-34: Armaments (4 armaments × 5 fields each),
    /// 35: EXP, 36: Gold, 37-42: Item Drops (3 drops × 2 fields each)
    pub fn import_enemy_ships<R: Read>(reader: R, existing: &[EnemyShip]) -> Result<Vec<EnemyShip>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut ships: Vec<EnemyShip> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(ship) = ships.iter_mut().find(|s| s.id == id) {
                ship.name = record.get(1).unwrap_or("").to_string();
                ship.max_hp = parse_or_default(record.get(2).unwrap_or("0"));
                ship.will = parse_or_default(record.get(3).unwrap_or("0"));
                ship.defense = parse_or_default(record.get(4).unwrap_or("0"));
                ship.mag_def = parse_or_default(record.get(5).unwrap_or("0"));
                ship.quick = parse_or_default(record.get(6).unwrap_or("0"));
                ship.agile = parse_or_default(record.get(7).unwrap_or("0"));
                ship.dodge = parse_or_default(record.get(8).unwrap_or("0"));
                
                for i in 0..6 {
                    ship.elements[i] = parse_or_default(record.get(9 + i).unwrap_or("0"));
                }
                
                // Armaments: 4 armaments starting at column 15, 5 fields each
                for i in 0..4 {
                    let base = 15 + i * 5;
                    ship.armaments[i].type_id = parse_or_default(record.get(base).unwrap_or("0"));
                    ship.armaments[i].attack = parse_or_default(record.get(base + 1).unwrap_or("0"));
                    ship.armaments[i].range = parse_or_default(record.get(base + 2).unwrap_or("0"));
                    ship.armaments[i].hit = parse_or_default(record.get(base + 3).unwrap_or("0"));
                    ship.armaments[i].element_id = parse_or_default(record.get(base + 4).unwrap_or("0"));
                }
                
                ship.exp = parse_or_default(record.get(35).unwrap_or("0"));
                ship.gold = parse_or_default(record.get(36).unwrap_or("0"));
                
                // Item drops: 3 drops starting at column 37, 2 fields each
                for i in 0..3 {
                    let base = 37 + i * 2;
                    ship.item_drops[i].drop_id = parse_or_default(record.get(base).unwrap_or("0"));
                    ship.item_drops[i].item_id = parse_or_default(record.get(base + 1).unwrap_or("0"));
                }
            }
        }
        
        Ok(ships)
    }
    
    /// Import enemy magic from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Category ID,
    /// 3: Effect ID, 4: [Effect Name], 5: Scope ID, 6: [Scope Name],
    /// 7: Effect Param ID, 8: Effect Base, 9: Element ID, 10: [Element Name],
    /// 11: Type ID, 12: State Inflict ID, 13: State Resist ID,
    /// 14: State ID, 15: [State Name], 16: State Miss%
    pub fn import_enemy_magic<R: Read>(reader: R, existing: &[EnemyMagic]) -> Result<Vec<EnemyMagic>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut magic: Vec<EnemyMagic> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(m) = magic.iter_mut().find(|m| m.id == id) {
                m.name = record.get(1).unwrap_or("").to_string();
                m.category_id = parse_or_default(record.get(2).unwrap_or("0"));
                m.effect_id = parse_or_default(record.get(3).unwrap_or("-1"));
                m.scope_id = parse_or_default(record.get(5).unwrap_or("0"));
                m.effect_param_id = parse_or_default(record.get(7).unwrap_or("0"));
                m.effect_base = parse_or_default(record.get(8).unwrap_or("0"));
                m.element_id = parse_or_default(record.get(9).unwrap_or("-1"));
                m.type_id = parse_or_default(record.get(11).unwrap_or("0"));
                m.state_infliction_id = parse_or_default(record.get(12).unwrap_or("0"));
                m.state_resistance_id = parse_or_default(record.get(13).unwrap_or("0"));
                m.state_id = parse_or_default(record.get(14).unwrap_or("0"));
                m.state_miss = parse_or_default(record.get(16).unwrap_or("0"));
            }
        }
        
        Ok(magic)
    }
    
    /// Import enemy super moves from CSV.
    /// Import enemy super moves from CSV, merging with existing data.
    /// 
    /// CSV columns (matching export):
    /// 0: Entry ID, 1: Entry US Name, 2: Category ID, 3: [Category Name],
    /// 4: Effect ID, 5: [Effect Name], 6: Scope ID, 7: [Scope Name],
    /// 8: Effect Param ID, 9: Effect Base, 10: Element ID, 11: [Element Name],
    /// 12: Type ID, 13: State Inflict ID, 14: State Resist ID,
    /// 15: State ID, 16: [State Name], 17: State Miss%
    pub fn import_enemy_super_moves<R: Read>(reader: R, existing: &[EnemySuperMove]) -> Result<Vec<EnemySuperMove>> {
        let mut rdr = csv::Reader::from_reader(reader);
        let mut moves: Vec<EnemySuperMove> = existing.to_vec();
        
        for result in rdr.records() {
            let record = result.map_err(|e| Error::ParseError { offset: 0, message: format!("CSV parse error: {}", e) })?;
            
            let id: u32 = parse_or_default(record.get(0).unwrap_or("0"));
            
            if let Some(m) = moves.iter_mut().find(|m| m.id == id) {
                m.name = record.get(1).unwrap_or("").to_string();
                m.category_id = parse_or_default(record.get(2).unwrap_or("0"));
                m.effect_id = parse_or_default(record.get(4).unwrap_or("-1"));
                m.scope_id = parse_or_default(record.get(6).unwrap_or("0"));
                m.effect_param_id = parse_or_default(record.get(8).unwrap_or("0"));
                m.effect_base = parse_or_default(record.get(9).unwrap_or("0"));
                m.element_id = parse_or_default(record.get(10).unwrap_or("-1"));
                m.type_id = parse_or_default(record.get(12).unwrap_or("0"));
                m.state_infliction_id = parse_or_default(record.get(13).unwrap_or("0"));
                m.state_resistance_id = parse_or_default(record.get(14).unwrap_or("0"));
                m.state_id = parse_or_default(record.get(15).unwrap_or("0"));
                m.state_miss = parse_or_default(record.get(17).unwrap_or("0"));
            }
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

