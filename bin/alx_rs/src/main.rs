//! ALX_RS - Skies of Arcadia Legends Data Exporter/Importer
//! 
//! A Rust CLI tool that extracts game data from a GameCube ISO
//! and exports it to CSV files, or imports CSV data back into the ISO.

use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use clap::Parser;
use alx::game::GameRoot;
use alx::csv::{CsvExporter, CsvImporter};

macro_rules! export_csv {
    ($game:expr, $output_dir:expr, $name:expr, $read_fn:ident, $export_fn:ident, $filename:expr) => {{
        print!("Exporting {}...", $name);
        let data = $game.$read_fn()?;
        CsvExporter::$export_fn(&data, File::create($output_dir.join($filename))?)?;
        println!(" {} entries", data.len());
    }};
}

#[derive(Parser, Debug)]
#[command(name = "alx_rs")]
#[command(author = "SOA Randomizer Team")]
#[command(version = "0.1.0")]
#[command(about = "Exports/imports Skies of Arcadia game data to/from CSV files", long_about = None)]
struct Args {
    /// Path to the GameCube ISO file
    #[arg(value_name = "ISO_FILE")]
    iso_path: PathBuf,

    /// Output directory for CSV files (export mode), or output ISO path (import mode)
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Import mode: read CSVs from folder and write to ISO
    /// Use --output to write to a copy instead of modifying the original
    #[arg(short, long, value_name = "IMPORT_DIR")]
    import: Option<PathBuf>,

    /// Skip confirmation prompts (auto-confirm overwrites)
    #[arg(short = 'y', long = "yes")]
    yes: bool,
}

/// Prompt user for confirmation to overwrite
fn confirm_overwrite() -> Result<bool, Box<dyn std::error::Error>> {
    print!("Are you sure you want to continue? [y/N]: ");
    std::io::stdout().flush()?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Validate ISO path
    if !args.iso_path.exists() {
        return Err(format!("ISO file not found: {}", args.iso_path.display()).into());
    }
    
    // Check if we're in import mode
    if let Some(import_dir) = args.import {
        return run_import(&args.iso_path, &import_dir, args.output.as_deref(), args.yes);
    }
    
    // Export mode
    run_export(&args.iso_path, args.output)
}

fn run_export(iso_path: &Path, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Determine output directory
    let output_dir = match output {
        Some(path) => path,
        None => {
            // Create 'data' folder next to ISO
            let iso_parent = iso_path.parent().unwrap_or(Path::new("."));
            iso_parent.join("data")
        }
    };
    
    // Create output directory
    fs::create_dir_all(&output_dir)?;
    
    println!("ALX_RS - Skies of Arcadia Data Exporter");
    println!("========================================");
    println!("ISO: {}", iso_path.display());
    println!("Output: {}", output_dir.display());
    println!();
    
    // Open the game
    println!("Loading game data...");
    let mut game = GameRoot::open(iso_path)?;
    
    println!("Detected: {} ({})", 
        game.version().region, 
        if game.version().is_gc() { "GameCube" } else { "Unknown" }
    );
    println!();
    
    // Export all data types
    export_all(&mut game, &output_dir)?;
    
    println!();
    println!("Export complete!");
    
    Ok(())
}

fn run_import(iso_path: &Path, import_dir: &Path, output_iso: Option<&Path>, auto_confirm: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Validate import directory
    if !import_dir.exists() {
        return Err(format!("Import directory not found: {}", import_dir.display()).into());
    }
    
    // Determine the target ISO path
    let target_iso = if let Some(output_path) = output_iso {
        // Check if output already exists
        if output_path.exists() && !auto_confirm {
            println!("Output file already exists: {}", output_path.display());
            if !confirm_overwrite()? {
                println!("Aborted.");
                return Ok(());
            }
        }
        
        // Copy the original ISO to the output path first
        println!("ALX_RS - Skies of Arcadia Data Importer");
        println!("========================================");
        println!("Source ISO: {}", iso_path.display());
        println!("Output ISO: {}", output_path.display());
        println!("Import from: {}", import_dir.display());
        println!();
        
        println!("Copying ISO to output path...");
        fs::copy(iso_path, output_path)?;
        println!("  Copy complete ({:.1} GB)", 
            fs::metadata(output_path)?.len() as f64 / 1_000_000_000.0);
        println!();
        
        output_path.to_path_buf()
    } else {
        // Modifying original ISO - require confirmation
        println!("ALX_RS - Skies of Arcadia Data Importer");
        println!("========================================");
        println!("ISO: {}", iso_path.display());
        println!("Import from: {}", import_dir.display());
        println!();
        
        if !auto_confirm {
            println!("WARNING: This will modify the original ISO in-place!");
            println!("         Use --output to write to a copy instead.");
            println!();
            if !confirm_overwrite()? {
                println!("Aborted.");
                return Ok(());
            }
        } else {
            println!("WARNING: Modifying ISO in-place. Use --output to write to a copy.");
            println!();
        }
        
        iso_path.to_path_buf()
    };
    
    // Open the game
    println!("Loading game data...");
    let mut game = GameRoot::open(&target_iso)?;
    
    println!("Detected: {} ({})", 
        game.version().region, 
        if game.version().is_gc() { "GameCube" } else { "Unknown" }
    );
    println!();
    
    // Import all data types
    import_all(&mut game, import_dir)?;
    
    // Save changes to DOL
    println!();
    println!("Saving changes to ISO...");
    game.save_dol()?;
    
    println!("Import complete!");
    
    Ok(())
}

