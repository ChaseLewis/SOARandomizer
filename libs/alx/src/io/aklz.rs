//! AKLZ compression/decompression for GameCube files.
//!
//! AKLZ is a sliding window LZ77-style compression format used in
//! Skies of Arcadia Legends for GameCube.

use crate::error::{Error, Result};

/// AKLZ file signature (12 bytes)
const FILE_SIG: [u8; 12] = [
    0x41, 0x4B, 0x4C, 0x5A, // "AKLZ"
    0x7E, 0x3F, 0x51, 0x64, 0x3D, 0xCC, 0xCC, 0xCD,
];

/// Begin of match amount (minimum match length)
const MATCH_BEG: usize = 3;

/// Size of match amount mask (max additional match length)
const MATCH_SIZE: u8 = 0x0F;

/// Maximum match length
const MAX_MATCH_LEN: usize = MATCH_BEG + MATCH_SIZE as usize; // 3 + 15 = 18

/// Size of sliding window buffer
const BUFFER_SIZE: usize = 0x1000;

/// Initial buffer pointer position
const BUFFER_BEG: usize = BUFFER_SIZE - MAX_MATCH_LEN;

/// Check if data is AKLZ compressed.
pub fn is_aklz(data: &[u8]) -> bool {
    data.len() >= 16 && data[..12] == FILE_SIG
}

/// Decompress AKLZ data.
/// Returns the decompressed data, or the original data if not compressed.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    // Check signature
    if !is_aklz(data) {
        // Not AKLZ compressed, return as-is
        return Ok(data.to_vec());
    }

    // Read decompressed size from header (offset 12, big-endian u32)
    let file_size = u32::from_be_bytes([data[12], data[13], data[14], data[15]]) as usize;

    let mut output = Vec::with_capacity(file_size);
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut buffer_ptr = BUFFER_BEG;

    let mut pos = 16; // Start after header

    while pos < data.len() && output.len() < file_size {
        let flag = data[pos];
        pos += 1;

        for i in 0..8 {
            if pos >= data.len() || output.len() >= file_size {
                break;
            }

            // Check if this chunk is compressed (bit 0 = 0) or raw (bit 0 = 1)
            if (flag >> i) & 1 == 0 {
                // Compressed: read 2 bytes for match position and size
                if pos + 1 >= data.len() {
                    break;
                }
                let b1 = data[pos] as usize;
                let b2 = data[pos + 1] as usize;
                pos += 2;

                let match_pos = b1 | ((b2 & !MATCH_SIZE as usize) << 4);
                let mut match_size = (b2 & MATCH_SIZE as usize) + MATCH_BEG;

                // Limit match size to remaining output
                match_size = match_size.min(file_size - output.len());

                // Copy from buffer
                for j in 0..match_size {
                    let byte = buffer[(match_pos + j) & (BUFFER_SIZE - 1)];
                    output.push(byte);

                    buffer[buffer_ptr] = byte;
                    buffer_ptr = (buffer_ptr + 1) & (BUFFER_SIZE - 1);
                }
            } else {
                // Not compressed: copy raw byte
                let byte = data[pos];
                pos += 1;

                output.push(byte);
                buffer[buffer_ptr] = byte;
                buffer_ptr = (buffer_ptr + 1) & (BUFFER_SIZE - 1);
            }
        }
    }

    if output.len() != file_size {
        return Err(Error::ParseError {
            offset: 12,
            message: format!(
                "AKLZ decompression size mismatch: got {} bytes, expected {}",
                output.len(),
                file_size
            ),
        });
    }

    Ok(output)
}

/// Compress data using AKLZ format.
///
/// This produces output compatible with the game's decompressor.
/// Based on the Ruby ALX implementation.
pub fn compress(data: &[u8]) -> Vec<u8> {
    // Pre-allocate output with header
    let mut output = Vec::with_capacity(data.len() + 16);

    // Write header
    output.extend_from_slice(&FILE_SIG);
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());

    // Initialize sliding window buffer (filled with zeros initially)
    // The buffer is indexed with (BUFFER_BEG + logical_pos) to match decompressor's initial state
    let mut buffer = vec![0u8; BUFFER_SIZE];

    // Logical position in the sliding window (0-based, like Ruby implementation)
    let mut buffer_ptr: usize = 0;

    let mut pos = 0;

    while pos < data.len() {
        // Process up to 8 chunks per flag byte
        let mut flag: u8 = 0;
        let mut chunk_data: Vec<u8> = Vec::with_capacity(16);

        for bit in 0..8 {
            if pos >= data.len() {
                break;
            }

            // Cannot find matches if we haven't processed enough data yet
            // or if there isn't enough data left (need at least MATCH_BEG bytes)
            let can_match = pos >= MATCH_BEG && (data.len() - pos) >= MATCH_BEG;

            let (match_pos, match_len) = if can_match {
                find_match(&buffer, buffer_ptr, pos, data)
            } else {
                (0, 0)
            };

            if match_len >= MATCH_BEG {
                // Found a match - encode as 2 bytes
                // The match position is in buffer coordinates (offset by BUFFER_BEG)
                let buffer_pos = (BUFFER_BEG + match_pos) & (BUFFER_SIZE - 1);

                let len_encoded = (match_len - MATCH_BEG) as u8;
                let b1 = (buffer_pos & 0xFF) as u8;
                let b2 = ((buffer_pos >> 4) as u8 & 0xF0) | len_encoded;

                chunk_data.push(b1);
                chunk_data.push(b2);

                // Update buffer with matched bytes
                for i in 0..match_len {
                    let phys_pos = (BUFFER_BEG + buffer_ptr) & (BUFFER_SIZE - 1);
                    buffer[phys_pos] = data[pos + i];
                    buffer_ptr += 1;
                }
                pos += match_len;

                // Flag bit 0 = compressed
            } else {
                // No match - store literal byte
                let byte = data[pos];
                chunk_data.push(byte);

                let phys_pos = (BUFFER_BEG + buffer_ptr) & (BUFFER_SIZE - 1);
                buffer[phys_pos] = byte;
                buffer_ptr += 1;
                pos += 1;

                // Flag bit 1 = literal
                flag |= 1 << bit;
            }
        }

        // Write flag byte followed by chunk data
        output.push(flag);
        output.extend_from_slice(&chunk_data);
    }

    output
}

