//! CSV export functionality matching original ALX format.

use std::io::Write;

use crate::entries::{
    Accessory, Armor, Character, CharacterMagic, CharacterSuperMove, CrewMember, Enemy, EnemyMagic,
    EnemyShip, EnemySuperMove, EnemyTask, ExpBoost, ExpCurve, MagicExpCurve, PlayableShip,
    ShipAccessory, ShipCannon, ShipItem, Shop, SpecialItem, SpiritCurve, Swashbuckler,
    TreasureChest, UsableItem, Weapon, WeaponEffect, TRAIT_NAMES,
};
use crate::error::Result;
use crate::items::ItemDatabase;
use crate::lookups::{EFFECT_NAMES, ELEMENT_NAMES, SCOPE_NAMES, SHIP_OCCASION_NAMES, STATE_NAMES};

/// CSV exporter for game data.
pub struct CsvExporter;

impl CsvExporter {
    /// Export accessories to CSV format matching original ALX output.
    pub fn export_accessories<W: Write>(accessories: &[Accessory], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Write header matching original ALX format
        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "PC Flags",
            "[V]",
            "[A]",
            "[F]",
            "[D]",
            "[E]",
            "[G]",
            "Sell%",
            "US Order 1",
            "US Order 2",
            "Pad 1",
            "Buy",
            "Trait 1 ID",
            "[Trait 1 Name]",
            "Pad 2",
            "Trait 1 Value",
            "Trait 2 ID",
            "[Trait 2 Name]",
            "Pad 3",
            "Trait 2 Value",
            "Trait 3 ID",
            "[Trait 3 Name]",
            "Pad 4",
            "Trait 3 Value",
            "Trait 4 ID",
            "[Trait 4 Name]",
            "Pad 5",
            "Trait 4 Value",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for acc in accessories {
            let char_cols = acc.character_flags.as_columns();
            let trait_name = |id: i8| -> &str {
                if id < 0 {
                    "None"
                } else {
                    TRAIT_NAMES.get(id as usize).unwrap_or(&"Unknown")
                }
            };

            wtr.write_record(&[
                acc.id.to_string(),
                acc.name.clone(),
                acc.character_flags.as_binary_string(),
                char_cols[0].to_string(),
                char_cols[1].to_string(),
                char_cols[2].to_string(),
                char_cols[3].to_string(),
                char_cols[4].to_string(),
                char_cols[5].to_string(),
                acc.sell_percent.to_string(),
                acc.order1.to_string(),
                acc.order2.to_string(),
                "0".to_string(), // Pad 1
                acc.buy_price.to_string(),
                acc.traits[0].id.to_string(),
                trait_name(acc.traits[0].id).to_string(),
                "0".to_string(), // Pad 2
                acc.traits[0].value.to_string(),
                acc.traits[1].id.to_string(),
                trait_name(acc.traits[1].id).to_string(),
                "0".to_string(), // Pad 3
                acc.traits[1].value.to_string(),
                acc.traits[2].id.to_string(),
                trait_name(acc.traits[2].id).to_string(),
                "0".to_string(), // Pad 4
                acc.traits[2].value.to_string(),
                acc.traits[3].id.to_string(),
                trait_name(acc.traits[3].id).to_string(),
                "0".to_string(), // Pad 5
                acc.traits[3].value.to_string(),
                format!("0x{:x}", acc.description_pos),
                acc.description_size.to_string(),
                acc.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export armors to CSV format.
    pub fn export_armors<W: Write>(armors: &[Armor], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "PC Flags",
            "[V]",
            "[A]",
            "[F]",
            "[D]",
            "[E]",
            "[G]",
            "Sell%",
            "US Order 1",
            "US Order 2",
            "Pad 1",
            "Buy",
            "Trait 1 ID",
            "[Trait 1 Name]",
            "Pad 2",
            "Trait 1 Value",
            "Trait 2 ID",
            "[Trait 2 Name]",
            "Pad 3",
            "Trait 2 Value",
            "Trait 3 ID",
            "[Trait 3 Name]",
            "Pad 4",
            "Trait 3 Value",
            "Trait 4 ID",
            "[Trait 4 Name]",
            "Pad 5",
            "Trait 4 Value",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for armor in armors {
            let char_cols = armor.character_flags.as_columns();
            let trait_name = |id: i8| -> &str {
                if id < 0 {
                    "None"
                } else {
                    TRAIT_NAMES.get(id as usize).unwrap_or(&"Unknown")
                }
            };

            wtr.write_record(&[
                armor.id.to_string(),
                armor.name.clone(),
                armor.character_flags.as_binary_string(),
                char_cols[0].to_string(),
                char_cols[1].to_string(),
                char_cols[2].to_string(),
                char_cols[3].to_string(),
                char_cols[4].to_string(),
                char_cols[5].to_string(),
                armor.sell_percent.to_string(),
                armor.order1.to_string(),
                armor.order2.to_string(),
                "0".to_string(),
                armor.buy_price.to_string(),
                armor.traits[0].id.to_string(),
                trait_name(armor.traits[0].id).to_string(),
                "0".to_string(),
                armor.traits[0].value.to_string(),
                armor.traits[1].id.to_string(),
                trait_name(armor.traits[1].id).to_string(),
                "0".to_string(),
                armor.traits[1].value.to_string(),
                armor.traits[2].id.to_string(),
                trait_name(armor.traits[2].id).to_string(),
                "0".to_string(),
                armor.traits[2].value.to_string(),
                armor.traits[3].id.to_string(),
                trait_name(armor.traits[3].id).to_string(),
                "0".to_string(),
                armor.traits[3].value.to_string(),
                format!("0x{:x}", armor.description_pos),
                armor.description_size.to_string(),
                armor.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export weapons to CSV format.
    ///
    /// The `weapon_effects` are used to look up effect descriptions.
    pub fn export_weapons<W: Write>(
        weapons: &[Weapon],
        writer: W,
        weapon_effects: &[WeaponEffect],
    ) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Weapons have a different structure than armor/accessories
        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "PC ID",
            "[PC Name]",
            "Sell%",
            "US Order 1",
            "US Order 2",
            "Effect ID",
            "[Effect Name]",
            "Buy",
            "Attack",
            "Hit%",
            "Trait ID",
            "[Trait Name]",
            "Pad 1",
            "Trait Value",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for weapon in weapons {
            let trait_name_str = if weapon.trait_data.id < 0 {
                "None"
            } else {
                TRAIT_NAMES
                    .get(weapon.trait_data.id as usize)
                    .unwrap_or(&"Unknown")
            };

            // Look up effect name from weapon effects
            let effect_name = if weapon.effect_id < 0 {
                "None".to_string()
            } else {
                weapon_effects
                    .get(weapon.effect_id as usize)
                    .map(|e| e.description())
                    .unwrap_or_else(|| "???".to_string())
            };

            wtr.write_record(&[
                weapon.id.to_string(),
                weapon.name.clone(),
                weapon.character_id.to_string(),
                weapon.character_name().to_string(),
                weapon.sell_percent.to_string(),
                weapon.order1.to_string(),
                weapon.order2.to_string(),
                weapon.effect_id.to_string(),
                effect_name,
                weapon.buy_price.to_string(),
                weapon.attack.to_string(),
                weapon.hit_percent.to_string(),
                weapon.trait_data.id.to_string(),
                trait_name_str.to_string(),
                "0".to_string(),
                weapon.trait_data.value.to_string(),
                format!("0x{:x}", weapon.description_pos),
                weapon.description_size.to_string(),
                weapon.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export character super moves (S-Moves) to CSV format matching original ALX output.
    pub fn export_character_super_moves<W: Write>(
        super_moves: &[CharacterSuperMove],
        writer: W,
    ) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Write header matching original ALX format
        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Element ID",
            "[Element Name]",
            "Order",
            "Occasion Flags",
            "[M]",
            "[B]",
            "[S]",
            "Effect ID",
            "[Effect Name]",
            "Scope ID",
            "[Scope Name]",
            "Category ID",
            "[Category Name]",
            "Effect Speed",
            "Effect SP",
            "Pad 1",
            "Pad 2",
            "Effect Base",
            "Type ID",
            "[Type Name]",
            "State ID",
            "[State Name]",
            "State Miss%",
            "Pad 3",
            "Pad 4",
            "Pad 5",
            "Ship Occ ID",
            "[Ship Occ Name]",
            "Pad 6",
            "Ship Eff ID",
            "[Ship Eff Name]",
            "Ship Eff SP",
            "Ship Eff Turns",
            "Ship Eff Base",
            "Unk",
            "Pad 7",
            "Pad 8",
            "Pad 9",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for sm in super_moves {
            let element_str = ELEMENT_NAMES.get(sm.element_id);
            let effect_str = EFFECT_NAMES.get(sm.effect_id as i16);
            let scope_str = SCOPE_NAMES.get(sm.scope_id);
            let category_str = crate::lookups::category_name(sm.category_id);
            let type_str = crate::lookups::type_name(sm.type_id);
            let state_str = STATE_NAMES.get(sm.state_id);
            let ship_occ_str = SHIP_OCCASION_NAMES.get(sm.ship_occasion_id);
            let ship_eff_str = EFFECT_NAMES.get(sm.ship_effect_id);

            // Occasion flags: M=4, B=2, S=1
            let m_flag = if sm.occasion_flags & 0x04 != 0 {
                "X"
            } else {
                ""
            };
            let b_flag = if sm.occasion_flags & 0x02 != 0 {
                "X"
            } else {
                ""
            };
            let s_flag = if sm.occasion_flags & 0x01 != 0 {
                "X"
            } else {
                ""
            };

            wtr.write_record(&[
                sm.id.to_string(),
                sm.name.clone(),
                sm.element_id.to_string(),
                element_str.to_string(),
                sm.order.to_string(),
                format!("0b{:04b}", sm.occasion_flags),
                m_flag.to_string(),
                b_flag.to_string(),
                s_flag.to_string(),
                sm.effect_id.to_string(),
                effect_str.to_string(),
                sm.scope_id.to_string(),
                scope_str.to_string(),
                sm.category_id.to_string(),
                category_str.to_string(),
                sm.effect_speed.to_string(),
                sm.effect_sp.to_string(),
                "0".to_string(), // Pad 1
                "0".to_string(), // Pad 2
                sm.effect_base.to_string(),
                sm.type_id.to_string(),
                type_str.to_string(),
                sm.state_id.to_string(),
                state_str.to_string(),
                sm.state_miss.to_string(),
                "0".to_string(), // Pad 3
                "0".to_string(), // Pad 4
                "0".to_string(), // Pad 5
                sm.ship_occasion_id.to_string(),
                ship_occ_str.to_string(),
                "0".to_string(), // Pad 6
                sm.ship_effect_id.to_string(),
                ship_eff_str.to_string(),
                sm.ship_effect_sp.to_string(),
                sm.ship_effect_turns.to_string(),
                sm.ship_effect_base.to_string(),
                sm.unknown.to_string(),
                "0".to_string(), // Pad 7
                "0".to_string(), // Pad 8
                "0".to_string(), // Pad 9
                format!("0x{:x}", sm.description_pos),
                sm.description_size.to_string(),
                sm.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export usable items to CSV format.
    pub fn export_usable_items<W: Write>(items: &[UsableItem], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Occasion Flags",
            "[M]",
            "[B]",
            "[S]",
            "Effect ID",
            "[Effect Name]",
            "Scope ID",
            "[Scope Name]",
            "Element ID",
            "[Element Name]",
            "Sell%",
            "US Order 1",
            "US Order 2",
            "Pad 1",
            "Buy",
            "Effect Base",
            "Type ID",
            "[Type Name]",
            "State ID",
            "[State Name]",
            "State Miss%",
            "Pad 2",
            "Pad 3",
            "Pad 4",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for item in items {
            let m = if item.occasion_flags.can_use_menu() {
                "X"
            } else {
                ""
            };
            let b = if item.occasion_flags.can_use_battle() {
                "X"
            } else {
                ""
            };
            let s = if item.occasion_flags.can_use_ship() {
                "X"
            } else {
                ""
            };

            wtr.write_record(&[
                item.id.to_string(),
                item.name.clone(),
                item.occasion_flags.as_binary_string(),
                m.to_string(),
                b.to_string(),
                s.to_string(),
                item.effect_id.to_string(),
                EFFECT_NAMES.get(item.effect_id as i16).to_string(),
                item.scope_id.to_string(),
                SCOPE_NAMES.get(item.scope_id).to_string(),
                item.element_id.to_string(),
                ELEMENT_NAMES.get(item.element_id).to_string(),
                item.sell_percent.to_string(),
                item.order1.to_string(),
                item.order2.to_string(),
                "0".to_string(),
                item.buy_price.to_string(),
                item.effect_base.to_string(),
                item.type_id.to_string(),
                crate::lookups::type_name(item.type_id).to_string(),
                item.state_id.to_string(),
                STATE_NAMES.get(item.state_id as i8).to_string(),
                item.state_miss.to_string(),
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
                format!("0x{:x}", item.description_pos),
                item.description_size.to_string(),
                item.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export special items to CSV format.
    pub fn export_special_items<W: Write>(items: &[SpecialItem], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Sell%",
            "US Order 1",
            "US Order 2",
            "Pad 1",
            "Pad 2",
            "Pad 3",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for item in items {
            wtr.write_record(&[
                item.id.to_string(),
                item.name.clone(),
                item.sell_percent.to_string(),
                item.order1.to_string(),
                item.order2.to_string(),
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
                format!("0x{:x}", item.description_pos),
                item.description_size.to_string(),
                item.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export characters to CSV format matching reference ALX format.
    pub fn export_characters<W: Write>(characters: &[Character], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Header matching reference ALX format (76 columns)
        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Age",
            "Gender ID",
            "[Gender Name]",
            "Width",
            "Depth",
            "MAXMP",
            "Element ID",
            "[Element Name]",
            "Pad 1",
            "Weapon ID",
            "[Weapon Name]",
            "Armor ID",
            "[Armor Name]",
            "Accessory ID",
            "[Accessory Name]",
            "Movement Flags",
            "[May Dodge]",
            "[Unk Damage]",
            "[Unk Ranged]",
            "[Unk Melee]",
            "[Ranged Atk]",
            "[Melee Atk]",
            "[Ranged Only]",
            "[Take Cover]",
            "[In Air]",
            "[On Ground]",
            "[Reserved]",
            "[May Move]",
            "HP",
            "MAXHP",
            "MAXHP Growth",
            "SP",
            "MAXSP",
            "Counter%",
            "Pad 2",
            "EXP",
            "MAXMP Growth",
            "Unk 1",
            "Green",
            "Red",
            "Purple",
            "Blue",
            "Yellow",
            "Silver",
            "Poison",
            "Unconscious",
            "Stone",
            "Sleep",
            "Confusion",
            "Silence",
            "Fatigue",
            "Revival",
            "Weak",
            "State 10",
            "State 11",
            "State 12",
            "State 13",
            "State 14",
            "State 15",
            "Danger",
            "Power",
            "Will",
            "Vigor",
            "Agile",
            "Quick",
            "Pad 3",
            "Power Growth",
            "Will Growth",
            "Vigor Growth",
            "Agile Growth",
            "Quick Growth",
            "Green EXP",
            "Red EXP",
            "Purple EXP",
            "Blue EXP",
            "Yellow EXP",
            "Silver EXP",
        ])?;

        for c in characters {
            // Movement flags columns
            let may_dodge = if (c.movement_flags & 0x800) != 0 {
                "X"
            } else {
                ""
            };
            let unk_damage = if (c.movement_flags & 0x400) != 0 {
                "X"
            } else {
                ""
            };
            let unk_ranged = if (c.movement_flags & 0x200) != 0 {
                "X"
            } else {
                ""
            };
            let unk_melee = if (c.movement_flags & 0x100) != 0 {
                "X"
            } else {
                ""
            };
            let ranged_atk = if (c.movement_flags & 0x080) != 0 {
                "X"
            } else {
                ""
            };
            let melee_atk = if (c.movement_flags & 0x040) != 0 {
                "X"
            } else {
                ""
            };
            let ranged_only = if (c.movement_flags & 0x020) != 0 {
                "X"
            } else {
                ""
            };
            let take_cover = if (c.movement_flags & 0x010) != 0 {
                "X"
            } else {
                ""
            };
            let in_air = if (c.movement_flags & 0x008) != 0 {
                "X"
            } else {
                ""
            };
            let on_ground = if (c.movement_flags & 0x004) != 0 {
                "X"
            } else {
                ""
            };
            let reserved = if (c.movement_flags & 0x002) != 0 {
                "X"
            } else {
                ""
            };
            let may_move = if (c.movement_flags & 0x001) != 0 {
                "X"
            } else {
                ""
            };

            wtr.write_record(&[
                c.id.to_string(),
                c.name.clone(),
                c.age.to_string(),
                c.gender_id.to_string(),
                c.gender_name().to_string(),
                c.width.to_string(),
                c.depth.to_string(),
                c.max_mp.to_string(),
                c.element_id.to_string(),
                c.element_name().to_string(),
                "0".to_string(), // Pad 1
                c.weapon_id.to_string(),
                "".to_string(), // [Weapon Name] - would need item database
                c.armor_id.to_string(),
                "".to_string(), // [Armor Name]
                c.accessory_id.to_string(),
                "".to_string(), // [Accessory Name]
                format!("0b{:012b}", c.movement_flags),
                may_dodge.to_string(),
                unk_damage.to_string(),
                unk_ranged.to_string(),
                unk_melee.to_string(),
                ranged_atk.to_string(),
                melee_atk.to_string(),
                ranged_only.to_string(),
                take_cover.to_string(),
                in_air.to_string(),
                on_ground.to_string(),
                reserved.to_string(),
                may_move.to_string(),
                c.hp.to_string(),
                c.max_hp.to_string(),
                c.max_hp_growth.to_string(),
                c.sp.to_string(),
                c.max_sp.to_string(),
                c.counter_percent.to_string(),
                "0".to_string(), // Pad 2
                c.exp.to_string(),
                format!("{:.2}", c.max_mp_growth),
                format!("{:.1}", c.unknown1),
                // Element resistances
                c.element_resistances[0].to_string(),
                c.element_resistances[1].to_string(),
                c.element_resistances[2].to_string(),
                c.element_resistances[3].to_string(),
                c.element_resistances[4].to_string(),
                c.element_resistances[5].to_string(),
                // State resistances (15)
                c.state_resistances[0].to_string(),
                c.state_resistances[1].to_string(),
                c.state_resistances[2].to_string(),
                c.state_resistances[3].to_string(),
                c.state_resistances[4].to_string(),
                c.state_resistances[5].to_string(),
                c.state_resistances[6].to_string(),
                c.state_resistances[7].to_string(),
                c.state_resistances[8].to_string(),
                c.state_resistances[9].to_string(),
                c.state_resistances[10].to_string(),
                c.state_resistances[11].to_string(),
                c.state_resistances[12].to_string(),
                c.state_resistances[13].to_string(),
                c.state_resistances[14].to_string(),
                c.danger.to_string(),
                c.power.to_string(),
                c.will.to_string(),
                c.vigor.to_string(),
                c.agile.to_string(),
                c.quick.to_string(),
                "0".to_string(), // Pad 3
                format!("{:.2}", c.power_growth),
                format!("{:.2}", c.will_growth),
                format!("{:.2}", c.vigor_growth),
                format!("{:.2}", c.agile_growth),
                format!("{:.2}", c.quick_growth),
                c.magic_exp[0].to_string(),
                c.magic_exp[1].to_string(),
                c.magic_exp[2].to_string(),
                c.magic_exp[3].to_string(),
                c.magic_exp[4].to_string(),
                c.magic_exp[5].to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export character magic to CSV format.
    pub fn export_character_magic<W: Write>(magic: &[CharacterMagic], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Element ID",
            "[Element Name]",
            "Order",
            "Occasion Flags",
            "[M]",
            "[B]",
            "[S]",
            "Effect ID",
            "[Effect Name]",
            "Scope ID",
            "[Scope Name]",
            "Category ID",
            "[Category Name]",
            "Effect Speed",
            "Effect SP",
            "Pad 1",
            "Pad 2",
            "Effect Base",
            "Type ID",
            "[Type Name]",
            "State ID",
            "[State Name]",
            "State Miss%",
            "Pad 3",
            "Pad 4",
            "Pad 5",
            "Ship Occ ID",
            "[Ship Occ Name]",
            "Pad 6",
            "Ship Eff ID",
            "[Ship Eff Name]",
            "Ship Eff SP",
            "Ship Eff Turns",
            "Ship Eff Base",
            "Unk",
            "Pad 7",
            "Pad 8",
            "Pad 9",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
            "[Ship US Descr Pos]",
            "[Ship US Descr Size]",
            "Ship US Descr Str",
        ])?;

        for m in magic {
            // Decode occasion flags: M=4, B=2, S=1 (matching Ruby ALX)
            let menu = if m.occasion_flags & 0x04 != 0 {
                "X"
            } else {
                ""
            };
            let battle = if m.occasion_flags & 0x02 != 0 {
                "X"
            } else {
                ""
            };
            let ship = if m.occasion_flags & 0x01 != 0 {
                "X"
            } else {
                ""
            };

            let ship_occ_name = SHIP_OCCASION_NAMES.get(m.ship_occasion_id);
            let ship_eff_name = EFFECT_NAMES.get(m.ship_effect_id);

            wtr.write_record(&[
                m.id.to_string(),
                m.name.clone(),
                m.element_id.to_string(),
                ELEMENT_NAMES.get(m.element_id).to_string(),
                m.order.to_string(),
                format!("0b{:04b}", m.occasion_flags),
                menu.to_string(),
                battle.to_string(),
                ship.to_string(),
                m.effect_id.to_string(),
                EFFECT_NAMES.get(m.effect_id as i16).to_string(),
                m.scope_id.to_string(),
                SCOPE_NAMES.get(m.scope_id).to_string(),
                m.category_id.to_string(),
                crate::lookups::category_name(m.category_id).to_string(),
                m.effect_speed.to_string(),
                m.effect_sp.to_string(),
                "0".to_string(), // Pad 1
                "0".to_string(), // Pad 2
                m.effect_base.to_string(),
                m.type_id.to_string(),
                crate::lookups::type_name(m.type_id).to_string(),
                m.state_id.to_string(),
                STATE_NAMES.get(m.state_id).to_string(),
                m.state_miss.to_string(),
                "0".to_string(), // Pad 3
                "0".to_string(), // Pad 4
                "0".to_string(), // Pad 5
                m.ship_occasion_id.to_string(),
                ship_occ_name.to_string(),
                "0".to_string(), // Pad 6
                m.ship_effect_id.to_string(),
                ship_eff_name.to_string(),
                m.ship_effect_sp.to_string(),
                m.ship_effect_turns.to_string(),
                m.ship_effect_base.to_string(),
                m.unknown.to_string(),
                "0".to_string(), // Pad 7
                "0".to_string(), // Pad 8
                "0".to_string(), // Pad 9
                format!("0x{:x}", m.description_pos),
                m.description_size.to_string(),
                m.description.clone(),
                format!("0x{:x}", m.ship_description_pos),
                m.ship_description_size.to_string(),
                m.ship_description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export shops to CSV format matching original ALX output.
    ///
    /// The `item_db` is used to look up item names for each shop slot.
    pub fn export_shops<W: Write>(shops: &[Shop], writer: W, item_db: &ItemDatabase) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Build header: Entry ID, Pad 1, US SOT Pos, [US Descr Pos], [US Descr Size], US Descr Str, Item 1 ID, [Item 1 Name], ...
        let mut header = vec![
            "Entry ID".to_string(),
            "Pad 1".to_string(),
            "US SOT Pos".to_string(),
            "[US Descr Pos]".to_string(),
            "[US Descr Size]".to_string(),
            "US Descr Str".to_string(),
        ];
        for i in 1..=48 {
            header.push(format!("Item {} ID", i));
            header.push(format!("[Item {} Name]", i));
        }

        wtr.write_record(&header)?;

        for shop in shops {
            let mut row = vec![
                shop.id.to_string(),
                "0".to_string(), // Pad 1
                format!("0x{:08x}", shop.sot_pos),
                format!("0x{:x}", shop.description_pos),
                shop.description_size.to_string(),
                shop.description.clone(),
            ];

            // Add 48 item slots with names
            for i in 0..48 {
                let item_id = shop.item_ids.get(i).copied().unwrap_or(-1);
                row.push(item_id.to_string());
                row.push(item_db.name_or_default(item_id as i32));
            }

            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export treasure chests to CSV format.
    ///
    /// The `item_db` is used to look up item names.
    pub fn export_treasure_chests<W: Write>(
        chests: &[TreasureChest],
        writer: W,
        item_db: &ItemDatabase,
    ) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&["Entry ID", "Item ID", "[Item Name]", "Amount"])?;

        for chest in chests {
            let item_name = item_db.name_or_default(chest.item_id);
            wtr.write_record(&[
                chest.id.to_string(),
                chest.item_id.to_string(),
                item_name,
                chest.item_amount.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export crew members to CSV format.
    pub fn export_crew_members<W: Write>(crew: &[CrewMember], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Position ID",
            "[Position Name]",
            "Trait ID",
            "[Trait Name]",
            "Trait Value",
            "Ship Eff ID",
            "Ship Eff SP",
            "Ship Eff Turns",
            "Ship Eff Base",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for c in crew {
            // Use ship trait names for crew members
            let trait_name = crate::lookups::ship_trait_name(c.trait_id);

            wtr.write_record(&[
                c.id.to_string(),
                c.name.clone(),
                c.position_id.to_string(),
                c.position_name().to_string(),
                c.trait_id.to_string(),
                trait_name.to_string(),
                c.trait_value.to_string(),
                c.ship_effect_id.to_string(),
                c.ship_effect_sp.to_string(),
                c.ship_effect_turns.to_string(),
                c.ship_effect_base.to_string(),
                format!("0x{:x}", c.description_pos),
                c.description_size.to_string(),
                c.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export playable ships to CSV format.
    pub fn export_playable_ships<W: Write>(ships: &[PlayableShip], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "MAXHP",
            "MAXSP",
            "SP",
            "Defense",
            "MagDef",
            "Quick",
            "Dodge%",
            "Green",
            "Red",
            "Purple",
            "Blue",
            "Yellow",
            "Silver",
            "Cannon 1",
            "Cannon 2",
            "Cannon 3",
            "Cannon 4",
            "Cannon 5",
            "Accessory 1",
            "Accessory 2",
            "Accessory 3",
        ])?;

        for ship in ships {
            wtr.write_record(&[
                ship.id.to_string(),
                ship.name.clone(),
                ship.max_hp.to_string(),
                ship.max_sp.to_string(),
                ship.sp.to_string(),
                ship.defense.to_string(),
                ship.mag_def.to_string(),
                ship.quick.to_string(),
                ship.dodge.to_string(),
                ship.elements[0].to_string(),
                ship.elements[1].to_string(),
                ship.elements[2].to_string(),
                ship.elements[3].to_string(),
                ship.elements[4].to_string(),
                ship.elements[5].to_string(),
                ship.cannon_ids[0].to_string(),
                ship.cannon_ids[1].to_string(),
                ship.cannon_ids[2].to_string(),
                ship.cannon_ids[3].to_string(),
                ship.cannon_ids[4].to_string(),
                ship.accessory_ids[0].to_string(),
                ship.accessory_ids[1].to_string(),
                ship.accessory_ids[2].to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export ship cannons to CSV format.
    pub fn export_ship_cannons<W: Write>(cannons: &[ShipCannon], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Ship Flags",
            "[LJ]",
            "[Del]",
            "[Dra]",
            "[Esm]",
            "[Gil]",
            "[Aik]",
            "Type ID",
            "[Type Name]",
            "Element ID",
            "[Element Name]",
            "Attack",
            "Hit%",
            "Limit",
            "SP Cost",
            "Trait ID",
            "[Trait Name]",
            "Trait Value",
            "Buy",
            "Sell%",
            "US Order",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for c in cannons {
            // Ship flag bits: 0=LittleJack, 1=Delphinus, 2=Drakkar, 3=Esmeralda, 4=Gilder's, 5=Aika's
            let lj = if c.ship_flags & 0x01 != 0 { "X" } else { "" };
            let del = if c.ship_flags & 0x02 != 0 { "X" } else { "" };
            let dra = if c.ship_flags & 0x04 != 0 { "X" } else { "" };
            let esm = if c.ship_flags & 0x08 != 0 { "X" } else { "" };
            let gil = if c.ship_flags & 0x10 != 0 { "X" } else { "" };
            let aik = if c.ship_flags & 0x20 != 0 { "X" } else { "" };
            let trait_name = if c.trait_id < 0 {
                "None"
            } else {
                TRAIT_NAMES.get(c.trait_id as usize).unwrap_or(&"Unknown")
            };

            wtr.write_record(&[
                c.id.to_string(),
                c.name.clone(),
                format!("0b{:06b}", c.ship_flags),
                lj.to_string(),
                del.to_string(),
                dra.to_string(),
                esm.to_string(),
                gil.to_string(),
                aik.to_string(),
                c.type_id.to_string(),
                crate::lookups::cannon_type_name(c.type_id).to_string(),
                c.element_id.to_string(),
                ELEMENT_NAMES.get(c.element_id).to_string(),
                c.attack.to_string(),
                c.hit.to_string(),
                c.limit.to_string(),
                c.sp.to_string(),
                c.trait_id.to_string(),
                trait_name.to_string(),
                c.trait_value.to_string(),
                c.buy_price.to_string(),
                c.sell_percent.to_string(),
                c.order1.to_string(),
                format!("0x{:x}", c.description_pos),
                c.description_size.to_string(),
                c.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export ship accessories to CSV format.
    pub fn export_ship_accessories<W: Write>(
        accessories: &[ShipAccessory],
        writer: W,
    ) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Ship Flags",
            "[LJ]",
            "[Del]",
            "[Dra]",
            "[Esm]",
            "[Gil]",
            "[Aik]",
            "Trait 1 ID",
            "[Trait 1 Name]",
            "Trait 1 Value",
            "Trait 2 ID",
            "[Trait 2 Name]",
            "Trait 2 Value",
            "Trait 3 ID",
            "[Trait 3 Name]",
            "Trait 3 Value",
            "Trait 4 ID",
            "[Trait 4 Name]",
            "Trait 4 Value",
            "Buy",
            "Sell%",
            "US Order",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for acc in accessories {
            // Use ship trait names for ship accessories
            let ship_trait = |id: i8| -> &str { crate::lookups::ship_trait_name(id) };
            // Ship flag bits: 0=LittleJack, 1=Delphinus, 2=Drakkar, 3=Esmeralda, 4=Gilder's, 5=Aika's
            let lj = if acc.ship_flags & 0x01 != 0 { "X" } else { "" };
            let del = if acc.ship_flags & 0x02 != 0 { "X" } else { "" };
            let dra = if acc.ship_flags & 0x04 != 0 { "X" } else { "" };
            let esm = if acc.ship_flags & 0x08 != 0 { "X" } else { "" };
            let gil = if acc.ship_flags & 0x10 != 0 { "X" } else { "" };
            let aik = if acc.ship_flags & 0x20 != 0 { "X" } else { "" };

            wtr.write_record(&[
                acc.id.to_string(),
                acc.name.clone(),
                format!("0b{:06b}", acc.ship_flags),
                lj.to_string(),
                del.to_string(),
                dra.to_string(),
                esm.to_string(),
                gil.to_string(),
                aik.to_string(),
                acc.traits[0].id.to_string(),
                ship_trait(acc.traits[0].id).to_string(),
                acc.traits[0].value.to_string(),
                acc.traits[1].id.to_string(),
                ship_trait(acc.traits[1].id).to_string(),
                acc.traits[1].value.to_string(),
                acc.traits[2].id.to_string(),
                ship_trait(acc.traits[2].id).to_string(),
                acc.traits[2].value.to_string(),
                acc.traits[3].id.to_string(),
                ship_trait(acc.traits[3].id).to_string(),
                acc.traits[3].value.to_string(),
                acc.buy_price.to_string(),
                acc.sell_percent.to_string(),
                acc.order1.to_string(),
                format!("0x{:x}", acc.description_pos),
                acc.description_size.to_string(),
                acc.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export ship items to CSV format.
    pub fn export_ship_items<W: Write>(items: &[ShipItem], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Occasion Flags",
            "[M]",
            "[B]",
            "[S]",
            "Ship Eff ID",
            "Ship Eff Turns",
            "Consume%",
            "Buy",
            "Sell%",
            "US Order 1",
            "US Order 2",
            "Ship Eff Base",
            "Element ID",
            "[Element Name]",
            "Unk 1",
            "Unk 2",
            "Hit%",
            "[US Descr Pos]",
            "[US Descr Size]",
            "US Descr Str",
        ])?;

        for item in items {
            let m = if item.usable_in_menu() { "X" } else { "" };
            let b = if item.usable_in_battle() { "X" } else { "" };
            let s = if item.usable_on_ship() { "X" } else { "" };

            wtr.write_record(&[
                item.id.to_string(),
                item.name.clone(),
                format!("0b{:04b}", item.occasion_flags),
                m.to_string(),
                b.to_string(),
                s.to_string(),
                item.ship_effect_id.to_string(),
                item.ship_effect_turns.to_string(),
                item.consume.to_string(),
                item.buy_price.to_string(),
                item.sell_percent.to_string(),
                item.order1.to_string(),
                item.order2.to_string(),
                item.ship_effect_base.to_string(),
                item.element_id.to_string(),
                ELEMENT_NAMES.get(item.element_id).to_string(),
                item.unknown1.to_string(),
                item.unknown2.to_string(),
                item.hit.to_string(),
                format!("0x{:x}", item.description_pos),
                item.description_size.to_string(),
                item.description.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export enemy ships to CSV format.
    pub fn export_enemy_ships<W: Write>(ships: &[EnemyShip], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        let mut header = vec![
            "Entry ID",
            "Entry US Name",
            "MAXHP",
            "Will",
            "Defense",
            "MagDef",
            "Quick",
            "Agile",
            "Dodge%",
            "Green",
            "Red",
            "Purple",
            "Blue",
            "Yellow",
            "Silver",
        ];
        for i in 1..=4 {
            header.push(Box::leak(format!("Arm {} Type ID", i).into_boxed_str()));
            header.push(Box::leak(format!("Arm {} Attack", i).into_boxed_str()));
            header.push(Box::leak(format!("Arm {} Range", i).into_boxed_str()));
            header.push(Box::leak(format!("Arm {} Hit%", i).into_boxed_str()));
            header.push(Box::leak(format!("Arm {} Element ID", i).into_boxed_str()));
        }
        header.push("EXP");
        header.push("Gold");
        for i in 1..=3 {
            header.push(Box::leak(format!("Item Drop {} ID", i).into_boxed_str()));
            header.push(Box::leak(format!("Item {} ID", i).into_boxed_str()));
        }

        wtr.write_record(&header)?;

        for ship in ships {
            let mut row = vec![
                ship.id.to_string(),
                ship.name.clone(),
                ship.max_hp.to_string(),
                ship.will.to_string(),
                ship.defense.to_string(),
                ship.mag_def.to_string(),
                ship.quick.to_string(),
                ship.agile.to_string(),
                ship.dodge.to_string(),
                ship.elements[0].to_string(),
                ship.elements[1].to_string(),
                ship.elements[2].to_string(),
                ship.elements[3].to_string(),
                ship.elements[4].to_string(),
                ship.elements[5].to_string(),
            ];
            for arm in &ship.armaments {
                row.push(arm.type_id.to_string());
                row.push(arm.attack.to_string());
                row.push(arm.range.to_string());
                row.push(arm.hit.to_string());
                row.push(arm.element_id.to_string());
            }
            row.push(ship.exp.to_string());
            row.push(ship.gold.to_string());
            for drop in &ship.item_drops {
                row.push(drop.drop_id.to_string());
                row.push(drop.item_id.to_string());
            }
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export enemy magic to CSV format.
    pub fn export_enemy_magic<W: Write>(magic: &[EnemyMagic], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Category ID",
            "Effect ID",
            "[Effect Name]",
            "Scope ID",
            "[Scope Name]",
            "Effect Param ID",
            "Effect Base",
            "Element ID",
            "[Element Name]",
            "Type ID",
            "State Inflict ID",
            "State Resist ID",
            "State ID",
            "[State Name]",
            "State Miss%",
        ])?;

        for m in magic {
            wtr.write_record(&[
                m.id.to_string(),
                m.name.clone(),
                m.category_id.to_string(),
                m.effect_id.to_string(),
                m.effect_name().to_string(),
                m.scope_id.to_string(),
                SCOPE_NAMES.get(m.scope_id).to_string(),
                m.effect_param_id.to_string(),
                m.effect_base.to_string(),
                m.element_id.to_string(),
                m.element_name().to_string(),
                m.type_id.to_string(),
                m.state_infliction_id.to_string(),
                m.state_resistance_id.to_string(),
                m.state_id.to_string(),
                m.state_name().to_string(),
                m.state_miss.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export enemy super moves to CSV format.
    pub fn export_enemy_super_moves<W: Write>(moves: &[EnemySuperMove], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Category ID",
            "[Category Name]",
            "Effect ID",
            "[Effect Name]",
            "Scope ID",
            "[Scope Name]",
            "Effect Param ID",
            "Effect Base",
            "Element ID",
            "[Element Name]",
            "Type ID",
            "State Inflict ID",
            "State Resist ID",
            "State ID",
            "[State Name]",
            "State Miss%",
        ])?;

        for m in moves {
            wtr.write_record(&[
                m.id.to_string(),
                m.name.clone(),
                m.category_id.to_string(),
                m.category_name().to_string(),
                m.effect_id.to_string(),
                m.effect_name().to_string(),
                m.scope_id.to_string(),
                SCOPE_NAMES.get(m.scope_id).to_string(),
                m.effect_param_id.to_string(),
                m.effect_base.to_string(),
                m.element_id.to_string(),
                m.element_name().to_string(),
                m.type_id.to_string(),
                m.state_infliction_id.to_string(),
                m.state_resistance_id.to_string(),
                m.state_id.to_string(),
                m.state_name().to_string(),
                m.state_miss.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export swashbucklers to CSV format.
    pub fn export_swashbucklers<W: Write>(ratings: &[Swashbuckler], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "Entry US Name",
            "Rating",
            "Regular Atk",
            "S-Move Atk",
            "Dodge%",
            "Run%",
        ])?;

        for r in ratings {
            wtr.write_record(&[
                r.id.to_string(),
                r.name.clone(),
                r.rating.to_string(),
                r.regular_attack.to_string(),
                r.super_move_attack.to_string(),
                r.dodge.to_string(),
                r.run.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export spirit curves to CSV format.
    pub fn export_spirit_curves<W: Write>(curves: &[SpiritCurve], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Build header with SP/MAXSP for each level
        let mut header = vec!["Entry ID".to_string(), "[PC Name]".to_string()];
        for i in 1..=99 {
            header.push(format!("SP {}", i));
            header.push(format!("MAXSP {}", i));
        }
        wtr.write_record(&header)?;

        for curve in curves {
            let mut row = vec![curve.id.to_string(), curve.character_name.clone()];
            for level in &curve.levels {
                row.push(level.sp.to_string());
                row.push(level.max_sp.to_string());
            }
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export exp boosts to CSV format.
    pub fn export_exp_boosts<W: Write>(boosts: &[ExpBoost], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Entry ID",
            "[PC Name]",
            "EXP",
            "Green EXP",
            "Red EXP",
            "Purple EXP",
            "Blue EXP",
            "Yellow EXP",
            "Silver EXP",
        ])?;

        for b in boosts {
            wtr.write_record(&[
                b.id.to_string(),
                b.character_name.clone(),
                b.exp.to_string(),
                b.green_exp.to_string(),
                b.red_exp.to_string(),
                b.purple_exp.to_string(),
                b.blue_exp.to_string(),
                b.yellow_exp.to_string(),
                b.silver_exp.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export enemies to CSV format matching original ALX format.
    ///
    /// The `item_db` is used to look up item names for drops.
    /// The `enemy_names` map converts enemy IDs to US names.
    pub fn export_enemies<W: Write>(
        enemies: &[Enemy],
        writer: W,
        item_db: &ItemDatabase,
        enemy_names: &std::collections::HashMap<u32, String>,
    ) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Header matching original ALX exactly
        wtr.write_record(&[
            "Entry ID",
            "[Filter]",
            "Entry JP Name",
            "[Entry US Name]",
            "Width",
            "Depth",
            "Element ID",
            "[Element Name]",
            "Pad 1",
            "Pad 2",
            "Movement Flags",
            "[May Dodge]",
            "[Unk Damage]",
            "[Unk Ranged]",
            "[Unk Melee]",
            "[Ranged Atk]",
            "[Melee Atk]",
            "[Ranged Only]",
            "[Take Cover]",
            "[In Air]",
            "[On Ground]",
            "[Reserved]",
            "[May Move]",
            "Counter%",
            "EXP",
            "Gold",
            "Pad 3",
            "Pad 4",
            "MAXHP",
            "Unk 1",
            "Green",
            "Red",
            "Purple",
            "Blue",
            "Yellow",
            "Silver",
            "Poison",
            "Unconscious",
            "Stone",
            "Sleep",
            "Confusion",
            "Silence",
            "Fatigue",
            "Revival",
            "Weak",
            "State 10",
            "State 11",
            "State 12",
            "State 13",
            "State 14",
            "State 15",
            "Danger",
            "Effect ID",
            "[Effect Name]",
            "State ID",
            "[State Name]",
            "State Miss%",
            "Pad 5",
            "Level",
            "Will",
            "Vigor",
            "Agile",
            "Quick",
            "Attack",
            "Defense",
            "MagDef",
            "Hit%",
            "Dodge%",
            "Pad 6",
            "Pad 7",
            "Item 1 Prob",
            "Item 1 Amount",
            "Item 1 ID",
            "[Item 1 Name]",
            "Item 2 Prob",
            "Item 2 Amount",
            "Item 2 ID",
            "[Item 2 Name]",
            "Item 3 Prob",
            "Item 3 Amount",
            "Item 3 ID",
            "[Item 3 Name]",
            "Item 4 Prob",
            "Item 4 Amount",
            "Item 4 ID",
            "[Item 4 Name]",
        ])?;

        // Process enemies: determine which entries should be marked as global ('*')
        // and sort by ID, then by file order
        let mut processed = process_enemy_filters(enemies);

        // Sort by ID, then global first, then by file name
        processed.sort_by(|(filter_a, a), (filter_b, b)| {
            let id_cmp = a.id.cmp(&b.id);
            if id_cmp != std::cmp::Ordering::Equal {
                return id_cmp;
            }
            // '*' filter comes first
            let a_is_global = *filter_a == "*";
            let b_is_global = *filter_b == "*";
            if a_is_global != b_is_global {
                return b_is_global.cmp(&a_is_global);
            }
            // Then by file name
            filter_a.cmp(filter_b)
        });

        let sorted = processed;

        for (filter, e) in sorted {
            // Movement flags as individual columns
            let may_dodge = if e.may_dodge() { "X" } else { "" };
            let unk_damage = if e.unk_damage() { "X" } else { "" };
            let unk_ranged = if e.unk_ranged() { "X" } else { "" };
            let unk_melee = if e.unk_melee() { "X" } else { "" };
            let ranged_atk = if e.ranged_atk() { "X" } else { "" };
            let melee_atk = if e.melee_atk() { "X" } else { "" };
            let ranged_only = if e.ranged_only() { "X" } else { "" };
            let take_cover = if e.take_cover() { "X" } else { "" };
            let in_air = if e.in_air() { "X" } else { "" };
            let on_ground = if e.on_ground() { "X" } else { "" };
            let reserved = if e.reserved() { "X" } else { "" };
            let may_move = if e.may_move() { "X" } else { "" };

            // US name lookup
            let us_name = enemy_names.get(&e.id).map(|s| s.as_str()).unwrap_or("???");

            // Item name lookups
            let item_name = |drop: &crate::entries::EnemyItemDrop| -> String {
                item_db.name_or_default(drop.item_id as i32)
            };

            wtr.write_record(&[
                e.id.to_string(),
                filter.clone(),
                e.name_jp.clone(),
                us_name.to_string(),
                e.width.to_string(),
                e.depth.to_string(),
                e.element_id.to_string(),
                e.element_name().to_string(),
                "-1".to_string(), // Pad 1
                "-1".to_string(), // Pad 2
                format!("0b{:012b}", e.movement_flags),
                may_dodge.to_string(),
                unk_damage.to_string(),
                unk_ranged.to_string(),
                unk_melee.to_string(),
                ranged_atk.to_string(),
                melee_atk.to_string(),
                ranged_only.to_string(),
                take_cover.to_string(),
                in_air.to_string(),
                on_ground.to_string(),
                reserved.to_string(),
                may_move.to_string(),
                e.counter.to_string(),
                e.exp.to_string(),
                e.gold.to_string(),
                "-1".to_string(), // Pad 3
                "-1".to_string(), // Pad 4
                e.max_hp.to_string(),
                format!("{:.1}", e.unknown_float),
                e.elements[0].to_string(),
                e.elements[1].to_string(),
                e.elements[2].to_string(),
                e.elements[3].to_string(),
                e.elements[4].to_string(),
                e.elements[5].to_string(),
                // State resistances (15 states)
                e.states[0].to_string(),
                e.states[1].to_string(),
                e.states[2].to_string(),
                e.states[3].to_string(),
                e.states[4].to_string(),
                e.states[5].to_string(),
                e.states[6].to_string(),
                e.states[7].to_string(),
                e.states[8].to_string(),
                e.states[9].to_string(),
                e.states[10].to_string(),
                e.states[11].to_string(),
                e.states[12].to_string(),
                e.states[13].to_string(),
                e.states[14].to_string(),
                e.danger.to_string(),
                e.effect_id.to_string(),
                e.effect_name().to_string(),
                e.state_id.to_string(),
                e.state_name().to_string(),
                e.state_miss.to_string(),
                "-1".to_string(), // Pad 5
                e.level.to_string(),
                e.will.to_string(),
                e.vigor.to_string(),
                e.agile.to_string(),
                e.quick.to_string(),
                e.attack.to_string(),
                e.defense.to_string(),
                e.mag_def.to_string(),
                e.hit.to_string(),
                e.dodge.to_string(),
                "-1".to_string(), // Pad 6
                "-1".to_string(), // Pad 7
                e.item_drops[0].probability.to_string(),
                e.item_drops[0].amount.to_string(),
                e.item_drops[0].item_id.to_string(),
                item_name(&e.item_drops[0]),
                e.item_drops[1].probability.to_string(),
                e.item_drops[1].amount.to_string(),
                e.item_drops[1].item_id.to_string(),
                item_name(&e.item_drops[1]),
                e.item_drops[2].probability.to_string(),
                e.item_drops[2].amount.to_string(),
                e.item_drops[2].item_id.to_string(),
                item_name(&e.item_drops[2]),
                e.item_drops[3].probability.to_string(),
                e.item_drops[3].amount.to_string(),
                e.item_drops[3].item_id.to_string(),
                item_name(&e.item_drops[3]),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export enemy tasks to CSV format.
    pub fn export_enemy_tasks<W: Write>(tasks: &[EnemyTask], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        wtr.write_record(&[
            "Task ID",
            "[Filter]",
            "Enemy ID",
            "Type ID",
            "[Type Name]",
            "Task ID",
            "Param ID",
        ])?;

        for t in tasks {
            wtr.write_record(&[
                t.id.to_string(),
                t.filter.clone(),
                t.enemy_id.to_string(),
                t.type_id.to_string(),
                t.type_name().to_string(),
                t.task_id.to_string(),
                t.param_id.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export EXP curves to CSV format matching original ALX output.
    pub fn export_exp_curves<W: Write>(curves: &[ExpCurve], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Build header: Entry ID, [PC Name], EXP 1-99
        let mut header = vec!["Entry ID".to_string(), "[PC Name]".to_string()];
        for i in 1..=99 {
            header.push(format!("EXP {}", i));
        }
        wtr.write_record(&header)?;

        for curve in curves {
            let mut row = vec![curve.id.to_string(), curve.character_name.clone()];
            for &exp in curve.exp_values.iter().take(99) {
                row.push(exp.to_string());
            }
            // Pad to 99 values if needed
            while row.len() < 101 {
                row.push("0".to_string());
            }
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export Magic EXP curves to CSV format matching original ALX output.
    pub fn export_magic_exp_curves<W: Write>(curves: &[MagicExpCurve], writer: W) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        // Build header: Entry ID, [PC Name], then 6 levels for each of 6 elements
        let mut header = vec!["Entry ID".to_string(), "[PC Name]".to_string()];
        let elements = ["Green", "Red", "Purple", "Blue", "Yellow", "Silver"];
        for element in &elements {
            for level in 1..=6 {
                header.push(format!("{} EXP {}", element, level));
            }
        }
        wtr.write_record(&header)?;

        for curve in curves {
            let mut row = vec![curve.id.to_string(), curve.character_name.clone()];
            for &exp in &curve.green_exp {
                row.push(exp.to_string());
            }
            for &exp in &curve.red_exp {
                row.push(exp.to_string());
            }
            for &exp in &curve.purple_exp {
                row.push(exp.to_string());
            }
            for &exp in &curve.blue_exp {
                row.push(exp.to_string());
            }
            for &exp in &curve.yellow_exp {
                row.push(exp.to_string());
            }
            for &exp in &curve.silver_exp {
                row.push(exp.to_string());
            }
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }
}

/// Determine file order based on file type (matching Ruby ALX logic).
/// - ENP files or '*': order 0
/// - EVP files: order 1
/// - EB/EC/DAT files: order 2
fn file_order(filter: &str) -> u8 {
    let lower = filter.to_lowercase();
    if filter == "*" || lower.ends_with("_ep.enp") || lower.ends_with("_ep.bin") {
        0
    } else if lower.ends_with(".evp") {
        1
    } else if lower.ends_with(".eb") || lower.ends_with(".ec") || lower.ends_with(".dat") {
        2
    } else {
        // Default order for other files
        3
    }
}

/// Process enemy filters to mark global entries with '*'.
/// For each enemy ID, the entry with the lowest file order gets marked as global.
fn process_enemy_filters(enemies: &[Enemy]) -> Vec<(String, &Enemy)> {
    use std::collections::HashMap;

    // Group enemies by ID, tracking the best candidate for global status
    let mut best_per_id: HashMap<u32, (u8, usize)> = HashMap::new(); // id -> (best_order, index)

    for (idx, enemy) in enemies.iter().enumerate() {
        let order = file_order(&enemy.filter);
        let entry = best_per_id.entry(enemy.id).or_insert((order, idx));
        if order < entry.0 {
            *entry = (order, idx);
        }
    }

    // Determine which indices should become global
    let global_indices: std::collections::HashSet<usize> = best_per_id
        .values()
        .filter(|(order, _)| *order <= 2) // Only mark as global if order <= 2
        .map(|(_, idx)| *idx)
        .collect();

    // Build result with updated filters
    enemies
        .iter()
        .enumerate()
        .map(|(idx, enemy)| {
            let filter = if global_indices.contains(&idx) {
                "*".to_string()
            } else {
                enemy.filter.clone()
            };
            (filter, enemy)
        })
        .collect()
}
