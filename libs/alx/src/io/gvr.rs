//! GVR (GameCube Ninja VR) texture handling for `.mld` model archives.
//!
//! Skies of Arcadia `.mld` files are AKLZ-compressed Sega Ninja model archives.
//! Once decompressed they contain a Ninja model plus one or more GameCube GVR
//! textures, each stored as a `GCIX` index chunk immediately followed by a
//! `GVRT` pixel chunk.
//!
//! This module locates those textures ([`carve_textures`]) and decodes their
//! base mip level to RGBA8 ([`decode_gvr`]) so they can be written out as PNGs.
//!
//! The de-tiling / decoding follows the GameCube `GX_TF_*` texture layouts
//! (the GVR data-format byte maps 1:1 onto the `GX` formats), matching the
//! well-tested Dolphin texture decoder.

use crate::error::{Error, Result};

/// Magic for the global texture index chunk.
const GCIX: &[u8; 4] = b"GCIX";
/// Alternate index chunk magic (PC builds use `GBIX`); accepted defensively.
const GBIX: &[u8; 4] = b"GBIX";
/// Magic for the pixel data chunk.
const GVRT: &[u8; 4] = b"GVRT";

/// A single texture carved out of a decompressed `.mld` archive.
#[derive(Debug, Clone)]
pub struct CarvedTexture {
    /// The standalone `.gvr` bytes (the `GCIX` chunk + the `GVRT` chunk).
    pub gvr: Vec<u8>,
    /// Byte offset of the `GCIX` chunk within the decompressed archive blob.
    /// `--repack` splices an edited texture back in at this offset.
    pub blob_offset: usize,
    /// Global texture index (first BE u32 of the `GCIX` body).
    pub global_index: u32,
    /// Texture width in pixels.
    pub width: u16,
    /// Texture height in pixels.
    pub height: u16,
    /// GVR data format byte (e.g. 0x0e = DXT1/CMP, 0x05 = RGB5A3).
    pub data_format: u8,
    /// GVR pixel-format/flags byte: high nibble = palette pixel format,
    /// low nibble = data flags (bit0 = mipmaps, bit1 = external palette,
    /// bit3 = internal palette).
    pub pixel_flags: u8,
}

/// A decoded texture: tightly packed RGBA8, row-major, top-to-bottom.
#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Walk a decompressed `.mld` blob and carve out every GVR texture.
///
/// Each `GCIX`/`GBIX` chunk is followed by a `GVRT` chunk; the chunk length
/// fields (little-endian u32 at offset +4) give the body sizes. A standalone
/// `.gvr` is the index chunk plus the `GVRT` chunk concatenated.
///
/// Robust to stray bytes: if a `GVRT` chunk does not immediately follow an
/// index chunk, the search resumes just past the index magic.
pub fn carve_textures(blob: &[u8]) -> Vec<CarvedTexture> {
    let mut out = Vec::new();
    let mut i = 0usize;

    while i + 8 <= blob.len() {
        let magic = &blob[i..i + 4];
        if magic != GCIX && magic != GBIX {
            i += 1;
            continue;
        }

        let idx_start = i;
        let idx_len = read_u32_le(blob, i + 4) as usize;
        let gvrt_start = match (i + 8).checked_add(idx_len) {
            Some(p) if p + 8 <= blob.len() => p,
            _ => {
                i += 4;
                continue;
            }
        };

        if &blob[gvrt_start..gvrt_start + 4] != GVRT {
            // Not a real texture header; keep scanning.
            i += 4;
            continue;
        }

        let gvrt_len = read_u32_le(blob, gvrt_start + 4) as usize;
        let gvrt_end = match gvrt_start.checked_add(8 + gvrt_len) {
            Some(p) if p <= blob.len() => p,
            _ => {
                i += 4;
                continue;
            }
        };

        // The GVRT header proper: width/height/format live just past the length.
        let global_index = read_u32_be(blob, idx_start + 8);
        let pixel_flags = blob[gvrt_start + 10];
        let data_format = blob[gvrt_start + 11];
        let width = read_u16_be(blob, gvrt_start + 12);
        let height = read_u16_be(blob, gvrt_start + 14);

        out.push(CarvedTexture {
            gvr: blob[idx_start..gvrt_end].to_vec(),
            blob_offset: idx_start,
            global_index,
            width,
            height,
            data_format,
            pixel_flags,
        });

        i = gvrt_end;
    }

    out
}

