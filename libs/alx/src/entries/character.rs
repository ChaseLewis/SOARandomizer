//! Playable character entry type.

use std::io::Cursor;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::game::region::GameVersion;
use crate::game::offsets::id_ranges;
use crate::io::BinaryReader;

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
    
    // Field offsets within entry (name at 0-10 is NEVER written)
    const OFF_AGE: usize = 11;
    const OFF_GENDER_ID: usize = 12;
    const OFF_WIDTH: usize = 13;
    const OFF_DEPTH: usize = 14;
    const OFF_MAX_MP: usize = 15;
    const OFF_ELEMENT_ID: usize = 16;
    // 17 = pad1
    const OFF_WEAPON_ID: usize = 18;
    const OFF_ARMOR_ID: usize = 20;
    const OFF_ACCESSORY_ID: usize = 22;
    const OFF_MOVEMENT_FLAGS: usize = 24;
    const OFF_HP: usize = 26;
    const OFF_MAX_HP: usize = 28;
    const OFF_MAX_HP_GROWTH: usize = 30;
    const OFF_SP: usize = 32;
    const OFF_MAX_SP: usize = 34;
    const OFF_COUNTER_PERCENT: usize = 36;
    // 38 = pad2
    const OFF_EXP: usize = 40;
    const OFF_MAX_MP_GROWTH: usize = 44;
    const OFF_UNKNOWN1: usize = 48;
    const OFF_ELEMENT_RESISTANCES: usize = 52;
    const OFF_STATE_RESISTANCES: usize = 64;
    const OFF_DANGER: usize = 94;
    const OFF_POWER: usize = 96;
    const OFF_WILL: usize = 98;
    const OFF_VIGOR: usize = 100;
    const OFF_AGILE: usize = 102;
    const OFF_QUICK: usize = 104;
    // 106 = pad3
    const OFF_POWER_GROWTH: usize = 108;
    const OFF_WILL_GROWTH: usize = 112;
    const OFF_VIGOR_GROWTH: usize = 116;
    const OFF_AGILE_GROWTH: usize = 120;
    const OFF_QUICK_GROWTH: usize = 124;
    const OFF_MAGIC_EXP: usize = 128;

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

    /// Patch a single character entry in a mutable buffer.
    /// Only writes numeric fields at their exact offsets - strings and padding are untouched.
    pub fn patch_entry(&self, buf: &mut [u8]) {
        // Helper to write BE values at offset
        fn put_i8(buf: &mut [u8], off: usize, v: i8) { buf[off] = v as u8; }
        fn put_u16(buf: &mut [u8], off: usize, v: u16) { buf[off..off+2].copy_from_slice(&v.to_be_bytes()); }
        fn put_i16(buf: &mut [u8], off: usize, v: i16) { buf[off..off+2].copy_from_slice(&v.to_be_bytes()); }
        fn put_u32(buf: &mut [u8], off: usize, v: u32) { buf[off..off+4].copy_from_slice(&v.to_be_bytes()); }
        fn put_i32(buf: &mut [u8], off: usize, v: i32) { buf[off..off+4].copy_from_slice(&v.to_be_bytes()); }
        fn put_f32(buf: &mut [u8], off: usize, v: f32) { buf[off..off+4].copy_from_slice(&v.to_be_bytes()); }
        
        // Write only numeric fields (skip name at 0-10, skip padding bytes)
        put_i8(buf, Self::OFF_AGE, self.age);
        put_i8(buf, Self::OFF_GENDER_ID, self.gender_id);
        put_i8(buf, Self::OFF_WIDTH, self.width);
        put_i8(buf, Self::OFF_DEPTH, self.depth);
        put_i8(buf, Self::OFF_MAX_MP, self.max_mp);
        put_i8(buf, Self::OFF_ELEMENT_ID, self.element_id);
        put_u16(buf, Self::OFF_WEAPON_ID, self.weapon_id);
        put_u16(buf, Self::OFF_ARMOR_ID, self.armor_id);
        put_u16(buf, Self::OFF_ACCESSORY_ID, self.accessory_id);
        put_i16(buf, Self::OFF_MOVEMENT_FLAGS, self.movement_flags);
        put_i16(buf, Self::OFF_HP, self.hp);
        put_i16(buf, Self::OFF_MAX_HP, self.max_hp);
        put_i16(buf, Self::OFF_MAX_HP_GROWTH, self.max_hp_growth);
        put_i16(buf, Self::OFF_SP, self.sp);
        put_i16(buf, Self::OFF_MAX_SP, self.max_sp);
        put_i16(buf, Self::OFF_COUNTER_PERCENT, self.counter_percent);
        put_u32(buf, Self::OFF_EXP, self.exp);
        put_f32(buf, Self::OFF_MAX_MP_GROWTH, self.max_mp_growth);
        put_f32(buf, Self::OFF_UNKNOWN1, self.unknown1);
        
        // Element resistances
        for (i, &res) in self.element_resistances.iter().enumerate() {
            put_i16(buf, Self::OFF_ELEMENT_RESISTANCES + i * 2, res);
        }
        
        // State resistances
        for (i, &res) in self.state_resistances.iter().enumerate() {
            put_i16(buf, Self::OFF_STATE_RESISTANCES + i * 2, res);
        }
        
        put_i16(buf, Self::OFF_DANGER, self.danger);
        put_i16(buf, Self::OFF_POWER, self.power);
        put_i16(buf, Self::OFF_WILL, self.will);
        put_i16(buf, Self::OFF_VIGOR, self.vigor);
        put_i16(buf, Self::OFF_AGILE, self.agile);
        put_i16(buf, Self::OFF_QUICK, self.quick);
        
        put_f32(buf, Self::OFF_POWER_GROWTH, self.power_growth);
        put_f32(buf, Self::OFF_WILL_GROWTH, self.will_growth);
        put_f32(buf, Self::OFF_VIGOR_GROWTH, self.vigor_growth);
        put_f32(buf, Self::OFF_AGILE_GROWTH, self.agile_growth);
        put_f32(buf, Self::OFF_QUICK_GROWTH, self.quick_growth);
        
        // Magic EXP
        for (i, &exp) in self.magic_exp.iter().enumerate() {
            put_i32(buf, Self::OFF_MAGIC_EXP + i * 4, exp);
        }
    }

    /// Patch all character entries into a buffer.
    /// Buffer must be the original DOL section data.
    pub fn patch_all(chars: &[Self], buf: &mut [u8]) {
        for c in chars {
            let start = c.id as usize * Self::ENTRY_SIZE;
            let end = start + Self::ENTRY_SIZE;
            if end <= buf.len() {
                c.patch_entry(&mut buf[start..end]);
            }
        }
    }
}
