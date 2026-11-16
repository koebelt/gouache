//! Perceptual image statistics for the Reference-Match Loop.
//! Exact pixel metrics are meaningless (geometry differs), so we compare distributions.

use image::RgbImage;
use serde::Serialize;

use crate::imageops::oklab;

/// Statistics of one image (or region).
#[derive(Clone, Debug, Serialize)]
pub struct Stats {
    /// Dominant OKLab colours (k-means) with weights, sorted by weight.
    pub palette: Vec<([f32; 3], f32)>,
    /// OKLab L histogram (32 bins, normalised).
    pub l_hist: Vec<f32>,
    /// Chroma histogram (16 bins over 0..0.25).
    pub c_hist: Vec<f32>,
    /// Chroma-weighted hue histogram (24 bins).
    pub h_hist: Vec<f32>,
    /// Mean L of darkest 20% / mean L of brightest 20%.
    pub shadow_light_ratio: f32,
    /// Warmth (b + 0.5 a) of brightest 25% minus darkest 25% (positive = warm light, cool shadow).
    pub warm_cool_sep: f32,
    /// Mean of per-8x8-block L standard deviation.
    pub local_contrast: f32,
    /// Fraction of pixels whose Sobel L gradient exceeds 0.08.
    pub edge_density: f32,
    /// Log radial power spectrum (16 bands, low to high frequency).
    pub spectrum: Vec<f32>,
    /// Mean L, mean chroma.
    pub mean_l: f32,
    /// Mean chroma.
    pub mean_c: f32,
}

fn lab_pixels(img: &RgbImage) -> Vec<[f32; 3]> {
    img.pixels().map(|p| oklab(*p)).collect()
}

fn hist_norm(h: &mut [f32]) {
    let s: f32 = h.iter().sum();
    if s > 0.0 {
        h.iter_mut().for_each(|x| *x /= s);
    }
}

/// Deterministic k-means in OKLab (k-means++-like seeding by luminance quantiles).
fn kmeans(px: &[[f32; 3]], k: usize) -> Vec<([f32; 3], f32)> {
    let step = (px.len() / 20_000).max(1);
    let s: Vec<[f32; 3]> = px.iter().step_by(step).copied().collect();
    let mut sorted = s.clone();
    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut c: Vec<[f32; 3]> = (0..k).map(|i| sorted[(i * 2 + 1) * sorted.len() / (2 * k)]).collect();
    let mut assign = vec![0usize; s.len()];
    for _ in 0..20 {
        for (i, p) in s.iter().enumerate() {
            assign[i] = (0..k).min_by(|&a, &b| d2(p, &c[a]).total_cmp(&d2(p, &c[b]))).unwrap_or(0);
        }
        let mut acc = vec![[0f32; 4]; k];
        for (i, p) in s.iter().enumerate() {
            let a = &mut acc[assign[i]];
            a[0] += p[0];
            a[1] += p[1];
            a[2] += p[2];
            a[3] += 1.0;
        }
        for j in 0..k {
            if acc[j][3] > 0.0 {
                c[j] = [acc[j][0] / acc[j][3], acc[j][1] / acc[j][3], acc[j][2] / acc[j][3]];
            }
        }
    }
    let mut w = vec![0f32; k];
    for &a in &assign {
        w[a] += 1.0;
    }
    let n = s.len() as f32;
    let mut out: Vec<([f32; 3], f32)> = c.into_iter().zip(w.into_iter().map(|x| x / n)).collect();
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out
}

fn d2(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f32::consts::PI / len as f32;
        for i in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (s, c) = (ang * k as f32).sin_cos();
                let (a, b) = (i + k, i + k + len / 2);
                let tr = re[b] * c - im[b] * s;
                let ti = re[b] * s + im[b] * c;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        len <<= 1;
    }
}

