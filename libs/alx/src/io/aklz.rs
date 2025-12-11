//! AKLZ compression/decompression for GameCube files.
//!
//! AKLZ is a sliding window LZ77-style compression format used in
//! Skies of Arcadia Legends for GameCube.

use crate::error::{Error, Result};

/// AKLZ file signature (12 bytes)
const FILE_SIG: [u8; 12] = [
    0x41, 0x4B, 0x4C, 0x5A,  // "AKLZ"
    0x7E, 0x3F, 0x51, 0x64,
    0x3D, 0xCC, 0xCC, 0xCD,
];

/// Begin of match amount
const MATCH_BEG: usize = 3;

/// Size of match amount mask
const MATCH_SIZE: u8 = 0x0F;

/// Size of sliding window buffer
const BUFFER_SIZE: usize = 0x1000;

/// Initial buffer pointer position
const BUFFER_BEG: usize = BUFFER_SIZE - (MATCH_BEG + MATCH_SIZE as usize);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_aklz() {
        let compressed = [
            0x41, 0x4B, 0x4C, 0x5A, 0x7E, 0x3F, 0x51, 0x64,
            0x3D, 0xCC, 0xCC, 0xCD, 0x00, 0x00, 0x00, 0x10,
        ];
        assert!(is_aklz(&compressed));
        
        let not_compressed = [0x00, 0x00, 0xFF, 0xFF, 0x00];
        assert!(!is_aklz(&not_compressed));
    }
}

