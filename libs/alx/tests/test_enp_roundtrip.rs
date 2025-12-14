//! Binary roundtrip tests for ENP file repackaging.
//!
//! These tests verify that multi-segment ENP files can be parsed into segments,
//! rebaked, and produce byte-identical output.

mod common;

use alx::io::{bake_enp_segments, decompress_aklz, A099A_BAKED_FILENAME, A099A_SEGMENTS};

/// Parse segment info from a multi-segment ENP file header.
/// Returns Vec of (segment_name, position, size) tuples.
fn parse_segment_info(data: &[u8]) -> Option<Vec<(String, usize, usize)>> {
    if data.len() < 8 {
        return None;
    }

    // Check for multi-segment signature: 00 00 FF FF
    if data[0..4] != [0x00, 0x00, 0xFF, 0xFF] {
        return None;
    }

    // Read number of segments (i16 BE at offset 4)
    let num_segments = i16::from_be_bytes([data[4], data[5]]) as usize;

    // Check value at offset 6 should be -1 (FF FF)
    let check = i16::from_be_bytes([data[6], data[7]]);
    if check != -1 {
        return None;
    }

    let mut segments = Vec::new();
    let mut offset = 8;

    for _ in 0..num_segments {
        if offset + 32 > data.len() {
            break;
        }

        // Read name (20 bytes, null-terminated)
        let name_bytes = &data[offset..offset + 20];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(20);
        let name = String::from_utf8_lossy(&name_bytes[..name_end]).to_string();
        offset += 20;

        // Read position (i32 BE)
        let pos = i32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        // Read size (i32 BE)
        let size = i32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        // Skip check value (4 bytes)
        offset += 4;

        segments.push((name, pos, size));
    }

    Some(segments)
}

