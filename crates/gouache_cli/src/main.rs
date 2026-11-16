//! `gouache_cli`: headless tools — shot, contact, match, golden, hash.

mod imageops;
mod matcher;
mod metrics;

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use gouache_render::{GpuContext, RenderSettings, Renderer, ViewMode};
use gouache_world::{Bookmark, CameraSpec, Look, SceneDoc, load_json};

#[derive(Parser)]
#[command(name = "gouache_cli", about = "Gouache headless tools")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Render a scene from a camera bookmark to PNG.
    Shot {
        /// Camera bookmark JSON (also names the scene).
        #[arg(long)]
        bookmark: PathBuf,
        /// Override scene JSON.
        #[arg(long)]
        scene: Option<PathBuf>,
        /// Output PNG.
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 1920)]
        width: u32,
        #[arg(long, default_value_t = 1080)]
        height: u32,
        /// Accumulated frames (GI convergence + supersampling).
        #[arg(long, default_value_t = 96)]
        frames: u32,
        /// View mode: lit, indirect, ao, shadow, normals.
        #[arg(long, default_value = "lit")]
        view: String,
        /// Look override JSON (merged over scene look).
        #[arg(long)]
        look: Option<PathBuf>,
    },
    /// Grid several PNGs into one contact sheet.
    Contact {
        /// Output PNG.
        #[arg(long)]
        out: PathBuf,
        /// Columns.
        #[arg(long, default_value_t = 3)]
        cols: u32,
        /// Tile width.
        #[arg(long, default_value_t = 640)]
        tile: u32,
        /// Inputs.
        inputs: Vec<PathBuf>,
    },
    /// Reference-match loop: render at the reference camera and compare (docs/MASTER_SPEC.md §11).
    Match {
        /// Reference name (docs/reference/<name>.match.json).
        #[arg(long = "ref")]
        reference: String,
        /// Accumulated frames.
        #[arg(long, default_value_t = 96)]
        frames: u32,
        /// Compare an existing render instead of rendering.
        #[arg(long)]
        ours: Option<PathBuf>,
        /// Iteration label (default: next free iter_NN).
        #[arg(long)]
        iter: Option<u32>,
    },
    /// Compare a render against a golden image with perceptual tolerance.
    Golden {
        /// Candidate PNG.
        #[arg(long)]
        candidate: PathBuf,
        /// Golden PNG.
        #[arg(long)]
        golden: PathBuf,
        /// Max allowed mean ΔE (OKLab×100).
        #[arg(long, default_value_t = 2.0)]
        max_mean_de: f32,
    },
    /// Crop rectangles (x,y,w,h) out of an image: `--rect name:x,y,w,h` (repeatable).
    Crop {
        /// Input image.
        #[arg(long)]
        input: PathBuf,
        /// Output directory.
        #[arg(long)]
        out_dir: PathBuf,
        /// name:x,y,w,h
        #[arg(long = "rect")]
        rects: Vec<String>,
    },
    /// Print deterministic content hashes of a scene's generated meshes.
    Hash {
        /// Scene JSON.
        #[arg(long)]
        scene: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Shot { bookmark, scene, out, width, height, frames, view, look } => {
            let bm: Bookmark = load_json(&bookmark)?;
            let scene_path = scene.unwrap_or_else(|| PathBuf::from(&bm.scene));
            let view = ViewMode::parse(&view).with_context(|| format!("unknown view {view}"))?;
            let look_override: Option<Look> = look.map(|p| load_json(&p)).transpose()?;
            let img = render_shot(
                &scene_path,
                &bm.camera,
                bm.look.clone(),
                look_override,
                width,
                height,
                frames,
                view,
            )?;
            save_png(&out, &img, width, height)?;
            println!("wrote {}", out.display());
        }
        Cmd::Contact { out, cols, tile, inputs } => {
            imageops::contact_sheet(&inputs, &out, cols, tile)?;
            println!("wrote {}", out.display());
        }
        Cmd::Match { reference, frames, ours, iter } => {
            matcher::run(&reference, frames, ours.as_deref(), iter)?
        }
        Cmd::Golden { candidate, golden, max_mean_de } => {
            let de = imageops::mean_delta_e(&candidate, &golden)?;
            println!("mean ΔE = {de:.3} (max {max_mean_de})");
            if de > max_mean_de {
                bail!("golden mismatch");
            }
        }
        Cmd::Crop { input, out_dir, rects } => {
            let img = imageops::load_rgb(&input)?;
            std::fs::create_dir_all(&out_dir)?;
            for r in rects {
                let (name, nums) = r.split_once(':').context("expected name:x,y,w,h")?;
                let v: Vec<u32> = nums.split(',').map(str::parse).collect::<Result<_, _>>()?;
                anyhow::ensure!(v.len() == 4, "expected 4 numbers in {r}");
                let c = image::imageops::crop_imm(&img, v[0], v[1], v[2], v[3]).to_image();
                c.save(out_dir.join(format!("{name}.png")))?;
            }
        }
        Cmd::Hash { scene } => {
            let doc: SceneDoc = load_json(&scene)?;
            let built = gouache_buildings::scene_build::build(&doc);
            for (i, m) in built.meshes.iter().enumerate() {
                println!("mesh {i}: tris={} hash={:016x}", m.triangles(), m.content_hash());
            }
        }
    }
    Ok(())
}

/// Render a scene file from a camera; returns tightly packed RGBA8.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_shot(
    scene_path: &Path,
    camera: &CameraSpec,
    bookmark_look: Option<Look>,
    look_override: Option<Look>,
    width: u32,
    height: u32,
    frames: u32,
    view: ViewMode,
) -> Result<Vec<u8>> {
    let t0 = Instant::now();
    let doc: SceneDoc = load_json(scene_path)?;
    let look = look_override.or(bookmark_look).unwrap_or_else(|| doc.look.clone());
    let built = gouache_buildings::scene_build::build(&doc);
    for m in &built.meshes {
        m.validate().context("generated mesh invalid")?;
    }
    let t_gen = t0.elapsed();
    let vol = gouache_render::volume::build_volume(Some(&built.heightfield), &built.meshes, 0.5, 384);
    let t_vol = t0.elapsed();
    let ctx = GpuContext::headless()?;
    let info = ctx.info();
    let mut r = Renderer::new(&ctx, RenderSettings { width, height, view, ..Default::default() })?;
    r.set_scene(&ctx, &built.meshes, &vol);
    for i in 0..frames.max(1) {
        r.render(&ctx, camera, &look, true, i as f32 / 60.0);
    }
    ctx.wait_idle();
    let t_render = t0.elapsed();
    let px = r.read_output(&ctx);
    eprintln!(
        "[shot] {} on {} ({}) | gen {:.0} ms, volume {:?}@{:.2}m {:.0} ms, {} frames {:.0} ms",
        doc.name,
        info.name,
        info.backend,
        t_gen.as_secs_f64() * 1e3,
        vol.dims,
        vol.voxel,
        (t_vol - t_gen).as_secs_f64() * 1e3,
        frames,
        (t_render - t_vol).as_secs_f64() * 1e3
    );
    Ok(px)
}

pub(crate) fn save_png(path: &Path, rgba: &[u8], w: u32, h: u32) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    image::save_buffer(path, rgba, w, h, image::ExtendedColorType::Rgba8)
        .with_context(|| format!("writing {}", path.display()))
}
