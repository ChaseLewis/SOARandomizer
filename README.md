# SOA Randomizer

A Rust toolkit for reading and modifying **Skies of Arcadia Legends** (GameCube) game data.

## Overview

This project provides tools to extract game data from a GameCube ISO, export it to editable CSV files, and import modified data back into the ISO. It's designed as the foundation for a future randomizer.

We will create a UI to make this easier to work with at some point but this is a CLI manual tool currently. Still very new so I recommend writing to a copy of your ISO in case of corruption. Feel free to report anything. I have done a lot of automated testing so far to make sure things are good, but entirely possible stuff got missed at this early stage.

## Caveats
- We are focusing on the USA version currently. The base tool we are basing this on has the capabilities to work with any Gamecube version.

- This does not allow code level tweaks. This means things like damage multipliers, treasure chest drops, ship weapon values, etc are all possible to be edited.

- We currently don't export the encounter tables and make them editable. That would be nice and give it a more 'randomizer' vibe if we could hotswap out bosses for versions tuned to the appropriate level. So you could edit an existing bosses strength, but not switch what models spawn where.

- Effects are mapped by an integer to a specified effect in a table. So changing what certain things do is limited to the effects currently in game. Though magic numbers themselves can be tuned.

## Components

```
SOARandomizer/
├── bin/alx_rs/       # CLI tool for exporting/importing game data
├── libs/alx/         # Core library for parsing game formats
└── submodules/alx/   # Reference Ruby implementation (for validation)
```

## Quick Start

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- A **Skies of Arcadia Legends (USA)** GameCube ISO

### Build

```bash
cargo build --release
```

The binary will be at `target/release/alx_rs.exe` (Windows) or `target/release/alx_rs` (Linux/macOS).

### Export Game Data

Extract all game data to CSV files:

```bash
# Export to 'data' folder next to the ISO
alx_rs "path/to/game.iso"

# Export to a custom directory
alx_rs "path/to/game.iso" --output my_data
```

This creates 23 CSV files containing:
- **Equipment**: Weapons, Armor, Accessories
- **Items**: Usable Items, Special Items, Ship Items
- **Characters**: Stats, Magic, Super Moves
- **Ships**: Playable Ships, Cannons, Accessories
- **Enemies**: Enemy stats, Magic, Super Moves, Ships
- **World**: Shops, Treasure Chests, Crew Members, Swashbucklers

### Import Modified Data

After editing the CSV files, import them back:

```bash
# Import to a COPY of the ISO (recommended)
alx_rs --import data_folder "path/to/game.iso" --output "modified_game.iso"

# Import and modify the original ISO (prompts for confirmation)
alx_rs --import data_folder "path/to/game.iso"

# Skip confirmation prompts with -y
alx_rs --import data_folder "path/to/game.iso" -y
```

The `--output` flag copies the original ISO first, keeping it untouched.
Without `--output`, you'll be prompted to confirm before modifying the original.

## Exported Data Types

| File | Description | Count |
|------|-------------|-------|
| `accessory.csv` | Accessory equipment | 80 |
| `armor.csv` | Armor equipment | 80 |
| `weapon.csv` | Weapon equipment | 80 |
| `usableitem.csv` | Consumable items | 80 |
| `specialitem.csv` | Key/story items | 80 |
| `character.csv` | Playable character stats | 6 |
| `charactermagic.csv` | Character spells | 36 |
| `charactersupermove.csv` | Character S-Moves | 26 |
| `shop.csv` | Shop inventories | 43 |
| `treasurechest.csv` | Chest contents | 119 |
| `crewmember.csv` | Recruitable crew | 22 |
| `playableship.csv` | Player ships | 5 |
| `shipcannon.csv` | Ship weapons | 40 |
| `shipaccessory.csv` | Ship accessories | 40 |
| `shipitem.csv` | Ship consumables | 30 |
| `enemyship.csv` | Enemy vessels | 45 |
| `enemy.csv` | Enemy stats | 344 |
| `enemytask.csv` | Enemy AI/moves | ~1000 |
| `enemymagic.csv` | Enemy spells | 36 |
| `enemysupermove.csv` | Enemy special attacks | 309 |
| `swashbuckler.csv` | Swashbuckler ratings | 24 |
| `spiritcurve.csv` | SP regeneration curves | 6 |
| `expboost.csv` | EXP multipliers | 3 |

## Validation

When importing, the tool validates each entry:
- ID ranges are checked
- Required fields are verified
- Data types are validated

Invalid entries are reported with specific error messages.

## Development

### Running Tests

```bash
# Run all tests (requires ISO at roms/Skies of Arcadia Legends (USA).iso)
cargo test

# Run specific test suite
cargo test --package alx test_accessories
```

### Project Structure

See [`libs/alx/README.md`](libs/alx/README.md) for library documentation.

## Credits

This project builds upon the work of the original [ALX Ruby toolkit](https://github.com/Tsjerk/alx) by Tsjerk Hoekstra, which provided the reference implementation and data format documentation.

## License

MIT License - See LICENSE for details.

