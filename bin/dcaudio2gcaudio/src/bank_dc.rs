//! Dreamcast `.MLT` ("SMLT") AICA sound-bank parser.
//!
//! An MLT is a Sega "Multi-Unit" bank: an `SMLT` header + entry table describing
//! what loads into AICA sound RAM, followed by the chunk bodies. Relevant chunks:
//! - `SMPB` — instrument/program bank (holds AICA 4-bit ADPCM waveform data)
//! - `SMSB` — sample bank: header + `SMSD` per-sample records (AICA voice params)
//! - `SFOB`/`SFPW`/`SFPB` — AICA DSP effect programs

use byteorder::{ByteOrder, LittleEndian as LE};

/// One entry in the `SMLT` table: a chunk destined for AICA RAM.
#[derive(Debug, Clone)]
pub struct SmltEntry {
    pub tag: [u8; 4],
    pub index: u32,
    /// Destination address in AICA sound RAM (not a file offset).
    pub ram_addr: u32,
    pub size: u32,
}

impl SmltEntry {
    pub fn tag_str(&self) -> String {
        String::from_utf8_lossy(&self.tag).to_string()
    }
}

/// A parsed `SMSD` sample record (raw fields; AICA params decoded lazily).
#[derive(Debug, Clone)]
pub struct SmsdRecord {
    /// Offset of the record within the SMSB body.
    pub offset: usize,
    pub raw: Vec<u8>,
}

/// A parsed DC MLT bank.
#[derive(Debug)]
pub struct MltBank {
    pub version: u32,
    pub entries: Vec<SmltEntry>,
    /// File offset of the `SMSB` body, if present.
    pub smsb_offset: Option<usize>,
    pub num_samples: usize,
    pub samples: Vec<SmsdRecord>,
}

/// All file offsets where `magic` occurs at/after `min_off`.
fn find_all(data: &[u8], magic: &[u8; 4], min_off: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = min_off;
    while i + 4 <= data.len() {
        if &data[i..i + 4] == magic {
            out.push(i);
        }
        i += 1;
    }
    out
}

/// Find the real `SMSB` body: the candidate whose header is self-consistent
/// (`num_samples` sane and the first sample offset == header+table size). The
/// MLT also contains `SMSB` tags in its secondary descriptor tables, which fail
/// this check.
fn find_smsb_body(data: &[u8], min_off: usize) -> Option<usize> {
    for o in find_all(data, b"SMSB", min_off) {
        if o + 0x14 > data.len() {
            continue;
        }
        let n = LE::read_u32(&data[o + 0xc..o + 0x10]) as usize;
        if n == 0 || n > 4096 || o + 0x10 + n * 4 > data.len() {
            continue;
        }
        let first = LE::read_u32(&data[o + 0x10..o + 0x14]) as usize;
        if first == 0x10 + n * 4 {
            return Some(o);
        }
    }
    None
}

/// Parse an MLT file's structure (header table + SMSB sample list).
pub fn parse(data: &[u8]) -> Result<MltBank, String> {
    if data.len() < 0x10 || &data[0..4] != b"SMLT" {
        return Err("not an SMLT file".into());
    }
    let version = LE::read_u32(&data[4..8]);
    let count = LE::read_u32(&data[8..12]) as usize;

    let mut entries = Vec::new();
    let table_end = 0x10 + count * 0x10;
    for i in 0..count {
        let o = 0x10 + i * 0x10;
        if o + 0x10 > data.len() {
            break;
        }
        let tag: [u8; 4] = data[o..o + 4].try_into().unwrap();
        if tag == [0xff, 0xff, 0xff, 0xff] {
            continue;
        }
        entries.push(SmltEntry {
            tag,
            index: LE::read_u32(&data[o + 4..o + 8]),
            ram_addr: LE::read_u32(&data[o + 8..o + 12]),
            size: LE::read_u32(&data[o + 12..o + 16]),
        });
    }

    // Locate the SMSB chunk body (after the entry table) and parse its samples.
    let mut num_samples = 0;
    let mut samples = Vec::new();
    let smsb_offset = find_smsb_body(data, table_end);
    if let Some(o) = smsb_offset {
        // SMSB: magic, u32 ver, u32 size, u32 num_samples, then num_samples u32
        // offsets (relative to SMSB start) to SMSD records.
        if o + 0x10 <= data.len() {
            num_samples = LE::read_u32(&data[o + 0xc..o + 0x10]) as usize;
            let mut offs = Vec::with_capacity(num_samples);
            for k in 0..num_samples {
                let to = o + 0x10 + k * 4;
                if to + 4 > data.len() {
                    break;
                }
                offs.push(LE::read_u32(&data[to..to + 4]) as usize);
            }
            for (k, &rel) in offs.iter().enumerate() {
                let start = o + rel;
                let end = if k + 1 < offs.len() {
                    o + offs[k + 1]
                } else {
                    data.len()
                };
                if start <= end && end <= data.len() {
                    samples.push(SmsdRecord {
                        offset: rel,
                        raw: data[start..end].to_vec(),
                    });
                }
            }
        }
    }

    Ok(MltBank {
        version,
        entries,
        smsb_offset,
        num_samples,
        samples,
    })
}

impl MltBank {
    /// Human-readable summary for `--inspect`.
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "SMLT v{} — {} entries\n",
            self.version,
            self.entries.len()
        ));
        for e in &self.entries {
            s.push_str(&format!(
                "  {:<5} idx={:<2} ram=0x{:06x} size=0x{:x}\n",
                e.tag_str(),
                e.index,
                e.ram_addr,
                e.size
            ));
        }
        match self.smsb_offset {
            Some(o) => s.push_str(&format!("SMSB @0x{:x}: {} samples\n", o, self.num_samples)),
            None => s.push_str("SMSB: none\n"),
        }
        for (i, r) in self.samples.iter().take(4).enumerate() {
            let head: Vec<String> = r.raw.iter().take(16).map(|b| format!("{b:02x}")).collect();
            s.push_str(&format!(
                "  sample[{i}] off=0x{:x} len={} : {}\n",
                r.offset,
                r.raw.len(),
                head.join(" ")
            ));
        }
        s
    }
}