/// Decode a standalone `.gvr` (as produced by [`carve_textures`]) into RGBA8.
///
/// Only the base (largest) mip level is decoded. Returns an error for formats
/// that are not supported, so the caller can keep the raw `.gvr` and skip PNG.
pub fn decode_gvr(gvr: &[u8]) -> Result<DecodedImage> {
    let gvrt = find_chunk(gvr, GVRT).ok_or_else(|| parse_err(0, "GVR has no GVRT chunk".into()))?;

    if gvrt + 16 > gvr.len() {
        return Err(parse_err(gvrt, "GVRT chunk truncated".into()));
    }

    let pixel_flags = gvr[gvrt + 10];
    let data_format = gvr[gvrt + 11];
    let width = read_u16_be(gvr, gvrt + 12) as usize;
    let height = read_u16_be(gvr, gvrt + 14) as usize;

    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err(parse_err(
            gvrt,
            format!("implausible GVR dimensions {width}x{height}"),
        ));
    }

    let palette_pixfmt = pixel_flags >> 4; // 0=IA8, 1=RGB565, 2=RGB5A3
    let flags = pixel_flags & 0x0f;
    let has_internal_palette = flags & 0x08 != 0;

    let mut pos = gvrt + 16;

    // Indexed formats may carry an internal palette before the pixel data.
    let palette = match data_format {
        0x08 | 0x09 if has_internal_palette => {
            let count = if data_format == 0x08 { 16 } else { 256 };
            let pal = read_palette(gvr, pos, count, palette_pixfmt)
                .ok_or_else(|| parse_err(pos, "GVR internal palette truncated".into()))?;
            pos += count * 2;
            Some(pal)
        }
        _ => None,
    };

    let src = &gvr[pos..];
    let mut rgba = vec![0u8; width * height * 4];

    match data_format {
        0x00 => decode_i4(src, width, height, &mut rgba)?,
        0x01 => decode_i8(src, width, height, &mut rgba)?,
        0x02 => decode_ia4(src, width, height, &mut rgba)?,
        0x03 => decode_ia8(src, width, height, &mut rgba)?,
        0x04 => decode_rgb565(src, width, height, &mut rgba)?,
        0x05 => decode_rgb5a3(src, width, height, &mut rgba)?,
        0x06 => decode_rgba8(src, width, height, &mut rgba)?,
        0x08 => decode_index4(src, width, height, palette.as_deref(), &mut rgba)?,
        0x09 => decode_index8(src, width, height, palette.as_deref(), &mut rgba)?,
        0x0e => decode_cmpr(src, width, height, &mut rgba)?,
        other => {
            return Err(parse_err(
                gvrt,
                format!("unsupported GVR data format 0x{other:02x}"),
            ));
        }
    }

    Ok(DecodedImage {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

// --------------------------------------------------------------------------
// Format decoders. GameCube textures are stored in tiles; tile dimensions are
// fixed per format. Pixels outside `width`/`height` (tile padding) are skipped.
// --------------------------------------------------------------------------

/// Set one RGBA pixel if it lies within the image bounds.
#[inline]
fn put(rgba: &mut [u8], width: usize, height: usize, x: usize, y: usize, p: [u8; 4]) {
    if x < width && y < height {
        let o = (y * width + x) * 4;
        rgba[o..o + 4].copy_from_slice(&p);
    }
}

#[inline]
fn need(src: &[u8], n: usize, what: &str) -> Result<()> {
    if src.len() < n {
        Err(parse_err(0, format!("GVR pixel data too short for {what}")))
    } else {
        Ok(())
    }
}

/// I4: 8x8 tiles, 4 bits/pixel intensity (grayscale, opaque).
fn decode_i4(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(8) {
        for tx in (0..width).step_by(8) {
            for y in 0..8 {
                for x in (0..8).step_by(2) {
                    need(src, o + 1, "I4")?;
                    let b = src[o];
                    o += 1;
                    let i0 = (b >> 4) * 0x11;
                    let i1 = (b & 0x0f) * 0x11;
                    put(rgba, width, height, tx + x, ty + y, [i0, i0, i0, 255]);
                    put(rgba, width, height, tx + x + 1, ty + y, [i1, i1, i1, 255]);
                }
            }
        }
    }
    Ok(())
}

/// I8: 8x4 tiles, 8 bits/pixel intensity.
fn decode_i8(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(8) {
            for y in 0..4 {
                for x in 0..8 {
                    need(src, o + 1, "I8")?;
                    let i = src[o];
                    o += 1;
                    put(rgba, width, height, tx + x, ty + y, [i, i, i, 255]);
                }
            }
        }
    }
    Ok(())
}

/// IA4: 8x4 tiles, 4 bits alpha + 4 bits intensity per pixel.
fn decode_ia4(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(8) {
            for y in 0..4 {
                for x in 0..8 {
                    need(src, o + 1, "IA4")?;
                    let b = src[o];
                    o += 1;
                    let a = (b >> 4) * 0x11;
                    let i = (b & 0x0f) * 0x11;
                    put(rgba, width, height, tx + x, ty + y, [i, i, i, a]);
                }
            }
        }
    }
    Ok(())
}

