//! Integration tests to verify import/export roundtrip preserves data.
//!
//! These tests export data from the ISO, then import it back and re-export,
//! verifying the CSVs are identical.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Path to the test ISO
const ISO_PATH: &str = "roms/Skies of Arcadia Legends (USA).iso";

/// Compare two CSV files line by line, returning differences
fn compare_csv_files(path1: &Path, path2: &Path) -> Vec<String> {
    let content1 = fs::read_to_string(path1).expect("Failed to read first file");
    let content2 = fs::read_to_string(path2).expect("Failed to read second file");

    let lines1: Vec<&str> = content1.lines().collect();
    let lines2: Vec<&str> = content2.lines().collect();

    let mut differences = Vec::new();

    // Check header
    if lines1.first() != lines2.first() {
        differences.push(format!(
            "Header mismatch:\n  Before: {:?}\n  After:  {:?}",
            lines1.first(),
            lines2.first()
        ));
    }

    // Check line counts
    if lines1.len() != lines2.len() {
        differences.push(format!(
            "Line count mismatch: {} vs {}",
            lines1.len(),
            lines2.len()
        ));
    }

    // Compare each line
    let max_lines = lines1.len().max(lines2.len());
    for i in 0..max_lines {
        let line1 = lines1.get(i);
        let line2 = lines2.get(i);

        if line1 != line2 {
            differences.push(format!(
                "Line {} differs:\n  Before: {:?}\n  After:  {:?}",
                i + 1,
                line1.unwrap_or(&"<missing>"),
                line2.unwrap_or(&"<missing>")
            ));

            // Show field-by-field diff for data rows
            if i > 0 {
                if let (Some(l1), Some(l2)) = (line1, line2) {
                    let fields1: Vec<&str> = l1.split(',').collect();
                    let fields2: Vec<&str> = l2.split(',').collect();

                    for (j, (f1, f2)) in fields1.iter().zip(fields2.iter()).enumerate() {
                        if f1 != f2 {
                            differences.push(format!("    Field {}: '{}' -> '{}'", j, f1, f2));
                        }
                    }
                }
            }
        }
    }

    differences
}

