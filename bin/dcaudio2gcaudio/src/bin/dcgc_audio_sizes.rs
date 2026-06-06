//! dcgc_audio_sizes — compare the on-disk size of the streamed audio that the
//! Dreamcast ships as a single stereo file (`<BASE>S.P04` / `.P16`) against the
//! GameCube's two mono files (`<BASE>_L.dsp` + `<BASE>_R.dsp`).
//!
//! Both platforms store this audio as 4-bit ADPCM, so raw byte sizes are a fair
//! proxy for "how much audio is here" — a GC track that is much smaller than its
//! DC counterpart is a candidate for data lost in the DC->GC conversion.
//!
//! Emits a per-track CSV and a summary-stats CSV, and prints the stats to stdout.

use clap::Parser;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "dcgc_audio_sizes")]
#[command(about = "Compare DC (stereo) vs GC (2x mono) streamed-audio file sizes", long_about = None)]
struct Args {
    /// Dreamcast SOUND directory holding the stereo `<BASE>S.P04` / `.P16` streams.
    #[arg(long, value_name = "DIR")]
    dc: PathBuf,

    /// GameCube sound directory holding the `<BASE>_L.dsp` / `<BASE>_R.dsp` streams.
    #[arg(long, value_name = "DIR")]
    gc: PathBuf,

    /// Per-track CSV output path. A sibling `<out>.summary.csv` is written too.
    #[arg(long, value_name = "FILE", default_value = "audio_size_report.csv")]
    out: PathBuf,

    /// DC stream extensions to consider (the stereo streams), comma-separated.
    #[arg(long, value_name = "EXTS", default_value = "p04,p16")]
    dc_ext: String,

    /// GC mono extension.
    #[arg(long, value_name = "EXT", default_value = "dsp")]
    gc_ext: String,
}

/// One track's matched DC stereo file and GC mono pair, keyed by base name.
#[derive(Default)]
struct Track {
    /// Original-cased base name as first seen (for display).
    display: String,
    dc_file: Option<PathBuf>,
    dc_bytes: Option<u64>,
    gc_l: Option<(PathBuf, u64)>,
    gc_r: Option<(PathBuf, u64)>,
}

impl Track {
    fn gc_total(&self) -> Option<u64> {
        match (&self.gc_l, &self.gc_r) {
            (None, None) => None,
            (l, r) => Some(l.as_ref().map_or(0, |x| x.1) + r.as_ref().map_or(0, |x| x.1)),
        }
    }

    /// matched / dc_only / gc_only, plus a note if a GC channel is missing.
    fn status(&self) -> String {
        let has_dc = self.dc_bytes.is_some();
        let has_gc = self.gc_l.is_some() || self.gc_r.is_some();
        let mut s = match (has_dc, has_gc) {
            (true, true) => "matched",
            (true, false) => "dc_only",
            (false, true) => "gc_only",
            (false, false) => "empty",
        }
        .to_string();
        if has_gc && (self.gc_l.is_none() || self.gc_r.is_none()) {
            s.push_str("+missing_channel");
        }
        s
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let dc_exts: Vec<String> = args
        .dc_ext
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let gc_ext = args.gc_ext.trim().to_lowercase();

    let mut tracks: BTreeMap<String, Track> = BTreeMap::new();

    // --- Scan DC: stereo `<BASE>S.<ext>` -> key on uppercase BASE (S stripped). ---
    for entry in std::fs::read_dir(&args.dc)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let (stem, ext) = match split_name(&path) {
            Some(v) => v,
            None => continue,
        };
        if !dc_exts.contains(&ext.to_lowercase()) {
            continue;
        }
        // Drop the trailing 'S' stereo marker if present.
        let base = stem
            .strip_suffix(['S', 's'])
            .unwrap_or(&stem)
            .to_string();
        let bytes = std::fs::metadata(&path)?.len();
        let t = tracks.entry(base.to_uppercase()).or_default();
        if t.display.is_empty() {
            t.display = base;
        }
        t.dc_file = Some(path);
        t.dc_bytes = Some(bytes);
    }