fn spectrum(img: &RgbImage) -> Vec<f32> {
    const N: usize = 128;
    let small = image::imageops::resize(img, N as u32, N as u32, image::imageops::FilterType::Triangle);
    let l: Vec<f32> = small.pixels().map(|p| oklab(*p)[0]).collect();
    let mean = l.iter().sum::<f32>() / l.len() as f32;
    let (mut re, mut im) = (vec![0f32; N * N], vec![0f32; N * N]);
    for y in 0..N {
        for x in 0..N {
            // Hann window against edge leakage.
            let w = (0.5 - 0.5 * (2.0 * std::f32::consts::PI * x as f32 / N as f32).cos())
                * (0.5 - 0.5 * (2.0 * std::f32::consts::PI * y as f32 / N as f32).cos());
            re[y * N + x] = (l[y * N + x] - mean) * w;
        }
    }
    for y in 0..N {
        fft(&mut re[y * N..(y + 1) * N], &mut im[y * N..(y + 1) * N]);
    }
    for x in 0..N {
        let (mut cr, mut ci): (Vec<f32>, Vec<f32>) = (0..N).map(|y| (re[y * N + x], im[y * N + x])).unzip();
        fft(&mut cr, &mut ci);
        for y in 0..N {
            re[y * N + x] = cr[y];
            im[y * N + x] = ci[y];
        }
    }
    let mut bands = [0f32; 16];
    let mut counts = [0f32; 16];
    for y in 0..N {
        for x in 0..N {
            let fx = if x < N / 2 { x as f32 } else { x as f32 - N as f32 };
            let fy = if y < N / 2 { y as f32 } else { y as f32 - N as f32 };
            let r = (fx * fx + fy * fy).sqrt() / (N as f32 / 2.0);
            if r <= 0.0 || r >= 1.0 {
                continue;
            }
            let b = (r * 16.0) as usize;
            bands[b] += re[y * N + x].powi(2) + im[y * N + x].powi(2);
            counts[b] += 1.0;
        }
    }
    bands.iter().zip(&counts).map(|(p, c)| (p / c.max(1.0) + 1e-9).log10()).collect()
}

/// Compute all statistics.
pub fn stats(img: &RgbImage) -> Stats {
    let px = lab_pixels(img);
    let n = px.len().max(1) as f32;
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut l_hist = vec![0f32; 32];
    let mut c_hist = vec![0f32; 16];
    let mut h_hist = vec![0f32; 24];
    let (mut ml, mut mc) = (0.0, 0.0);
    for p in &px {
        let c = (p[1] * p[1] + p[2] * p[2]).sqrt();
        l_hist[((p[0].clamp(0.0, 0.9999)) * 32.0) as usize] += 1.0;
        c_hist[((c / 0.25).clamp(0.0, 0.9999) * 16.0) as usize] += 1.0;
        let hue = p[2].atan2(p[1]) / (2.0 * std::f32::consts::PI) + 0.5;
        h_hist[(hue.clamp(0.0, 0.9999) * 24.0) as usize] += c;
        ml += p[0];
        mc += c;
    }
    hist_norm(&mut l_hist);
    hist_norm(&mut c_hist);
    hist_norm(&mut h_hist);
    let mut by_l: Vec<&[f32; 3]> = px.iter().collect();
    by_l.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mean_of = |s: &[&[f32; 3]], f: &dyn Fn(&[f32; 3]) -> f32| {
        s.iter().map(|p| f(p)).sum::<f32>() / s.len().max(1) as f32
    };
    let k20 = by_l.len() / 5;
    let k25 = by_l.len() / 4;
    let dark = &by_l[..k20.max(1)];
    let bright = &by_l[by_l.len() - k20.max(1)..];
    let shadow_light_ratio = mean_of(dark, &|p| p[0]) / mean_of(bright, &|p| p[0]).max(1e-4);
    let warmth = |p: &[f32; 3]| p[2] + 0.5 * p[1];
    let warm_cool_sep =
        mean_of(&by_l[by_l.len() - k25.max(1)..], &warmth) - mean_of(&by_l[..k25.max(1)], &warmth);
    // Local contrast (8x8 blocks).
    let mut lc = 0.0;
    let mut blocks = 0.0;
    for by in (0..h.saturating_sub(7)).step_by(8) {
        for bx in (0..w.saturating_sub(7)).step_by(8) {
            let (mut s, mut s2) = (0.0, 0.0);
            for y in by..by + 8 {
                for x in bx..bx + 8 {
                    let l = px[y * w + x][0];
                    s += l;
                    s2 += l * l;
                }
            }
            lc += (s2 / 64.0 - (s / 64.0f32).powi(2)).max(0.0).sqrt();
            blocks += 1.0;
        }
    }
    // Edge density.
    let mut edges = 0.0;
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let l = |dx: isize, dy: isize| px[(y as isize + dy) as usize * w + (x as isize + dx) as usize][0];
            let gx = l(1, -1) + 2.0 * l(1, 0) + l(1, 1) - l(-1, -1) - 2.0 * l(-1, 0) - l(-1, 1);
            let gy = l(-1, 1) + 2.0 * l(0, 1) + l(1, 1) - l(-1, -1) - 2.0 * l(0, -1) - l(1, -1);
            if (gx * gx + gy * gy).sqrt() > 0.08 {
                edges += 1.0;
            }
        }
    }
    Stats {
        palette: kmeans(&px, 6),
        l_hist,
        c_hist,
        h_hist,
        shadow_light_ratio,
        warm_cool_sep,
        local_contrast: lc / f32::max(blocks, 1.0),
        edge_density: edges / n,
        spectrum: spectrum(img),
        mean_l: ml / n,
        mean_c: mc / n,
    }
}

