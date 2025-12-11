# alx - Skies of Arcadia Data Library

A Rust library for reading and writing game data from **Skies of Arcadia Legends** (GameCube) ISOs.

## Features

- **ISO Parsing**: Read files directly from GameCube ISO images
- **AKLZ Decompression**: Handle compressed game files (`.enp`, `.evp`, `.dat`)
- **Binary Parsing**: Read/write game data structures with correct endianness
- **CSV Export/Import**: Convert game data to/from editable CSV format
- **Full Round-Trip**: Data can be read, modified, and written back

## Usage

### Opening a Game

```rust
use alx::game::GameRoot;

let mut game = GameRoot::open("path/to/game.iso")?;

println!("Region: {}", game.version().region);  // "US"
println!("Platform: {}", if game.version().is_gc() { "GameCube" } else { "Dreamcast" });
```

### Reading Game Data

```rust
// Read all accessories
let accessories = game.read_accessories()?;
for acc in &accessories {
    println!("{}: {} - Buy: {}g, Sell: {}g", 
        acc.id, acc.name_us, acc.buy_price, acc.sell_price);
}

// Read all weapons
let weapons = game.read_weapons()?;

// Read enemies (from ENP, EVP, and DAT files)
let enemies = game.read_enemies()?;
println!("Found {} enemies", enemies.len());  // 344
```

### Writing Game Data

```rust
// Read, modify, and write back
let mut accessories = game.read_accessories()?;

// Double all buy prices
for acc in &mut accessories {
    acc.buy_price *= 2;
}

// Write changes to the DOL
game.write_accessories(&accessories)?;

// Save the DOL back to the ISO
game.save_dol()?;
```

### CSV Export

```rust
use alx::csv::CsvExporter;
use std::fs::File;

let accessories = game.read_accessories()?;
let file = File::create("accessories.csv")?;
CsvExporter::export_accessories(&accessories, file)?;
```

### CSV Import

```rust
use alx::csv::CsvImporter;
use std::fs::File;
use std::io::BufReader;

let file = File::open("accessories.csv")?;
let reader = BufReader::new(file);
let accessories = CsvImporter::import_accessories(reader)?;

// Validate before writing
for acc in &accessories {
    // Validation is performed automatically during import
}

game.write_accessories(&accessories)?;
game.save_dol()?;
```

## Module Structure

```
alx/
├── game/           # High-level game interface
│   ├── root.rs     # GameRoot - main entry point
│   ├── offsets.rs  # Memory addresses for each data type
│   └── region.rs   # Region/version detection
├── entries/        # Data structure definitions
│   ├── accessory.rs
│   ├── weapon.rs
│   ├── enemy.rs
│   └── ... (24 entry types)
├── io/             # Low-level I/O
│   ├── iso.rs      # ISO file reading
│   ├── binary.rs   # Binary read/write traits
│   ├── aklz.rs     # AKLZ decompression
│   ├── enp.rs      # ENP/EVP/DAT parsing
│   └── strings.rs  # Text encoding (Shift-JIS, Windows-1252)
├── csv/            # CSV handling
│   ├── export.rs   # Data → CSV
│   └── import.rs   # CSV → Data (with validation)
├── items.rs        # Item database with effect lookups
└── lookups.rs      # Name/vocabulary lookups
```

## Entry Types

### Equipment
- `Accessory` - Rings, cloaks, etc.
- `Armor` - Body armor
- `Weapon` - Character weapons

### Items
- `UsableItem` - Consumables (healing, buffs)
- `SpecialItem` - Key items, story items
- `ShipItem` - Ship consumables

### Characters
- `Character` - Base stats, growth rates
- `CharacterMagic` - Spells (6 per element × 6 elements)
- `CharacterSuperMove` - S-Moves

### Ships
- `PlayableShip` - Player vessels (Little Jack, Delphinus, etc.)
- `ShipCannon` - Ship weapons
- `ShipAccessory` - Ship equipment

### Enemies
- `Enemy` - Stats, drops, element (344 unique)
- `EnemyTask` - AI behaviors and attack patterns
- `EnemyMagic` - Enemy spells
- `EnemySuperMove` - Boss attacks
- `EnemyShip` - Enemy vessels

### World
- `Shop` - Store inventories (up to 32 items each)
- `TreasureChest` - Chest contents and locations
- `CrewMember` - Recruitable crew with bonuses
- `Swashbuckler` - Rating thresholds

### Progression
- `SpiritCurve` - SP regeneration per character
- `ExpBoost` - Experience multipliers

## Data Sources

Game data is read from multiple sources within the ISO:

| Source | Data Types |
|--------|------------|
| `Start.dol` | Most equipment, items, characters, ships, shops |
| `*.enp` files | Enemy stats (AKLZ compressed) |
| `epevent.evp` | Additional enemies (AKLZ compressed) |
| `ecinit*.dat` | Area-specific enemies (AKLZ compressed) |
| `ebinit*.dat` | Boss enemies (AKLZ compressed) |

## Binary Format

All data uses **big-endian** byte order (GameCube native). The `BinaryRead` and `BinaryWrite` traits handle serialization:

```rust
use alx::io::BinaryRead;

impl BinaryRead for MyStruct {
    fn read_one<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            id: reader.read_i8()?,
            value: reader.read_i16_be()?,
            name: reader.read_fixed_string(16)?,
        })
    }
}
```

## Testing

The test suite validates against the reference Ruby ALX implementation:

```bash
# Run all tests
cargo test

# Test specific entry type
cargo test test_accessories

# Test round-trip (read → write → read)
cargo test test_roundtrip
```

Tests compare:
- Entry counts match reference CSVs
- All field values match exactly
- Binary round-trips produce identical data

## Dependencies

- `byteorder` - Big-endian binary I/O
- `encoding_rs` - Shift-JIS and Windows-1252 text
- `csv` - CSV parsing and generation
- `gc_fst` - GameCube FST (file system table) parsing