/// Import a CSV file if it exists, returning the parsed data.
macro_rules! import_csv {
    ($import_dir:expr, $filename:expr, $import_fn:ident, $type_name:expr) => {{
        let path = $import_dir.join($filename);
        if path.exists() {
            print!("Importing {}...", $type_name);
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let result = CsvImporter::$import_fn(reader);
            match result {
                Ok(data) => {
                    println!(" {} entries", data.len());
                    Some(data)
                }
                Err(e) => {
                    println!(" ERROR: {}", e);
                    return Err(format!("Failed to import {}: {}", $type_name, e).into());
                }
            }
        } else {
            println!("Skipping {} (file not found)", $type_name);
            None
        }
    }};
}

fn import_all(game: &mut GameRoot, import_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Import accessories
    if let Some(data) = import_csv!(import_dir, "accessory.csv", import_accessories, "accessories") {
        game.write_accessories(&data)?;
    }
    
    // Import armors
    if let Some(data) = import_csv!(import_dir, "armor.csv", import_armors, "armors") {
        game.write_armors(&data)?;
    }
    
    // Import weapons
    if let Some(data) = import_csv!(import_dir, "weapon.csv", import_weapons, "weapons") {
        game.write_weapons(&data)?;
    }
    
    // Import usable items
    if let Some(data) = import_csv!(import_dir, "usableitem.csv", import_usable_items, "usable items") {
        game.write_usable_items(&data)?;
    }
    
    // Import special items
    if let Some(data) = import_csv!(import_dir, "specialitem.csv", import_special_items, "special items") {
        game.write_special_items(&data)?;
    }
    
    // Import characters
    if let Some(data) = import_csv!(import_dir, "character.csv", import_characters, "characters") {
        game.write_characters(&data)?;
    }
    
    // Import character magic
    if let Some(data) = import_csv!(import_dir, "charactermagic.csv", import_character_magic, "character magic") {
        game.write_character_magic(&data)?;
    }
    
    // Import character super moves
    if let Some(data) = import_csv!(import_dir, "charactersupermove.csv", import_character_super_moves, "character super moves") {
        game.write_character_super_moves(&data)?;
    }
    
    // Import shops
    if let Some(data) = import_csv!(import_dir, "shop.csv", import_shops, "shops") {
        game.write_shops(&data)?;
    }
    
    // Import treasure chests
    if let Some(data) = import_csv!(import_dir, "treasurechest.csv", import_treasure_chests, "treasure chests") {
        game.write_treasure_chests(&data)?;
    }
    
    // Import crew members
    if let Some(data) = import_csv!(import_dir, "crewmember.csv", import_crew_members, "crew members") {
        game.write_crew_members(&data)?;
    }
    
    // Import playable ships
    if let Some(data) = import_csv!(import_dir, "playableship.csv", import_playable_ships, "playable ships") {
        game.write_playable_ships(&data)?;
    }
    
    // Import ship cannons
    if let Some(data) = import_csv!(import_dir, "shipcannon.csv", import_ship_cannons, "ship cannons") {
        game.write_ship_cannons(&data)?;
    }
    
    // Import ship accessories
    if let Some(data) = import_csv!(import_dir, "shipaccessory.csv", import_ship_accessories, "ship accessories") {
        game.write_ship_accessories(&data)?;
    }
    
    // Import ship items
    if let Some(data) = import_csv!(import_dir, "shipitem.csv", import_ship_items, "ship items") {
        game.write_ship_items(&data)?;
    }
    
    // Import enemy ships
    if let Some(data) = import_csv!(import_dir, "enemyship.csv", import_enemy_ships, "enemy ships") {
        game.write_enemy_ships(&data)?;
    }
    
    // Import enemy magic
    if let Some(data) = import_csv!(import_dir, "enemymagic.csv", import_enemy_magic, "enemy magic") {
        game.write_enemy_magic(&data)?;
    }
    
    // Import enemy super moves
    if let Some(data) = import_csv!(import_dir, "enemysupermove.csv", import_enemy_super_moves, "enemy super moves") {
        game.write_enemy_super_moves(&data)?;
    }
    
    // Import swashbucklers
    if let Some(data) = import_csv!(import_dir, "swashbuckler.csv", import_swashbucklers, "swashbucklers") {
        game.write_swashbucklers(&data)?;
    }
    
    // Import spirit curves
    if let Some(data) = import_csv!(import_dir, "spiritcurve.csv", import_spirit_curves, "spirit curves") {
        game.write_spirit_curves(&data)?;
    }
    
    // Import exp boosts
    if let Some(data) = import_csv!(import_dir, "expboost.csv", import_exp_boosts, "exp boosts") {
        game.write_exp_boosts(&data)?;
    }
    
    // Note: Enemies from ENP/DAT files are not yet supported for import
    // (they require writing back to file-based storage, not just DOL)
    
    Ok(())
}

