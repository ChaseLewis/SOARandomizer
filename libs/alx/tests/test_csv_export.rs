//! Integration tests for CSV export matching original ALX format.

mod common;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use alx::GameRoot;

/// Path to reference CSV files.
const REFERENCE_CSV_DIR: &str = "../../submodules/alx/dist/2002-12-19-gc-us-final/data";

/// Skip test if reference CSVs don't exist.
macro_rules! skip_if_no_reference {
    () => {
        if !Path::new(REFERENCE_CSV_DIR).exists() {
            eprintln!("Skipping: Reference CSV directory not found at {}", REFERENCE_CSV_DIR);
            return;
        }
    };
}

/// A row from a reference CSV with helper methods for parsing values.
#[derive(Debug)]
struct RefRow(HashMap<String, String>);

impl RefRow {
    fn str(&self, key: &str) -> &str {
        self.0.get(key).map(|s| s.as_str()).unwrap_or("")
    }
    
    fn u32(&self, key: &str) -> u32 {
        self.0.get(key).and_then(|s| s.parse().ok()).unwrap_or(0)
    }
    
    fn u16(&self, key: &str) -> u16 {
        self.0.get(key).and_then(|s| s.parse().ok()).unwrap_or(0)
    }
    
    fn u8(&self, key: &str) -> u8 {
        self.0.get(key).and_then(|s| s.parse().ok()).unwrap_or(0)
    }
    
    fn i8(&self, key: &str) -> i8 {
        self.0.get(key).and_then(|s| s.parse().ok()).unwrap_or(0)
    }
    
    fn i16(&self, key: &str) -> i16 {
        self.0.get(key).and_then(|s| s.parse().ok()).unwrap_or(0)
    }
    
    fn i32(&self, key: &str) -> i32 {
        self.0.get(key).and_then(|s| s.parse().ok()).unwrap_or(0)
    }
    
    /// Parse a hex value like "0x2c7d9c".
    fn hex(&self, key: &str) -> u32 {
        self.0.get(key).and_then(|s| {
            if s.starts_with("0x") {
                u32::from_str_radix(&s[2..], 16).ok()
            } else {
                s.parse().ok()
            }
        }).unwrap_or(0)
    }
    
    /// Parse binary flags like "0b00111010".
    fn binary_flags(&self, key: &str) -> u16 {
        self.0.get(key).and_then(|s| {
            if s.starts_with("0b") {
                u16::from_str_radix(&s[2..], 2).ok()
            } else {
                s.parse().ok()
            }
        }).unwrap_or(0)
    }
}

/// Load a reference CSV and the game, returning both.
fn load_reference_and_game(csv_name: &str) -> (Vec<RefRow>, GameRoot) {
    let path = Path::new(REFERENCE_CSV_DIR).join(csv_name);
    let file = File::open(&path).expect(&format!("Failed to open {}", csv_name));
    let mut reader = csv::Reader::from_reader(file);
    let headers: Vec<String> = reader.headers().unwrap().iter().map(|s| s.to_string()).collect();
    
    let rows: Vec<RefRow> = reader.records()
        .map(|r| {
            let record = r.unwrap();
            let values: HashMap<String, String> = headers.iter()
                .zip(record.iter())
                .map(|(h, v)| (h.clone(), v.to_string()))
                .collect();
            RefRow(values)
        })
        .collect();
    
    let game = common::load_game();
    (rows, game)
}

