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
    /// Path to the GameCube ISO file. Optional only for --build-iso, which
    /// rebuilds from an unpack directory and needs no source ISO.
    #[arg(value_name = "ISO_FILE")]
    iso_path: Option<PathBuf>,

    /// Output directory for CSV files (export mode), or output ISO path (import mode)
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Import mode: read CSVs from folder and write to ISO
    /// Use --output to write to a copy instead of modifying the original
    #[arg(short, long, value_name = "IMPORT_DIR")]
    import: Option<PathBuf>,

    /// Dump an ENP file's structure to JSON for debugging
    /// Example: --dump-enp a101b_ep.enp
    #[arg(long, value_name = "ENP_FILE")]
    dump_enp: Option<String>,

    /// Dump the EVP file's structure to JSON for debugging
    /// Example: --dump-evp
    #[arg(long)]
    dump_evp: bool,

    /// List all files in the ISO filesystem
    #[arg(long)]
    list_files: bool,

    /// Dump a file from the ISO as hex (substring match on filename)
    /// Example: --dump-file r500a.tec
    #[arg(long, value_name = "FILENAME")]
    dump_file: Option<String>,

    /// Extract a file from the ISO (decompresses AKLZ) to --output path
    #[arg(long, value_name = "FILENAME")]
    extract: Option<String>,

    /// Unpack every .mld archive in the ISO into per-file folders containing
    /// the decompressed blob, each GVR texture (.gvr + .png), and a
    /// manifest.json describing how to repack. Pass the output directory.
    #[arg(long, value_name = "OUTPUT_DIR")]
    full_unpack: Option<PathBuf>,

    /// Repack a folder produced by --full-unpack back into a copy of the ISO.
    /// Pass the unpack directory; use --output for the target ISO path.
    /// Only textures whose PNG actually changed are re-encoded.
    #[arg(long, value_name = "UNPACK_DIR")]
    repack: Option<PathBuf>,

    /// Unpack the ENTIRE ISO into an editable tree: every file decompressed
    /// (AKLZ) under files/, each .mld's textures under mld/, original compressed
    /// bytes cached under cache/, system files under &&systemdata/, plus a
    /// metadata.json. Rebuild later with --build-iso. Pass the output directory.
    #[arg(long, value_name = "OUTPUT_DIR")]
    unpack_iso: Option<PathBuf>,

    /// Rebuild a complete ISO from scratch from a --unpack-iso directory (no
    /// source ISO needed). Pass the unpack directory; use --output for the
    /// target ISO path. Unedited files reuse cached compressed bytes.
    #[arg(long, value_name = "UNPACK_DIR")]
    build_iso: Option<PathBuf>,

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

    // --build-iso rebuilds from an unpack directory and needs no source ISO,
    // so handle it before requiring the ISO_FILE argument.
    if let Some(ref unpack_dir) = args.build_iso {
        let output = args
            .output
            .as_deref()
            .ok_or("--build-iso requires --output <ISO> for the target ISO path")?;
        return run_build_iso(unpack_dir, output, args.yes);
    }

    // Every other command needs a source ISO.
    let iso_path = args
        .iso_path
        .as_deref()
        .ok_or("ISO_FILE argument is required")?;
    if !iso_path.exists() {
        return Err(format!("ISO file not found: {}", iso_path.display()).into());
    }

    // Check if we're in extract mode
    if let Some(ref filename) = args.extract {
        let output = args.output.as_deref().unwrap_or(Path::new("."));
        return run_extract(iso_path, filename, output);
    }

    // Check if we're in full-unpack mode
    if let Some(ref out_dir) = args.full_unpack {
        return run_full_unpack(iso_path, out_dir);
    }

    // Check if we're in unpack-iso mode (full ISO -> editable tree)
    if let Some(ref out_dir) = args.unpack_iso {
        return run_unpack_iso(iso_path, out_dir);
    }

    // Check if we're in repack mode
    if let Some(ref unpack_dir) = args.repack {
        let output = args
            .output
            .as_deref()
            .ok_or("--repack requires --output <ISO> for the target ISO path")?;
        return run_repack(iso_path, unpack_dir, output, args.yes);
    }

    // Check if we're in list-files mode
    if args.list_files {
        return run_list_files(iso_path, args.output.as_deref());
    }

    // Check if we're in dump-file mode
    if let Some(ref filename) = args.dump_file {
        return run_dump_file(iso_path, filename);
    }

    // Check if we're in dump-enp mode
    if let Some(enp_name) = args.dump_enp {
        return run_dump_enp(iso_path, &enp_name, args.output.as_deref());
    }

    // Check if we're in dump-evp mode
    if args.dump_evp {
        return run_dump_evp(iso_path, args.output.as_deref());
    }

    // Check if we're in import mode
    if let Some(import_dir) = args.import {
        return run_import(iso_path, &import_dir, args.output.as_deref(), args.yes);
    }

    // Export mode
    run_export(iso_path, args.output)
}

fn run_extract(
    iso_path: &Path,
    filename: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut iso = alx::io::IsoFile::open(iso_path)?;
    let matching = iso.list_files_matching(filename)?;
    if matching.is_empty() {
        return Err(format!("No files matching '{}'", filename).into());
    }
    for entry in &matching {
        let raw = iso.read_file_direct(entry)?;
        let data = if raw.len() >= 4 && &raw[0..4] == b"AKLZ" {
            let d = alx::io::decompress_aklz(&raw)?;
            println!("Decompressed {} -> {} bytes", raw.len(), d.len());
            d
        } else {
            raw
        };
        let out_file = if output_path.is_dir() {
            output_path.join(entry.path.file_name().unwrap_or_default())
        } else {
            output_path.to_path_buf()
        };
        std::fs::write(&out_file, &data)?;
        println!(
            "Extracted {} to {}",
            entry.path.display(),
            out_file.display()
        );
    }
    Ok(())
}

/// Write an RGBA8 image to a PNG file.
fn write_png(
    path: &Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    Ok(())
}

/// JSON-escape a string for manual manifest writing.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// Result of unpacking one `.mld` into its texture folder.
#[derive(Default)]
struct MldUnpackStats {
    textures: usize,
    pngs: usize,
    skipped: usize,
    unsupported: std::collections::BTreeSet<u8>,
}

