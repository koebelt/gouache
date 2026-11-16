//! Gouache deterministic math.
//!
//! Everything in this crate must produce bit-identical results on every OS:
//! integer hashing, `libm` transcendentals, no fast-math, no `HashMap` iteration.
//! It has no GPU dependency so the future Mandate server can reuse it.

pub mod curve;
pub mod mesh;
pub mod rng;
pub mod tile;
pub mod world;

pub use curve::CatmullRom;
pub use rng::{Pcg32, hash_u32, hash_u64, hash2, hash3};
pub use tile::TileCoord;
pub use world::WorldPos;

/// Smooth Hermite step, deterministic.
#[inline]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