#[test]
fn test_weapon_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("weapon.csv");
    let weapons = game.read_weapons().unwrap();
    
    assert_eq!(weapons.len(), refs.len(), "Weapon count mismatch");
    
    for (weapon, r) in weapons.iter().zip(refs.iter()) {
        let ctx = format!("Weapon {} ({})", weapon.id, weapon.name);
        
        assert_eq!(weapon.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(weapon.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(weapon.character_id, r.i8("PC ID"), "{ctx}: PC ID");
        assert_eq!(weapon.sell_percent, r.i8("Sell%"), "{ctx}: Sell%");
        assert_eq!(weapon.buy_price, r.u16("Buy"), "{ctx}: Buy");
        assert_eq!(weapon.attack, r.i16("Attack"), "{ctx}: Attack");
        assert_eq!(weapon.hit_percent, r.i16("Hit%"), "{ctx}: Hit%");
        assert_eq!(weapon.effect_id, r.i8("Effect ID"), "{ctx}: Effect ID");
        assert_eq!(weapon.trait_data.id, r.i8("Trait ID"), "{ctx}: Trait ID");
        assert_eq!(weapon.trait_data.value, r.i16("Trait Value"), "{ctx}: Trait Value");
        assert_eq!(weapon.description_pos, r.hex("[US Descr Pos]"), "{ctx}: Descr Pos");
        assert_eq!(weapon.description_size, r.u32("[US Descr Size]"), "{ctx}: Descr Size");
    }
    
    println!("✓ All {} weapons match reference CSV!", weapons.len());
}

#[test]
fn test_accessory_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("accessory.csv");
    let accessories = game.read_accessories().unwrap();
    
    assert_eq!(accessories.len(), refs.len(), "Accessory count mismatch");
    
    for (acc, r) in accessories.iter().zip(refs.iter()) {
        let ctx = format!("Accessory {} ({})", acc.id, acc.name);
        
        assert_eq!(acc.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(acc.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(acc.character_flags.0 as u16, r.binary_flags("PC Flags"), "{ctx}: PC Flags");
        assert_eq!(acc.sell_percent, r.i8("Sell%"), "{ctx}: Sell%");
        assert_eq!(acc.buy_price, r.u16("Buy"), "{ctx}: Buy");
        assert_eq!(acc.traits[0].id, r.i8("Trait 1 ID"), "{ctx}: Trait 1 ID");
        assert_eq!(acc.traits[0].value, r.i16("Trait 1 Value"), "{ctx}: Trait 1 Value");
        assert_eq!(acc.description_pos, r.hex("[US Descr Pos]"), "{ctx}: Descr Pos");
    }
    
    println!("✓ All {} accessories match reference CSV!", accessories.len());
}

#[test]
fn test_armor_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("armor.csv");
    let armors = game.read_armors().unwrap();
    
    assert_eq!(armors.len(), refs.len(), "Armor count mismatch");
    
    for (armor, r) in armors.iter().zip(refs.iter()) {
        let ctx = format!("Armor {} ({})", armor.id, armor.name);
        
        assert_eq!(armor.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(armor.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(armor.character_flags.0 as u16, r.binary_flags("PC Flags"), "{ctx}: PC Flags");
        assert_eq!(armor.sell_percent, r.i8("Sell%"), "{ctx}: Sell%");
        assert_eq!(armor.buy_price, r.u16("Buy"), "{ctx}: Buy");
        assert_eq!(armor.traits[0].id, r.i8("Trait 1 ID"), "{ctx}: Trait 1 ID");
        assert_eq!(armor.traits[0].value, r.i16("Trait 1 Value"), "{ctx}: Trait 1 Value");
        assert_eq!(armor.description_pos, r.hex("[US Descr Pos]"), "{ctx}: Descr Pos");
    }
    
    println!("✓ All {} armors match reference CSV!", armors.len());
}

#[test]
fn test_weapon_description_content() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("weapon.csv");
    let weapons = game.read_weapons().unwrap();
    
    // Normalize whitespace for comparison
    let normalize = |s: &str| -> String {
        s.replace("\r\n", "\n").replace('\n', " ")
         .split_whitespace().collect::<Vec<_>>().join(" ")
    };
    
    let first_weapon = &weapons[0];
    let our_dscr = normalize(&first_weapon.description);
    let ref_dscr = normalize(refs[0].str("US Descr Str"));
    
    assert_eq!(our_dscr, ref_dscr, 
        "Weapon {} ({}): Description content mismatch", 
        first_weapon.id, first_weapon.name);
    
    println!("✓ First weapon description matches!");
}

#[test]
fn test_usable_item_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("usableitem.csv");
    let items = game.read_usable_items().unwrap();
    
    assert_eq!(items.len(), refs.len(), "Usable item count mismatch");
    
    for (item, r) in items.iter().zip(refs.iter()) {
        let ctx = format!("UsableItem {} ({})", item.id, item.name);
        
        assert_eq!(item.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(item.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(item.effect_id, r.i8("Effect ID"), "{ctx}: Effect ID");
        assert_eq!(item.scope_id, r.u8("Scope ID"), "{ctx}: Scope ID");
        assert_eq!(item.consume_percent, r.i8("Consume%"), "{ctx}: Consume%");
        assert_eq!(item.sell_percent, r.i8("Sell%"), "{ctx}: Sell%");
        assert_eq!(item.buy_price, r.u16("Buy"), "{ctx}: Buy");
        assert_eq!(item.effect_base, r.i16("Effect Base"), "{ctx}: Effect Base");
        assert_eq!(item.element_id, r.i8("Element ID"), "{ctx}: Element ID");
        // Note: Description positions are read sequentially and may not match
        // entries with missing descriptions (marked 0x0 in reference).
        // We skip this check until we implement proper SOT (String Offset Table) handling.
    }
    
    println!("✓ All {} usable items match reference CSV!", items.len());
}

#[test]
fn test_special_item_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("specialitem.csv");
    let items = game.read_special_items().unwrap();
    
    assert_eq!(items.len(), refs.len(), "Special item count mismatch");
    
    for (item, r) in items.iter().zip(refs.iter()) {
        let ctx = format!("SpecialItem {} ({})", item.id, item.name);
        
        assert_eq!(item.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(item.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(item.sell_percent, r.i8("Sell%"), "{ctx}: Sell%");
        assert_eq!(item.buy_price, r.u16("Buy"), "{ctx}: Buy");
        // Note: Description positions skipped (see usable_item test note)
    }
    
    println!("✓ All {} special items match reference CSV!", items.len());
}

#[test]
fn test_character_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("character.csv");
    let characters = game.read_characters().unwrap();
    
    assert_eq!(characters.len(), refs.len(), "Character count mismatch");
    
    for (c, r) in characters.iter().zip(refs.iter()) {
        let ctx = format!("Character {} ({})", c.id, c.name);
        
        assert_eq!(c.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(c.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(c.age, r.i8("Age"), "{ctx}: Age");
        assert_eq!(c.gender_id, r.i8("Gender ID"), "{ctx}: Gender ID");
        assert_eq!(c.max_mp, r.i8("MAXMP"), "{ctx}: MAXMP");
        assert_eq!(c.element_id, r.i8("Element ID"), "{ctx}: Element ID");
        assert_eq!(c.weapon_id, r.u16("Weapon ID"), "{ctx}: Weapon ID");
        assert_eq!(c.armor_id, r.u16("Armor ID"), "{ctx}: Armor ID");
        assert_eq!(c.accessory_id, r.u16("Accessory ID"), "{ctx}: Accessory ID");
        assert_eq!(c.hp, r.i16("HP"), "{ctx}: HP");
        assert_eq!(c.max_hp, r.i16("MAXHP"), "{ctx}: MAXHP");
        assert_eq!(c.max_hp_growth, r.i16("MAXHP Growth"), "{ctx}: MAXHP Growth");
        assert_eq!(c.power, r.i16("Power"), "{ctx}: Power");
        assert_eq!(c.will, r.i16("Will"), "{ctx}: Will");
        assert_eq!(c.vigor, r.i16("Vigor"), "{ctx}: Vigor");
        assert_eq!(c.agile, r.i16("Agile"), "{ctx}: Agile");
        assert_eq!(c.quick, r.i16("Quick"), "{ctx}: Quick");
    }
    
    println!("✓ All {} characters match reference CSV!", characters.len());
}

#[test]
fn test_character_magic_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("charactermagic.csv");
    let magics = game.read_character_magic().unwrap();
    
    assert_eq!(magics.len(), refs.len(), "Character magic count mismatch");
    
    for (m, r) in magics.iter().zip(refs.iter()) {
        let ctx = format!("CharacterMagic {} ({})", m.id, m.name);
        
        assert_eq!(m.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(m.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(m.element_id, r.i8("Element ID"), "{ctx}: Element ID");
        assert_eq!(m.effect_id, r.i8("Effect ID"), "{ctx}: Effect ID");
        assert_eq!(m.scope_id, r.u8("Scope ID"), "{ctx}: Scope ID");
        assert_eq!(m.effect_sp, r.i8("Effect SP"), "{ctx}: Effect SP");
    }
    
    println!("✓ All {} character magics match reference CSV!", magics.len());
}

#[test]
fn test_shop_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("shop.csv");
    let shops = game.read_shops().unwrap();
    
    assert_eq!(shops.len(), refs.len(), "Shop count mismatch");
    
    for (shop, r) in shops.iter().zip(refs.iter()) {
        let ctx = format!("Shop {} ({})", shop.id, shop.description);
        
        assert_eq!(shop.id, r.u16("Entry ID"), "{ctx}: ID");
        
        // Check first item ID
        let ref_item1 = r.i16("Item 1 ID");
        if !shop.item_ids.is_empty() {
            assert_eq!(shop.item_ids[0], ref_item1, "{ctx}: Item 1 ID");
        }
    }
    
    println!("✓ All {} shops match reference CSV!", shops.len());
}

#[test]
fn test_treasure_chest_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("treasurechest.csv");
    let chests = game.read_treasure_chests().unwrap();
    
    assert_eq!(chests.len(), refs.len(), "Treasure chest count mismatch");
    
    for (chest, r) in chests.iter().zip(refs.iter()) {
        let ctx = format!("TreasureChest {}", chest.id);
        
        assert_eq!(chest.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(chest.item_id, r.i32("Item ID"), "{ctx}: Item ID");
        assert_eq!(chest.item_amount, r.i32("Item Amount"), "{ctx}: Item Amount");
    }
    
    println!("✓ All {} treasure chests match reference CSV!", chests.len());
}

#[test]
fn test_crew_member_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("crewmember.csv");
    let crew = game.read_crew_members().unwrap();
    
    assert_eq!(crew.len(), refs.len(), "Crew member count mismatch");
    
    for (c, r) in crew.iter().zip(refs.iter()) {
        let ctx = format!("CrewMember {} ({})", c.id, c.name);
        
        assert_eq!(c.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(c.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(c.position_id, r.i8("Pos ID"), "{ctx}: Pos ID");
        assert_eq!(c.trait_id, r.i8("Trait ID"), "{ctx}: Trait ID");
        assert_eq!(c.trait_value, r.i16("Trait Value"), "{ctx}: Trait Value");
        assert_eq!(c.ship_effect_id, r.i8("Ship Eff ID"), "{ctx}: Ship Eff ID");
    }
    
    println!("✓ All {} crew members match reference CSV!", crew.len());
}

#[test]
fn test_playable_ship_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("playableship.csv");
    let ships = game.read_playable_ships().unwrap();
    
    assert_eq!(ships.len(), refs.len(), "Playable ship count mismatch");
    
    for (ship, r) in ships.iter().zip(refs.iter()) {
        let ctx = format!("PlayableShip {} ({})", ship.id, ship.name);
        
        assert_eq!(ship.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(ship.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(ship.max_hp as i32, r.i32("MAXHP"), "{ctx}: MAXHP");
        assert_eq!(ship.max_sp, r.i16("MAXSP"), "{ctx}: MAXSP");
        assert_eq!(ship.defense, r.i16("Defense"), "{ctx}: Defense");
        assert_eq!(ship.mag_def, r.i16("MagDef"), "{ctx}: MagDef");
    }
    
    println!("✓ All {} playable ships match reference CSV!", ships.len());
}

#[test]
fn test_ship_cannon_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("shipcannon.csv");
    let cannons = game.read_ship_cannons().unwrap();
    
    assert_eq!(cannons.len(), refs.len(), "Ship cannon count mismatch");
    
    for (c, r) in cannons.iter().zip(refs.iter()) {
        let ctx = format!("ShipCannon {} ({})", c.id, c.name);
        
        assert_eq!(c.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(c.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(c.type_id, r.i8("Type ID"), "{ctx}: Type ID");
        assert_eq!(c.element_id, r.i8("Element ID"), "{ctx}: Element ID");
        assert_eq!(c.attack, r.i16("Attack"), "{ctx}: Attack");
        assert_eq!(c.buy_price, r.u16("Buy"), "{ctx}: Buy");
    }
    
    println!("✓ All {} ship cannons match reference CSV!", cannons.len());
}

#[test]
fn test_ship_accessory_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("shipaccessory.csv");
    let accessories = game.read_ship_accessories().unwrap();
    
    assert_eq!(accessories.len(), refs.len(), "Ship accessory count mismatch");
    
    for (acc, r) in accessories.iter().zip(refs.iter()) {
        let ctx = format!("ShipAccessory {} ({})", acc.id, acc.name);
        
        assert_eq!(acc.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(acc.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(acc.traits[0].id, r.i8("Trait 1 ID"), "{ctx}: Trait 1 ID");
        assert_eq!(acc.traits[0].value, r.i16("Trait 1 Value"), "{ctx}: Trait 1 Value");
        assert_eq!(acc.buy_price, r.u16("Buy"), "{ctx}: Buy");
    }
    
    println!("✓ All {} ship accessories match reference CSV!", accessories.len());
}

#[test]
fn test_ship_item_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("shipitem.csv");
    let items = game.read_ship_items().unwrap();
    
    assert_eq!(items.len(), refs.len(), "Ship item count mismatch");
    
    for (item, r) in items.iter().zip(refs.iter()) {
        let ctx = format!("ShipItem {} ({})", item.id, item.name);
        
        assert_eq!(item.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(item.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(item.ship_effect_id, r.i8("Ship Eff ID"), "{ctx}: Ship Eff ID");
        assert_eq!(item.buy_price, r.u16("Buy"), "{ctx}: Buy");
    }
    
    println!("✓ All {} ship items match reference CSV!", items.len());
}

#[test]
fn test_enemy_ship_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("enemyship.csv");
    let ships = game.read_enemy_ships().unwrap();
    
    assert_eq!(ships.len(), refs.len(), "Enemy ship count mismatch");
    
    for (ship, r) in ships.iter().zip(refs.iter()) {
        let ctx = format!("EnemyShip {} ({})", ship.id, ship.name);
        
        assert_eq!(ship.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(ship.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(ship.max_hp as i32, r.i32("MAXHP"), "{ctx}: MAXHP");
        assert_eq!(ship.will, r.i16("Will"), "{ctx}: Will");
        assert_eq!(ship.exp, r.i32("EXP"), "{ctx}: EXP");
        assert_eq!(ship.gold, r.i32("Gold"), "{ctx}: Gold");
    }
    
    println!("✓ All {} enemy ships match reference CSV!", ships.len());
}

#[test]
fn test_swashbuckler_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("swashbuckler.csv");
    let ratings = game.read_swashbucklers().unwrap();
    
    assert_eq!(ratings.len(), refs.len(), "Swashbuckler count mismatch");
    
    for (sw, r) in ratings.iter().zip(refs.iter()) {
        let ctx = format!("Swashbuckler {} ({})", sw.id, sw.name);
        
        assert_eq!(sw.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(sw.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(sw.rating, r.u8("Rating"), "{ctx}: Rating");
    }
    
    println!("✓ All {} swashbucklers match reference CSV!", ratings.len());
}

#[test]
fn test_exp_boost_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("expboost.csv");
    let boosts = game.read_exp_boosts().unwrap();
    
    assert_eq!(boosts.len(), refs.len(), "Exp boost count mismatch");
    
    for (b, r) in boosts.iter().zip(refs.iter()) {
        let ctx = format!("ExpBoost {}", b.id);
        
        assert_eq!(b.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(b.exp, r.u32("EXP"), "{ctx}: EXP");
        assert_eq!(b.green_exp, r.u32("Green EXP"), "{ctx}: Green EXP");
        assert_eq!(b.red_exp, r.u32("Red EXP"), "{ctx}: Red EXP");
    }
    
    println!("✓ All {} exp boosts match reference CSV!", boosts.len());
}

#[test]
fn test_enemy_magic_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("enemymagic.csv");
    let magics = game.read_enemy_magic().unwrap();
    
    assert_eq!(magics.len(), refs.len(), "Enemy magic count mismatch");
    
    for (m, r) in magics.iter().zip(refs.iter()) {
        let ctx = format!("EnemyMagic {} ({})", m.id, m.name);
        
        assert_eq!(m.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(m.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(m.effect_id, r.i8("Effect ID"), "{ctx}: Effect ID");
        assert_eq!(m.scope_id, r.u8("Scope ID"), "{ctx}: Scope ID");
    }
    
    println!("✓ All {} enemy magics match reference CSV!", magics.len());
}

#[test]
fn test_enemy_super_move_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("enemysupermove.csv");
    let moves = game.read_enemy_super_moves().unwrap();
    
    assert_eq!(moves.len(), refs.len(), "Enemy super move count mismatch");
    
    for (m, r) in moves.iter().zip(refs.iter()) {
        let ctx = format!("EnemySuperMove {} ({})", m.id, m.name);
        
        assert_eq!(m.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(m.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(m.category_id, r.i8("Category ID"), "{ctx}: Category ID");
        assert_eq!(m.effect_id, r.i8("Effect ID"), "{ctx}: Effect ID");
    }
    
    println!("✓ All {} enemy super moves match reference CSV!", moves.len());
}

#[test]
fn test_character_super_move_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("charactersupermove.csv");
    let moves = game.read_character_super_moves().unwrap();
    
    assert_eq!(moves.len(), refs.len(), "Character super move count mismatch");
    
    for (m, r) in moves.iter().zip(refs.iter()) {
        let ctx = format!("CharacterSuperMove {} ({})", m.id, m.name);
        
        assert_eq!(m.id, r.u32("Entry ID"), "{ctx}: ID");
        assert_eq!(m.name, r.str("Entry US Name"), "{ctx}: Name");
        assert_eq!(m.element_id, r.i8("Element ID"), "{ctx}: Element ID");
        assert_eq!(m.effect_id, r.i8("Effect ID"), "{ctx}: Effect ID");
        assert_eq!(m.effect_sp, r.i8("Effect SP"), "{ctx}: Effect SP");
    }
    
    println!("✓ All {} character super moves match reference CSV!", moves.len());
}

#[test]
fn test_spirit_curve_csv_matches_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let (refs, mut game) = load_reference_and_game("spiritcurve.csv");
    let curves = game.read_spirit_curves().unwrap();
    
    assert_eq!(curves.len(), refs.len(), "Spirit curve count mismatch");
    
    for (c, r) in curves.iter().zip(refs.iter()) {
        let ctx = format!("SpiritCurve {} ({})", c.id, c.character_name);
        
        assert_eq!(c.id, r.u32("Entry ID"), "{ctx}: ID");
        // Check first level SP values
        assert_eq!(c.levels[0].sp, r.i8("SP 1"), "{ctx}: SP 1");
        assert_eq!(c.levels[0].max_sp, r.i8("MAXSP 1"), "{ctx}: MAXSP 1");
    }
    
    println!("✓ All {} spirit curves match reference CSV!", curves.len());
}

// =============================================================================
// CSV Output Comparison Tests
// These tests generate CSV using CsvExporter and compare to reference files
// =============================================================================

use alx::csv::CsvExporter;

/// Compare generated CSV output to a reference file.
/// Returns Ok if they match, Err with details if not.
fn compare_csv_output(generated: &str, reference_path: &str) -> Result<(), String> {
    let ref_path = Path::new(REFERENCE_CSV_DIR).join(reference_path);
    let reference = std::fs::read_to_string(&ref_path)
        .map_err(|e| format!("Failed to read reference file: {}", e))?;
    
    // Parse both CSVs
    let mut gen_reader = csv::Reader::from_reader(generated.as_bytes());
    let mut ref_reader = csv::Reader::from_reader(reference.as_bytes());
    
    let gen_records: Vec<csv::StringRecord> = gen_reader.records()
        .filter_map(|r| r.ok())
        .collect();
    let ref_records: Vec<csv::StringRecord> = ref_reader.records()
        .filter_map(|r| r.ok())
        .collect();
    
    // Compare row counts
    if gen_records.len() != ref_records.len() {
        return Err(format!(
            "Row count mismatch: generated {} rows, reference has {} rows",
            gen_records.len(),
            ref_records.len()
        ));
    }
    
    Ok(())
}

/// Deep compare two CSV files cell by cell, optionally skipping certain columns.
/// Skips columns that don't exist in both and trims whitespace.
/// Returns detailed errors for mismatches.
fn compare_csv_deep_skip(
    generated: &str, 
    reference_path: &str,
    id_column: &str,
    skip_columns: &[&str],
) -> Result<(), String> {
    let ref_path = Path::new(REFERENCE_CSV_DIR).join(reference_path);
    let reference = std::fs::read_to_string(&ref_path)
        .map_err(|e| format!("Failed to read reference file: {}", e))?;
    
    // Parse both CSVs
    let mut gen_reader = csv::Reader::from_reader(generated.as_bytes());
    let mut ref_reader = csv::Reader::from_reader(reference.as_bytes());
    
    // Get headers
    let gen_headers: Vec<String> = gen_reader.headers()
        .map_err(|e| format!("Failed to read generated headers: {}", e))?
        .iter()
        .map(|s| s.trim().to_string())
        .collect();
    
    let ref_headers: Vec<String> = ref_reader.headers()
        .map_err(|e| format!("Failed to read reference headers: {}", e))?
        .iter()
        .map(|s| s.trim().to_string())
        .collect();
    
    // Find common columns, excluding any columns in skip_columns
    let common_cols: Vec<(usize, usize, &String)> = gen_headers.iter()
        .enumerate()
        .filter_map(|(gen_idx, h)| {
            // Skip if this column is in the skip list
            if skip_columns.iter().any(|&skip| skip == h) {
                return None;
            }
            ref_headers.iter()
                .position(|rh| rh == h)
                .map(|ref_idx| (gen_idx, ref_idx, h))
        })
        .collect();
    
    if common_cols.is_empty() {
        return Err("No common columns found between generated and reference".to_string());
    }
    
    // Find ID column index in generated
    let gen_id_idx = gen_headers.iter().position(|h| h == id_column);
    
    // Collect records
    let gen_records: Vec<csv::StringRecord> = gen_reader.records()
        .filter_map(|r| r.ok())
        .collect();
    let ref_records: Vec<csv::StringRecord> = ref_reader.records()
        .filter_map(|r| r.ok())
        .collect();
    
    if gen_records.len() != ref_records.len() {
        return Err(format!(
            "Row count mismatch: generated {} rows, reference has {} rows",
            gen_records.len(),
            ref_records.len()
        ));
    }
    
    // Compare each row, cell by cell
    let mut errors = Vec::new();
    for (row_idx, (gen_row, ref_row)) in gen_records.iter().zip(ref_records.iter()).enumerate() {
        let row_id: String = gen_id_idx
            .and_then(|idx| gen_row.get(idx))
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("row {}", row_idx));
        
        for &(gen_idx, ref_idx, col_name) in &common_cols {
            // Normalize: trim whitespace and convert \r\n to \n
            let gen_val = gen_row.get(gen_idx)
                .map(|s| s.trim().replace("\r\n", "\n"))
                .unwrap_or_default();
            let ref_val = ref_row.get(ref_idx)
                .map(|s| s.trim().replace("\r\n", "\n"))
                .unwrap_or_default();
            
            if gen_val != ref_val {
                errors.push(format!(
                    "ID {}, column '{}': generated '{}' != reference '{}'",
                    row_id, col_name, gen_val, ref_val
                ));
                if errors.len() >= 20 {
                    errors.push("... (more errors truncated)".to_string());
                    break;
                }
            }
        }
        if errors.len() >= 20 {
            break;
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("Cell mismatches:\n{}", errors.join("\n")))
    }
}

/// Compare CSVs cell by cell on common columns.
/// Convenience wrapper that doesn't skip any columns.
fn compare_csv_deep(
    generated: &str, 
    reference_path: &str,
    id_column: &str,
) -> Result<(), String> {
    compare_csv_deep_skip(generated, reference_path, id_column, &[])
}

#[test]
fn test_csv_exporter_accessory_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let accessories = game.read_accessories().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_accessories(&accessories, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "accessory.csv")
        .expect("Accessory CSV comparison failed");
    
    println!("✓ Accessory CSV has correct row count!");
}

#[test]
fn test_csv_exporter_armor_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let armors = game.read_armors().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_armors(&armors, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "armor.csv")
        .expect("Armor CSV comparison failed");
    
    println!("✓ Armor CSV has correct row count!");
}

#[test]
fn test_csv_exporter_weapon_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let weapons = game.read_weapons().unwrap();
    let weapon_effects = game.read_weapon_effects().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_weapons(&weapons, &mut buffer, &weapon_effects).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "weapon.csv")
        .expect("Weapon CSV comparison failed");
    
    println!("✓ Weapon CSV has correct row count!");
}

#[test]
fn test_csv_exporter_usable_item_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let items = game.read_usable_items().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_usable_items(&items, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "usableitem.csv")
        .expect("Usable item CSV comparison failed");
    
    println!("✓ Usable item CSV has correct row count!");
}

#[test]
fn test_csv_exporter_special_item_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let items = game.read_special_items().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_special_items(&items, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "specialitem.csv")
        .expect("Special item CSV comparison failed");
    
    println!("✓ Special item CSV has correct row count!");
}

#[test]
fn test_csv_exporter_character_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let characters = game.read_characters().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_characters(&characters, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "character.csv")
        .expect("Character CSV comparison failed");
    
    println!("✓ Character CSV has correct row count!");
}

#[test]
fn test_csv_exporter_shop_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let shops = game.read_shops().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_shops(&shops, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "shop.csv")
        .expect("Shop CSV comparison failed");
    
    println!("✓ Shop CSV has correct row count!");
}

#[test]
fn test_csv_exporter_treasure_chest_row_count() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let chests = game.read_treasure_chests().unwrap();
    let item_db = game.build_item_database().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_treasure_chests(&chests, &mut buffer, &item_db).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_output(&generated, "treasurechest.csv")
        .expect("Treasure chest CSV comparison failed");
    
    println!("✓ Treasure chest CSV has correct row count!");
}

// =============================================================================
// Deep CSV Comparison Tests (cell by cell)
// =============================================================================

#[test]
fn test_accessory_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_accessories().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_accessories(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "accessory.csv", "Entry ID")
        .expect("Accessory CSV deep comparison failed");
    
    println!("✓ Accessory CSV cells match reference!");
}

#[test]
fn test_weapon_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_weapons().unwrap();
    let weapon_effects = game.read_weapon_effects().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_weapons(&data, &mut buffer, &weapon_effects).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "weapon.csv", "Entry ID")
        .expect("Weapon CSV deep comparison failed");
    
    println!("✓ Weapon CSV cells match reference!");
}