/// IA8: 4x4 tiles, 8 bits alpha + 8 bits intensity (alpha byte first).
fn decode_ia8(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            for y in 0..4 {
                for x in 0..4 {
                    need(src, o + 2, "IA8")?;
                    let a = src[o];
                    let i = src[o + 1];
                    o += 2;
                    put(rgba, width, height, tx + x, ty + y, [i, i, i, a]);
                }
            }
        }
    }
    Ok(())
}

/// RGB565: 4x4 tiles, big-endian u16 per pixel, opaque.
fn decode_rgb565(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            for y in 0..4 {
                for x in 0..4 {
                    need(src, o + 2, "RGB565")?;
                    let v = u16::from_be_bytes([src[o], src[o + 1]]);
                    o += 2;
                    put(rgba, width, height, tx + x, ty + y, rgb565_to_rgba(v));
                }
            }
        }
    }
    Ok(())
}

/// RGB5A3: 4x4 tiles, big-endian u16 per pixel.
fn decode_rgb5a3(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            for y in 0..4 {
                for x in 0..4 {
                    need(src, o + 2, "RGB5A3")?;
                    let v = u16::from_be_bytes([src[o], src[o + 1]]);
                    o += 2;
                    put(rgba, width, height, tx + x, ty + y, rgb5a3_to_rgba(v));
                }
            }
        }
    }
    Ok(())
}

/// RGBA8 (ARGB8888): 4x4 tiles split into two cache lines: 32 bytes of AR
/// pairs followed by 32 bytes of GB pairs for the same 16 pixels.
fn decode_rgba8(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            need(src, o + 64, "RGBA8")?;
            for k in 0..16 {
                let a = src[o + k * 2];
                let r = src[o + k * 2 + 1];
                let g = src[o + 32 + k * 2];
                let b = src[o + 32 + k * 2 + 1];
                let x = tx + (k % 4);
                let y = ty + (k / 4);
                put(rgba, width, height, x, y, [r, g, b, a]);
            }
            o += 64;
        }
    }
    Ok(())
}

/// C4 (Index4): 8x8 tiles, 4 bits/pixel palette index.
fn decode_index4(
    src: &[u8],
    width: usize,
    height: usize,
    palette: Option<&[[u8; 4]]>,
    rgba: &mut [u8],
) -> Result<()> {
    let pal = palette.ok_or_else(|| parse_err(0, "Index4 texture without palette".into()))?;
    let mut o = 0;
    for ty in (0..height).step_by(8) {
        for tx in (0..width).step_by(8) {
            for y in 0..8 {
                for x in (0..8).step_by(2) {
                    need(src, o + 1, "Index4")?;
                    let b = src[o];
                    o += 1;
                    let p0 = pal[(b >> 4) as usize & (pal.len() - 1)];
                    let p1 = pal[(b & 0x0f) as usize & (pal.len() - 1)];
                    put(rgba, width, height, tx + x, ty + y, p0);
                    put(rgba, width, height, tx + x + 1, ty + y, p1);
                }
            }
        }
    }
    Ok(())
}

/// C8 (Index8): 8x4 tiles, 8 bits/pixel palette index.
fn decode_index8(
    src: &[u8],
    width: usize,
    height: usize,
    palette: Option<&[[u8; 4]]>,
    rgba: &mut [u8],
) -> Result<()> {
    let pal = palette.ok_or_else(|| parse_err(0, "Index8 texture without palette".into()))?;
    let mut o = 0;
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(8) {
            for y in 0..4 {
                for x in 0..8 {
                    need(src, o + 1, "Index8")?;
                    let idx = src[o] as usize;
                    o += 1;
                    let p = pal[idx & (pal.len() - 1)];
                    put(rgba, width, height, tx + x, ty + y, p);
                }
            }
        }
    }
    Ok(())
}

