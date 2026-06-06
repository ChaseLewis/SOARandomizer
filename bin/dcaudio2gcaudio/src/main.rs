//! dcaudio2gcaudio — render Skies of Arcadia DC/GC music, compare, and tune the
//! GameCube mix toward the Dreamcast. See the project plan for the staged design.
//!
//! Phase 0 surface: `--inspect` dumps the DC `.MLT` and GC `.info`/`.samp` bank
//! structures so the sample/level/pan fields can be reverse-engineered.

mod bank_dc;
mod bank_gc;

use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "dcaudio2gcaudio")]
#[command(about = "Render/compare/tune Skies of Arcadia DC vs GC music banks", long_about = None)]
struct Args {
    /// Inspect a bank by base name (e.g. b7000000): dumps DC .MLT and/or GC .info/.samp.
    #[arg(long, value_name = "BANK")]
    inspect: Option<String>,

    /// Directory holding the Dreamcast SOUND/*.MLT files.
    #[arg(long, value_name = "DIR")]
    dc_sound: Option<PathBuf>,

    /// Directory holding the GameCube sound/*.info + *.samp files.
    #[arg(long, value_name = "DIR")]
    gc_sound: Option<PathBuf>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if let Some(bank) = args.inspect {
        return run_inspect(&bank, args.dc_sound.as_deref(), args.gc_sound.as_deref());
    }

    Err("no command given (try --inspect <bank> --dc-sound <dir> --gc-sound <dir>)".into())
}

/// Case-insensitive lookup of `<base>.<ext>` in `dir`.
fn find_file(dir: &Path, base: &str, ext: &str) -> Option<PathBuf> {
    let want = format!("{base}.{ext}").to_lowercase();
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let p = e.path();
        let name = p.file_name()?.to_string_lossy().to_lowercase();
        (name == want).then_some(p)
    })
}

fn run_inspect(
    bank: &str,
    dc_sound: Option<&Path>,
    gc_sound: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    if dc_sound.is_none() && gc_sound.is_none() {
        return Err("--inspect needs --dc-sound and/or --gc-sound".into());
    }

    if let Some(dir) = dc_sound {
        println!("=== DC {bank}.MLT ===");
        match find_file(dir, bank, "mlt") {
            Some(p) => {
                let data = std::fs::read(&p)?;
                match bank_dc::parse(&data) {
                    Ok(b) => print!("{}", b.summary()),
                    Err(e) => println!("  parse error: {e}"),
                }
            }
            None => println!("  not found in {}", dir.display()),
        }
    }

    if let Some(dir) = gc_sound {
        println!("=== GC {bank}.info/.samp ===");
        match (find_file(dir, bank, "info"), find_file(dir, bank, "samp")) {
            (Some(ip), sp) => {
                let info = std::fs::read(&ip)?;
                let samp_len = sp
                    .as_ref()
                    .and_then(|p| std::fs::metadata(p).ok())
                    .map(|m| m.len() as usize)
                    .unwrap_or(0);
                match bank_gc::parse(&info, samp_len) {
                    Ok(b) => print!("{}", b.summary()),
                    Err(e) => println!("  parse error: {e}"),
                }
            }
            _ => println!("  .info not found in {}", dir.display()),
        }
    }

    Ok(())
}
