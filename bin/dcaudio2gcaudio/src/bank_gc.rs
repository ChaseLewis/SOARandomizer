//! GameCube `.info` + `.samp` sound-bank parser.
//!
//! `.samp` holds the PCM16 waveform data. `.info` holds, near as the spike
//! determined, a big-endian u16 offset table followed by fixed 0x30-byte
//! per-instrument records (incrementing index). The exact field semantics
//! (level/pan/loop/rate) are still being reverse-engineered — this parser
//! exposes the structure so `--inspect` can pin them down.

use byteorder::{BigEndian as BE, ByteOrder};

/// Size of each per-instrument record in `.info`.
pub const INFO_RECORD_SIZE: usize = 0x30;

#[derive(Debug, Clone)]
pub struct InfoRecord {
    pub offset: usize,
    pub raw: [u8; INFO_RECORD_SIZE],
}

#[derive(Debug)]
pub struct GcBank {
    /// Leading big-endian u16 offset table (one per sample).
    pub offset_table: Vec<u16>,
    /// File offset where the per-instrument records begin.
    pub records_start: usize,
    pub records: Vec<InfoRecord>,
    pub samp_len: usize,
}

/// Parse a GC `.info` (+ `.samp` length) bank.
pub fn parse(info: &[u8], samp_len: usize) -> Result<GcBank, String> {
    if info.len() < 4 {
        return Err("empty .info".into());
    }

    // Leading offset table: rising BE u16 values until the first zero.
    let mut offset_table = Vec::new();
    let mut i = 0;
    while i + 2 <= info.len() {
        let v = BE::read_u16(&info[i..i + 2]);
        if i > 0 && v == 0 {
            break;
        }
        offset_table.push(v);
        i += 2;
    }

    // Records begin at the next non-zero 4-aligned position after the table.
    let mut records_start = i;
    while records_start < info.len() && info[records_start] == 0 {
        records_start += 1;
    }
    records_start &= !3; // align down to 4

    let mut records = Vec::new();
    let mut o = records_start;
    while o + INFO_RECORD_SIZE <= info.len() {
        let raw: [u8; INFO_RECORD_SIZE] = info[o..o + INFO_RECORD_SIZE].try_into().unwrap();
        // Stop at an all-zero record (end padding).
        if raw.iter().all(|&b| b == 0) {
            break;
        }
        records.push(InfoRecord { offset: o, raw });
        o += INFO_RECORD_SIZE;
    }

    Ok(GcBank {
        offset_table,
        records_start,
        records,
        samp_len,
    })
}

impl GcBank {
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            ".info: {} offset-table entries, {} records @0x{:x}; .samp {} bytes\n",
            self.offset_table.len(),
            self.records.len(),
            self.records_start,
            self.samp_len
        ));
        let head: Vec<String> = self
            .offset_table
            .iter()
            .take(12)
            .map(|v| format!("0x{v:x}"))
            .collect();
        s.push_str(&format!("  offsets[0..12]: {}\n", head.join(" ")));
        for (i, r) in self.records.iter().take(4).enumerate() {
            let hx: Vec<String> = r.raw.iter().map(|b| format!("{b:02x}")).collect();
            s.push_str(&format!("  rec[{i}] @0x{:x}: {}\n", r.offset, hx.join(" ")));
        }
        s
    }
}
