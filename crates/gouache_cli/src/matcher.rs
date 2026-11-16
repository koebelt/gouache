//! Reference-Match Loop driver (docs/MASTER_SPEC.md §11.2).
//!
//! Writes `docs/shots/match/<ref>/iter_NN/` with ours/ref/side_by_side/flicker/diff_heatmap,
//! per-region crops, palette swatches, `report.json`/`report.md`, and refreshes `progress.png`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gouache_render::ViewMode;
use gouache_world::{CameraSpec, Look, load_json};
use image::{Rgb, RgbImage};
use serde::{Deserialize, Serialize};

use crate::imageops::{self, load_rgb, oklab, oklab_to_rgb, resize, tag};
use crate::metrics::{self, Stats};

/// Camera estimate for a reference.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MatchCamera {
    /// Camera.
    #[serde(flatten)]
    pub camera: CameraSpec,
    /// Free-form time of day.
    #[serde(default)]
    pub time_of_day: String,
    /// Season.
    #[serde(default)]
    pub season: String,
}

/// A region isolating one thing to compare.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Region {
    /// Name (e.g. shadow_edge).
    pub name: String,
    /// x, y, w, h in reference pixels.
    pub rect: [u32; 4],
    /// What it should show.
    pub what_it_should_show: String,
    /// Gates this region at the current milestone (others are reported but informative).
    #[serde(default = "yes")]
    pub gated: bool,
    /// Compute SSIM (only for regions meant to match structurally, e.g. sky, UI).
    #[serde(default)]
    pub structural: bool,
}

fn yes() -> bool {
    true
}

/// Pass thresholds.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Thresholds {
    /// Max weighted palette ΔE (OKLab×100).
    pub palette_de: f32,
    /// Max luminance histogram EMD.
    pub l_emd: f32,
    /// Max chroma histogram L1.
    pub chroma_l1: f32,
    /// Max hue histogram L1.
    pub hue_l1: f32,
    /// Max |Δ shadow/light ratio|.
    pub shadow_light: f32,
    /// Max |Δ warm/cool separation|.
    pub warm_cool: f32,
    /// Allowed ours/ref ratio range for local contrast.
    pub local_contrast: [f32; 2],
    /// Allowed ours/ref ratio range for edge density.
    pub edge_density: [f32; 2],
    /// Max log-spectrum RMS distance.
    pub spectrum: f32,
    /// Min SSIM for structural regions.
    pub ssim: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            palette_de: 6.0,
            l_emd: 0.035,
            chroma_l1: 0.5,
            hue_l1: 0.6,
            shadow_light: 0.08,
            warm_cool: 0.025,
            local_contrast: [0.7, 1.45],
            edge_density: [0.6, 1.7],
            spectrum: 0.45,
            ssim: 0.5,
        }
    }
}

/// `<name>.match.json`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MatchSpec {
    /// Reference image file name (inside docs/reference).
    pub image: String,
    /// Scene JSON used to reproduce it.
    pub target_scene: String,
    /// Estimated camera.
    pub camera: MatchCamera,
    /// Look for the reproduction (sun estimated from shadows).
    #[serde(default)]
    pub look: Option<Look>,
    /// Regions.
    pub regions: Vec<Region>,
    /// Thresholds.
    #[serde(default)]
    pub thresholds: Thresholds,
    /// Notes.
    #[serde(default)]
    pub notes: String,
}

#[derive(Serialize)]
struct Check {
    metric: &'static str,
    reference: f32,
    ours: f32,
    distance: f32,
    limit: String,
    pass: bool,
}

#[derive(Serialize)]
struct RegionReport {
    name: String,
    gated: bool,
    checks: Vec<Check>,
    pass: bool,
}