/// CMPR (DXT1/S3TC): 8x8 tiles, each a 2x2 arrangement of 4x4 DXT1 sub-blocks.
/// GameCube stores the 565 colours big-endian and reads the 2-bit indices
/// MSB-first within each row byte.
fn decode_cmpr(src: &[u8], width: usize, height: usize, rgba: &mut [u8]) -> Result<()> {
    let mut o = 0;
    for ty in (0..height).step_by(8) {
        for tx in (0..width).step_by(8) {
            // Sub-block order: (0,0), (4,0), (0,4), (4,4).
            for sy in (0..8).step_by(4) {
                for sx in (0..8).step_by(4) {
                    need(src, o + 8, "CMPR")?;
                    let c0 = u16::from_be_bytes([src[o], src[o + 1]]);
                    let c1 = u16::from_be_bytes([src[o + 2], src[o + 3]]);
                    let colors = dxt1_palette(c0, c1);
                    for row in 0..4 {
                        let bits = src[o + 4 + row];
                        for col in 0..4 {
                            let idx = (bits >> (6 - col * 2)) & 0x03;
                            put(
                                rgba,
                                width,
                                height,
                                tx + sx + col,
                                ty + sy + row,
                                colors[idx as usize],
                            );
                        }
                    }
                    o += 8;
                }
            }
        }
    }
    Ok(())
}

// --------------------------------------------------------------------------
// Encoding (RGBA8 -> GVR), for repacking edited textures.
// --------------------------------------------------------------------------

/// Re-encode an edited texture back into a GVR, preserving the original
/// `template` GVR's header, index chunk, palette, and (for mipmapped textures)
/// every mip level beyond the base.
///
/// Only the **base** (largest) mip level is re-encoded, and it is spliced in
/// place over the template's base-level bytes. Because GameCube formats are
/// fixed bits-per-pixel, the byte length is identical, so the result is the
/// exact same size as `template` — safe to drop back into the archive blob.
///
/// `rgba` must be `width*height*4` bytes matching the template's dimensions.
/// Returns an error for formats without an encoder.
pub fn encode_gvr(template: &[u8], rgba: &[u8]) -> Result<Vec<u8>> {
    let gvrt =
        find_chunk(template, GVRT).ok_or_else(|| parse_err(0, "GVR has no GVRT chunk".into()))?;
    if gvrt + 16 > template.len() {
        return Err(parse_err(gvrt, "GVRT chunk truncated".into()));
    }

    let pixel_flags = template[gvrt + 10];
    let data_format = template[gvrt + 11];
    let width = read_u16_be(template, gvrt + 12) as usize;
    let height = read_u16_be(template, gvrt + 14) as usize;

    if rgba.len() != width * height * 4 {
        return Err(parse_err(
            gvrt,
            format!("RGBA size {} does not match {width}x{height}", rgba.len()),
        ));
    }

    let palette_pixfmt = pixel_flags >> 4;
    let flags = pixel_flags & 0x0f;
    let has_internal_palette = flags & 0x08 != 0;

    let mut pos = gvrt + 16;
    let palette = match data_format {
        0x08 | 0x09 if has_internal_palette => {
            let count = if data_format == 0x08 { 16 } else { 256 };
            let pal = read_palette(template, pos, count, palette_pixfmt)
                .ok_or_else(|| parse_err(pos, "GVR internal palette truncated".into()))?;
            pos += count * 2;
            Some(pal)
        }
        _ => None,
    };

    let base = match data_format {
        0x00 => enc_i4(rgba, width, height),
        0x01 => enc_i8(rgba, width, height),
        0x02 => enc_ia4(rgba, width, height),
        0x03 => enc_ia8(rgba, width, height),
        0x04 => enc_rgb565(rgba, width, height),
        0x05 => enc_rgb5a3(rgba, width, height),
        0x06 => enc_rgba8(rgba, width, height),
        0x08 => enc_index4(rgba, width, height, palette.as_deref())?,
        0x09 => enc_index8(rgba, width, height, palette.as_deref())?,
        0x0e => enc_cmpr(rgba, width, height),
        other => {
            return Err(parse_err(
                gvrt,
                format!("no GVR encoder for data format 0x{other:02x}"),
            ));
        }
    };

    if pos + base.len() > template.len() {
        return Err(parse_err(
            pos,
            "encoded base level larger than template".into(),
        ));
    }

    let mut out = template.to_vec();
    out[pos..pos + base.len()].copy_from_slice(&base);
    Ok(out)
}

/// Fetch an RGBA pixel, clamping coordinates into the valid region so tile
/// padding reuses edge pixels (avoids skewing DXT endpoints).
#[inline]
fn get(rgba: &[u8], width: usize, height: usize, x: usize, y: usize) -> [u8; 4] {
    let xx = x.min(width - 1);
    let yy = y.min(height - 1);
    let o = (yy * width + xx) * 4;
    [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
}

#[inline]
fn luma(p: [u8; 4]) -> u8 {
    ((p[0] as u32 * 77 + p[1] as u32 * 150 + p[2] as u32 * 29) >> 8) as u8
}

fn enc_i4(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(8) {
        for tx in (0..width).step_by(8) {
            for y in 0..8 {
                for x in (0..8).step_by(2) {
                    let a = luma(get(rgba, width, height, tx + x, ty + y));
                    let b = luma(get(rgba, width, height, tx + x + 1, ty + y));
                    out.push((a & 0xf0) | (b >> 4));
                }
            }
        }
    }
    out
}

fn enc_i8(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(8) {
            for y in 0..4 {
                for x in 0..8 {
                    out.push(luma(get(rgba, width, height, tx + x, ty + y)));
                }
            }
        }
    }
    out
}

