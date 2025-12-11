//! Binary reading and writing utilities with endianness support.
//!
//! GameCube uses big-endian byte order.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use crate::error::{Error, Result};

/// Extension trait for reading binary data in big-endian format.
pub trait BinaryReader: Read + Seek {
    /// Read an unsigned 8-bit integer.
    fn read_u8(&mut self) -> io::Result<u8> {
        ReadBytesExt::read_u8(self)
    }

    /// Read a signed 8-bit integer.
    fn read_i8(&mut self) -> io::Result<i8> {
        ReadBytesExt::read_i8(self)
    }

    /// Read an unsigned 16-bit integer (big-endian).
    fn read_u16_be(&mut self) -> io::Result<u16> {
        ReadBytesExt::read_u16::<BigEndian>(self)
    }

    /// Read a signed 16-bit integer (big-endian).
    fn read_i16_be(&mut self) -> io::Result<i16> {
        ReadBytesExt::read_i16::<BigEndian>(self)
    }

    /// Read an unsigned 32-bit integer (big-endian).
    fn read_u32_be(&mut self) -> io::Result<u32> {
        ReadBytesExt::read_u32::<BigEndian>(self)
    }

    /// Read a signed 32-bit integer (big-endian).
    fn read_i32_be(&mut self) -> io::Result<i32> {
        ReadBytesExt::read_i32::<BigEndian>(self)
    }

    /// Read a 32-bit float (big-endian).
    fn read_f32_be(&mut self) -> io::Result<f32> {
        ReadBytesExt::read_f32::<BigEndian>(self)
    }

    /// Read a fixed-length string, decoding from Shift-JIS to UTF-8.
    /// The string is null-terminated within the fixed length.
    fn read_string_fixed(&mut self, len: usize) -> Result<String> {
        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;

        // Find null terminator
        let end = buf.iter().position(|&b| b == 0).unwrap_or(len);
        let bytes = &buf[..end];

        if bytes.is_empty() {
            return Ok(String::new());
        }

        // Decode from Shift-JIS
        let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(bytes);
        if had_errors {
            // Try Windows-1252 as fallback (used in some EU versions)
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            Ok(decoded.into_owned())
        } else {
            Ok(decoded.into_owned())
        }
    }

    /// Read exact bytes into a buffer.
    fn read_bytes(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Get current position in the stream.
    fn position(&mut self) -> io::Result<u64> {
        self.stream_position()
    }

    /// Seek to an absolute position.
    fn seek_to(&mut self, pos: u64) -> io::Result<u64> {
        self.seek(SeekFrom::Start(pos))
    }
}

/// Extension trait for writing binary data in big-endian format.
pub trait BinaryWriter: Write + Seek {
    /// Write an unsigned 8-bit integer.
    fn write_u8(&mut self, v: u8) -> io::Result<()> {
        WriteBytesExt::write_u8(self, v)
    }

    /// Write a signed 8-bit integer.
    fn write_i8(&mut self, v: i8) -> io::Result<()> {
        WriteBytesExt::write_i8(self, v)
    }

    /// Write an unsigned 16-bit integer (big-endian).
    fn write_u16_be(&mut self, v: u16) -> io::Result<()> {
        WriteBytesExt::write_u16::<BigEndian>(self, v)
    }

    /// Write a signed 16-bit integer (big-endian).
    fn write_i16_be(&mut self, v: i16) -> io::Result<()> {
        WriteBytesExt::write_i16::<BigEndian>(self, v)
    }

    /// Write an unsigned 32-bit integer (big-endian).
    fn write_u32_be(&mut self, v: u32) -> io::Result<()> {
        WriteBytesExt::write_u32::<BigEndian>(self, v)
    }

    /// Write a signed 32-bit integer (big-endian).
    fn write_i32_be(&mut self, v: i32) -> io::Result<()> {
        WriteBytesExt::write_i32::<BigEndian>(self, v)
    }

    /// Write a 32-bit float (big-endian).
    fn write_f32_be(&mut self, v: f32) -> io::Result<()> {
        WriteBytesExt::write_f32::<BigEndian>(self, v)
    }

    /// Write a fixed-length string, encoding from UTF-8 to Shift-JIS.
    /// Pads with nulls if shorter than the fixed length.
    fn write_string_fixed(&mut self, s: &str, len: usize) -> Result<()> {
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(s);
        let bytes = encoded.as_ref();

        if bytes.len() > len {
            return Err(Error::ValidationError(format!(
                "String too long: {} bytes, max {} bytes",
                bytes.len(),
                len
            )));
        }

        self.write_all(bytes)?;

        // Pad with nulls
        let padding = len - bytes.len();
        if padding > 0 {
            self.write_all(&vec![0u8; padding])?;
        }

        Ok(())
    }
}

// Implement traits for common types
impl<T: Read + Seek> BinaryReader for T {}
impl<T: Write + Seek> BinaryWriter for T {}

/// A cursor over a byte buffer for in-memory binary operations.
#[allow(dead_code)]
pub type ByteCursor = Cursor<Vec<u8>>;

/// Create a new cursor for reading from a byte slice.
#[allow(dead_code)]
pub fn read_cursor(data: &[u8]) -> Cursor<&[u8]> {
    Cursor::new(data)
}

/// Create a new cursor for reading/writing to a byte vector.
#[allow(dead_code)]
pub fn write_cursor() -> ByteCursor {
    Cursor::new(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u16_be() {
        let data = [0x12, 0x34];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u16_be().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u32_be() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u32_be().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_string_fixed() {
        // ASCII string with null padding
        let data = b"Hello\x00\x00\x00";
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_string_fixed(8).unwrap(), "Hello");
    }

    #[test]
    fn test_write_string_fixed() {
        let mut cursor = write_cursor();
        cursor.write_string_fixed("Hi", 5).unwrap();
        let result = cursor.into_inner();
        assert_eq!(result, vec![b'H', b'i', 0, 0, 0]);
    }
}
