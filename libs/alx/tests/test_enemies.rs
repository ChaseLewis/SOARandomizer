//! Integration tests for Enemy entries.

mod common;

#[test]
fn test_read_enemies() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let (enemies, tasks) = game.read_enemies().unwrap();
    
    // Should have many enemies from ENP files
    assert!(!enemies.is_empty(), "Should have found some enemies");
    assert!(!tasks.is_empty(), "Should have found some tasks");
    
    // Count by filter type
    let global_count = enemies.iter().filter(|e| e.filter == "*").count();
    let file_specific = enemies.len() - global_count;
    
    println!("✓ Read {} enemies ({} global, {} file-specific) and {} tasks", 
        enemies.len(), global_count, file_specific, tasks.len());
}

#[test]
fn test_enemy_filter_breakdown() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let (enemies, _tasks) = game.read_enemies().unwrap();
    
    // Count by filter type
    let global_count = enemies.iter().filter(|e| e.filter == "*").count();
    let evp_count = enemies.iter().filter(|e| e.filter.contains("evp")).count();
    let enp_count = enemies.iter().filter(|e| e.filter.contains("enp")).count();
    let dat_count = enemies.iter().filter(|e| e.filter.contains("dat")).count();
    
    println!("Enemy filter breakdown:");
    println!("  Global (*): {}", global_count);
    println!("  EVP files: {}", evp_count);
    println!("  ENP files: {}", enp_count);
    println!("  DAT files: {}", dat_count);
    println!("  Total: {}", enemies.len());
    
    // Show some sample IDs and their filters
    println!("\nSample enemies:");
    for enemy in enemies.iter().take(10) {
        println!("  ID {}: filter='{}', name='{}'", enemy.id, enemy.filter, enemy.name_jp);
    }
    
    // Check for ecinit specifically
    let ecinit_count = enemies.iter().filter(|e| e.filter.contains("ecinit")).count();
    let ebinit_count = enemies.iter().filter(|e| e.filter.contains("ebinit")).count();
    println!("\nDAT breakdown: ecinit={}, ebinit={}", ecinit_count, ebinit_count);
    
    // List all unique ecinit/ebinit files
    let mut dat_files: Vec<_> = enemies.iter()
        .filter(|e| e.filter.contains("init") && e.filter.contains(".dat"))
        .map(|e| e.filter.clone())
        .collect();
    dat_files.sort();
    dat_files.dedup();
    println!("\nDAT files found ({}):", dat_files.len());
    for f in &dat_files {
        println!("  {}", f);
    }
    
    // Check for specific missing files
    let missing = vec!["ecinit015", "ecinit018", "ebinit037", "ebinit038", "ebinit039", 
                       "ebinit040", "ebinit041", "ebinit100", "ebinit101", "ebinit102", 
                       "ebinit103", "ebinit104"];
    println!("\nMissing file check:");
    for m in &missing {
        let found = enemies.iter().any(|e| e.filter.contains(m));
        println!("  {}: {}", m, if found { "FOUND" } else { "MISSING" });
    }
}

#[test]
fn test_debug_id_165() {
    skip_if_no_iso!();
    
    use alx::io::{decompress_aklz, parse_evp, parse_dat_file};
    
    let mut game = common::load_game();
    let version = game.version().clone();
    
    // Check EVP for ID 165
    println!("Checking EVP for ID 165:");
    if let Ok(files) = game.list_iso_files_matching("epevent.evp") {
        for entry in &files {
            let raw = game.read_file_direct(entry).unwrap();
            let data = decompress_aklz(&raw).unwrap();
            let parsed = parse_evp(&data, "epevent.evp", &version).unwrap();
            let found: Vec<_> = parsed.enemies.iter().filter(|e| e.id == 165).collect();
            println!("  EVP has {} entries for ID 165", found.len());
            for e in found {
                println!("    filter='{}' hp={} exp={}", e.filter, e.max_hp, e.exp);
            }
        }
    }
    
    // Check ebinit037.dat for ID 165
    println!("\nChecking ebinit037.dat for ID 165:");
    if let Ok(files) = game.list_iso_files_matching("ebinit037") {
        for entry in &files {
            let raw = game.read_file_direct(entry).unwrap();
            let data = decompress_aklz(&raw).unwrap();
            let filename = entry.path.file_name().unwrap().to_string_lossy().to_string();
            let parsed = parse_dat_file(&data, &filename, &version).unwrap();
            let found: Vec<_> = parsed.enemies.iter().filter(|e| e.id == 165).collect();
            println!("  DAT has {} entries for ID 165", found.len());
            for e in found {
                println!("    filter='{}' hp={} exp={}", e.filter, e.max_hp, e.exp);
            }
        }
    }
}

#[test]
fn test_enemy_stats() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let (enemies, _tasks) = game.read_enemies().unwrap();
    
    // All enemies should have positive HP
    for enemy in &enemies {
        assert!(enemy.max_hp > 0, "Enemy {} should have HP > 0", enemy.id);
    }
    
    println!("✓ Enemy stats verified for {} enemies", enemies.len());
}

#[test]
fn test_enemy_tasks() {
    skip_if_no_iso!();
    
    let mut game = common::load_game();
    let (_enemies, tasks) = game.read_enemies().unwrap();
    
    // Tasks should have valid type IDs
    for task in &tasks {
        assert!(task.type_id >= -1 && task.type_id <= 1, 
            "Task type ID should be -1, 0, or 1, got {}", task.type_id);
    }
    
    println!("✓ Enemy tasks verified: {} tasks", tasks.len());
}

