# SOA Randomizer

A Rust toolkit for reading and modifying **Skies of Arcadia Legends** (GameCube) game data.

## Overview

This project provides tools to extract game data from a GameCube ISO, export it to editable CSV files, and import modified data back into the ISO. It's designed as the foundation for a future randomizer.

We will create a UI to make this easier to work with at some point but this is a CLI manual tool currently. Still very new so I recommend writing to a copy of your ISO in case of corruption. Feel free to report anything. I have done a lot of automated testing so far to make sure things are good, but entirely possible stuff got missed at this early stage.

## Caveats
- We are focusing on the USA version currently. The base tool we are basing this on has the capabilities to work with any Gamecube version.

- This does not allow code level tweaks. This means things like damage multipliers, treasure chest drops, ship weapon values, etc are all possible to be edited.

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

This creates CSV files and JSON encounter definitions containing:
- **Equipment**: Weapons, Armor, Accessories
- **Items**: Usable Items, Special Items, Ship Items
- **Characters**: Stats, Magic, Super Moves
- **Ships**: Playable Ships, Cannons, Accessories
- **Enemies**: Enemy stats, Magic, Super Moves, Ships
- **World**: Shops, Treasure Chests, Crew Members, Swashbucklers
- **Encounters**: ENP files (area encounters) and EVP files (scripted event battles)

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

## Texture Modding (`.mld` unpack / repack)

The game's models and UI live in **`.mld`** archives (AKLZ-compressed Sega Ninja
model containers). Each holds one or more **GVR** textures. The tool can unpack
every `.mld` into editable PNGs and repack edited PNGs back into the ISO.

### Unpack all textures

```bash
# Unpack every .mld in the ISO into per-file folders under "unpacked/"
alx_rs "path/to/game.iso" --full-unpack unpacked
```

For an ISO file like `battle\command.mld`, this produces
`unpacked/battle/command/` containing:

| File | Description |
|------|-------------|
| `command.bin` | The full AKLZ-decompressed archive (the repack base). |
| `texNN.png` | Each texture decoded to RGBA8 — **this is what you edit.** |
| `texNN.gvr` | The original carved GVR texture (raw, for reference). |
| `manifest.json` | The repack contract: per-texture format, offset, dimensions, and a pixel hash used to detect edits. |

The whole USA ISO unpacks to ~1791 folders and ~88,000 PNGs.

### Edit

Open any `texNN.png` in an image editor and change it. Keep the **same
dimensions** — resizing a texture will be rejected on repack. You can leave the
other files alone; `manifest.json` lists exactly which files belong to the
package, so stray files an editor drops in (lock files, backups) are ignored.

### Repack into the ISO

```bash
# Repack the edited folder into a COPY of the ISO
alx_rs "path/to/game.iso" --repack unpacked --output "modified_game.iso"

# Skip the overwrite prompt with -y
alx_rs "path/to/game.iso" --repack unpacked --output "modified_game.iso" -y
```

Repack copies the source ISO, then **re-encodes only the textures whose pixels
actually changed** (detected by comparing each PNG's pixels to the hash recorded
at unpack time). Each changed texture is encoded back to GVR and spliced into its
archive, which is then AKLZ-recompressed and written into the output ISO.
Untouched textures keep their original bytes exactly — no quality loss. It prints
each texture it re-encodes, e.g. `+ title/ts900000.mld tex00.png`.

**Caveats**
- Editing is per-PNG; `--repack` needs the same `manifest.json`/`.bin` layout
  produced by `--full-unpack`, so unpack and repack the same folder.
- Lossy formats (DXT1/CMP) lose a little quality only on textures you edit;
  unedited textures are never re-encoded.
- For **mipmapped** textures, only the base (full-size) level is re-encoded; the
  smaller mip levels keep their original pixels, so a heavy edit can look stale
  when the texture is drawn small/at a distance.

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

### Encounter Files (JSON)

In addition to CSV files, the tool exports editable JSON files for encounter data:

| Folder | Description | Files |
|--------|-------------|-------|
| `data/enp/` | Area encounter definitions (`.enp.json`) | ~90 |
| `data/evp/` | Scripted event battles (`epevent.evp.json`) | 1 |

**ENP files** define random encounters for each area, including:
- Which enemies can spawn in each encounter slot
- Enemy stats overrides for that area
- Encounter group configurations

**EVP files** define scripted story battles (boss fights, forced encounters), including:
- Character and enemy positions on the battlefield
- Battle conditions (can escape, defeat conditions)
- Enemy stats for event-specific encounters

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

### Pre-commit hook

A tracked pre-commit hook in [`.githooks/`](.githooks/pre-commit) runs the same
fast checks CI enforces (`cargo fmt --check` and `cargo clippy -D warnings`) so a
commit can't break the build on those steps. Enable it once per clone:

```bash
git config core.hooksPath .githooks
```

Bypass it for a single commit with `git commit --no-verify`. Tests are left out
of the hook (they need the ISO and are slow); see the comment in the hook to
enable them.

### Project Structure

See [`libs/alx/README.md`](libs/alx/README.md) for library documentation.

## Credits

This project builds upon the work of the original [ALX Ruby toolkit](https://github.com/Tsjerk/alx) by Tsjerk Hoekstra, which provided the reference implementation and data format documentation.

## License

MIT License - See LICENSE for details.