fn enc_ia4(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(8) {
            for y in 0..4 {
                for x in 0..8 {
                    let p = get(rgba, width, height, tx + x, ty + y);
                    out.push((p[3] & 0xf0) | (luma(p) >> 4));
                }
            }
        }
    }
    out
}

fn enc_ia8(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            for y in 0..4 {
                for x in 0..4 {
                    let p = get(rgba, width, height, tx + x, ty + y);
                    out.push(p[3]);
                    out.push(luma(p));
                }
            }
        }
    }
    out
}

fn enc_rgb565(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            for y in 0..4 {
                for x in 0..4 {
                    let p = get(rgba, width, height, tx + x, ty + y);
                    out.extend_from_slice(&rgba_to_rgb565(p).to_be_bytes());
                }
            }
        }
    }
    out
}

fn enc_rgb5a3(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            for y in 0..4 {
                for x in 0..4 {
                    let p = get(rgba, width, height, tx + x, ty + y);
                    out.extend_from_slice(&rgba_to_rgb5a3(p).to_be_bytes());
                }
            }
        }
    }
    out
}

fn enc_rgba8(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(4) {
            // AR plane for 16 pixels, then GB plane.
            for k in 0..16 {
                let p = get(rgba, width, height, tx + (k % 4), ty + (k / 4));
                out.push(p[3]);
                out.push(p[0]);
            }
            for k in 0..16 {
                let p = get(rgba, width, height, tx + (k % 4), ty + (k / 4));
                out.push(p[1]);
                out.push(p[2]);
            }
        }
    }
    out
}