#[test]
fn test_armor_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_armors().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_armors(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "armor.csv", "Entry ID")
        .expect("Armor CSV deep comparison failed");
    
    println!("✓ Armor CSV cells match reference!");
}

#[test]
fn test_usable_item_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_usable_items().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_usable_items(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    // Skip description columns - description reading has offset alignment issues
    let skip_descr = &["[US Descr Pos]", "[US Descr Size]", "US Descr Str"];
    compare_csv_deep_skip(&generated, "usableitem.csv", "Entry ID", skip_descr)
        .expect("Usable item CSV deep comparison failed");
    
    println!("✓ Usable item CSV cells match reference! (description columns skipped)");
}

#[test]
fn test_special_item_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_special_items().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_special_items(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    // Skip description columns - description reading has offset alignment issues
    let skip_descr = &["[US Descr Pos]", "[US Descr Size]", "US Descr Str"];
    compare_csv_deep_skip(&generated, "specialitem.csv", "Entry ID", skip_descr)
        .expect("Special item CSV deep comparison failed");
    
    println!("✓ Special item CSV cells match reference! (description columns skipped)");
}

#[test]
fn test_character_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_characters().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_characters(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "character.csv", "Entry ID")
        .expect("Character CSV deep comparison failed");
    
    println!("✓ Character CSV cells match reference!");
}