fn compare(r: &Stats, o: &Stats, t: &Thresholds, ssim: Option<f32>) -> Vec<Check> {
    let mut v = vec![
        Check {
            metric: "palette_de",
            reference: 0.0,
            ours: 0.0,
            distance: metrics::palette_de(&r.palette, &o.palette),
            limit: format!("<= {}", t.palette_de),
            pass: false,
        },
        Check {
            metric: "l_hist_emd",
            reference: r.mean_l,
            ours: o.mean_l,
            distance: metrics::emd(&r.l_hist, &o.l_hist),
            limit: format!("<= {}", t.l_emd),
            pass: false,
        },
        Check {
            metric: "chroma_hist_l1",
            reference: r.mean_c,
            ours: o.mean_c,
            distance: metrics::l1(&r.c_hist, &o.c_hist),
            limit: format!("<= {}", t.chroma_l1),
            pass: false,
        },
        Check {
            metric: "hue_hist_l1",
            reference: 0.0,
            ours: 0.0,
            distance: metrics::l1(&r.h_hist, &o.h_hist),
            limit: format!("<= {}", t.hue_l1),
            pass: false,
        },
        Check {
            metric: "shadow_light_ratio",
            reference: r.shadow_light_ratio,
            ours: o.shadow_light_ratio,
            distance: (r.shadow_light_ratio - o.shadow_light_ratio).abs(),
            limit: format!("<= {}", t.shadow_light),
            pass: false,
        },
        Check {
            metric: "warm_cool_sep",
            reference: r.warm_cool_sep,
            ours: o.warm_cool_sep,
            distance: (r.warm_cool_sep - o.warm_cool_sep).abs(),
            limit: format!("<= {}", t.warm_cool),
            pass: false,
        },
        Check {
            metric: "local_contrast",
            reference: r.local_contrast,
            ours: o.local_contrast,
            distance: o.local_contrast / r.local_contrast.max(1e-5),
            limit: format!("ratio in {:?}", t.local_contrast),
            pass: false,
        },
        Check {
            metric: "edge_density",
            reference: r.edge_density,
            ours: o.edge_density,
            distance: o.edge_density / r.edge_density.max(1e-5),
            limit: format!("ratio in {:?}", t.edge_density),
            pass: false,
        },
        Check {
            metric: "spectrum_rms",
            reference: 0.0,
            ours: 0.0,
            distance: metrics::spectrum_dist(&r.spectrum, &o.spectrum),
            limit: format!("<= {}", t.spectrum),
            pass: false,
        },
    ];
    if let Some(s) = ssim {
        v.push(Check {
            metric: "ssim",
            reference: 1.0,
            ours: s,
            distance: s,
            limit: format!(">= {}", t.ssim),
            pass: false,
        });
    }
    for c in &mut v {
        c.pass = match c.metric {
            "palette_de" => c.distance <= t.palette_de,
            "l_hist_emd" => c.distance <= t.l_emd,
            "chroma_hist_l1" => c.distance <= t.chroma_l1,
            "hue_hist_l1" => c.distance <= t.hue_l1,
            "shadow_light_ratio" => c.distance <= t.shadow_light,
            "warm_cool_sep" => c.distance <= t.warm_cool,
            "local_contrast" => (t.local_contrast[0]..=t.local_contrast[1]).contains(&c.distance),
            "edge_density" => (t.edge_density[0]..=t.edge_density[1]).contains(&c.distance),
            "spectrum_rms" => c.distance <= t.spectrum,
            "ssim" => c.distance >= t.ssim,
            _ => false,
        };
    }
    v
}

fn crop(img: &RgbImage, r: [u32; 4]) -> RgbImage {
    image::imageops::crop_imm(img, r[0], r[1], r[2].min(img.width() - r[0]), r[3].min(img.height() - r[1]))
        .to_image()
}

fn side_by_side(a: &RgbImage, b: &RgbImage, la: &str, lb: &str, height: u32) -> RgbImage {
    let wa = (height as f32 * a.width() as f32 / a.height() as f32) as u32;
    let ra = resize(a, wa, height);
    let rb = resize(b, wa, height);
    let mut out = RgbImage::from_pixel(wa * 2 + 8, height, Rgb([12, 12, 14]));
    image::imageops::overlay(&mut out, &ra, 0, 0);
    image::imageops::overlay(&mut out, &rb, i64::from(wa + 8), 0);
    let s = (height / 180).max(2);
    tag(&mut out, 8, 8, la, s);
    tag(&mut out, wa + 16, 8, lb, s);
    out
}

fn blur(img: &RgbImage, f: u32) -> RgbImage {
    let s = resize(img, (img.width() / f).max(1), (img.height() / f).max(1));
    resize(&s, img.width(), img.height())
}

fn heat(t: f32) -> Rgb<u8> {
    let t = t.clamp(0.0, 1.0);
    let c = [
        (t * 3.0).clamp(0.0, 1.0),
        (t * 3.0 - 1.0).clamp(0.0, 1.0),
        (t * 3.0 - 2.0).clamp(0.0, 1.0) * 0.6 + (1.0 - t * 3.0).clamp(0.0, 1.0) * 0.3,
    ];
    Rgb([(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8])
}

fn palette_image(r: &Stats, o: &Stats) -> RgbImage {
    let (w, h) = (600u32, 60u32);
    let mut img = RgbImage::from_pixel(w + 80, h * 2 + 10, Rgb([24, 24, 28]));
    for (row, st) in [r, o].iter().enumerate() {
        let mut x = 80u32;
        for (c, wt) in &st.palette {
            let ww = (wt * w as f32).round() as u32;
            imageops::fill(&mut img, x, row as u32 * (h + 10), ww, h, oklab_to_rgb(*c));
            x += ww;
        }
        tag(&mut img, 4, row as u32 * (h + 10) + 20, if row == 0 { "REF" } else { "OURS" }, 2);
    }
    img
}

fn next_iter(dir: &Path) -> u32 {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str()?.strip_prefix("iter_")?.parse::<u32>().ok())
                .max()
                .map_or(0, |m| m + 1)
        })
        .unwrap_or(0)
}

