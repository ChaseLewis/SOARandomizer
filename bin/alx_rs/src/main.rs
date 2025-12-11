//! ALX_RS - Skies of Arcadia Legends Data Exporter/Importer
//!
//! A Rust CLI tool that extracts game data from a GameCube ISO
//! and exports it to CSV files, or imports CSV data back into the ISO.

use alx::csv::{CsvExporter, CsvImporter};
use alx::game::GameRoot;
use clap::Parser;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

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
        return run_import(
            &args.iso_path,
            &import_dir,
            args.output.as_deref(),
            args.yes,
        );
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

    println!(
        "Detected: {} ({})",
        game.version().region,
        if game.version().is_gc() {
            "GameCube"
        } else {
            "Unknown"
        }
    );
    println!();

    // Export all data types
    export_all(&mut game, &output_dir)?;

    println!();
    println!("Export complete!");

    Ok(())
}

fn run_import(
    iso_path: &Path,
    import_dir: &Path,
    output_iso: Option<&Path>,
    auto_confirm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
        println!(
            "  Copy complete ({:.1} GB)",
            fs::metadata(output_path)?.len() as f64 / 1_000_000_000.0
        );
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

    println!(
        "Detected: {} ({})",
        game.version().region,
        if game.version().is_gc() {
            "GameCube"
        } else {
            "Unknown"
        }
    );
    println!();

    // Import all data types
    import_all(&mut game, import_dir)?;

    // Save changes to ISO
    println!();
    println!("Saving changes to ISO...");
    game.save_dol()?;
    game.save_level()?;

    println!("Import complete!");

    Ok(())
}

