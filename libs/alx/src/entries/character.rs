//! Playable character entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::{BinaryReader, BinaryWriter};

/// Playable character entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// Entry ID (0-5 for the 6 playable characters)
    pub id: u32,
    /// Character name
    pub name: String,
    /// Age
    pub age: i8,
    /// Gender ID (0=Male, 1=Female)
    pub gender_id: i8,
    /// Width (collision)
    pub width: i8,
    /// Depth (collision)
    pub depth: i8,
    /// Max MP
    pub max_mp: i8,
    /// Element ID (affinity)
    pub element_id: i8,
    /// Equipped weapon ID
    pub weapon_id: u16,
    /// Equipped armor ID
    pub armor_id: u16,
    /// Equipped accessory ID
    pub accessory_id: u16,
    /// Movement flags
    pub movement_flags: i16,
    /// Current HP
    pub hp: i16,
    /// Max HP
    pub max_hp: i16,
    /// Max HP growth per level
    pub max_hp_growth: i16,
    /// Current SP
    pub sp: i16,
    /// Max SP
    pub max_sp: i16,
    /// Counter percentage
    pub counter_percent: i16,
    /// Current EXP
    pub exp: u32,
    /// Max MP growth per level
    pub max_mp_growth: f32,
    /// Unknown float
    pub unknown1: f32,
    /// Element resistances (Green, Red, Purple, Blue, Yellow, Silver)
    pub element_resistances: [i16; 6],
    /// State resistances (15 states)
    pub state_resistances: [i16; 15],
    /// Danger rating
    pub danger: i16,
    /// Power stat
    pub power: i16,
    /// Will stat
    pub will: i16,
    /// Vigor stat
    pub vigor: i16,
    /// Agile stat
    pub agile: i16,
    /// Quick stat
    pub quick: i16,
    /// Power growth per level
    pub power_growth: f32,
    /// Will growth per level
    pub will_growth: f32,
    /// Vigor growth per level
    pub vigor_growth: f32,
    /// Agile growth per level
    pub agile_growth: f32,
    /// Quick growth per level
    pub quick_growth: f32,
    /// Magic EXP per element
    pub magic_exp: [i32; 6],
}

impl Default for Character {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            age: 0,
            gender_id: 0,
            width: 0,
            depth: 0,
            max_mp: 0,
            element_id: 0,
            weapon_id: 0,
            armor_id: 0,
            accessory_id: 0,
            movement_flags: 0,
            hp: 0,
            max_hp: 0,
            max_hp_growth: 0,
            sp: 0,
            max_sp: 0,
            counter_percent: 0,
            exp: 0,
            max_mp_growth: 0.0,
            unknown1: 0.0,
            element_resistances: [0; 6],
            state_resistances: [0; 15],
            danger: 0,
            power: 0,
            will: 0,
            vigor: 0,
            agile: 0,
            quick: 0,
            power_growth: 0.0,
            will_growth: 0.0,
            vigor_growth: 0.0,
            agile_growth: 0.0,
            quick_growth: 0.0,
            magic_exp: [0; 6],
        }
    }
}

impl Character {
    /// Size of one character entry in bytes.
    /// Based on Ruby: 11 + many fields = ~152 bytes
    pub const ENTRY_SIZE: usize = 152;

