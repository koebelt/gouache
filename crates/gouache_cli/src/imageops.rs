//! Image helpers: loading, resizing, OKLab, contact sheets, labels.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{Rgb, RgbImage, imageops::FilterType};

/// Load any image as RGB8.
pub fn load_rgb(p: &Path) -> Result<RgbImage> {
    Ok(image::open(p).with_context(|| format!("opening {}", p.display()))?.to_rgb8())
}

/// Resize to exactly (w, h).
pub fn resize(img: &RgbImage, w: u32, h: u32) -> RgbImage {
    if img.width() == w && img.height() == h {
        return img.clone();
    }
    image::imageops::resize(img, w, h, FilterType::Lanczos3)
}

fn srgb_to_lin(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// sRGB8 -> OKLab (L in 0..1).
pub fn oklab(p: Rgb<u8>) -> [f32; 3] {
    let (r, g, b) = (srgb_to_lin(p[0]), srgb_to_lin(p[1]), srgb_to_lin(p[2]));
    let l = 0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// OKLab -> sRGB8 (for swatches).
pub fn oklab_to_rgb(c: [f32; 3]) -> Rgb<u8> {
    let l_ = c[0] + 0.396_337_78 * c[1] + 0.215_803_76 * c[2];
    let m_ = c[0] - 0.105_561_346 * c[1] - 0.063_854_17 * c[2];
    let s_ = c[0] - 0.089_484_18 * c[1] - 1.291_485_5 * c[2];
    let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    let r = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
    let g = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
    let b = -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s;
    let enc = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        let v = if x <= 0.003_130_8 { x * 12.92 } else { 1.055 * x.powf(1.0 / 2.4) - 0.055 };
        (v * 255.0 + 0.5) as u8
    };
    Rgb([enc(r), enc(g), enc(b)])
}

/// Mean OKLab ΔE×100 between two images (resized to the first).
pub fn mean_delta_e(a: &Path, b: &Path) -> Result<f32> {
    let ia = load_rgb(a)?;
    let ib = resize(&load_rgb(b)?, ia.width(), ia.height());
    let mut s = 0.0f64;
    for (p, q) in ia.pixels().zip(ib.pixels()) {
        let (x, y) = (oklab(*p), oklab(*q));
        s += f64::from(((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2) + (x[2] - y[2]).powi(2)).sqrt());
    }
    Ok((s / f64::from(ia.width() * ia.height()) * 100.0) as f32)
}

/// Draw a filled rectangle.
pub fn fill(img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32, c: Rgb<u8>) {
    for yy in y..(y + h).min(img.height()) {
        for xx in x..(x + w).min(img.width()) {
            img.put_pixel(xx, yy, c);
        }
    }
}

/// Tiny 3x5 bitmap font for labels (A-Z, 0-9, a few symbols). Uppercases input.
pub fn label(img: &mut RgbImage, x: u32, y: u32, text: &str, scale: u32, c: Rgb<u8>) {
    const GLYPHS: &[(char, [u8; 5])] = &[
        ('A', [7, 5, 7, 5, 5]),
        ('B', [6, 5, 6, 5, 6]),
        ('C', [7, 4, 4, 4, 7]),
        ('D', [6, 5, 5, 5, 6]),
        ('E', [7, 4, 6, 4, 7]),
        ('F', [7, 4, 6, 4, 4]),
        ('G', [7, 4, 5, 5, 7]),
        ('H', [5, 5, 7, 5, 5]),
        ('I', [7, 2, 2, 2, 7]),
        ('J', [1, 1, 1, 5, 7]),
        ('K', [5, 5, 6, 5, 5]),
        ('L', [4, 4, 4, 4, 7]),
        ('M', [5, 7, 7, 5, 5]),
        ('N', [6, 5, 5, 5, 5]),
        ('O', [7, 5, 5, 5, 7]),
        ('P', [7, 5, 7, 4, 4]),
        ('Q', [7, 5, 5, 7, 1]),
        ('R', [7, 5, 6, 5, 5]),
        ('S', [7, 4, 7, 1, 7]),
        ('T', [7, 2, 2, 2, 2]),
        ('U', [5, 5, 5, 5, 7]),
        ('V', [5, 5, 5, 5, 2]),
        ('W', [5, 5, 7, 7, 5]),
        ('X', [5, 5, 2, 5, 5]),
        ('Y', [5, 5, 2, 2, 2]),
        ('Z', [7, 1, 2, 4, 7]),
        ('0', [7, 5, 5, 5, 7]),
        ('1', [2, 6, 2, 2, 7]),
        ('2', [7, 1, 7, 4, 7]),
        ('3', [7, 1, 7, 1, 7]),
        ('4', [5, 5, 7, 1, 1]),
        ('5', [7, 4, 7, 1, 7]),
        ('6', [7, 4, 7, 5, 7]),
        ('7', [7, 1, 1, 1, 1]),
        ('8', [7, 5, 7, 5, 7]),
        ('9', [7, 5, 7, 1, 7]),
        ('.', [0, 0, 0, 0, 2]),
        ('-', [0, 0, 7, 0, 0]),
        ('_', [0, 0, 0, 0, 7]),
        (':', [0, 2, 0, 2, 0]),
        ('/', [1, 1, 2, 4, 4]),
        ('=', [0, 7, 0, 7, 0]),
        ('>', [4, 2, 1, 2, 4]),
    ];
    let mut cx = x;
    for ch in text.chars() {
        let ch = ch.to_ascii_uppercase();
        if let Some((_, rows)) = GLYPHS.iter().find(|g| g.0 == ch) {
            for (ry, row) in rows.iter().enumerate() {
                for rx in 0..3u32 {
                    if row & (4 >> rx) != 0 {
                        fill(img, cx + rx * scale, y + ry as u32 * scale, scale, scale, c);
                    }
                }
            }
        }
        cx += 4 * scale;
    }
}

/// Label with a dark backing box for legibility.
pub fn tag(img: &mut RgbImage, x: u32, y: u32, text: &str, scale: u32) {
    let w = text.len() as u32 * 4 * scale + 2 * scale;
    fill(img, x, y, w, 7 * scale, Rgb([20, 20, 24]));
    label(img, x + scale, y + scale, text, scale, Rgb([240, 240, 240]));
}

/// Grid of images, each scaled to `tile` width, labelled with the file stem.
pub fn contact_sheet(inputs: &[PathBuf], out: &Path, cols: u32, tile: u32) -> Result<()> {
    let imgs: Vec<(String, RgbImage)> = inputs
        .iter()
        .map(|p| {
            let i = load_rgb(p)?;
            let h = (tile as f32 * i.height() as f32 / i.width() as f32) as u32;
            let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            // Generic names (ours.png) are labelled by their directory (iter_NN).
            let name = if stem == "ours" {
                p.parent()
                    .and_then(|d| d.file_name())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or(stem)
            } else {
                stem
            };
            Ok((name, resize(&i, tile, h)))
        })
        .collect::<Result<_>>()?;
    let th = imgs.iter().map(|i| i.1.height()).max().unwrap_or(1);
    let rows = (imgs.len() as u32).div_ceil(cols.max(1));
    let mut sheet = RgbImage::from_pixel(cols * tile, rows * th, Rgb([30, 30, 34]));
    for (k, (name, im)) in imgs.iter().enumerate() {
        let (x, y) = ((k as u32 % cols) * tile, (k as u32 / cols) * th);
        image::imageops::overlay(&mut sheet, im, i64::from(x), i64::from(y));
        tag(&mut sheet, x + 6, y + 6, name, 2);
    }
    if let Some(d) = out.parent() {
        std::fs::create_dir_all(d)?;
    }
    sheet.save(out)?;
    Ok(())
}