/// Find the best match in the sliding window buffer.
/// Returns (logical_match_position, match_length).
///
/// The match position is a logical offset (0-based), which will be converted
/// to buffer coordinates by adding BUFFER_BEG.
///
/// Key constraint: match_pos + match_len < buffer_ptr (prevents overlapping reads)
fn find_match(buffer: &[u8], buffer_ptr: usize, data_pos: usize, data: &[u8]) -> (usize, usize) {
    let remaining = data.len() - data_pos;
    if remaining < MATCH_BEG {
        return (0, 0);
    }

    let max_len = remaining.min(MAX_MATCH_LEN);
    let mut best_pos = 0;
    let mut best_len = 0;

    // Search through all positions we've written to
    // Start from the beginning to find older matches first
    let search_start = buffer_ptr.saturating_sub(BUFFER_SIZE);

    for match_start in search_start..buffer_ptr {
        // Key constraint from Ruby: match_start + match_size < buffer_ptr
        // This prevents overlapping reads
        let max_match_at_pos = (buffer_ptr - match_start).min(max_len);
        if max_match_at_pos < MATCH_BEG {
            continue;
        }

        let mut len = 0;

        // Check how many bytes match
        while len < max_match_at_pos {
            let phys_pos = (BUFFER_BEG + match_start + len) & (BUFFER_SIZE - 1);
            let buffer_byte = buffer[phys_pos];
            if buffer_byte != data[data_pos + len] {
                break;
            }
            len += 1;
        }

        // Keep track of best match (prefer longer matches)
        if len >= MATCH_BEG && len > best_len {
            best_len = len;
            best_pos = match_start;

            // Early exit if we found maximum length match
            if best_len == MAX_MATCH_LEN {
                break;
            }
        }
    }

    (best_pos, best_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_aklz() {
        let compressed = [
            0x41, 0x4B, 0x4C, 0x5A, 0x7E, 0x3F, 0x51, 0x64, 0x3D, 0xCC, 0xCC, 0xCD, 0x00, 0x00,
            0x00, 0x10,
        ];
        assert!(is_aklz(&compressed));

        let not_compressed = [0x00, 0x00, 0xFF, 0xFF, 0x00];
        assert!(!is_aklz(&not_compressed));
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        // Test repeated data
        let original = vec![0u8; 100];
        let compressed = compress(&original);
        assert!(is_aklz(&compressed));
        assert_eq!(decompress(&compressed).unwrap(), original);

        // Test sequential bytes
        let original: Vec<u8> = (0..=255).collect();
        let compressed = compress(&original);
        assert_eq!(decompress(&compressed).unwrap(), original);

        // Test data with repeated subsequences
        let mut original = Vec::new();
        for _ in 0..10 {
            original.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        }
        let compressed = compress(&original);
        assert_eq!(decompress(&compressed).unwrap(), original);
    }

    #[test]
    fn test_compress_mixed_data() {
        // Test data pattern similar to ENP files:
        // - lots of 255s (empty slots)
        // - some zeros interspersed
        // - structured data
        let mut original = Vec::new();

        // Header-like data
        original.extend_from_slice(&[0, 0, 0, 99, 0, 0, 2, 220]);

        // Lots of 255s
        for _ in 0..500 {
            original.push(255);
        }

        // Some zeros in the middle
        original.extend_from_slice(&[0, 0, 10, 156]);

        // More 255s
        for _ in 0..100 {
            original.push(255);
        }

        // More structured data
        original.extend_from_slice(&[102, 2, 42, 255, 255, 255]);

        let compressed = compress(&original);
        let decompressed = decompress(&compressed).unwrap();

        // Find first difference if any
        for (i, (a, b)) in original.iter().zip(decompressed.iter()).enumerate() {
            if a != b {
                // Show context
                let start = i.saturating_sub(5);
                let end = (i + 5).min(original.len());
                eprintln!("Context around position {}:", i);
                eprintln!("  Original:     {:?}", &original[start..end]);
                eprintln!("  Decompressed: {:?}", &decompressed[start..end]);
                panic!(
                    "Mismatch at position {}: original={}, decompressed={}",
                    i, a, b
                );
            }
        }
        assert_eq!(original.len(), decompressed.len());
    }
}