/// Import a CSV file if it exists, returning the parsed data.
/// This version doesn't need existing data (for types where CSV has all fields).
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
    // Ensure DOL is loaded before any writes
    game.load_dol()?;

    // Import accessories
    if let Some(data) = import_csv!(
        import_dir,
        "accessory.csv",
        import_accessories,
        "accessories"
    ) {
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

    // Import usable items (merge with existing)
    {
        let path = import_dir.join("usableitem.csv");
        if path.exists() {
            print!("Importing usable items...");
            let existing = game.read_usable_items()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_usable_items(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_usable_items(&data)?;
        } else {
            println!("Skipping usable items (file not found)");
        }
    }

    // Import special items
    if let Some(data) = import_csv!(
        import_dir,
        "specialitem.csv",
        import_special_items,
        "special items"
    ) {
        game.write_special_items(&data)?;
    }

    // Import characters (merge with existing - CSV only has subset of fields)
    {
        let path = import_dir.join("character.csv");
        if path.exists() {
            print!("Importing characters...");
            let existing = game.read_characters()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_characters(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_characters(&data)?;
        } else {
            println!("Skipping characters (file not found)");
        }
    }

    // Import character magic (merge with existing)
    {
        let path = import_dir.join("charactermagic.csv");
        if path.exists() {
            print!("Importing character magic...");
            let existing = game.read_character_magic()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_character_magic(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_character_magic(&data)?;
        } else {
            println!("Skipping character magic (file not found)");
        }
    }

    // Import character super moves (merge with existing)
    {
        let path = import_dir.join("charactersupermove.csv");
        if path.exists() {
            print!("Importing character super moves...");
            let existing = game.read_character_super_moves()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_character_super_moves(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_character_super_moves(&data)?;
        } else {
            println!("Skipping character super moves (file not found)");
        }
    }

    // Import shops (merge with existing)
    {
        let path = import_dir.join("shop.csv");
        if path.exists() {
            print!("Importing shops...");
            let existing = game.read_shops()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_shops(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_shops(&data)?;
        } else {
            println!("Skipping shops (file not found)");
        }
    }

    // Import treasure chests
    if let Some(data) = import_csv!(
        import_dir,
        "treasurechest.csv",
        import_treasure_chests,
        "treasure chests"
    ) {
        game.write_treasure_chests(&data)?;
    }

    // Import crew members (merge with existing)
    {
        let path = import_dir.join("crewmember.csv");
        if path.exists() {
            print!("Importing crew members...");
            let existing = game.read_crew_members()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_crew_members(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_crew_members(&data)?;
        } else {
            println!("Skipping crew members (file not found)");
        }
    }

    // Import playable ships (merge with existing)
    {
        let path = import_dir.join("playableship.csv");
        if path.exists() {
            print!("Importing playable ships...");
            let existing = game.read_playable_ships()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_playable_ships(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_playable_ships(&data)?;
        } else {
            println!("Skipping playable ships (file not found)");
        }
    }

    // Import ship cannons (merge with existing)
    {
        let path = import_dir.join("shipcannon.csv");
        if path.exists() {
            print!("Importing ship cannons...");
            let existing = game.read_ship_cannons()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_ship_cannons(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_ship_cannons(&data)?;
        } else {
            println!("Skipping ship cannons (file not found)");
        }
    }

    // Import ship accessories (merge with existing)
    {
        let path = import_dir.join("shipaccessory.csv");
        if path.exists() {
            print!("Importing ship accessories...");
            let existing = game.read_ship_accessories()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_ship_accessories(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_ship_accessories(&data)?;
        } else {
            println!("Skipping ship accessories (file not found)");
        }
    }

    // Import ship items (merge with existing)
    {
        let path = import_dir.join("shipitem.csv");
        if path.exists() {
            print!("Importing ship items...");
            let existing = game.read_ship_items()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_ship_items(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_ship_items(&data)?;
        } else {
            println!("Skipping ship items (file not found)");
        }
    }

    // Import enemy ships (merge with existing)
    {
        let path = import_dir.join("enemyship.csv");
        if path.exists() {
            print!("Importing enemy ships...");
            let existing = game.read_enemy_ships()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_enemy_ships(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_enemy_ships(&data)?;
        } else {
            println!("Skipping enemy ships (file not found)");
        }
    }

    // Import enemy magic (merge with existing)
    {
        let path = import_dir.join("enemymagic.csv");
        if path.exists() {
            print!("Importing enemy magic...");
            let existing = game.read_enemy_magic()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_enemy_magic(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_enemy_magic(&data)?;
        } else {
            println!("Skipping enemy magic (file not found)");
        }
    }

    // Import enemy super moves (merge with existing)
    {
        let path = import_dir.join("enemysupermove.csv");
        if path.exists() {
            print!("Importing enemy super moves...");
            let existing = game.read_enemy_super_moves()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_enemy_super_moves(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_enemy_super_moves(&data)?;
        } else {
            println!("Skipping enemy super moves (file not found)");
        }
    }

    // Import swashbucklers
    if let Some(data) = import_csv!(
        import_dir,
        "swashbuckler.csv",
        import_swashbucklers,
        "swashbucklers"
    ) {
        game.write_swashbucklers(&data)?;
    }

    // Import spirit curves
    if let Some(data) = import_csv!(
        import_dir,
        "spiritcurve.csv",
        import_spirit_curves,
        "spirit curves"
    ) {
        game.write_spirit_curves(&data)?;
    }

    // Import exp boosts
    if let Some(data) = import_csv!(import_dir, "expboost.csv", import_exp_boosts, "exp boosts") {
        game.write_exp_boosts(&data)?;
    }

    // Import EXP curves (from level file)
    {
        let path = import_dir.join("expcurve.csv");
        if path.exists() {
            print!("Importing exp curves...");
            // Need to load level file first
            game.load_level_file()?;
            let existing = game.read_exp_curves()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_exp_curves(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_exp_curves(&data)?;
        } else {
            println!("Skipping exp curves (file not found)");
        }
    }

    // Import Magic EXP curves (from level file)
    {
        let path = import_dir.join("magicexpcurve.csv");
        if path.exists() {
            print!("Importing magic exp curves...");
            // Need to load level file first (may already be loaded)
            game.load_level_file()?;
            let existing = game.read_magic_exp_curves()?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let data = CsvImporter::import_magic_exp_curves(reader, &existing)?;
            println!(" {} entries", data.len());
            game.write_magic_exp_curves(&data)?;
        } else {
            println!("Skipping magic exp curves (file not found)");
        }
    }

    // Note: Enemies from ENP/DAT files are not yet supported for import
    // (they require writing back to file-based storage, not just DOL)

    Ok(())
}

fn export_all(game: &mut GameRoot, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    export_csv!(
        game,
        output_dir,
        "accessories",
        read_accessories,
        export_accessories,
        "accessory.csv"
    );
    export_csv!(
        game,
        output_dir,
        "armors",
        read_armors,
        export_armors,
        "armor.csv"
    );

    // Weapons need weapon effects for effect name lookup
    print!("Exporting weapons...");
    let weapons = game.read_weapons()?;
    let weapon_effects = game.read_weapon_effects()?;
    CsvExporter::export_weapons(
        &weapons,
        File::create(output_dir.join("weapon.csv"))?,
        &weapon_effects,
    )?;
    println!(" {} entries", weapons.len());
    export_csv!(
        game,
        output_dir,
        "usable items",
        read_usable_items,
        export_usable_items,
        "usableitem.csv"
    );
    export_csv!(
        game,
        output_dir,
        "special items",
        read_special_items,
        export_special_items,
        "specialitem.csv"
    );
    export_csv!(
        game,
        output_dir,
        "characters",
        read_characters,
        export_characters,
        "character.csv"
    );
    export_csv!(
        game,
        output_dir,
        "character magic",
        read_character_magic,
        export_character_magic,
        "charactermagic.csv"
    );
    export_csv!(
        game,
        output_dir,
        "character super moves",
        read_character_super_moves,
        export_character_super_moves,
        "charactersupermove.csv"
    );

    // Build item database early for lookups (shops, treasure chests, and enemies need it)
    let item_db = game.build_item_database()?;

    // Shops need item database for item name lookup
    print!("Exporting shops...");
    let shops = game.read_shops()?;
    CsvExporter::export_shops(&shops, File::create(output_dir.join("shop.csv"))?, &item_db)?;
    println!(" {} entries", shops.len());

    // Treasure chests need item database for item name lookup
    print!("Exporting treasure chests...");
    let chests = game.read_treasure_chests()?;
    CsvExporter::export_treasure_chests(
        &chests,
        File::create(output_dir.join("treasurechest.csv"))?,
        &item_db,
    )?;
    println!(" {} entries", chests.len());

    export_csv!(
        game,
        output_dir,
        "crew members",
        read_crew_members,
        export_crew_members,
        "crewmember.csv"
    );
    export_csv!(
        game,
        output_dir,
        "playable ships",
        read_playable_ships,
        export_playable_ships,
        "playableship.csv"
    );
    export_csv!(
        game,
        output_dir,
        "ship cannons",
        read_ship_cannons,
        export_ship_cannons,
        "shipcannon.csv"
    );
    export_csv!(
        game,
        output_dir,
        "ship accessories",
        read_ship_accessories,
        export_ship_accessories,
        "shipaccessory.csv"
    );
    export_csv!(
        game,
        output_dir,
        "ship items",
        read_ship_items,
        export_ship_items,
        "shipitem.csv"
    );
    export_csv!(
        game,
        output_dir,
        "enemy ships",
        read_enemy_ships,
        export_enemy_ships,
        "enemyship.csv"
    );
    export_csv!(
        game,
        output_dir,
        "enemy magic",
        read_enemy_magic,
        export_enemy_magic,
        "enemymagic.csv"
    );
    export_csv!(
        game,
        output_dir,
        "enemy super moves",
        read_enemy_super_moves,
        export_enemy_super_moves,
        "enemysupermove.csv"
    );
    export_csv!(
        game,
        output_dir,
        "swashbucklers",
        read_swashbucklers,
        export_swashbucklers,
        "swashbuckler.csv"
    );
    export_csv!(
        game,
        output_dir,
        "spirit curves",
        read_spirit_curves,
        export_spirit_curves,
        "spiritcurve.csv"
    );
    export_csv!(
        game,
        output_dir,
        "exp boosts",
        read_exp_boosts,
        export_exp_boosts,
        "expboost.csv"
    );
    export_csv!(
        game,
        output_dir,
        "exp curves",
        read_exp_curves,
        export_exp_curves,
        "expcurve.csv"
    );
    export_csv!(
        game,
        output_dir,
        "magic exp curves",
        read_magic_exp_curves,
        export_magic_exp_curves,
        "magicexpcurve.csv"
    );

    // Enemies (from ENP files) - special handling for two outputs
    print!("Exporting enemies...");
    let (enemies, tasks) = game.read_enemies()?;
    // Use US enemy names from vocabulary
    let enemy_names = alx::lookups::enemy_names_map();
    CsvExporter::export_enemies(
        &enemies,
        File::create(output_dir.join("enemy.csv"))?,
        &item_db,
        &enemy_names,
    )?;
    CsvExporter::export_enemy_tasks(&tasks, File::create(output_dir.join("enemytask.csv"))?)?;
    println!(" {} enemies, {} tasks", enemies.len(), tasks.len());

    Ok(())
}