/// Test that a099a_ep.enp can be parsed into segments and rebaked to produce
/// byte-identical output.
#[test]
fn test_a099a_ep_binary_roundtrip() {
    skip_if_no_iso!();

    let mut game = common::load_game();

    // Find and read a099a_ep.enp
    let enp_files = game
        .iso_mut()
        .list_files_matching(A099A_BAKED_FILENAME)
        .expect("Failed to list ENP files");

    let entry = enp_files
        .iter()
        .find(|e| {
            e.path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                == Some(A099A_BAKED_FILENAME.to_lowercase())
        })
        .expect("a099a_ep.enp not found in ISO");

    // Read the original file (compressed)
    let raw_data = game
        .read_file_direct(entry)
        .expect("Failed to read a099a_ep.enp");

    // Decompress
    let original_data = decompress_aklz(&raw_data).expect("Failed to decompress a099a_ep.enp");

    println!("Original a099a_ep.enp size: {} bytes", original_data.len());

    // Parse segment info from header
    let segments_info =
        parse_segment_info(&original_data).expect("Failed to parse segment info from header");

    println!("Found {} segments:", segments_info.len());
    for (name, pos, size) in &segments_info {
        println!("  {} at offset {}, size {} bytes", name, pos, size);
    }

    // Verify we have the expected 13 segments
    assert_eq!(
        segments_info.len(),
        A099A_SEGMENTS.len(),
        "Expected {} segments, found {}",
        A099A_SEGMENTS.len(),
        segments_info.len()
    );

    // Check for padding between segments
    println!("\nChecking padding between segments:");
    for i in 0..segments_info.len() {
        let (name, pos, size) = &segments_info[i];
        let seg_end = pos + size;
        
        let next_start = if i + 1 < segments_info.len() {
            segments_info[i + 1].1
        } else {
            original_data.len()
        };
        
        let padding = next_start - seg_end;
        if padding > 0 {
            println!(
                "  After {} (ends at {}): {} bytes padding = {:02X?}",
                name,
                seg_end,
                padding,
                &original_data[seg_end..next_start]
            );
        }
    }

    // Extract each segment's raw data
    let mut segment_data: Vec<(&str, Vec<u8>)> = Vec::new();
    for (i, (name, pos, size)) in segments_info.iter().enumerate() {
        if *pos + *size > original_data.len() {
            panic!(
                "Segment {} extends beyond file: pos={}, size={}, file_len={}",
                name,
                pos,
                size,
                original_data.len()
            );
        }

        let data = original_data[*pos..*pos + *size].to_vec();

        // Convert .bin back to .enp for baking (bake_enp_segments expects .enp extension)
        let enp_name = name.replace(".bin", ".enp");

        // Verify segment name matches expected
        let expected_name = A099A_SEGMENTS[i];
        assert_eq!(
            enp_name.to_lowercase(),
            expected_name.to_lowercase(),
            "Segment {} name mismatch: expected {}, got {}",
            i,
            expected_name,
            enp_name
        );

        segment_data.push((expected_name, data));
    }

    // Rebake the segments
    let segments_for_bake: Vec<(&str, &[u8])> = segment_data
        .iter()
        .map(|(name, data)| (*name, data.as_slice()))
        .collect();

    let rebaked = bake_enp_segments(&segments_for_bake).expect("Failed to rebake segments");

    println!("Rebaked a099a_ep.enp size: {} bytes", rebaked.len());

    // Compare sizes first
    assert_eq!(
        original_data.len(),
        rebaked.len(),
        "Size mismatch: original {} bytes, rebaked {} bytes",
        original_data.len(),
        rebaked.len()
    );

    // Compare byte-by-byte
    let mut first_diff: Option<usize> = None;
    let mut diff_count = 0;

    for (i, (orig, new)) in original_data.iter().zip(rebaked.iter()).enumerate() {
        if orig != new {
            if first_diff.is_none() {
                first_diff = Some(i);
            }
            diff_count += 1;
        }
    }

    if let Some(first) = first_diff {
        // Show context around first difference
        let start = first.saturating_sub(8);
        let end = (first + 24).min(original_data.len());

        println!("\nFirst difference at offset {}:", first);
        println!(
            "Original: {:02X?}",
            &original_data[start..end.min(original_data.len())]
        );
        println!(
            "Rebaked:  {:02X?}",
            &rebaked[start..end.min(rebaked.len())]
        );
        println!("Total differences: {} bytes", diff_count);

        panic!(
            "Binary mismatch: {} bytes differ, first at offset {}",
            diff_count, first
        );
    }

    // Also verify using CRC32
    let orig_crc = common::crc32_checksum(&original_data);
    let new_crc = common::crc32_checksum(&rebaked);
    assert_eq!(
        orig_crc, new_crc,
        "CRC32 mismatch: original {:08X}, rebaked {:08X}",
        orig_crc, new_crc
    );

    println!("✓ Binary roundtrip verified! CRC32: {:08X}", orig_crc);
}

/// Test that header structure is correctly preserved.
#[test]
fn test_a099a_ep_header_structure() {
    skip_if_no_iso!();

    let mut game = common::load_game();

    let enp_files = game
        .iso_mut()
        .list_files_matching(A099A_BAKED_FILENAME)
        .expect("Failed to list ENP files");

    let entry = enp_files
        .iter()
        .find(|e| {
            e.path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                == Some(A099A_BAKED_FILENAME.to_lowercase())
        })
        .expect("a099a_ep.enp not found in ISO");

    let raw_data = game
        .read_file_direct(entry)
        .expect("Failed to read a099a_ep.enp");
    let data = decompress_aklz(&raw_data).expect("Failed to decompress");

    // Verify header signature
    assert_eq!(
        &data[0..4],
        &[0x00, 0x00, 0xFF, 0xFF],
        "Invalid header signature"
    );

    // Verify segment count
    let num_segments = i16::from_be_bytes([data[4], data[5]]);
    assert_eq!(num_segments, 13, "Expected 13 segments");

    // Verify check value
    let check = i16::from_be_bytes([data[6], data[7]]);
    assert_eq!(check, -1, "Invalid check value");

    // Verify header size: 8 + 13 * 32 = 424 bytes
    let expected_header_size = 8 + 13 * 32;
    assert!(
        data.len() > expected_header_size,
        "File too small for header"
    );

    println!("✓ Header structure verified");
    println!("  Segments: {}", num_segments);
    println!("  Header size: {} bytes", expected_header_size);
}