    /// Get gender name.
    pub fn gender_name(&self) -> &'static str {
        match self.gender_id {
            0 => "Male",
            1 => "Female",
            _ => "???",
        }
    }

    /// Get element name.
    pub fn element_name(&self) -> &'static str {
        crate::lookups::element_name(self.element_id)
    }

    /// Read a single character entry from binary data.
    pub fn read_one(cursor: &mut Cursor<&[u8]>, id: u32, _version: &GameVersion) -> Result<Self> {
        let name = cursor.read_string_fixed(11)?;
        let age = cursor.read_i8()?;
        let gender_id = cursor.read_i8()?;
        let width = cursor.read_i8()?;
        let depth = cursor.read_i8()?;
        let max_mp = cursor.read_i8()?;
        let element_id = cursor.read_i8()?;
        let _pad1 = cursor.read_u8()?;
        let weapon_id = cursor.read_u16_be()?;
        let armor_id = cursor.read_u16_be()?;
        let accessory_id = cursor.read_u16_be()?;
        let movement_flags = cursor.read_i16_be()?;
        let hp = cursor.read_i16_be()?;
        let max_hp = cursor.read_i16_be()?;
        let max_hp_growth = cursor.read_i16_be()?;
        let sp = cursor.read_i16_be()?;
        let max_sp = cursor.read_i16_be()?;
        let counter_percent = cursor.read_i16_be()?;
        let _pad2 = cursor.read_i16_be()?;
        let exp = cursor.read_u32_be()?;
        let max_mp_growth = cursor.read_f32_be()?;
        let unknown1 = cursor.read_f32_be()?;
        
        let mut element_resistances = [0i16; 6];
        for i in 0..6 {
            element_resistances[i] = cursor.read_i16_be()?;
        }
        
        let mut state_resistances = [0i16; 15];
        for i in 0..15 {
            state_resistances[i] = cursor.read_i16_be()?;
        }
        
        let danger = cursor.read_i16_be()?;
        let power = cursor.read_i16_be()?;
        let will = cursor.read_i16_be()?;
        let vigor = cursor.read_i16_be()?;
        let agile = cursor.read_i16_be()?;
        let quick = cursor.read_i16_be()?;
        let _pad3 = cursor.read_i16_be()?;
        
        let power_growth = cursor.read_f32_be()?;
        let will_growth = cursor.read_f32_be()?;
        let vigor_growth = cursor.read_f32_be()?;
        let agile_growth = cursor.read_f32_be()?;
        let quick_growth = cursor.read_f32_be()?;
        
        let mut magic_exp = [0i32; 6];
        for i in 0..6 {
            magic_exp[i] = cursor.read_i32_be()?;
        }
        
        Ok(Self {
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
        })
    }

    /// Read all character entries from binary data.
    pub fn read_all_data(data: &[u8], version: &GameVersion) -> Result<Vec<Self>> {
        let mut characters = Vec::new();
        let mut cursor = Cursor::new(data);
        
        let id_range = id_ranges::CHARACTER;
        
        for id in id_range {
            if cursor.position() as usize + Self::ENTRY_SIZE > data.len() {
                break;
            }
            let character = Self::read_one(&mut cursor, id, version)?;
            characters.push(character);
        }
        
        Ok(characters)
    }

    /// Write a single character entry to binary data.
    pub fn write_one<W: BinaryWriter>(&self, writer: &mut W, _version: &GameVersion) -> Result<()> {
        writer.write_string_fixed(&self.name, 11)?;
        writer.write_i8(self.age)?;
        writer.write_i8(self.gender_id)?;
        writer.write_i8(self.width)?;
        writer.write_i8(self.depth)?;
        writer.write_i8(self.max_mp)?;
        writer.write_i8(self.element_id)?;
        writer.write_u8(0)?; // pad1
        writer.write_u16_be(self.weapon_id)?;
        writer.write_u16_be(self.armor_id)?;
        writer.write_u16_be(self.accessory_id)?;
        writer.write_i16_be(self.movement_flags)?;
        writer.write_i16_be(self.hp)?;
        writer.write_i16_be(self.max_hp)?;
        writer.write_i16_be(self.max_hp_growth)?;
        writer.write_i16_be(self.sp)?;
        writer.write_i16_be(self.max_sp)?;
        writer.write_i16_be(self.counter_percent)?;
        writer.write_i16_be(0)?; // pad2
        writer.write_u32_be(self.exp)?;
        writer.write_f32_be(self.max_mp_growth)?;
        writer.write_f32_be(self.unknown1)?;
        
        for &res in &self.element_resistances {
            writer.write_i16_be(res)?;
        }
        
        for &res in &self.state_resistances {
            writer.write_i16_be(res)?;
        }
        
        writer.write_i16_be(self.danger)?;
        writer.write_i16_be(self.power)?;
        writer.write_i16_be(self.will)?;
        writer.write_i16_be(self.vigor)?;
        writer.write_i16_be(self.agile)?;
        writer.write_i16_be(self.quick)?;
        writer.write_i16_be(0)?; // pad3
        
        writer.write_f32_be(self.power_growth)?;
        writer.write_f32_be(self.will_growth)?;
        writer.write_f32_be(self.vigor_growth)?;
        writer.write_f32_be(self.agile_growth)?;
        writer.write_f32_be(self.quick_growth)?;
        
        for &exp in &self.magic_exp {
            writer.write_i32_be(exp)?;
        }
        
        Ok(())
    }

    /// Write all character entries to binary data.
    pub fn write_all_data<W: BinaryWriter>(chars: &[Self], writer: &mut W, version: &GameVersion) -> Result<()> {
        for c in chars {
            c.write_one(writer, version)?;
        }
        Ok(())
    }
}
