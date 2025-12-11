//! ISO file abstraction for reading/writing GameCube disc images.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Header offsets in the ISO.
const HEADER_INFO_OFFSET: u64 = 0x420;

/// Represents an opened GameCube ISO file.
pub struct IsoFile {
    path: PathBuf,
    file: File,
}

impl IsoFile {
    /// Open an ISO file for reading.
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            path: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
            file,
        })
    }

    /// Get the path to the ISO file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the DOL (executable) offset from the ISO header.
    pub fn read_dol_offset(&mut self) -> Result<u32> {
        self.file.seek(SeekFrom::Start(HEADER_INFO_OFFSET))?;
        let mut buf = [0u8; 4];
        self.file.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Read the FST (file system table) offset from the ISO header.
    pub fn read_fst_offset(&mut self) -> Result<u32> {
        self.file.seek(SeekFrom::Start(HEADER_INFO_OFFSET + 4))?;
        let mut buf = [0u8; 4];
        self.file.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Read raw bytes from a specific offset in the ISO.
    pub fn read_bytes_at(&mut self, offset: u64, len: usize) -> Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; len];
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Read the game ID from the ISO header (first 6 bytes).
    pub fn read_game_id(&mut self) -> Result<String> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8; 6];
        self.file.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    /// Read the game title from the ISO header.
    pub fn read_game_title(&mut self) -> Result<String> {
        self.file.seek(SeekFrom::Start(0x20))?;
        let mut buf = [0u8; 0x3E0];
        self.file.read_exact(&mut buf)?;
        
        // Find null terminator
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        Ok(String::from_utf8_lossy(&buf[..end]).to_string())
    }

    /// Extract a file from the ISO to a destination path.
    pub fn extract_file(&self, iso_path: &Path, dest_path: &Path) -> Result<()> {
        // Ensure destination directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let files = [(iso_path, dest_path)];
        gc_fst::read_iso_files(self.path(), &files)?;
        Ok(())
    }

    /// Replace a file in the ISO with new content.
    pub fn replace_file(&self, iso_path: &Path, source_path: &Path) -> Result<()> {
        let ops = [gc_fst::IsoOp::Insert {
            iso_path,
            input_path: source_path,
        }];
        gc_fst::operate_on_iso(self.path(), &ops)?;
        Ok(())
    }

    /// Write file data to a path in the ISO.
    /// This writes to a temp file and replaces the ISO file.
    pub fn write_file(&self, iso_path: &Path, data: &[u8]) -> Result<()> {
        // Create a unique temp file
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_file = temp_dir.join(format!(
            "alx_write_{}_{}",
            unique_id,
            iso_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        
        std::fs::write(&temp_file, data)?;
        self.replace_file(iso_path, &temp_file)?;
        let _ = std::fs::remove_file(&temp_file);
        
        Ok(())
    }

    /// Read a file directly from the ISO into memory.
    /// This extracts to a temp file and reads it.
    pub fn read_file(&self, iso_path: &Path) -> Result<Vec<u8>> {
        // Create a unique temp file path
        let temp_dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_file = temp_dir.join(format!(
            "alx_{}_{}",
            unique_id,
            iso_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        
        self.extract_file(iso_path, &temp_file)?;
        let data = std::fs::read(&temp_file)?;
        let _ = std::fs::remove_file(&temp_file);
        
        Ok(data)
    }
}

/// Read the Start.dol executable from the ISO.
/// Note: gc_fst uses "Start.dol" as a special path identifier.
#[allow(dead_code)]
pub fn read_start_dol(iso: &IsoFile) -> Result<Vec<u8>> {
    let dol_path = Path::new("Start.dol");
    iso.read_file(dol_path)
}

/// A file entry in the ISO filesystem.
#[derive(Debug, Clone)]
pub struct IsoFileEntry {
    /// Path within the ISO
    pub path: PathBuf,
    /// Offset in the ISO
    pub offset: u32,
    /// Size in bytes
    pub size: u32,
}

impl IsoFile {
    /// List all files in the ISO filesystem.
    pub fn list_files(&mut self) -> Result<Vec<IsoFileEntry>> {
        // Read header info
        self.file.seek(SeekFrom::Start(HEADER_INFO_OFFSET))?;
        let mut buf = [0u8; 12];
        self.file.read_exact(&mut buf)?;
        let fst_offset = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let fs_size = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        
        // Read entry count
        self.file.seek(SeekFrom::Start((fst_offset + 8) as u64))?;
        let mut u32_buf = [0u8; 4];
        self.file.read_exact(&mut u32_buf)?;
        let entry_count = u32::from_be_bytes(u32_buf);
        
        let string_table_offset = fst_offset + entry_count * 0xC;
        let entry_start_offset = fst_offset + 0xC;
        
        // Read FST data
        let string_table_offset_in_buf = string_table_offset - entry_start_offset;
        self.file.seek(SeekFrom::Start(entry_start_offset as u64))?;
        let mut fst_buf = vec![0u8; fs_size as usize];
        self.file.read_exact(&mut fst_buf)?;
        
        let mut files = Vec::new();
        let mut dir_end_indices: Vec<u32> = Vec::with_capacity(8);
        let mut offset = 0u32;
        let mut entry_index = 1u32;
        let mut path = PathBuf::with_capacity(64);
        
        while offset < string_table_offset_in_buf {
            while Some(entry_index) == dir_end_indices.last().copied() {
                dir_end_indices.pop();
                path.pop();
            }
            
            let is_file = fst_buf[offset as usize] == 0;
            
            let name_offset = u32::from_be_bytes([
                0,
                fst_buf[offset as usize + 1],
                fst_buf[offset as usize + 2],
                fst_buf[offset as usize + 3],
            ]);
            
            // Read filename from string table
            let name_start = (string_table_offset_in_buf + name_offset) as usize;
            let mut name_end = name_start;
            while name_end < fst_buf.len() && fst_buf[name_end] != 0 {
                name_end += 1;
            }
            let name = String::from_utf8_lossy(&fst_buf[name_start..name_end]).to_string();
            
            if is_file {
                let file_offset = u32::from_be_bytes([
                    fst_buf[offset as usize + 4],
                    fst_buf[offset as usize + 5],
                    fst_buf[offset as usize + 6],
                    fst_buf[offset as usize + 7],
                ]);
                let file_size = u32::from_be_bytes([
                    fst_buf[offset as usize + 8],
                    fst_buf[offset as usize + 9],
                    fst_buf[offset as usize + 10],
                    fst_buf[offset as usize + 11],
                ]);
                
                let mut file_path = path.clone();
                file_path.push(&name);
                
                files.push(IsoFileEntry {
                    path: file_path,
                    offset: file_offset,
                    size: file_size,
                });
            } else {
                path.push(&name);
                let next_idx = u32::from_be_bytes([
                    fst_buf[offset as usize + 8],
                    fst_buf[offset as usize + 9],
                    fst_buf[offset as usize + 10],
                    fst_buf[offset as usize + 11],
                ]);
                dir_end_indices.push(next_idx);
            }
            
            offset += 0xC;
            entry_index += 1;
        }
        
        Ok(files)
    }
    
    /// List files matching a pattern (substring matching).
    pub fn list_files_matching(&mut self, pattern: &str) -> Result<Vec<IsoFileEntry>> {
        let all_files = self.list_files()?;
        let pattern_lower = pattern.to_lowercase();
        Ok(all_files
            .into_iter()
            .filter(|f| f.path.to_string_lossy().to_lowercase().contains(&pattern_lower))
            .collect())
    }
    
    /// Read a file directly by offset and size (faster than by path).
    pub fn read_file_direct(&mut self, entry: &IsoFileEntry) -> Result<Vec<u8>> {
        self.read_bytes_at(entry.offset as u64, entry.size as usize)
    }
}

