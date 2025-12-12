//! Integration tests for EnemyEncounter entries.

mod common;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use alx::csv::{CsvExporter, CsvImporter};
use alx::lookups::enemy_names_map;

/// Path to reference CSV files.
const REFERENCE_CSV_DIR: &str = "../../submodules/alx/dist/2002-12-19-gc-us-final/data";

#[test]
fn test_read_enemy_encounters() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let encounters = game.read_enemy_encounters().unwrap();

    // Should have many encounters from ENP files
    assert!(
        !encounters.is_empty(),
        "Should have found some enemy encounters"
    );

    // Count unique files
    let mut files: Vec<String> = encounters.iter().map(|e| e.filter.clone()).collect();
    files.sort();
    files.dedup();

    println!(
        "✓ Read {} enemy encounters from {} files",
        encounters.len(),
        files.len()
    );

    // Show sample encounters
    println!("\nSample encounters:");
    for enc in encounters.iter().take(5) {
        let enemy_count = enc.enemy_slots.iter().filter(|s| s.enemy_id != 255).count();
        println!(
            "  ID {}, file='{}', initiative={}, magic_exp={}, enemies={}",
            enc.id, enc.filter, enc.initiative, enc.magic_exp, enemy_count
        );
    }
}

#[test]
fn test_encounter_structure() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let encounters = game.read_enemy_encounters().unwrap();

    // Count encounters with various properties
    let mut with_enemies = 0;
    let mut empty_encounters = 0;

    for enc in &encounters {
        // Enemy slots are u8, all values are valid (0-254 for enemy ID, 255 for none)
        // Just count them for statistics
        let _ = enc.enemy_slots.iter().count();

        let has_enemies = enc.enemy_slots.iter().any(|s| s.enemy_id != 255);
        if has_enemies {
            with_enemies += 1;
        } else {
            empty_encounters += 1;
        }
    }

    // Most encounters should have enemies, but some may be empty placeholders
    assert!(
        with_enemies > empty_encounters,
        "Most encounters should have enemies: {} with vs {} empty",
        with_enemies,
        empty_encounters
    );

    println!(
        "✓ Encounter structure verified: {} with enemies, {} empty (total: {})",
        with_enemies,
        empty_encounters,
        encounters.len()
    );
}

#[test]
fn test_enemy_encounter_csv_export() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let encounters = game.read_enemy_encounters().unwrap();

    // Build enemy name lookup using vocabulary for US names
    let (enemies, _) = game.read_enemies().unwrap();
    let us_names = enemy_names_map();
    let mut enemy_names: HashMap<u32, (String, String)> = HashMap::new();
    for enemy in &enemies {
        let us_name = us_names
            .get(&enemy.id)
            .cloned()
            .unwrap_or_else(|| "???".to_string());
        enemy_names.insert(enemy.id, (enemy.name_jp.clone(), us_name));
    }

    // Export to CSV
    let mut csv_output = Vec::new();
    CsvExporter::export_enemy_encounters(&encounters, &mut csv_output, &enemy_names).unwrap();

    let csv_str = String::from_utf8(csv_output).unwrap();

    // Verify header
    let first_line = csv_str.lines().next().unwrap();
    assert!(
        first_line.contains("Entry ID"),
        "Header should contain 'Entry ID'"
    );
    assert!(
        first_line.contains("[Filter]"),
        "Header should contain '[Filter]'"
    );
    assert!(
        first_line.contains("Initiative"),
        "Header should contain 'Initiative'"
    );
    assert!(
        first_line.contains("Magic EXP"),
        "Header should contain 'Magic EXP'"
    );
    assert!(
        first_line.contains("EC1 ID"),
        "Header should contain 'EC1 ID'"
    );

    // Count data rows (excluding header)
    let data_lines = csv_str.lines().skip(1).count();
    assert_eq!(
        data_lines,
        encounters.len(),
        "CSV should have one line per encounter"
    );

    println!("✓ CSV export produced {} rows", data_lines);
}

#[test]
fn test_enemy_encounter_csv_roundtrip() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let original_encounters = game.read_enemy_encounters().unwrap();

    // Build enemy name lookup using vocabulary for US names
    let (enemies, _) = game.read_enemies().unwrap();
    let us_names = enemy_names_map();
    let mut enemy_names: HashMap<u32, (String, String)> = HashMap::new();
    for enemy in &enemies {
        let us_name = us_names
            .get(&enemy.id)
            .cloned()
            .unwrap_or_else(|| "???".to_string());
        enemy_names.insert(enemy.id, (enemy.name_jp.clone(), us_name));
    }

    // Export to CSV
    let mut csv_output = Vec::new();
    CsvExporter::export_enemy_encounters(&original_encounters, &mut csv_output, &enemy_names)
        .unwrap();

    // Import back from CSV
    let imported = CsvImporter::import_enemy_encounters(&csv_output[..], &[]).unwrap();

    // Compare
    assert_eq!(
        original_encounters.len(),
        imported.len(),
        "Should have same number of encounters after roundtrip"
    );

    for (orig, imp) in original_encounters.iter().zip(imported.iter()) {
        assert_eq!(
            orig.id, imp.id,
            "ID should match after roundtrip"
        );
        assert_eq!(
            orig.filter, imp.filter,
            "Filter should match after roundtrip"
        );
        assert_eq!(
            orig.initiative, imp.initiative,
            "Initiative should match after roundtrip for {} in {}",
            orig.id, orig.filter
        );
        assert_eq!(
            orig.magic_exp, imp.magic_exp,
            "Magic EXP should match after roundtrip for {} in {}",
            orig.id, orig.filter
        );

        for i in 0..8 {
            assert_eq!(
                orig.enemy_slots[i].enemy_id, imp.enemy_slots[i].enemy_id,
                "Enemy slot {} should match after roundtrip for {} in {}",
                i + 1,
                orig.id,
                orig.filter
            );
        }
    }

    println!(
        "✓ CSV roundtrip verified for {} encounters",
        original_encounters.len()
    );
}