fn nearest_index(pal: &[[u8; 4]], p: [u8; 4]) -> u8 {
    let mut best = 0usize;
    let mut best_d = u32::MAX;
    for (i, c) in pal.iter().enumerate() {
        let dr = p[0] as i32 - c[0] as i32;
        let dg = p[1] as i32 - c[1] as i32;
        let db = p[2] as i32 - c[2] as i32;
        let da = p[3] as i32 - c[3] as i32;
        let d = (dr * dr + dg * dg + db * db + da * da) as u32;
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best as u8
}

fn enc_index4(
    rgba: &[u8],
    width: usize,
    height: usize,
    palette: Option<&[[u8; 4]]>,
) -> Result<Vec<u8>> {
    let pal = palette.ok_or_else(|| parse_err(0, "Index4 encode without palette".into()))?;
    let mut out = Vec::new();
    for ty in (0..height).step_by(8) {
        for tx in (0..width).step_by(8) {
            for y in 0..8 {
                for x in (0..8).step_by(2) {
                    let a = nearest_index(pal, get(rgba, width, height, tx + x, ty + y));
                    let b = nearest_index(pal, get(rgba, width, height, tx + x + 1, ty + y));
                    out.push((a << 4) | (b & 0x0f));
                }
            }
        }
    }
    Ok(out)
}

fn enc_index8(
    rgba: &[u8],
    width: usize,
    height: usize,
    palette: Option<&[[u8; 4]]>,
) -> Result<Vec<u8>> {
    let pal = palette.ok_or_else(|| parse_err(0, "Index8 encode without palette".into()))?;
    let mut out = Vec::new();
    for ty in (0..height).step_by(4) {
        for tx in (0..width).step_by(8) {
            for y in 0..4 {
                for x in 0..8 {
                    out.push(nearest_index(pal, get(rgba, width, height, tx + x, ty + y)));
                }
            }
        }
    }
    Ok(out)
}

/// Encode one 4x4 DXT1 block (16 RGBA pixels) to 8 GameCube CMPR bytes.
fn encode_dxt1_block(block: &[[u8; 4]; 16]) -> [u8; 8] {
    let has_alpha = block.iter().any(|p| p[3] < 128);

    // Bounding box of the opaque colours.
    let mut lo = [255u8; 3];
    let mut hi = [0u8; 3];
    let mut any = false;
    for p in block.iter() {
        if p[3] >= 128 {
            any = true;
            for c in 0..3 {
                lo[c] = lo[c].min(p[c]);
                hi[c] = hi[c].max(p[c]);
            }
        }
    }
    if !any {
        lo = [0, 0, 0];
        hi = [0, 0, 0];
    }

    let mut c_hi = rgba_to_rgb565([hi[0], hi[1], hi[2], 255]);
    let mut c_lo = rgba_to_rgb565([lo[0], lo[1], lo[2], 255]);

    let mut out = [0u8; 8];
    if !has_alpha {
        // 4-colour opaque mode requires c0 > c1.
        if c_hi <= c_lo {
            if c_hi == c_lo {
                // Flat block: nudge so c0 > c1, all indices 0.
                out[0..2].copy_from_slice(&c_hi.to_be_bytes());
                out[2..4].copy_from_slice(&c_hi.to_be_bytes());
                return out;
            }
            std::mem::swap(&mut c_hi, &mut c_lo);
        }
        let pal = dxt1_palette(c_hi, c_lo);
        out[0..2].copy_from_slice(&c_hi.to_be_bytes());
        out[2..4].copy_from_slice(&c_lo.to_be_bytes());
        for row in 0..4 {
            let mut bits = 0u8;
            for col in 0..4 {
                let p = block[row * 4 + col];
                let idx = nearest_index(&pal, [p[0], p[1], p[2], 255]);
                bits |= idx << (6 - col * 2);
            }
            out[4 + row] = bits;
        }
    } else {
        // 3-colour + transparent mode requires c0 <= c1.
        if c_hi > c_lo {
            std::mem::swap(&mut c_hi, &mut c_lo);
        }
        let pal = dxt1_palette(c_hi, c_lo); // pal[3] is transparent
        out[0..2].copy_from_slice(&c_hi.to_be_bytes());
        out[2..4].copy_from_slice(&c_lo.to_be_bytes());
        for row in 0..4 {
            let mut bits = 0u8;
            for col in 0..4 {
                let p = block[row * 4 + col];
                let idx = if p[3] < 128 {
                    3
                } else {
                    nearest_index(&pal[0..3], [p[0], p[1], p[2], 255])
                };
                bits |= idx << (6 - col * 2);
            }
            out[4 + row] = bits;
        }
    }
    out
}

fn enc_cmpr(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for ty in (0..height).step_by(8) {
        for tx in (0..width).step_by(8) {
            for sy in (0..8).step_by(4) {
                for sx in (0..8).step_by(4) {
                    let mut block = [[0u8; 4]; 16];
                    for row in 0..4 {
                        for col in 0..4 {
                            block[row * 4 + col] =
                                get(rgba, width, height, tx + sx + col, ty + sy + row);
                        }
                    }
                    out.extend_from_slice(&encode_dxt1_block(&block));
                }
            }
        }
    }
    out
}

#[inline]
fn rgba_to_rgb565(p: [u8; 4]) -> u16 {
    ((p[0] as u16 >> 3) << 11) | ((p[1] as u16 >> 2) << 5) | (p[2] as u16 >> 3)
}

#[inline]
fn rgba_to_rgb5a3(p: [u8; 4]) -> u16 {
    if p[3] >= 0xe0 {
        // Opaque -> RGB555.
        0x8000 | ((p[0] as u16 >> 3) << 10) | ((p[1] as u16 >> 3) << 5) | (p[2] as u16 >> 3)
    } else {
        // ARGB3444.
        ((p[3] as u16 >> 5) << 12)
            | ((p[0] as u16 >> 4) << 8)
            | ((p[1] as u16 >> 4) << 4)
            | (p[2] as u16 >> 4)
    }
}

// --------------------------------------------------------------------------
// Colour helpers
// --------------------------------------------------------------------------

#[inline]
fn rgb565_to_rgba(v: u16) -> [u8; 4] {
    let r = ((v >> 11) & 0x1f) as u8;
    let g = ((v >> 5) & 0x3f) as u8;
    let b = (v & 0x1f) as u8;
    [
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
        255,
    ]
}

#[inline]
fn rgb5a3_to_rgba(v: u16) -> [u8; 4] {
    if v & 0x8000 != 0 {
        // RGB555, opaque.
        let r = ((v >> 10) & 0x1f) as u8;
        let g = ((v >> 5) & 0x1f) as u8;
        let b = (v & 0x1f) as u8;
        [
            (r << 3) | (r >> 2),
            (g << 3) | (g >> 2),
            (b << 3) | (b >> 2),
            255,
        ]
    } else {
        // ARGB3444.
        let a = ((v >> 12) & 0x07) as u8;
        let r = ((v >> 8) & 0x0f) as u8;
        let g = ((v >> 4) & 0x0f) as u8;
        let b = (v & 0x0f) as u8;
        [
            (r << 4) | r,
            (g << 4) | g,
            (b << 4) | b,
            (a << 5) | (a << 2) | (a >> 1),
        ]
    }
}

/// Build the 4-entry colour table for a DXT1 block.
fn dxt1_palette(c0: u16, c1: u16) -> [[u8; 4]; 4] {
    let a = rgb565_to_rgba(c0);
    let b = rgb565_to_rgba(c1);
    let mut p = [[0u8; 4]; 4];
    p[0] = a;
    p[1] = b;
    if c0 > c1 {
        for i in 0..3 {
            p[2][i] = ((2 * a[i] as u16 + b[i] as u16) / 3) as u8;
            p[3][i] = ((a[i] as u16 + 2 * b[i] as u16) / 3) as u8;
        }
        p[2][3] = 255;
        p[3][3] = 255;
    } else {
        for i in 0..3 {
            p[2][i] = ((a[i] as u16 + b[i] as u16) / 2) as u8;
        }
        p[2][3] = 255;
        p[3] = [0, 0, 0, 0]; // transparent
    }
    p
}

/// Read an internal palette of `count` entries in the given pixel format.
fn read_palette(src: &[u8], pos: usize, count: usize, pixfmt: u8) -> Option<Vec<[u8; 4]>> {
    if pos + count * 2 > src.len() {
        return None;
    }
    let mut pal = Vec::with_capacity(count);
    for i in 0..count {
        let v = u16::from_be_bytes([src[pos + i * 2], src[pos + i * 2 + 1]]);
        let c = match pixfmt {
            0 => {
                // IA8
                let a = (v >> 8) as u8;
                let l = (v & 0xff) as u8;
                [l, l, l, a]
            }
            1 => rgb565_to_rgba(v),
            _ => rgb5a3_to_rgba(v),
        };
        pal.push(c);
    }
    Some(pal)
}

// --------------------------------------------------------------------------
// Byte helpers
// --------------------------------------------------------------------------

#[inline]
fn read_u16_be(b: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([b[o], b[o + 1]])
}

#[inline]
fn read_u32_be(b: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

#[inline]
fn read_u32_le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// Find the offset of a 4-byte chunk magic within `data`.
fn find_chunk(data: &[u8], magic: &[u8; 4]) -> Option<usize> {
    data.windows(4).position(|w| w == magic)
}

fn parse_err(offset: usize, message: String) -> Error {
    Error::ParseError { offset, message }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal GCIX+GVRT buffer for a single tile of a given format.
    fn make_gvr(global_index: u32, data_format: u8, w: u16, h: u16, pixels: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(GCIX);
        v.extend_from_slice(&8u32.to_le_bytes());
        v.extend_from_slice(&global_index.to_be_bytes());
        v.extend_from_slice(&[0, 0, 0, 0]);
        v.extend_from_slice(GVRT);
        v.extend_from_slice(&((8 + pixels.len()) as u32).to_le_bytes());
        v.extend_from_slice(&[0, 0, 0, data_format]);
        v.extend_from_slice(&w.to_be_bytes());
        v.extend_from_slice(&h.to_be_bytes());
        v.extend_from_slice(pixels);
        v
    }

    #[test]
    fn carve_finds_textures_with_offsets() {
        let t0 = make_gvr(11, 0x05, 4, 4, &[0u8; 32]);
        let t1 = make_gvr(12, 0x05, 4, 4, &[0u8; 32]);
        let mut blob = vec![0xab; 5]; // leading junk
        let off0 = blob.len();
        blob.extend_from_slice(&t0);
        let off1 = blob.len();
        blob.extend_from_slice(&t1);
        blob.extend_from_slice(&[0xcd; 3]); // trailing junk

        let texs = carve_textures(&blob);
        assert_eq!(texs.len(), 2);
        assert_eq!(texs[0].blob_offset, off0);
        assert_eq!(texs[0].global_index, 11);
        assert_eq!(texs[0].gvr, t0);
        assert_eq!(texs[1].blob_offset, off1);
        assert_eq!(texs[1].global_index, 12);
        assert_eq!(texs[1].data_format, 0x05);
        assert_eq!((texs[1].width, texs[1].height), (4, 4));
    }

    #[test]
    fn decode_rgb5a3_opaque_white() {
        // 0xFFFF -> top bit set (RGB555), all channels max -> white opaque.
        let px = [0xffu8; 32]; // one 4x4 tile, 16 px * 2 bytes
        let gvr = make_gvr(0, 0x05, 4, 4, &px);
        let img = decode_gvr(&gvr).unwrap();
        assert_eq!((img.width, img.height), (4, 4));
        assert_eq!(&img.rgba[0..4], &[255, 255, 255, 255]);
        assert_eq!(img.rgba.len(), 4 * 4 * 4);
    }

    #[test]
    fn decode_cmpr_solid_color() {
        // GameCube CMPR pads to an 8x8 tile = four 4x4 DXT1 sub-blocks (32 bytes).
        // A 4x4 image lives entirely in the first sub-block; the rest is padding.
        // First sub-block: c0=c1=0xFFFF (white), all indices 0 -> solid white.
        let mut px = vec![0u8; 32];
        px[0] = 0xff;
        px[1] = 0xff; // c0
        px[2] = 0xff;
        px[3] = 0xff; // c1
                      // index bytes 4..8 = 0 -> color[0] everywhere
        let gvr = make_gvr(0, 0x0e, 4, 4, &px);
        let img = decode_gvr(&gvr).unwrap();
        assert_eq!((img.width, img.height), (4, 4));
        for px in img.rgba.chunks(4) {
            assert_eq!(px, &[255, 255, 255, 255]);
        }
    }

    #[test]
    fn unsupported_format_errors() {
        let gvr = make_gvr(0, 0x77, 4, 4, &[0u8; 32]);
        assert!(decode_gvr(&gvr).is_err());
    }

    /// Encoding then decoding must preserve the image (exactly, for lossless
    /// formats) and keep the GVR byte length identical to the template.
    fn assert_roundtrip(data_format: u8, w: u16, h: u16, byte_len: usize, lossless: bool) {
        // Distinct per-pixel colours so tiling/order bugs surface.
        let mut rgba = vec![0u8; w as usize * h as usize * 4];
        for (i, px) in rgba.chunks_mut(4).enumerate() {
            px[0] = (i * 7) as u8;
            px[1] = (i * 13) as u8;
            px[2] = (i * 29) as u8;
            px[3] = 255;
        }
        let template = make_gvr(0, data_format, w, h, &vec![0u8; byte_len]);
        let re = encode_gvr(&template, &rgba).unwrap();
        assert_eq!(re.len(), template.len(), "fmt 0x{data_format:02x} size");
        let img = decode_gvr(&re).unwrap();
        assert_eq!((img.width, img.height), (w as u32, h as u32));
        if lossless {
            assert_eq!(img.rgba, rgba, "fmt 0x{data_format:02x} pixels");
        }
    }

    #[test]
    fn encode_roundtrip_rgba8() {
        // ARGB8888 is fully lossless. One 4x4 tile = 64 bytes.
        assert_roundtrip(0x06, 4, 4, 64, true);
    }

    #[test]
    fn encode_roundtrip_rgb5a3_opaque() {
        // Opaque pixels become RGB555 (5 bits/chan); not byte-lossless, but
        // the size and structure must hold. Use a larger tile.
        assert_roundtrip(0x05, 8, 8, 8 * 8 * 2, false);
    }

    #[test]
    fn encode_roundtrip_cmpr_preserves_size() {
        // DXT1 is lossy, but encode must yield the exact template size and a
        // decodable result. 8x8 = one macro tile = 32 bytes.
        assert_roundtrip(0x0e, 8, 8, 32, false);
    }

    #[test]
    fn encode_index8_uses_palette() {
        // Palette of 256 RGB565 entries + 8x4 tile of indices (32 bytes).
        let w = 8u16;
        let h = 4u16;
        let mut body = Vec::new();
        // Internal palette: entry i = grayscale i in RGB565.
        for i in 0..256u16 {
            let g = (i >> 3) as u16;
            let v = (g << 11) | ((i >> 2) << 5) | g;
            body.extend_from_slice(&v.to_be_bytes());
        }
        body.extend_from_slice(&vec![0u8; (w * h) as usize]);
        // pixel_flags: pixfmt=1 (RGB565) in high nibble, internal palette bit (0x08).
        let mut template = make_gvr(0, 0x09, w, h, &body);
        let gvrt = find_chunk(&template, GVRT).unwrap();
        template[gvrt + 10] = (1 << 4) | 0x08;

        // A flat mid-gray image should encode and decode without error.
        let rgba = vec![128u8; (w * h * 4) as usize]
            .iter()
            .enumerate()
            .map(|(i, _)| if i % 4 == 3 { 255 } else { 128 })
            .collect::<Vec<u8>>();
        let re = encode_gvr(&template, &rgba).unwrap();
        assert_eq!(re.len(), template.len());
        let img = decode_gvr(&re).unwrap();
        assert_eq!((img.width, img.height), (w as u32, h as u32));
    }
}