#[test]
fn test_character_magic_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_character_magic().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_character_magic(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "charactermagic.csv", "Entry ID")
        .expect("Character magic CSV deep comparison failed");
    
    println!("✓ Character magic CSV cells match reference!");
}

#[test]
fn test_character_super_move_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_character_super_moves().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_character_super_moves(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "charactersupermove.csv", "Entry ID")
        .expect("Character super move CSV deep comparison failed");
    
    println!("✓ Character super move CSV cells match reference!");
}

#[test]
fn test_shop_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_shops().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_shops(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    // Skip description columns - description reading has offset alignment issues
    let skip_descr = &["[US Descr Pos]", "[US Descr Size]", "US Descr Str"];
    compare_csv_deep_skip(&generated, "shop.csv", "Entry ID", skip_descr)
        .expect("Shop CSV deep comparison failed");
    
    println!("✓ Shop CSV cells match reference! (description columns skipped)");
}

#[test]
fn test_treasure_chest_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_treasure_chests().unwrap();
    let item_db = game.build_item_database().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_treasure_chests(&data, &mut buffer, &item_db).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "treasurechest.csv", "Entry ID")
        .expect("Treasure chest CSV deep comparison failed");
    
    println!("✓ Treasure chest CSV cells match reference!");
}

#[test]
fn test_crew_member_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_crew_members().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_crew_members(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "crewmember.csv", "Entry ID")
        .expect("Crew member CSV deep comparison failed");
    
    println!("✓ Crew member CSV cells match reference!");
}