/// Unpack one decompressed `.mld` blob into `folder`: writes `<stem>.bin`, each
/// `texNN.gvr` + decoded `texNN.png`, and a `manifest.json` (the repack
/// contract, with a `files` allowlist and per-texture `rgba_crc32`).
///
/// Shared by `--full-unpack` and `--unpack-iso`.
fn unpack_mld_folder(
    folder: &Path,
    stem: &str,
    iso_path_str: &str,
    blob: &[u8],
    aklz: bool,
) -> Result<MldUnpackStats, Box<dyn std::error::Error>> {
    use alx::io::{carve_textures, decode_gvr};

    fs::create_dir_all(folder)?;

    let blob_name = format!("{stem}.bin");
    fs::write(folder.join(&blob_name), blob)?;

    // Every file that belongs to this package. Repack only ever touches files
    // listed here / referenced by a texture, so stray files an image editor may
    // drop in (lock files, .tmp, backups) are ignored.
    let mut package_files: Vec<String> = vec![blob_name.clone()];
    let mut stats = MldUnpackStats::default();

    let textures = carve_textures(blob);
    stats.textures = textures.len();

    let mut tex_entries: Vec<String> = Vec::with_capacity(textures.len());
    for (i, tex) in textures.iter().enumerate() {
        let gvr_name = format!("tex{i:02}.gvr");
        fs::write(folder.join(&gvr_name), &tex.gvr)?;
        package_files.push(gvr_name.clone());

        let mut decoded = false;
        let mut crc_field = String::from("null");
        let mut png_field = String::from("null");

        match decode_gvr(&tex.gvr) {
            Ok(img) => {
                let png_name = format!("tex{i:02}.png");
                write_png(&folder.join(&png_name), img.width, img.height, &img.rgba)?;
                package_files.push(png_name.clone());
                let crc = crc32fast::hash(&img.rgba);
                decoded = true;
                stats.pngs += 1;
                crc_field = format!("\"0x{crc:08x}\"");
                png_field = format!("\"{}\"", json_escape(&png_name));
            }
            Err(_) => {
                stats.skipped += 1;
                stats.unsupported.insert(tex.data_format);
            }
        }

        tex_entries.push(format!(
            "    {{ \"index\": {}, \"png\": {}, \"gvr\": \"{}\", \
             \"blob_offset\": {}, \"gvr_len\": {}, \
             \"data_format\": {}, \"pixel_flags\": {}, \
             \"width\": {}, \"height\": {}, \"global_index\": {}, \
             \"decoded\": {}, \"rgba_crc32\": {} }}",
            i,
            png_field,
            json_escape(&gvr_name),
            tex.blob_offset,
            tex.gvr.len(),
            tex.data_format,
            tex.pixel_flags,
            tex.width,
            tex.height,
            tex.global_index,
            decoded,
            crc_field,
        ));
    }

    let files_list = package_files
        .iter()
        .map(|f| format!("\"{}\"", json_escape(f)))
        .collect::<Vec<_>>()
        .join(", ");
    let manifest = format!(
        "{{\n  \"iso_path\": \"{}\",\n  \"aklz\": {},\n  \"blob\": \"{}\",\n  \"files\": [{}],\n  \"textures\": [\n{}\n  ]\n}}\n",
        json_escape(iso_path_str),
        aklz,
        json_escape(&blob_name),
        files_list,
        tex_entries.join(",\n"),
    );
    fs::write(folder.join("manifest.json"), manifest)?;

    Ok(stats)
}

/// Reconstruct a `.mld`'s decompressed blob from its unpack `folder` + parsed
/// `manifest.json`: starts from `<blob>.bin` and, for every texture whose PNG's
/// pixels differ from the recorded `rgba_crc32`, re-encodes it to GVR and
/// splices it back in place. Returns `(blob, changed_textures, errors)`.
///
/// Shared by `--repack` and `--build-iso`.
fn rebuild_mld_blob(
    folder: &Path,
    manifest: &serde_json::Value,
) -> Result<(Vec<u8>, usize, usize), Box<dyn std::error::Error>> {
    use alx::io::encode_gvr;

    let iso_path = manifest["iso_path"].as_str().unwrap_or("<unknown>");
    let blob_name = manifest["blob"]
        .as_str()
        .ok_or_else(|| format!("{}: manifest missing blob", folder.display()))?;
    let mut blob = fs::read(folder.join(blob_name))?;

    let mut changed = 0usize;
    let mut errors = 0usize;

    if let Some(textures) = manifest["textures"].as_array() {
        for tex in textures {
            if !tex["decoded"].as_bool().unwrap_or(false) {
                continue;
            }
            let png_name = match tex["png"].as_str() {
                Some(p) => p,
                None => continue,
            };
            let png_path = folder.join(png_name);
            if !png_path.exists() {
                continue;
            }
            let off = tex["blob_offset"].as_u64().unwrap_or(0) as usize;
            let len = tex["gvr_len"].as_u64().unwrap_or(0) as usize;
            let expected = tex["rgba_crc32"]
                .as_str()
                .and_then(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).ok());

            let (_, _, rgba) = read_png_rgba(&png_path)?;
            let crc = crc32fast::hash(&rgba);
            if Some(crc) == expected {
                continue; // pixels unchanged -> keep original bytes
            }

            if off + len > blob.len() {
                eprintln!("  {iso_path} {png_name}: texture range out of blob bounds; skipping");
                errors += 1;
                continue;
            }
            let template = blob[off..off + len].to_vec();
            match encode_gvr(&template, &rgba) {
                Ok(new_gvr) if new_gvr.len() == len => {
                    blob[off..off + len].copy_from_slice(&new_gvr);
                    changed += 1;
                    println!("  + {iso_path} {png_name}");
                }
                Ok(_) => {
                    eprintln!("  {iso_path} {png_name}: re-encoded size mismatch; skipping");
                    errors += 1;
                }
                Err(e) => {
                    eprintln!("  {iso_path} {png_name}: {e}");
                    errors += 1;
                }
            }
        }
    }

    Ok((blob, changed, errors))
}