    // --- Scan GC: `<BASE>_L.dsp` / `<BASE>_R.dsp` -> key on uppercase BASE. ---
    for entry in std::fs::read_dir(&args.gc)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let (stem, ext) = match split_name(&path) {
            Some(v) => v,
            None => continue,
        };
        if ext.to_lowercase() != gc_ext {
            continue;
        }
        let lower = stem.to_lowercase();
        let (base, is_left) = if let Some(b) = lower.strip_suffix("_l") {
            (b.to_string(), Some(true))
        } else if let Some(b) = lower.strip_suffix("_r") {
            (b.to_string(), Some(false))
        } else {
            // Mono file without an _L/_R marker — record under its own base.
            (lower.clone(), None)
        };
        let bytes = std::fs::metadata(&path)?.len();
        let t = tracks.entry(base.to_uppercase()).or_default();
        if t.display.is_empty() {
            // Preserve the original casing of the part before _L/_R.
            t.display = stem[..base.len().min(stem.len())].to_string();
        }
        match is_left {
            Some(true) => t.gc_l = Some((path, bytes)),
            Some(false) => t.gc_r = Some((path, bytes)),
            // No channel marker: park it in the left slot so its bytes still count.
            None => t.gc_l = Some((path, bytes)),
        }
    }

    // --- Write the per-track CSV. ---
    let mut csv = String::new();
    csv.push_str(
        "base,status,dc_file,dc_bytes,gc_l_bytes,gc_r_bytes,gc_total_bytes,gc_minus_dc_bytes,pct_diff_gc_vs_dc\n",
    );

    // Accumulators for stats over fully-matched tracks (DC + at least one GC channel).
    let mut byte_diffs: Vec<i64> = Vec::new(); // gc_total - dc
    let mut pct_diffs: Vec<f64> = Vec::new(); // (gc-dc)/dc * 100
    let mut total_dc: u64 = 0;
    let mut total_gc: u64 = 0;
    let (mut n_matched, mut n_dc_only, mut n_gc_only) = (0u64, 0u64, 0u64);
    let mut n_gc_smaller = 0u64;
    let mut n_gc_larger = 0u64;

    for track in tracks.values() {
        let dc = track.dc_bytes;
        let gc = track.gc_total();
        let l = track.gc_l.as_ref().map(|x| x.1);
        let r = track.gc_r.as_ref().map(|x| x.1);

        let (diff, pct) = match (dc, gc) {
            (Some(d), Some(g)) => {
                let diff = g as i64 - d as i64;
                let pct = if d > 0 {
                    (g as f64 - d as f64) / d as f64 * 100.0
                } else {
                    0.0
                };
                (Some(diff), Some(pct))
            }
            _ => (None, None),
        };

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            track.display,
            track.status(),
            track
                .dc_file
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            opt(dc),
            opt(l),
            opt(r),
            opt(gc),
            diff.map(|d| d.to_string()).unwrap_or_default(),
            pct.map(|p| format!("{p:.2}")).unwrap_or_default(),
        ));

        match (dc, gc) {
            (Some(d), Some(g)) => {
                n_matched += 1;
                total_dc += d;
                total_gc += g;
                byte_diffs.push(g as i64 - d as i64);
                if let Some(p) = pct {
                    pct_diffs.push(p);
                }
                if g < d {
                    n_gc_smaller += 1;
                } else if g > d {
                    n_gc_larger += 1;
                }
            }
            (Some(_), None) => n_dc_only += 1,
            (None, Some(_)) => n_gc_only += 1,
            (None, None) => {}
        }
    }

    std::fs::write(&args.out, &csv)?;

    // --- Compute + emit stats. ---
    let mean_diff = mean_i(&byte_diffs);
    let median_diff = median_i(&mut byte_diffs.clone());
    let mean_pct = mean_f(&pct_diffs);
    let median_pct = median_f(&mut pct_diffs.clone());
    let max_diff = byte_diffs.iter().copied().max(); // most GC-larger
    let min_diff = byte_diffs.iter().copied().min(); // most GC-smaller (biggest loss)
    let max_abs = byte_diffs.iter().map(|d| d.abs()).max();
    let overall_pct = if total_dc > 0 {
        (total_gc as f64 - total_dc as f64) / total_dc as f64 * 100.0
    } else {
        0.0
    };

    let mut stats = String::new();
    stats.push_str("metric,value\n");
    stats.push_str(&format!("matched_tracks,{n_matched}\n"));
    stats.push_str(&format!("dc_only_tracks,{n_dc_only}\n"));
    stats.push_str(&format!("gc_only_tracks,{n_gc_only}\n"));
    stats.push_str(&format!("total_dc_bytes,{total_dc}\n"));
    stats.push_str(&format!("total_gc_bytes,{total_gc}\n"));
    stats.push_str(&format!(
        "total_gc_minus_dc_bytes,{}\n",
        total_gc as i64 - total_dc as i64
    ));
    stats.push_str(&format!("overall_pct_diff_gc_vs_dc,{overall_pct:.2}\n"));
    stats.push_str(&format!("mean_byte_diff,{mean_diff:.1}\n"));
    stats.push_str(&format!("median_byte_diff,{median_diff:.1}\n"));
    stats.push_str(&format!("mean_pct_diff,{mean_pct:.2}\n"));
    stats.push_str(&format!("median_pct_diff,{median_pct:.2}\n"));
    stats.push_str(&format!("max_byte_diff_gc_larger,{}\n", opt_i(max_diff)));
    stats.push_str(&format!("min_byte_diff_gc_smaller,{}\n", opt_i(min_diff)));
    stats.push_str(&format!("max_abs_byte_diff,{}\n", opt_i(max_abs)));
    stats.push_str(&format!("tracks_gc_smaller_than_dc,{n_gc_smaller}\n"));
    stats.push_str(&format!("tracks_gc_larger_than_dc,{n_gc_larger}\n"));

    let summary_path = sibling(&args.out, "summary.csv");
    std::fs::write(&summary_path, &stats)?;

    // Pretty print to stdout.
    println!("Per-track CSV : {}", args.out.display());
    println!("Summary CSV   : {}", summary_path.display());
    println!();
    print!("{stats}");

    Ok(())
}

/// Split a path into (file stem, extension-without-dot). Returns None if no extension.
fn split_name(path: &Path) -> Option<(String, String)> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let ext = path.extension()?.to_string_lossy().into_owned();
    Some((stem, ext))
}

/// CSV path with its final extension replaced (e.g. `report.csv` -> `report.summary.csv`).
fn sibling(out: &Path, new_ext: &str) -> PathBuf {
    out.with_extension(new_ext)
}

fn opt(v: Option<u64>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}

fn opt_i(v: Option<i64>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}

fn mean_i(xs: &[i64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<i64>() as f64 / xs.len() as f64
}

fn mean_f(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn median_i(xs: &mut [i64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_unstable();
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2] as f64
    } else {
        (xs[n / 2 - 1] as f64 + xs[n / 2] as f64) / 2.0
    }
}

fn median_f(xs: &mut [f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2]
    } else {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    }
}