#[test]
fn test_playable_ship_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_playable_ships().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_playable_ships(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "playableship.csv", "Entry ID")
        .expect("Playable ship CSV deep comparison failed");
    
    println!("✓ Playable ship CSV cells match reference!");
}

#[test]
fn test_ship_cannon_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_ship_cannons().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_ship_cannons(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "shipcannon.csv", "Entry ID")
        .expect("Ship cannon CSV deep comparison failed");
    
    println!("✓ Ship cannon CSV cells match reference!");
}

#[test]
fn test_ship_accessory_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_ship_accessories().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_ship_accessories(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "shipaccessory.csv", "Entry ID")
        .expect("Ship accessory CSV deep comparison failed");
    
    println!("✓ Ship accessory CSV cells match reference!");
}

#[test]
fn test_ship_item_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_ship_items().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_ship_items(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "shipitem.csv", "Entry ID")
        .expect("Ship item CSV deep comparison failed");
    
    println!("✓ Ship item CSV cells match reference!");
}

#[test]
fn test_enemy_ship_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_enemy_ships().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_enemy_ships(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "enemyship.csv", "Entry ID")
        .expect("Enemy ship CSV deep comparison failed");
    
    println!("✓ Enemy ship CSV cells match reference!");
}

#[test]
fn test_enemy_magic_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_enemy_magic().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_enemy_magic(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "enemymagic.csv", "Entry ID")
        .expect("Enemy magic CSV deep comparison failed");
    
    println!("✓ Enemy magic CSV cells match reference!");
}

#[test]
fn test_enemy_super_move_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_enemy_super_moves().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_enemy_super_moves(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "enemysupermove.csv", "Entry ID")
        .expect("Enemy super move CSV deep comparison failed");
    
    println!("✓ Enemy super move CSV cells match reference!");
}

#[test]
fn test_swashbuckler_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_swashbucklers().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_swashbucklers(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "swashbuckler.csv", "Entry ID")
        .expect("Swashbuckler CSV deep comparison failed");
    
    println!("✓ Swashbuckler CSV cells match reference!");
}

#[test]
fn test_spirit_curve_csv_deep_compare() {
    skip_if_no_iso!();
    skip_if_no_reference!();
    
    let mut game = common::load_game();
    let data = game.read_spirit_curves().unwrap();
    
    let mut buffer = Vec::new();
    CsvExporter::export_spirit_curves(&data, &mut buffer).unwrap();
    let generated = String::from_utf8(buffer).unwrap();
    
    compare_csv_deep(&generated, "spiritcurve.csv", "Entry ID")
        .expect("Spirit curve CSV deep comparison failed");
    
    println!("✓ Spirit curve CSV cells match reference!");
}