#[test]
fn test_compare_with_reference_csv() {
    skip_if_no_iso!();

    let reference_path = Path::new(REFERENCE_CSV_DIR).join("enemyencounter.csv");
    if !reference_path.exists() {
        eprintln!(
            "Skipping: Reference CSV not found at {}",
            reference_path.display()
        );
        return;
    }

    // Load reference CSV
    let file = File::open(&reference_path).expect("Failed to open reference CSV");
    let mut reader = csv::Reader::from_reader(file);
    let ref_headers: Vec<String> = reader
        .headers()
        .unwrap()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut ref_rows: Vec<HashMap<String, String>> = Vec::new();
    for result in reader.records() {
        let record = result.unwrap();
        let mut row = HashMap::new();
        for (i, value) in record.iter().enumerate() {
            if i < ref_headers.len() {
                row.insert(ref_headers[i].clone(), value.to_string());
            }
        }
        ref_rows.push(row);
    }

    println!(
        "Reference CSV has {} columns and {} rows",
        ref_headers.len(),
        ref_rows.len()
    );

    // Load game encounters
    let mut game = common::load_game();
    let encounters = game.read_enemy_encounters().unwrap();

    // Build enemy name lookup using vocabulary for US names
    let (enemies, _) = game.read_enemies().unwrap();
    let us_names = enemy_names_map();
    let mut enemy_names: HashMap<u32, (String, String)> = HashMap::new();
    for enemy in &enemies {
        let us_name = us_names
            .get(&enemy.id)
            .cloned()
            .unwrap_or_else(|| "???".to_string());
        enemy_names.insert(enemy.id, (enemy.name_jp.clone(), us_name));
    }

    // Export to CSV
    let mut csv_output = Vec::new();
    CsvExporter::export_enemy_encounters(&encounters, &mut csv_output, &enemy_names).unwrap();

    // Parse our CSV output
    let mut our_reader = csv::Reader::from_reader(&csv_output[..]);
    let our_headers: Vec<String> = our_reader
        .headers()
        .unwrap()
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut our_rows: Vec<HashMap<String, String>> = Vec::new();
    for result in our_reader.records() {
        let record = result.unwrap();
        let mut row = HashMap::new();
        for (i, value) in record.iter().enumerate() {
            if i < our_headers.len() {
                row.insert(our_headers[i].clone(), value.to_string());
            }
        }
        our_rows.push(row);
    }

    println!(
        "Our CSV has {} columns and {} rows",
        our_headers.len(),
        our_rows.len()
    );

    // Assert row counts match
    assert_eq!(
        ref_rows.len(),
        our_rows.len(),
        "Row count mismatch: reference has {}, we have {}",
        ref_rows.len(),
        our_rows.len()
    );

    // Columns to compare (skip JP names as encoding may differ between systems)
    let columns_to_compare = [
        "Entry ID",
        "[Filter]",
        "Initiative",
        "Magic EXP",
        "EC1 ID",
        "EC2 ID",
        "EC3 ID",
        "EC4 ID",
        "EC5 ID",
        "EC6 ID",
        "EC7 ID",
        "EC8 ID",
        "[EC1 US Name]",
        "[EC2 US Name]",
        "[EC3 US Name]",
        "[EC4 US Name]",
        "[EC5 US Name]",
        "[EC6 US Name]",
        "[EC7 US Name]",
        "[EC8 US Name]",
    ];

    // Detailed comparison
    let mut mismatches: Vec<String> = Vec::new();
    for (i, (ref_row, our_row)) in ref_rows.iter().zip(our_rows.iter()).enumerate() {
        for col in &columns_to_compare {
            let ref_val = ref_row.get(*col).map(|s| s.as_str()).unwrap_or("");
            let our_val = our_row.get(*col).map(|s| s.as_str()).unwrap_or("");

            if ref_val != our_val {
                let ref_id = ref_row.get("Entry ID").map(|s| s.as_str()).unwrap_or("?");
                let ref_filter = ref_row.get("[Filter]").map(|s| s.as_str()).unwrap_or("?");
                mismatches.push(format!(
                    "Row {} (ID={}, {}): column '{}' mismatch - ref='{}', ours='{}'",
                    i, ref_id, ref_filter, col, ref_val, our_val
                ));
            }
        }
    }

    if !mismatches.is_empty() {
        // Print first 10 mismatches for debugging
        for m in mismatches.iter().take(10) {
            println!("{}", m);
        }
        if mismatches.len() > 10 {
            println!("... and {} more mismatches", mismatches.len() - 10);
        }
        panic!(
            "Reference CSV comparison failed: {} mismatches found",
            mismatches.len()
        );
    }

    println!("✓ All {} rows match the reference CSV!", ref_rows.len());
}

#[test]
fn test_encounter_file_breakdown() {
    skip_if_no_iso!();

    let mut game = common::load_game();
    let encounters = game.read_enemy_encounters().unwrap();

    // Group by file
    let mut by_file: HashMap<String, usize> = HashMap::new();
    for enc in &encounters {
        *by_file.entry(enc.filter.clone()).or_insert(0) += 1;
    }

    println!("Encounters by file:");
    let mut files: Vec<_> = by_file.iter().collect();
    files.sort_by_key(|(f, _)| f.as_str());
    for (file, count) in files.iter().take(20) {
        println!("  {}: {} encounters", file, count);
    }
    if files.len() > 20 {
        println!("  ... and {} more files", files.len() - 20);
    }

    println!("\nTotal: {} files, {} encounters", by_file.len(), encounters.len());
}