fn export_all(game: &mut GameRoot, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    export_csv!(game, output_dir, "accessories", read_accessories, export_accessories, "accessory.csv");
    export_csv!(game, output_dir, "armors", read_armors, export_armors, "armor.csv");
    
    // Weapons need weapon effects for effect name lookup
    print!("Exporting weapons...");
    let weapons = game.read_weapons()?;
    let weapon_effects = game.read_weapon_effects()?;
    CsvExporter::export_weapons(&weapons, File::create(output_dir.join("weapon.csv"))?, &weapon_effects)?;
    println!(" {} entries", weapons.len());
    export_csv!(game, output_dir, "usable items", read_usable_items, export_usable_items, "usableitem.csv");
    export_csv!(game, output_dir, "special items", read_special_items, export_special_items, "specialitem.csv");
    export_csv!(game, output_dir, "characters", read_characters, export_characters, "character.csv");
    export_csv!(game, output_dir, "character magic", read_character_magic, export_character_magic, "charactermagic.csv");
    export_csv!(game, output_dir, "character super moves", read_character_super_moves, export_character_super_moves, "charactersupermove.csv");
    export_csv!(game, output_dir, "shops", read_shops, export_shops, "shop.csv");
    
    // Build item database early for lookups (treasure chests and enemies need it)
    let item_db = game.build_item_database()?;
    
    // Treasure chests need item database for item name lookup
    print!("Exporting treasure chests...");
    let chests = game.read_treasure_chests()?;
    CsvExporter::export_treasure_chests(&chests, File::create(output_dir.join("treasurechest.csv"))?, &item_db)?;
    println!(" {} entries", chests.len());
    
    export_csv!(game, output_dir, "crew members", read_crew_members, export_crew_members, "crewmember.csv");
    export_csv!(game, output_dir, "playable ships", read_playable_ships, export_playable_ships, "playableship.csv");
    export_csv!(game, output_dir, "ship cannons", read_ship_cannons, export_ship_cannons, "shipcannon.csv");
    export_csv!(game, output_dir, "ship accessories", read_ship_accessories, export_ship_accessories, "shipaccessory.csv");
    export_csv!(game, output_dir, "ship items", read_ship_items, export_ship_items, "shipitem.csv");
    export_csv!(game, output_dir, "enemy ships", read_enemy_ships, export_enemy_ships, "enemyship.csv");
    export_csv!(game, output_dir, "enemy magic", read_enemy_magic, export_enemy_magic, "enemymagic.csv");
    export_csv!(game, output_dir, "enemy super moves", read_enemy_super_moves, export_enemy_super_moves, "enemysupermove.csv");
    export_csv!(game, output_dir, "swashbucklers", read_swashbucklers, export_swashbucklers, "swashbuckler.csv");
    export_csv!(game, output_dir, "spirit curves", read_spirit_curves, export_spirit_curves, "spiritcurve.csv");
    export_csv!(game, output_dir, "exp boosts", read_exp_boosts, export_exp_boosts, "expboost.csv");
    
    // Enemies (from ENP files) - special handling for two outputs
    print!("Exporting enemies...");
    let (enemies, tasks) = game.read_enemies()?;
    // Use US enemy names from vocabulary
    let enemy_names = alx::lookups::enemy_names_map();
    CsvExporter::export_enemies(&enemies, File::create(output_dir.join("enemy.csv"))?, &item_db, &enemy_names)?;
    CsvExporter::export_enemy_tasks(&tasks, File::create(output_dir.join("enemytask.csv"))?)?;
    println!(" {} enemies, {} tasks", enemies.len(), tasks.len());
    
    Ok(())
}
