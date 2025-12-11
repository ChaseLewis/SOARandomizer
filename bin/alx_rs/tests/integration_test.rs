//! Integration tests for the alx_rs binary.
//!
//! These tests run the alx_rs executable and verify its output matches
//! the reference CSV files from the original ALX Ruby tool.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Path to the ISO file for testing.
const TEST_ISO_PATH: &str = "../../roms/Skies of Arcadia Legends (USA).iso";

/// Path to reference CSV files from original ALX.
const REFERENCE_DIR: &str = "../../submodules/alx/dist/2002-12-19-gc-us-final/data";

/// Skip tests if ISO doesn't exist.
macro_rules! skip_if_no_iso {
    () => {
        if !Path::new(TEST_ISO_PATH).exists() {
            eprintln!("Skipping: Test ISO not found at {}", TEST_ISO_PATH);
            return;
        }
    };
}

/// Skip tests if reference files don't exist.
macro_rules! skip_if_no_reference {
    () => {
        if !Path::new(REFERENCE_DIR).exists() {
            eprintln!(
                "Skipping: Reference directory not found at {}",
                REFERENCE_DIR
            );
            return;
        }
    };
}

/// Get the path to the alx_rs binary.
fn get_binary_path() -> String {
    // The binary is built in the target directory relative to the workspace
    let debug_path = "../../target/debug/alx_rs.exe";
    let release_path = "../../target/release/alx_rs.exe";

    // Also check without .exe for Unix
    let debug_unix = "../../target/debug/alx_rs";
    let release_unix = "../../target/release/alx_rs";

    if Path::new(release_path).exists() {
        release_path.to_string()
    } else if Path::new(debug_path).exists() {
        debug_path.to_string()
    } else if Path::new(release_unix).exists() {
        release_unix.to_string()
    } else if Path::new(debug_unix).exists() {
        debug_unix.to_string()
    } else {
        panic!("alx_rs binary not found. Run `cargo build --package alx_rs` first.");
    }
}

/// Count rows in a CSV file (excluding header) using proper CSV parsing.
fn count_csv_rows(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }
    let file = fs::File::open(path).ok();
    if file.is_none() {
        return 0;
    }
    let mut reader = csv::Reader::from_reader(file.unwrap());
    reader.records().count()
}

#[test]
fn test_binary_runs_successfully() {
    skip_if_no_iso!();

    let binary = get_binary_path();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().join("data");

    let output = Command::new(&binary)
        .arg(TEST_ISO_PATH)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to run alx_rs");

    assert!(
        output.status.success(),
        "alx_rs failed with: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that output contains expected messages
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ALX_RS"), "Missing header in output");
    assert!(
        stdout.contains("Export complete!"),
        "Missing completion message"
    );

    println!("✓ Binary runs successfully!");
}

#[test]
fn test_binary_creates_all_csv_files() {
    skip_if_no_iso!();

    let binary = get_binary_path();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().join("data");

    let output = Command::new(&binary)
        .arg(TEST_ISO_PATH)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to run alx_rs");

    assert!(output.status.success(), "alx_rs failed");

    // Expected CSV files
    let expected_files = [
        "accessory.csv",
        "armor.csv",
        "weapon.csv",
        "usableitem.csv",
        "specialitem.csv",
        "character.csv",
        "charactermagic.csv",
        "charactersupermove.csv",
        "shop.csv",
        "treasurechest.csv",
        "crewmember.csv",
        "playableship.csv",
        "shipcannon.csv",
        "shipaccessory.csv",
        "shipitem.csv",
        "enemyship.csv",
        "enemymagic.csv",
        "enemysupermove.csv",
        "swashbuckler.csv",
        "spiritcurve.csv",
        "expboost.csv",
        "enemy.csv",
        "enemytask.csv",
    ];

    for file in &expected_files {
        let path = output_dir.join(file);
        assert!(path.exists(), "Missing output file: {}", file);

        // Check file is not empty
        let size = fs::metadata(&path).unwrap().len();
        assert!(size > 0, "File {} is empty", file);
    }

    println!("✓ All {} CSV files created!", expected_files.len());
}

#[test]
fn test_binary_output_row_counts_match_reference() {
    skip_if_no_iso!();
    skip_if_no_reference!();

    let binary = get_binary_path();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().join("data");

    let output = Command::new(&binary)
        .arg(TEST_ISO_PATH)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .expect("Failed to run alx_rs");

    assert!(output.status.success(), "alx_rs failed");

    // Files to compare (must exist in both generated and reference)
    let files_to_compare = [
        "accessory.csv",
        "armor.csv",
        "weapon.csv",
        "usableitem.csv",
        "specialitem.csv",
        "character.csv",
        "charactermagic.csv",
        "charactersupermove.csv",
        "shop.csv",
        "treasurechest.csv",
        "crewmember.csv",
        "playableship.csv",
        "shipcannon.csv",
        "shipaccessory.csv",
        "shipitem.csv",
        "enemyship.csv",
        "enemymagic.csv",
        "enemysupermove.csv",
        "swashbuckler.csv",
        "spiritcurve.csv",
        "expboost.csv",
    ];

    let mut mismatches = Vec::new();

    for file in &files_to_compare {
        let generated_path = output_dir.join(file);
        let reference_path = Path::new(REFERENCE_DIR).join(file);

        if !reference_path.exists() {
            println!("⚠ Reference file not found: {}", file);
            continue;
        }

        let gen_rows = count_csv_rows(&generated_path);
        let ref_rows = count_csv_rows(&reference_path);

        if gen_rows != ref_rows {
            mismatches.push(format!(
                "{}: generated {} rows, reference has {} rows",
                file, gen_rows, ref_rows
            ));
        } else {
            println!("✓ {} row count matches ({} rows)", file, gen_rows);
        }
    }

    if !mismatches.is_empty() {
        panic!("Row count mismatches:\n{}", mismatches.join("\n"));
    }

    println!("✓ All file row counts match reference!");
}

#[test]
fn test_binary_default_output_dir() {
    skip_if_no_iso!();

    // This test verifies the default output directory behavior
    // We can't actually test this without affecting the real ISO directory,
    // so we just verify the help text mentions the behavior

    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to run alx_rs --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("data"), "Help should mention 'data' folder");
    assert!(
        stdout.contains("Output directory"),
        "Help should mention output directory"
    );

    println!("✓ Help text is correct!");
}

#[test]
fn test_binary_handles_missing_iso() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("nonexistent.iso")
        .output()
        .expect("Failed to run alx_rs");

    // Should fail with error
    assert!(!output.status.success(), "Should fail for missing ISO");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("Error"),
        "Should report error for missing file"
    );

    println!("✓ Handles missing ISO correctly!");
}

#[test]
fn test_binary_version_flag() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run alx_rs --version");

    assert!(output.status.success(), "Version flag should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("alx_rs") || stdout.contains("0.1"),
        "Should show version info"
    );

    println!("✓ Version flag works!");
}
