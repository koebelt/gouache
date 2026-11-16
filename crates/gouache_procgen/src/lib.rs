//! Deterministic procedural generation (terrain, noise). No GPU.

pub mod noise;
pub mod terrain;

pub use terrain::{Heightfield, TerrainSpec};