/// Earth mover's distance between two 1D histograms (in bins / bin count).
pub fn emd(a: &[f32], b: &[f32]) -> f32 {
    let (mut ca, mut cb, mut d) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        ca += x;
        cb += y;
        d += f32::abs(ca - cb);
    }
    d / a.len() as f32
}

/// L1 distance between histograms (0..2).
pub fn l1(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum()
}

/// Weighted palette distance: each reference cluster to its nearest ours cluster (OKLab ΔE×100).
pub fn palette_de(r: &[([f32; 3], f32)], o: &[([f32; 3], f32)]) -> f32 {
    let mut s = 0.0;
    for (rc, rw) in r {
        let best = o.iter().map(|(oc, _)| d2(rc, oc).sqrt()).fold(f32::MAX, f32::min);
        s += best * rw;
    }
    s * 100.0
}

/// Mean squared difference of log spectra.
pub fn spectrum_dist(a: &[f32], b: &[f32]) -> f32 {
    (a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f32>() / a.len() as f32).sqrt()
}

/// Mean SSIM on OKLab L (8x8 windows), for regions meant to match structurally.
pub fn ssim(a: &RgbImage, b: &RgbImage) -> f32 {
    let la: Vec<f32> = a.pixels().map(|p| oklab(*p)[0]).collect();
    let lb: Vec<f32> = b.pixels().map(|p| oklab(*p)[0]).collect();
    let (w, h) = (a.width() as usize, a.height() as usize);
    let (c1, c2) = (0.01f32.powi(2), 0.03f32.powi(2));
    let (mut s, mut n) = (0.0, 0.0);
    for by in (0..h.saturating_sub(7)).step_by(4) {
        for bx in (0..w.saturating_sub(7)).step_by(4) {
            let (mut ma, mut mb, mut va, mut vb, mut cov) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for y in by..by + 8 {
                for x in bx..bx + 8 {
                    ma += la[y * w + x];
                    mb += lb[y * w + x];
                }
            }
            ma /= 64.0;
            mb /= 64.0;
            for y in by..by + 8 {
                for x in bx..bx + 8 {
                    let (da, db) = (la[y * w + x] - ma, lb[y * w + x] - mb);
                    va += da * da;
                    vb += db * db;
                    cov += da * db;
                }
            }
            va /= 63.0;
            vb /= 63.0;
            cov /= 63.0;
            s += ((2.0 * ma * mb + c1) * (2.0 * cov + c2)) / ((ma * ma + mb * mb + c1) * (va + vb + c2));
            n += 1.0;
        }
    }
    s / f32::max(n, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_have_zero_distances() {
        let img = RgbImage::from_fn(64, 48, |x, y| image::Rgb([(x * 4) as u8, (y * 5) as u8, 90]));
        let a = stats(&img);
        assert!(emd(&a.l_hist, &a.l_hist) < 1e-6);
        assert!(palette_de(&a.palette, &a.palette) < 1e-4);
        assert!((ssim(&img, &img) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn fft_impulse_is_flat() {
        let (mut re, mut im) = (vec![0.0; 8], vec![0.0; 8]);
        re[0] = 1.0;
        fft(&mut re, &mut im);
        assert!(re.iter().all(|x| (x - 1.0).abs() < 1e-6));
    }
}