/// Unpack every `.mld` archive in the ISO into editable per-file folders.
///
/// For each `battle\foo.mld` the output is `<out_dir>/battle/foo/` containing:
///   - `foo.bin`       the full AKLZ-decompressed archive (repack base)
///   - `texNN.gvr`     each carved GVR texture (lossless source / inspection)
///   - `texNN.png`     the decoded texture (editable; canonical edit source)
///   - `manifest.json` the repack contract (offsets, formats, pixel CRC32s)
fn run_full_unpack(iso_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::decompress_aklz;

    println!("ALX_RS - Full .mld Unpacker");
    println!("==========================");
    println!("ISO: {}", iso_path.display());
    println!("Output: {}", out_dir.display());

    let mut iso = alx::io::IsoFile::open(iso_path)?;
    let all_files = iso.list_files()?;
    let mld_files: Vec<_> = all_files
        .into_iter()
        .filter(|e| {
            e.path
                .file_name()
                .map(|s| s.to_string_lossy().to_lowercase().ends_with(".mld"))
                .unwrap_or(false)
        })
        .collect();

    println!("Found {} .mld files\n", mld_files.len());

    let mut mld_ok = 0usize;
    let mut tex_total = 0usize;
    let mut png_total = 0usize;
    let mut png_skipped = 0usize;
    let mut unsupported_formats: std::collections::BTreeSet<u8> = std::collections::BTreeSet::new();

    for (n, entry) in mld_files.iter().enumerate() {
        // ISO path uses backslashes; normalize to forward slashes for output.
        let iso_path_str = entry.path.to_string_lossy().replace('\\', "/");
        let stem = entry
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("mld{n}"));
        let parent = entry
            .path
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        // Output folder mirrors the ISO tree, with the .mld becoming a folder.
        let mut folder = out_dir.to_path_buf();
        if !parent.is_empty() {
            for part in parent.split('/') {
                folder.push(part);
            }
        }
        folder.push(&stem);

        // Read and (if needed) AKLZ-decompress.
        let raw = iso.read_file_direct(entry)?;
        let was_aklz = raw.len() >= 4 && &raw[0..4] == b"AKLZ";
        let blob = if was_aklz {
            decompress_aklz(&raw)?
        } else {
            raw
        };

        let stats = unpack_mld_folder(&folder, &stem, &iso_path_str, &blob, was_aklz)?;
        tex_total += stats.textures;
        png_total += stats.pngs;
        png_skipped += stats.skipped;
        unsupported_formats.extend(stats.unsupported);

        mld_ok += 1;
        if (n + 1) % 100 == 0 || n + 1 == mld_files.len() {
            println!(
                "  [{}/{}] {} ({} textures)",
                n + 1,
                mld_files.len(),
                iso_path_str,
                stats.textures
            );
        }
    }

    println!("\nDone.");
    println!("  .mld unpacked: {mld_ok}");
    println!("  textures carved: {tex_total}");
    println!("  PNGs written: {png_total}");
    println!("  PNGs skipped (unsupported): {png_skipped}");
    if !unsupported_formats.is_empty() {
        let fmts: Vec<String> = unsupported_formats
            .iter()
            .map(|f| format!("0x{f:02x}"))
            .collect();
        println!("  unsupported formats seen: {}", fmts.join(", "));
    }

    Ok(())
}

/// Read a PNG file as tightly-packed RGBA8.
fn read_png_rgba(path: &Path) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    // Expand palettes/low-bit gray to 8-bit channels and drop 16-bit depth.
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let data = &buf[..info.buffer_size()];
    let (w, h) = (info.width, info.height);
    let n = (w * h) as usize;

    let rgba = match info.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity(n * 4);
            for px in data.chunks(3) {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        }
        png::ColorType::GrayscaleAlpha => {
            let mut out = Vec::with_capacity(n * 4);
            for px in data.chunks(2) {
                out.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
            }
            out
        }
        png::ColorType::Grayscale => {
            let mut out = Vec::with_capacity(n * 4);
            for &g in data {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        png::ColorType::Indexed => {
            return Err(format!("unexpected indexed PNG after expand: {}", path.display()).into());
        }
    };
    Ok((w, h, rgba))
}

