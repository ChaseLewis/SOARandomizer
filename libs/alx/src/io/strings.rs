//! String reading utilities for game data.

use std::io::Cursor;

use crate::error::Result;

/// Read a null-terminated string with 4-byte block alignment.
/// This matches the ALX behavior: read until null, then align to next 4-byte boundary.
pub fn read_aligned_string(
    cursor: &mut Cursor<&[u8]>,
    block_size: usize,
) -> Result<(String, usize)> {
    let start_pos = cursor.position() as usize;
    let data = cursor.get_ref();

    // Find the null terminator
    let mut end_pos = start_pos;
    while end_pos < data.len() && data[end_pos] != 0 {
        end_pos += 1;
    }

    // Read the string bytes (excluding null terminator)
    let string_bytes = &data[start_pos..end_pos];

    // Decode as Windows-1252 (common for US game data)
    let text = decode_windows1252(string_bytes);

    // Calculate the total size with block alignment
    let raw_size = end_pos - start_pos + 1; // +1 for null terminator
    let aligned_size = if block_size > 1 {
        raw_size.div_ceil(block_size) * block_size
    } else {
        raw_size
    };

    // Seek past the aligned block
    cursor.set_position((start_pos + aligned_size) as u64);

    Ok((text, aligned_size))
}

/// Decode bytes using appropriate encoding based on content.
/// Handles Japanese (Shift-JIS) and Western (Windows-1252) text.
/// Also performs character translation to match original ALX output.
pub fn decode_game_string(bytes: &[u8]) -> String {
    use encoding_rs::SHIFT_JIS;

    // Check if text has Shift-JIS quote sequences embedded in otherwise ASCII/Latin text
    // These sequences need to be converted before decoding
    let has_shiftjis_quotes = has_shiftjis_quote_sequences(bytes);

    // Check if text is primarily ASCII (with possible embedded Shift-JIS quotes)
    // ASCII is a subset of both encodings, so check for non-ASCII non-quote bytes
    let is_primarily_ascii = bytes.iter().all(|&b| {
        b < 0x80 || b == 0x81 // Allow ASCII and the Shift-JIS prefix byte
    });

    if has_shiftjis_quotes && is_primarily_ascii {
        // This is ASCII/Latin text with embedded Shift-JIS quotes - use Windows-1252 mode
        return decode_as_windows1252_with_shiftjis_quotes(bytes);
    }

    // Try pure Shift-JIS decoding
    let (decoded, _, had_errors) = SHIFT_JIS.decode(bytes);

    let text = if !had_errors {
        // Pure Shift-JIS (likely Japanese text)
        decoded.trim_end_matches('\0').to_string()
    } else {
        // Fall back to Windows-1252 with quote translation
        decode_as_windows1252_with_shiftjis_quotes(bytes)
    };

    // ALX always translates [] to "" (curly quotes) for all text
    translate_brackets_to_quotes(&text)
}

/// Translate square brackets to curly quotes, matching ALX behavior.
fn translate_brackets_to_quotes(text: &str) -> String {
    text.replace('[', "\u{201C}") // Left double quotation mark
        .replace(']', "\u{201D}") // Right double quotation mark
}

/// Check if bytes contain Shift-JIS quote sequences (0x81 0x67, 0x81 0x68, etc.)
fn has_shiftjis_quote_sequences(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len().saturating_sub(1) {
        if bytes[i] == 0x81 {
            match bytes[i + 1] {
                0x63 | 0x66 | 0x67 | 0x68 | 0x73 | 0x74 | 0x40 => return true,
                _ => {}
            }
        }
        i += 1;
    }
    false
}

/// Decode bytes as Windows-1252, first converting Shift-JIS quote sequences.
fn decode_as_windows1252_with_shiftjis_quotes(bytes: &[u8]) -> String {
    use encoding_rs::WINDOWS_1252;

    // Pre-process: convert Shift-JIS quote characters to ASCII/Windows-1252 replacements
    let mut processed = bytes.to_vec();
    let mut i = 0;
    while i < processed.len().saturating_sub(1) {
        if processed[i] == 0x81 {
            let replacement = match processed[i + 1] {
                0x63 => Some(0x85),  // ellipsis
                0x66 => Some(b'\''), // single quote
                0x67 => Some(b'['),  // left quote
                0x68 => Some(b']'),  // right quote
                0x73 => Some(0xab),  // left guillemet
                0x74 => Some(0xbb),  // right guillemet
                0x40 => Some(0x7f),  // ideographic space
                _ => None,
            };
            if let Some(repl) = replacement {
                processed[i] = repl;
                processed.remove(i + 1);
            }
        }
        i += 1;
    }

    // Decode as Windows-1252
    let (decoded, _, _) = WINDOWS_1252.decode(&processed);
    decoded.trim_end_matches('\0').to_string()
}

/// Decode bytes as Windows-1252 (legacy, use decode_game_string for new code).
pub fn decode_windows1252(bytes: &[u8]) -> String {
    decode_game_string(bytes)
}

/// Read all description strings from a description data range.
/// Returns a vector of (position, size, text) tuples.
pub fn read_description_strings(
    data: &[u8],
    base_offset: usize,
    count: usize,
    block_size: usize,
) -> Result<Vec<(u32, u32, String)>> {
    let mut cursor = Cursor::new(data);
    let mut descriptions = Vec::with_capacity(count);

    for _ in 0..count {
        if cursor.position() as usize >= data.len() {
            break;
        }

        let pos = base_offset + cursor.position() as usize;
        let (text, size) = read_aligned_string(&mut cursor, block_size)?;

        descriptions.push((pos as u32, size as u32, text));
    }

    Ok(descriptions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_aligned_string() {
        // "Hi\0\0" - 2 chars + null + 1 padding = 4 bytes
        let data = b"Hi\x00\x00Next";
        let mut cursor = Cursor::new(&data[..]);

        let (text, size) = read_aligned_string(&mut cursor, 4).unwrap();
        assert_eq!(text, "Hi");
        assert_eq!(size, 4);
        assert_eq!(cursor.position(), 4);
    }

    #[test]
    fn test_read_description_strings() {
        // Two strings with 4-byte alignment
        // "Hi\0\0" (4 bytes) + "Test\0\0\0\0" (8 bytes)
        let data = b"Hi\x00\x00Test\x00\x00\x00\x00";

        let descriptions = read_description_strings(data, 0x1000, 2, 4).unwrap();

        assert_eq!(descriptions.len(), 2);
        assert_eq!(descriptions[0], (0x1000, 4, "Hi".to_string()));
        assert_eq!(descriptions[1], (0x1004, 8, "Test".to_string()));
    }
}