/// Run one iteration of the loop.
pub fn run(reference: &str, frames: u32, ours: Option<&Path>, iter: Option<u32>) -> Result<()> {
    let refdir = PathBuf::from("docs/reference");
    let spec: MatchSpec = load_json(&refdir.join(format!("{reference}.match.json")))?;
    let ref_img = load_rgb(&refdir.join(&spec.image))?;
    let (w, h) = (ref_img.width(), ref_img.height());
    let base = PathBuf::from("docs/shots/match").join(reference);
    let it = iter.unwrap_or_else(|| next_iter(&base));
    let out = base.join(format!("iter_{it:02}"));
    std::fs::create_dir_all(out.join("regions"))?;

    let ours_img = match ours {
        Some(p) => resize(&load_rgb(p)?, w, h),
        None => {
            let px = crate::render_shot(
                Path::new(&spec.target_scene),
                &spec.camera.camera,
                spec.look.clone(),
                None,
                w,
                h,
                frames,
                ViewMode::Lit,
            )?;
            let rgba = image::RgbaImage::from_raw(w, h, px).context("render size")?;
            image::DynamicImage::ImageRgba8(rgba).to_rgb8()
        }
    };
    ours_img.save(out.join("ours.png"))?;
    ref_img.save(out.join("ref.png"))?;
    side_by_side(&ref_img, &ours_img, "REF", &format!("OURS ITER {it:02}"), 720)
        .save(out.join("side_by_side.png"))?;

    // Flicker: checkerboard interleave (96 px cells).
    let flicker = RgbImage::from_fn(w, h, |x, y| {
        if ((x / 96) + (y / 96)) % 2 == 0 { *ref_img.get_pixel(x, y) } else { *ours_img.get_pixel(x, y) }
    });
    flicker.save(out.join("flicker.png"))?;
    // Low-pass ΔE heatmap (geometry differs, so compare blurred colour fields).
    let (br, bo) = (blur(&ref_img, 24), blur(&ours_img, 24));
    let heatmap = RgbImage::from_fn(w, h, |x, y| {
        let (a, b) = (oklab(*br.get_pixel(x, y)), oklab(*bo.get_pixel(x, y)));
        let de = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        heat(de / 0.25)
    });
    heatmap.save(out.join("diff_heatmap.png"))?;

    let t = &spec.thresholds;
    let (rs, os) = (metrics::stats(&ref_img), metrics::stats(&ours_img));
    palette_image(&rs, &os).save(out.join("palette.png"))?;
    let mut reports = vec![RegionReport {
        name: "whole_frame".into(),
        gated: false,
        checks: compare(&rs, &os, t, None),
        pass: false,
    }];
    for r in &spec.regions {
        let (rc, oc) = (crop(&ref_img, r.rect), crop(&ours_img, r.rect));
        side_by_side(&rc, &oc, &format!("REF {}", r.name), "OURS", 400)
            .save(out.join("regions").join(format!("{}.png", r.name)))?;
        let ssim = r.structural.then(|| metrics::ssim(&rc, &oc));
        reports.push(RegionReport {
            name: r.name.clone(),
            gated: r.gated,
            checks: compare(&metrics::stats(&rc), &metrics::stats(&oc), t, ssim),
            pass: false,
        });
    }
    for rep in &mut reports {
        rep.pass = rep.checks.iter().all(|c| c.pass);
    }
    let all_gated_pass = reports.iter().filter(|r| r.gated).all(|r| r.pass);

    let mut md = String::new();
    writeln!(md, "# Match `{reference}` iter {it:02}\n")?;
    writeln!(md, "Gated regions pass: **{}**\n", if all_gated_pass { "YES" } else { "NO" })?;
    for rep in &reports {
        writeln!(
            md,
            "## {} {}{}\n",
            rep.name,
            if rep.pass { "PASS" } else { "FAIL" },
            if rep.gated { " (gated)" } else { "" }
        )?;
        writeln!(md, "| metric | ref | ours | distance | limit | |\n|---|---|---|---|---|---|")?;
        for c in &rep.checks {
            writeln!(
                md,
                "| {} | {:.3} | {:.3} | {:.3} | {} | {} |",
                c.metric,
                c.reference,
                c.ours,
                c.distance,
                c.limit,
                if c.pass { "ok" } else { "**x**" }
            )?;
        }
        writeln!(md)?;
    }
    std::fs::write(out.join("report.md"), &md)?;
    std::fs::write(
        out.join("report.json"),
        serde_json::to_string_pretty(
            &serde_json::json!({ "reference": reference, "iter": it, "gated_pass": all_gated_pass, "regions": reports }),
        )?,
    )?;

    // Filmstrip of all iterations.
    let mut iters: Vec<PathBuf> = std::fs::read_dir(&base)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("ours.png").exists())
        .collect();
    iters.sort();
    let thumbs: Vec<PathBuf> =
        std::iter::once(out.join("ref.png")).chain(iters.iter().map(|p| p.join("ours.png"))).collect();
    imageops::contact_sheet(&thumbs, &base.join("progress.png"), 4, 480)?;

    println!("{md}");
    println!("images: {}", out.display());
    Ok(())
}