/// Recursively collect every `manifest.json` under `dir`.
fn collect_manifests(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_manifests(&path, out)?;
        } else if path
            .file_name()
            .map(|n| n == "manifest.json")
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Repack a `--full-unpack` directory back into a copy of the ISO.
///
/// Each `manifest.json` is the contract: the `.bin` blob is the base, and for
/// every texture whose PNG's decoded pixels differ from the recorded
/// `rgba_crc32`, the PNG is re-encoded to GVR and spliced back into the blob
/// (in place, same byte length). Unchanged textures keep their original bytes,
/// so untouched textures never lose quality. Only `.mld` with at least one
/// changed texture are recompressed and written; everything else is inherited
/// unchanged from the copied ISO. Stray files in a package folder are ignored
/// because only manifest-referenced files are read.
fn run_repack(
    source_iso: &Path,
    unpack_dir: &Path,
    output_iso: &Path,
    auto_confirm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::compress_aklz;

    println!("ALX_RS - .mld Repacker");
    println!("=====================");
    println!("Source ISO: {}", source_iso.display());
    println!("Unpack dir: {}", unpack_dir.display());
    println!("Output ISO: {}", output_iso.display());

    if !unpack_dir.exists() {
        return Err(format!("Unpack directory not found: {}", unpack_dir.display()).into());
    }

    if output_iso.exists() && !auto_confirm {
        println!("\nOutput file already exists: {}", output_iso.display());
        if !confirm_overwrite()? {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Start from a fresh copy of the source ISO.
    println!("\nCopying ISO to output path...");
    fs::copy(source_iso, output_iso)?;
    println!(
        "  Copy complete ({:.1} GB)",
        fs::metadata(output_iso)?.len() as f64 / 1_000_000_000.0
    );

    let mut manifests = Vec::new();
    collect_manifests(unpack_dir, &mut manifests)?;
    manifests.sort();
    println!("Found {} packages\n", manifests.len());

    // Temp area for recompressed .mld awaiting batch insertion.
    let temp_root = std::env::temp_dir().join(format!(
        "alx_repack_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&temp_root)?;

    let mut inserts: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut changed_tex = 0usize;
    let mut changed_mld = 0usize;
    let mut errors = 0usize;

    for (mi, mpath) in manifests.iter().enumerate() {
        let folder = mpath.parent().unwrap_or(unpack_dir);
        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(mpath)?)?;

        let iso_path = json["iso_path"]
            .as_str()
            .ok_or_else(|| format!("{}: missing iso_path", mpath.display()))?;
        let aklz = json["aklz"].as_bool().unwrap_or(true);

        let (blob, changed, errs) = rebuild_mld_blob(folder, &json)?;
        changed_tex += changed;
        errors += errs;

        if changed > 0 {
            let final_bytes = if aklz { compress_aklz(&blob) } else { blob };
            let tmp = temp_root.join(format!("repack_{changed_mld}.mld"));
            fs::write(&tmp, &final_bytes)?;
            inserts.push((PathBuf::from(iso_path), tmp));
            changed_mld += 1;
        }

        if (mi + 1) % 200 == 0 || mi + 1 == manifests.len() {
            println!("  scanned [{}/{}]", mi + 1, manifests.len());
        }
    }

    if inserts.is_empty() {
        println!("\nNo textures changed. Output ISO is a faithful copy of the source.");
    } else {
        println!(
            "\nWriting {} changed .mld ({} textures) into the ISO...",
            changed_mld, changed_tex
        );
        let iso = alx::io::IsoFile::open(output_iso)?;
        iso.replace_files(&inserts)?;
    }

    let _ = fs::remove_dir_all(&temp_root);

    println!("\nDone.");
    println!("  packages scanned: {}", manifests.len());
    println!("  .mld changed: {changed_mld}");
    println!("  textures re-encoded: {changed_tex}");
    if errors > 0 {
        println!("  errors (textures skipped): {errors}");
    }
    println!("  output: {}", output_iso.display());

    Ok(())
}

/// Join a forward-slash-separated relative ISO path onto a base directory.
fn rel_join(base: &Path, rel: &str) -> PathBuf {
    let mut p = base.to_path_buf();
    for part in rel.split('/') {
        if !part.is_empty() {
            p.push(part);
        }
    }
    p
}

/// Ensure the parent directory of `path` exists.
fn ensure_parent(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Unpack the ENTIRE ISO into an editable tree (see `--unpack-iso`).
fn run_unpack_iso(iso_path: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{decompress_aklz, is_aklz};

    println!("ALX_RS - Full ISO Unpacker");
    println!("==========================");
    println!("ISO: {}", iso_path.display());
    println!("Output: {}", out_dir.display());

    let mut iso = alx::io::IsoFile::open(iso_path)?;

    // System files needed for a from-scratch rebuild.
    iso.extract_system_files(&out_dir.join("&&systemdata"))?;

    let files = iso.list_files()?;
    println!("Found {} files\n", files.len());

    let files_dir = out_dir.join("files");
    let mld_dir = out_dir.join("mld");
    let cache_dir = out_dir.join("cache");

    let mut meta_entries: Vec<String> = Vec::with_capacity(files.len());
    let mut n_mld = 0usize;
    let mut n_aklz = 0usize;
    let mut tex_total = 0usize;

    for (n, entry) in files.iter().enumerate() {
        let iso_path_str = entry.path.to_string_lossy().replace('\\', "/");
        let raw = iso.read_file_direct(entry)?;
        let aklz = is_aklz(&raw);
        let decompressed = if aklz {
            decompress_aklz(&raw)?
        } else {
            raw.clone()
        };

        // Cache the original compressed bytes for fast, lossless rebuild.
        if aklz {
            let cpath = rel_join(&cache_dir, &iso_path_str);
            ensure_parent(&cpath)?;
            fs::write(&cpath, &raw)?;
            n_aklz += 1;
        }

        let crc = crc32fast::hash(&decompressed);
        let is_mld = iso_path_str.to_lowercase().ends_with(".mld");

        if is_mld {
            let stem = entry
                .path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("mld{n}"));
            // mld/<dir-with-.mld-stripped>, e.g. battle/command.mld -> mld/battle/command
            let dir_rel = iso_path_str
                .strip_suffix(".mld")
                .or_else(|| iso_path_str.strip_suffix(".MLD"))
                .unwrap_or(&iso_path_str)
                .to_string();
            let folder = rel_join(&mld_dir, &dir_rel);
            let stats = unpack_mld_folder(&folder, &stem, &iso_path_str, &decompressed, aklz)?;
            tex_total += stats.textures;
            n_mld += 1;
            meta_entries.push(format!(
                "    {{ \"path\": \"{}\", \"aklz\": {}, \"kind\": \"mld\", \"dir\": \"{}\", \"crc32\": \"0x{:08x}\" }}",
                json_escape(&iso_path_str),
                aklz,
                json_escape(&dir_rel),
                crc
            ));
        } else {
            let fpath = rel_join(&files_dir, &iso_path_str);
            ensure_parent(&fpath)?;
            fs::write(&fpath, &decompressed)?;
            meta_entries.push(format!(
                "    {{ \"path\": \"{}\", \"aklz\": {}, \"kind\": \"file\", \"crc32\": \"0x{:08x}\" }}",
                json_escape(&iso_path_str),
                aklz,
                crc
            ));
        }

        if (n + 1) % 500 == 0 || n + 1 == files.len() {
            println!("  [{}/{}]", n + 1, files.len());
        }
    }

    let metadata = format!(
        "{{\n  \"files\": [\n{}\n  ]\n}}\n",
        meta_entries.join(",\n")
    );
    fs::write(out_dir.join("metadata.json"), metadata)?;

    println!("\nDone.");
    println!("  files unpacked: {}", files.len());
    println!("  AKLZ files (cached): {n_aklz}");
    println!("  .mld with textures: {n_mld} ({tex_total} textures)");
    println!("  output: {}", out_dir.display());

    Ok(())
}

/// Rebuild a complete ISO from a `--unpack-iso` directory (see `--build-iso`).
fn run_build_iso(
    unpack_dir: &Path,
    output_iso: &Path,
    auto_confirm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{build_iso, compress_aklz};

    println!("ALX_RS - ISO Builder (from scratch)");
    println!("===================================");
    println!("Unpack dir: {}", unpack_dir.display());
    println!("Output ISO: {}", output_iso.display());

    let meta_path = unpack_dir.join("metadata.json");
    if !meta_path.exists() {
        return Err(format!("metadata.json not found in {}", unpack_dir.display()).into());
    }
    if output_iso.exists() && !auto_confirm {
        println!("\nOutput file already exists: {}", output_iso.display());
        if !confirm_overwrite()? {
            println!("Aborted.");
            return Ok(());
        }
    }

    let meta: serde_json::Value = serde_json::from_str(&fs::read_to_string(&meta_path)?)?;
    let entries = meta["files"]
        .as_array()
        .ok_or("metadata.json: missing files array")?;

    let files_dir = unpack_dir.join("files");
    let mld_dir = unpack_dir.join("mld");
    let cache_dir = unpack_dir.join("cache");
    let sysdata = unpack_dir.join("&&systemdata");
    if !sysdata.join("ISO.hdr").exists() {
        return Err(format!("missing &&systemdata in {}", unpack_dir.display()).into());
    }

    // Stage the disc tree (with final, compressed-where-needed bytes) into a
    // temp dir, then let gc_fst build the ISO from it.
    let staging = std::env::temp_dir().join(format!(
        "alx_build_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&staging)?;

    // Copy system files into staging/&&systemdata.
    let staging_sys = staging.join("&&systemdata");
    fs::create_dir_all(&staging_sys)?;
    for f in ["ISO.hdr", "AppLoader.ldr", "Start.dol"] {
        fs::copy(sysdata.join(f), staging_sys.join(f))?;
    }

    let mut cached = 0usize;
    let mut recompressed = 0usize;
    let mut changed_mld = 0usize;

    for (i, entry) in entries.iter().enumerate() {
        let path = entry["path"].as_str().ok_or("file entry missing path")?;
        let aklz = entry["aklz"].as_bool().unwrap_or(false);
        let kind = entry["kind"].as_str().unwrap_or("file");
        let expected_crc = entry["crc32"]
            .as_str()
            .and_then(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).ok());

        // Reconstruct the decompressed bytes.
        let decompressed: Vec<u8> = if kind == "mld" {
            let dir = entry["dir"].as_str().ok_or("mld entry missing dir")?;
            let folder = rel_join(&mld_dir, dir);
            let manifest: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(folder.join("manifest.json"))?)?;
            let (blob, changed, _errs) = rebuild_mld_blob(&folder, &manifest)?;
            if changed > 0 {
                changed_mld += 1;
            }
            blob
        } else {
            fs::read(rel_join(&files_dir, path))?
        };

        // Choose final bytes: reuse cache if unchanged, else (re)compress.
        let final_bytes: Vec<u8> = if aklz {
            let crc = crc32fast::hash(&decompressed);
            let cache_path = rel_join(&cache_dir, path);
            if Some(crc) == expected_crc && cache_path.exists() {
                cached += 1;
                fs::read(&cache_path)?
            } else {
                recompressed += 1;
                compress_aklz(&decompressed)
            }
        } else {
            decompressed
        };

        let staged = rel_join(&staging, path);
        ensure_parent(&staged)?;
        fs::write(&staged, &final_bytes)?;

        if (i + 1) % 500 == 0 || i + 1 == entries.len() {
            println!("  staged [{}/{}]", i + 1, entries.len());
        }
    }

    println!("\nBuilding ISO image...");
    let iso_bytes = build_iso(&staging)?;
    fs::write(output_iso, &iso_bytes)?;

    let _ = fs::remove_dir_all(&staging);

    println!("\nDone.");
    println!("  files written: {}", entries.len());
    println!("  reused from cache: {cached}");
    println!("  recompressed: {recompressed}");
    println!("  .mld with edited textures: {changed_mld}");
    println!(
        "  output: {} ({:.2} GB)",
        output_iso.display(),
        iso_bytes.len() as f64 / 1_000_000_000.0
    );

    Ok(())
}

fn run_list_files(
    iso_path: &Path,
    output_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut iso = alx::io::IsoFile::open(iso_path)?;
    let files = iso.list_files()?;

    let mut output_lines = Vec::new();
    for f in &files {
        output_lines.push(format!(
            "{:>10}  0x{:08X}  {}",
            f.size,
            f.offset,
            f.path.display()
        ));
    }

    let text = output_lines.join("\n");
    if let Some(output) = output_path {
        std::fs::write(output, &text)?;
        println!("Listed {} files to {}", files.len(), output.display());
    } else {
        println!("{}", text);
        println!("\n{} files total", files.len());
    }

    Ok(())
}

fn run_dump_file(iso_path: &Path, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Special mode: "search-dol:VALUE" to search for i16 patterns in Start.dol
    if let Some(search_val) = filename.strip_prefix("search-dol:") {
        return run_search_dol(iso_path, search_val);
    }
    // Special mode: "dol-range:START..END" to dump a DOL range
    if let Some(range_str) = filename.strip_prefix("dol-range:") {
        return run_dump_dol_range(iso_path, range_str);
    }
    // Special mode: "search-bytes:HEXBYTES" to search ISO files for byte pattern
    if let Some(hex_str) = filename.strip_prefix("search-bytes:") {
        return run_search_bytes(iso_path, hex_str);
    }

    let mut iso = alx::io::IsoFile::open(iso_path)?;
    let matching = iso.list_files_matching(filename)?;

    if matching.is_empty() {
        return Err(format!("No files matching '{}'", filename).into());
    }

    for entry in &matching {
        println!(
            "=== {} (offset=0x{:08X}, size={}) ===",
            entry.path.display(),
            entry.offset,
            entry.size
        );

        let raw_data = iso.read_file_direct(entry)?;

        // Decompress AKLZ if applicable
        let data = if raw_data.len() >= 4 && &raw_data[0..4] == b"AKLZ" {
            let decompressed = alx::io::decompress_aklz(&raw_data)?;
            println!(
                "  (AKLZ compressed: {} -> {} bytes)",
                raw_data.len(),
                decompressed.len()
            );
            decompressed
        } else {
            raw_data
        };

        // Print hex dump (limited to first 512 bytes for sanity)
        let limit = data.len().min(512);
        for (i, chunk) in data[..limit].chunks(16).enumerate() {
            let offset = i * 16;
            // Hex part
            let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
            let hex_str = hex.join(" ");
            // ASCII part
            let ascii: String = chunk
                .iter()
                .map(|&b| {
                    if (0x20..=0x7e).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            println!("{:08X}  {:48}  {}", offset, hex_str, ascii);
        }
        if data.len() > limit {
            println!("... ({} more bytes)", data.len() - limit);
        }

        // Also print as i16 big-endian values for the first 128 values
        println!("\n--- As i16 BE values ---");
        let i16_limit = data.len().min(256);
        for (i, chunk) in data[..i16_limit].chunks(2).enumerate() {
            if chunk.len() == 2 {
                let val = i16::from_be_bytes([chunk[0], chunk[1]]);
                if i % 8 == 0 {
                    if i > 0 {
                        println!();
                    }
                    print!("{:04X}: ", i * 2);
                }
                print!("{:>6} ", val);
            }
        }
        println!("\n");
    }

    Ok(())
}

/// Search all files in the ISO for a byte pattern (hex string).
fn run_search_bytes(iso_path: &Path, hex_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse hex string
    let pattern: Vec<u8> = hex_str
        .split_whitespace()
        .flat_map(|s| {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>()
        })
        .collect();

    println!(
        "Searching for pattern: {:02X?} ({} bytes)",
        pattern,
        pattern.len()
    );

    let mut iso = alx::io::IsoFile::open(iso_path)?;
    let files = iso.list_files()?;

    for entry in &files {
        let path_str = entry.path.to_string_lossy().to_string();
        // Skip very large files (models/textures)
        if entry.size > 500_000 {
            continue;
        }

        let raw_data = match iso.read_file_direct(entry) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Try decompressed if AKLZ
        let data = if raw_data.len() >= 4 && &raw_data[0..4] == b"AKLZ" {
            match alx::io::decompress_aklz(&raw_data) {
                Ok(d) => d,
                Err(_) => raw_data.clone(),
            }
        } else {
            raw_data.clone()
        };

        // Search for pattern
        for i in 0..data.len().saturating_sub(pattern.len()) {
            if data[i..i + pattern.len()] == pattern[..] {
                println!(
                    "FOUND in {} at offset 0x{:04X} (file size: {}, decompressed: {})",
                    path_str,
                    i,
                    entry.size,
                    data.len()
                );
                // Show context
                let ctx_start = i.saturating_sub(16);
                let ctx_end = (i + pattern.len() + 16).min(data.len());
                for (j, chunk) in data[ctx_start..ctx_end].chunks(16).enumerate() {
                    let offset = ctx_start + j * 16;
                    let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
                    println!("  {:04X}: {}", offset, hex.join(" "));
                }
            }
        }
    }

    println!("Search complete.");
    Ok(())
}

/// Dump a range of Start.dol as i16 values
fn run_dump_dol_range(iso_path: &Path, range_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = range_str.split("..").collect();
    if parts.len() != 2 {
        return Err("Expected format: start..end (hex, e.g. 2d168c..2d29e8)".into());
    }
    let start = usize::from_str_radix(parts[0], 16)?;
    let end = usize::from_str_radix(parts[1], 16)?;

    let game = GameRoot::open(iso_path)?;
    let dol_path = Path::new("Start.dol");
    let data = game.iso().read_file(dol_path)?;

    println!(
        "Dumping DOL range 0x{:06X}..0x{:06X} ({} bytes)",
        start,
        end,
        end - start
    );
    let slice = &data[start..end];

    // Hex dump
    for (i, chunk) in slice.chunks(16).enumerate() {
        let offset = start + i * 16;
        let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
        let hex_str = hex.join(" ");
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if (0x20..=0x7e).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        println!("{:06X}  {:48}  {}", offset, hex_str, ascii);
    }

    // As i16 values with alignment annotations
    println!("\n--- As i16 BE values (8 per line) ---");
    for (i, chunk) in slice.chunks(2).enumerate() {
        if chunk.len() == 2 {
            let offset = start + i * 2;
            let val = i16::from_be_bytes([chunk[0], chunk[1]]);
            if i % 8 == 0 {
                if i > 0 {
                    println!();
                }
                print!("{:06X}: ", offset);
            }
            print!("{:>6} ", val);
        }
    }
    println!();

    Ok(())
}

/// Search Start.dol for regions containing clusters of a specific i16 value.
/// This helps find data tables with known default values.
fn run_search_dol(iso_path: &Path, search_val: &str) -> Result<(), Box<dyn std::error::Error>> {
    let game = GameRoot::open(iso_path)?;
    let dol_path = Path::new("Start.dol");
    let data = game.iso().read_file(dol_path)?;

    let target: i16 = search_val.parse()?;
    let target_bytes = target.to_be_bytes();

    println!(
        "Searching Start.dol ({} bytes) for clusters of i16 value {}...",
        data.len(),
        target
    );
    println!(
        "Target bytes: {:02X} {:02X}",
        target_bytes[0], target_bytes[1]
    );

    // Find all positions where the target appears
    let mut positions = Vec::new();
    for i in 0..data.len() - 1 {
        if data[i] == target_bytes[0] && data[i + 1] == target_bytes[1] {
            positions.push(i);
        }
    }

    println!("Found {} occurrences total", positions.len());

    // Find clusters (groups of occurrences within 64 bytes of each other)
    let mut cluster_start = 0;
    let mut cluster_count = 1;
    let mut clusters = Vec::new();

    for i in 1..positions.len() {
        if positions[i] - positions[i - 1] <= 64 {
            cluster_count += 1;
        } else {
            if cluster_count >= 3 {
                clusters.push((positions[cluster_start], positions[i - 1], cluster_count));
            }
            cluster_start = i;
            cluster_count = 1;
        }
    }
    if cluster_count >= 3 {
        clusters.push((
            positions[cluster_start],
            *positions.last().unwrap(),
            cluster_count,
        ));
    }

    println!("\nClusters of 3+ occurrences within 64 bytes:");
    for (start, end, count) in &clusters {
        // Show context around the cluster
        let ctx_start = start.saturating_sub(16);
        let ctx_end = (end + 16).min(data.len());
        println!(
            "\n--- Cluster at 0x{:06X}..0x{:06X} ({} hits) ---",
            start, end, count
        );

        // Print as i16 values
        let aligned_start = ctx_start & !1; // align to 2-byte boundary
        for (i, chunk) in data[aligned_start..ctx_end].chunks(2).enumerate() {
            if chunk.len() == 2 {
                let offset = aligned_start + i * 2;
                let val = i16::from_be_bytes([chunk[0], chunk[1]]);
                if i % 16 == 0 {
                    if i > 0 {
                        println!();
                    }
                    print!("{:06X}: ", offset);
                }
                if val == target {
                    print!("[{:>4}]", val);
                } else {
                    print!(" {:>4} ", val);
                }
            }
        }
        println!();
    }

    Ok(())
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

fn run_dump_enp(
    iso_path: &Path,
    enp_name: &str,
    output_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{decompress_aklz, dump_enp_editable};

    println!("ALX_RS - ENP File Dumper");
    println!("========================");
    println!("ISO: {}", iso_path.display());
    println!("ENP: {}", enp_name);

    // Load ISO
    let mut game = GameRoot::open(iso_path)?;
    println!("Detected: {:?}", game.version());

    // Build item database for item name lookups
    let item_db = game.build_item_database()?;

    // Find the ENP file
    let pattern = if enp_name.contains(".enp") {
        enp_name.replace(".enp", "")
    } else {
        enp_name.to_string()
    };

    let matching_files = game.iso_mut().list_files_matching(&pattern)?;

    let mut found = false;
    for entry in &matching_files {
        let filename = entry
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        if !filename.ends_with(".enp") {
            continue;
        }

        if !filename.contains(&pattern) {
            continue;
        }

        println!("\nFound: {}", filename);

        // Read and decompress the file
        let raw_data = game.iso_mut().read_file_direct(entry)?;
        let data = decompress_aklz(&raw_data)?;

        println!("  Compressed size: {} bytes", raw_data.len());
        println!("  Decompressed size: {} bytes", data.len());

        // Dump the structure using simplified editable format
        let dump = dump_enp_editable(&data, &filename, game.version(), &item_db)?;

        // Convert to JSON
        let json = serde_json::to_string_pretty(&dump)?;

        // Output
        if let Some(output) = output_path {
            let output_file = if output.is_dir() {
                output.join(format!("{}.json", filename))
            } else {
                output.to_path_buf()
            };
            std::fs::write(&output_file, &json)?;
            println!("  Written to: {}", output_file.display());
        } else {
            println!("\n{}", json);
        }

        found = true;
        break;
    }

    if !found {
        return Err(format!("ENP file not found: {}", enp_name).into());
    }

    Ok(())
}

fn run_dump_evp(
    iso_path: &Path,
    output_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{decompress_aklz, dump_evp_editable};

    println!("ALX_RS - EVP File Dumper");
    println!("========================");
    println!("ISO: {}", iso_path.display());

    // Load ISO
    let mut game = GameRoot::open(iso_path)?;
    println!("Detected: {:?}", game.version());

    // Build item database for item name lookups
    let item_db = game.build_item_database()?;

    // Find the EVP file
    let matching_files = game.iso_mut().list_files_matching("epevent.evp")?;

    let entry = matching_files
        .first()
        .ok_or("EVP file (epevent.evp) not found")?;

    let filename = entry
        .path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    println!("\nDumping: {}", filename);

    // Read and decompress the file
    let raw_data = game.iso_mut().read_file_direct(entry)?;
    let data = decompress_aklz(&raw_data)?;
    println!("  File size: {} bytes (uncompressed)", data.len());

    // Dump the structure using simplified editable format
    let dump = dump_evp_editable(&data, &filename, game.version(), &item_db)?;

    println!("  Enemies: {}", dump.enemies.len());
    println!("  Events: {}", dump.events.len());

    // Convert to JSON
    let json = serde_json::to_string_pretty(&dump)?;

    // Output
    if let Some(output) = output_path {
        let output_file = if output.is_dir() {
            output.join(format!("{}.json", filename))
        } else {
            output.to_path_buf()
        };
        std::fs::write(&output_file, &json)?;
        println!("  Written to: {}", output_file.display());
    } else {
        println!("\n{}", json);
    }

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

    // Note: Enemy encounters are now imported via ENP JSON files, not CSV
    // The CSV export is kept for reference/documentation purposes

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

    // Import ENP files from JSON
    import_enp_files(game, import_dir)?;

    // Import EVP file from JSON
    import_evp_file(game, import_dir)?;

    Ok(())
}

fn import_evp_file(
    game: &mut GameRoot,
    import_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{build_evp, EvpDefinition};

    let evp_dir = import_dir.join("evp");
    if !evp_dir.exists() {
        println!("Skipping EVP file (evp/ directory not found)");
        return Ok(());
    }

    // Look for the epevent.evp.json file
    let evp_file = evp_dir.join("epevent.evp.json");
    if !evp_file.exists() {
        println!("Skipping EVP file (epevent.evp.json not found)");
        return Ok(());
    }

    print!("Importing EVP file...");
    std::io::Write::flush(&mut std::io::stdout())?;

    // Build item database for reverse lookup (name -> ID)
    let item_db = game.build_item_database()?;

    // Build enemy database from the original EVP file
    let file_db = game.build_enemy_database_for_evp()?;

    // Build global enemy database as fallback
    let global_db = game.build_global_enemy_database()?;

    // Read and parse JSON
    let json_content = std::fs::read_to_string(&evp_file)?;
    let def: EvpDefinition = match serde_json::from_str(&json_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("\n  Error parsing {}: {}", evp_file.display(), e);
            return Err(e.into());
        }
    };

    // Build the EVP file with patched data
    let evp_data = match build_evp(&def, &file_db, Some(&global_db), &item_db) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("\n  Error building EVP: {}", e);
            return Err(e.into());
        }
    };

    // Write back to ISO
    match game.write_evp_file(&evp_data) {
        Ok(()) => {
            println!(
                " {} enemies, {} events",
                def.enemies.len(),
                def.events.len()
            );
        }
        Err(e) => {
            eprintln!("\n  Error writing EVP: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

fn import_enp_files(
    game: &mut GameRoot,
    import_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{
        bake_enp_segments, build_enp, decompress_aklz, EnpDefinition, A099A_BAKED_FILENAME,
        A099A_SEGMENTS,
    };

    let enp_dir = import_dir.join("enp");
    if !enp_dir.exists() {
        println!("Skipping ENP files (enp/ directory not found)");
        return Ok(());
    }

    print!("Importing ENP files...");
    std::io::Write::flush(&mut std::io::stdout())?;

    // Build item database for reverse lookup (name -> ID)
    let item_db = game.build_item_database()?;

    // Build global enemy database (all enemies from all files)
    // This is used as a fallback when an enemy isn't in the current file
    let global_db = game.build_global_enemy_database()?;

    // Track if any a099a files were imported (need rebaking)
    let mut a099a_imported = false;

    // Find all JSON files in enp directory
    let mut count = 0;
    let mut errors = 0;

    for entry in std::fs::read_dir(&enp_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            // Read and parse JSON
            let json_content = std::fs::read_to_string(&path)?;
            let def: EnpDefinition = match serde_json::from_str(&json_content) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("\n  Error parsing {}: {}", path.display(), e);
                    errors += 1;
                    continue;
                }
            };

            // Check if this is an a099a segment file
            if A099A_SEGMENTS.contains(&def.filename.as_str()) {
                a099a_imported = true;
            }

            // Build enemy database from THIS specific ENP file's original data
            let file_db = match game.build_enemy_database_for_file(&def.filename) {
                Ok(db) => db,
                Err(e) => {
                    eprintln!("\n  Error reading original {}: {}", def.filename, e);
                    errors += 1;
                    continue;
                }
            };

            // Build the ENP file with patched data
            // Uses file-specific DB first, then falls back to global DB for "stolen" enemies
            let enp_data = match build_enp(&def, &file_db, Some(&global_db), &item_db) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("\n  Error building {}: {}", def.filename, e);
                    errors += 1;
                    continue;
                }
            };

            // Write back to ISO
            match game.write_enp_file(&def.filename, &enp_data) {
                Ok(()) => count += 1,
                Err(e) => {
                    eprintln!("\n  Error writing {}: {}", def.filename, e);
                    errors += 1;
                }
            }
        }
    }

    if errors > 0 {
        println!(" {} files ({} errors)", count, errors);
    } else {
        println!(" {} files", count);
    }

    // Rebake a099a_ep.enp if any segment files were imported
    if a099a_imported {
        print!("Rebaking {}...", A099A_BAKED_FILENAME);
        std::io::Write::flush(&mut std::io::stdout())?;

        // Read all 13 segment files from the ISO (they've just been updated)
        let mut segments: Vec<(String, Vec<u8>)> = Vec::new();
        let mut rebake_ok = true;

        for seg_name in A099A_SEGMENTS {
            // Find and read the segment file from ISO
            match game.read_enp_file_raw(seg_name) {
                Ok(compressed) => {
                    // Decompress the segment
                    match decompress_aklz(&compressed) {
                        Ok(decompressed) => {
                            segments.push((seg_name.to_string(), decompressed));
                        }
                        Err(e) => {
                            eprintln!("\n  Error decompressing {}: {}", seg_name, e);
                            rebake_ok = false;
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n  Error reading {}: {}", seg_name, e);
                    rebake_ok = false;
                    break;
                }
            }
        }

        if rebake_ok {
            // Create segment references for baking
            let segment_refs: Vec<(&str, &[u8])> = segments
                .iter()
                .map(|(name, data)| (name.as_str(), data.as_slice()))
                .collect();

            // Bake into multi-segment format
            match bake_enp_segments(&segment_refs) {
                Ok(baked) => {
                    // Write to ISO (write_enp_file handles compression)
                    match game.write_enp_file(A099A_BAKED_FILENAME, &baked) {
                        Ok(()) => println!(" done ({} bytes uncompressed)", baked.len()),
                        Err(e) => eprintln!("\n  Error writing {}: {}", A099A_BAKED_FILENAME, e),
                    }
                }
                Err(e) => {
                    eprintln!("\n  Error baking {}: {}", A099A_BAKED_FILENAME, e);
                }
            }
        }
    }

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

    // Build item database early for lookups (characters, shops, treasure chests, and enemies need it)
    let item_db = game.build_item_database()?;

    // Characters need item database for equipment name lookup
    print!("Exporting characters...");
    let characters = game.read_characters()?;
    CsvExporter::export_characters(
        &characters,
        &item_db,
        File::create(output_dir.join("character.csv"))?,
    )?;
    println!(" {} entries", characters.len());

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

    // Build lookups for enemy task names (magic and super moves)
    let enemy_magic_data = game.read_enemy_magic()?;
    let enemy_super_moves_data = game.read_enemy_super_moves()?;

    let mut enemy_magic_names: std::collections::HashMap<u32, String> =
        std::collections::HashMap::new();
    for m in &enemy_magic_data {
        enemy_magic_names.insert(m.id, m.name.clone());
    }

    let mut enemy_super_move_names: std::collections::HashMap<u32, String> =
        std::collections::HashMap::new();
    for s in &enemy_super_moves_data {
        enemy_super_move_names.insert(s.id, s.name.clone());
    }

    CsvExporter::export_enemy_tasks(
        &tasks,
        &enemies,
        &enemy_magic_names,
        &enemy_super_move_names,
        File::create(output_dir.join("enemytask.csv"))?,
    )?;
    println!(" {} enemies, {} tasks", enemies.len(), tasks.len());

    // Enemy encounters (from ENP files)
    print!("Exporting enemy encounters...");
    let encounters = game.read_enemy_encounters()?;
    // Build enemy name lookup map for encounters (id -> (jp_name, us_name))
    let mut encounter_enemy_names: std::collections::HashMap<u32, (String, String)> =
        std::collections::HashMap::new();
    for enemy in &enemies {
        let us_name = enemy_names
            .get(&enemy.id)
            .cloned()
            .unwrap_or_else(|| "???".to_string());
        encounter_enemy_names.insert(enemy.id, (enemy.name_jp.clone(), us_name));
    }
    CsvExporter::export_enemy_encounters(
        &encounters,
        File::create(output_dir.join("enemyencounter.csv"))?,
        &encounter_enemy_names,
    )?;
    println!(" {} encounters", encounters.len());

    // Enemy events (from EVP file - scripted battles)
    print!("Exporting enemy events...");
    let events = game.read_enemy_events()?;
    CsvExporter::export_enemy_events(
        &events,
        File::create(output_dir.join("enemyevent.csv"))?,
        &encounter_enemy_names,
    )?;
    println!(" {} events", events.len());

    // Export ENP file dumps
    export_enp_dumps(game, output_dir, &item_db)?;

    // Export EVP file dump
    export_evp_dump(game, output_dir, &item_db)?;

    Ok(())
}

fn export_evp_dump(
    game: &mut GameRoot,
    output_dir: &Path,
    item_db: &alx::items::ItemDatabase,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{decompress_aklz, dump_evp_editable};

    let evp_dir = output_dir.join("evp");
    fs::create_dir_all(&evp_dir)?;

    print!("Exporting EVP file dump...");

    // Find EVP file
    let matching_files = game.iso_mut().list_files_matching("epevent.evp")?;

    if matching_files.is_empty() {
        println!(" not found");
        return Ok(());
    }

    for entry in &matching_files {
        let filename = entry
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Read and decompress the file
        let raw_data = match game.iso_mut().read_file_direct(entry) {
            Ok(data) => data,
            Err(_) => continue,
        };

        let data = match decompress_aklz(&raw_data) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Dump the structure using simplified editable format
        let dump = match dump_evp_editable(&data, &filename, game.version(), item_db) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Convert to JSON
        let json = serde_json::to_string_pretty(&dump)?;

        // Write to evp subfolder
        let output_file = evp_dir.join(format!("{}.json", filename));
        fs::write(&output_file, &json)?;

        println!(
            " {} enemies, {} events",
            dump.enemies.len(),
            dump.events.len()
        );
        break;
    }

    Ok(())
}

fn export_enp_dumps(
    game: &mut GameRoot,
    output_dir: &Path,
    item_db: &alx::items::ItemDatabase,
) -> Result<(), Box<dyn std::error::Error>> {
    use alx::io::{decompress_aklz, dump_enp_editable};

    let enp_dir = output_dir.join("enp");
    fs::create_dir_all(&enp_dir)?;

    print!("Exporting ENP file dumps...");

    // Find all ENP files
    let all_files = game.iso_mut().list_files_matching("")?;
    let enp_files: Vec<_> = all_files
        .iter()
        .filter(|e| {
            e.path
                .file_name()
                .map(|s| s.to_string_lossy().ends_with(".enp"))
                .unwrap_or(false)
        })
        .collect();

    let mut count = 0;
    for entry in &enp_files {
        let filename = entry
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // Read and decompress the file
        let raw_data = match game.iso_mut().read_file_direct(entry) {
            Ok(data) => data,
            Err(_) => continue,
        };

        let data = match decompress_aklz(&raw_data) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Dump the structure using simplified editable format
        let dump = match dump_enp_editable(&data, &filename, game.version(), item_db) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Skip files with no enemies (likely multi-segment or special format)
        if dump.enemies.is_empty() {
            continue;
        }

        // Convert to JSON
        let json = serde_json::to_string_pretty(&dump)?;

        // Write to enp subfolder
        let output_file = enp_dir.join(format!("{}.json", filename));
        fs::write(&output_file, &json)?;
        count += 1;
    }

    println!(" {} files", count);
    Ok(())
}