/// Run the alx_rs binary with given arguments
fn run_alx_rs(args: &[&str]) -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["run", "-p", "alx_rs", "--quiet", "--"])
        .args(args)
        .output()
        .expect("Failed to run alx_rs");

    if !output.status.success() {
        return Err(format!(
            "alx_rs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Test that all CSV files are identical after import/export roundtrip
#[test]
#[ignore] // Run with: cargo test --package alx_rs import_roundtrip -- --ignored
fn test_import_export_roundtrip() {
    if !Path::new(ISO_PATH).exists() {
        eprintln!("Skipping test: ISO not found at {}", ISO_PATH);
        return;
    }

    let test_dir = Path::new("target/test_import_roundtrip");
    let before_dir = test_dir.join("before");
    let after_dir = test_dir.join("after");
    let test_iso = test_dir.join("test.iso");

    // Clean up from previous runs
    let _ = fs::remove_dir_all(test_dir);
    fs::create_dir_all(&before_dir).expect("Failed to create before dir");
    fs::create_dir_all(&after_dir).expect("Failed to create after dir");

    // Step 1: Export from original ISO
    println!("Step 1: Exporting from original ISO...");
    run_alx_rs(&[ISO_PATH, "--output", before_dir.to_str().unwrap()]).expect("Export failed");

    // Step 2: Copy ISO and import the exported data
    println!("Step 2: Importing to copy of ISO...");
    run_alx_rs(&[
        "--import",
        before_dir.to_str().unwrap(),
        ISO_PATH,
        "--output",
        test_iso.to_str().unwrap(),
        "-y",
    ])
    .expect("Import failed");

    // Step 3: Export from the modified ISO
    println!("Step 3: Exporting from modified ISO...");
    run_alx_rs(&[
        test_iso.to_str().unwrap(),
        "--output",
        after_dir.to_str().unwrap(),
    ])
    .expect("Re-export failed");

    // Step 4: Compare all CSV files
    println!("Step 4: Comparing CSV files...");

    let csv_files = [
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

    let mut all_passed = true;
    let mut failures = Vec::new();

    for csv_file in &csv_files {
        let before_path = before_dir.join(csv_file);
        let after_path = after_dir.join(csv_file);

        if !before_path.exists() {
            failures.push(format!("{}: Before file missing", csv_file));
            all_passed = false;
            continue;
        }

        if !after_path.exists() {
            failures.push(format!("{}: After file missing", csv_file));
            all_passed = false;
            continue;
        }

        let diffs = compare_csv_files(&before_path, &after_path);

        if diffs.is_empty() {
            println!("  ✓ {}", csv_file);
        } else {
            println!("  ✗ {} - {} differences", csv_file, diffs.len());
            for diff in &diffs {
                println!("    {}", diff);
            }
            failures.push(format!("{}: {} differences", csv_file, diffs.len()));
            all_passed = false;
        }
    }

    // Clean up on success
    if all_passed {
        let _ = fs::remove_dir_all(test_dir);
    }

    assert!(
        all_passed,
        "CSV comparison failures:\n{}",
        failures.join("\n")
    );
}

/// Individual file roundtrip tests for faster debugging
macro_rules! roundtrip_test {
    ($name:ident, $csv_file:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            if !Path::new(ISO_PATH).exists() {
                eprintln!("Skipping test: ISO not found at {}", ISO_PATH);
                return;
            }

            let test_dir = Path::new(concat!("target/test_rt_", stringify!($name)));
            let before_dir = test_dir.join("before");
            let after_dir = test_dir.join("after");
            let test_iso = test_dir.join("test.iso");

            let _ = fs::remove_dir_all(test_dir);
            fs::create_dir_all(&before_dir).unwrap();
            fs::create_dir_all(&after_dir).unwrap();

            // Export
            run_alx_rs(&[ISO_PATH, "--output", before_dir.to_str().unwrap()]).unwrap();

            // Import
            run_alx_rs(&[
                "--import",
                before_dir.to_str().unwrap(),
                ISO_PATH,
                "--output",
                test_iso.to_str().unwrap(),
                "-y",
            ])
            .unwrap();

            // Re-export
            run_alx_rs(&[
                test_iso.to_str().unwrap(),
                "--output",
                after_dir.to_str().unwrap(),
            ])
            .unwrap();

            // Compare
            let diffs = compare_csv_files(&before_dir.join($csv_file), &after_dir.join($csv_file));

            if diffs.is_empty() {
                let _ = fs::remove_dir_all(test_dir);
            }

            assert!(
                diffs.is_empty(),
                "Differences in {}:\n{}",
                $csv_file,
                diffs.join("\n")
            );
        }
    };
}

roundtrip_test!(test_rt_accessory, "accessory.csv");
roundtrip_test!(test_rt_armor, "armor.csv");
roundtrip_test!(test_rt_weapon, "weapon.csv");
roundtrip_test!(test_rt_usableitem, "usableitem.csv");
roundtrip_test!(test_rt_specialitem, "specialitem.csv");
roundtrip_test!(test_rt_character, "character.csv");
roundtrip_test!(test_rt_charactermagic, "charactermagic.csv");
roundtrip_test!(test_rt_charactersupermove, "charactersupermove.csv");
roundtrip_test!(test_rt_shop, "shop.csv");
roundtrip_test!(test_rt_treasurechest, "treasurechest.csv");
roundtrip_test!(test_rt_crewmember, "crewmember.csv");
roundtrip_test!(test_rt_playableship, "playableship.csv");
roundtrip_test!(test_rt_shipcannon, "shipcannon.csv");
roundtrip_test!(test_rt_shipaccessory, "shipaccessory.csv");
roundtrip_test!(test_rt_shipitem, "shipitem.csv");
roundtrip_test!(test_rt_enemyship, "enemyship.csv");
roundtrip_test!(test_rt_enemymagic, "enemymagic.csv");
roundtrip_test!(test_rt_enemysupermove, "enemysupermove.csv");
roundtrip_test!(test_rt_swashbuckler, "swashbuckler.csv");
roundtrip_test!(test_rt_spiritcurve, "spiritcurve.csv");
roundtrip_test!(test_rt_expboost, "expboost.csv");
