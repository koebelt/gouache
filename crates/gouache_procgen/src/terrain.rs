//! Diorama heightfield: deterministic base function, smooth normals, slope/curvature colouring.

use glam::Vec3;
use gouache_math::mesh::{MeshData, Vertex, unorm8};
use gouache_math::{lerp, smoothstep};
use serde::{Deserialize, Serialize};

use crate::noise::fbm2;

/// sRGB colour as 0..1 floats (palette entries).
pub type Srgb = [f32; 3];

/// Parameters of the procedural terrain base.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainSpec {
    /// Generator seed.
    pub seed: u32,
    /// Edge length of the square patch (m), centred on the origin.
    pub size_m: f32,
    /// Sample spacing (m).
    pub cell_m: f32,
    /// Hill amplitude (m).
    pub amplitude_m: f32,
    /// Dominant feature wavelength (m).
    pub feature_m: f32,
    /// Radius of the gently flattened centre (m); 0 disables.
    pub flat_radius_m: f32,
    /// Grass colour.
    pub grass: Srgb,
    /// Dry/warm grass colour (blended by noise).
    pub grass_dry: Srgb,
    /// Dirt colour.
    pub dirt: Srgb,
    /// Rock colour on steep slopes.
    pub rock: Srgb,
}

impl Default for TerrainSpec {
    fn default() -> Self {
        Self {
            seed: 1,
            size_m: 150.0,
            cell_m: 0.5,
            amplitude_m: 6.0,
            feature_m: 60.0,
            flat_radius_m: 25.0,
            grass: [0.62, 0.62, 0.30],
            grass_dry: [0.80, 0.66, 0.32],
            dirt: [0.62, 0.48, 0.34],
            rock: [0.60, 0.57, 0.52],
        }
    }
}

/// Regular grid of heights.
#[derive(Clone, Debug)]
pub struct Heightfield {
    /// Samples per side.
    pub n: usize,
    /// Spacing (m).
    pub cell: f32,
    /// World x/z of sample (0, 0).
    pub origin: [f32; 2],
    /// Row-major heights (z rows, x columns).
    pub h: Vec<f32>,
}

impl Heightfield {
    /// Evaluate the deterministic base function for `spec`.
    pub fn generate(spec: &TerrainSpec) -> Self {
        let n = (spec.size_m / spec.cell_m).round() as usize + 1;
        let half = spec.size_m * 0.5;
        let mut h = vec![0.0; n * n];
        for z in 0..n {
            for x in 0..n {
                let wx = -half + x as f32 * spec.cell_m;
                let wz = -half + z as f32 * spec.cell_m;
                h[z * n + x] = base_height(spec, wx, wz);
            }
        }
        Self { n, cell: spec.cell_m, origin: [-half, -half], h }
    }

    /// Height at grid sample (clamped).
    pub fn at(&self, x: isize, z: isize) -> f32 {
        let c = |v: isize| v.clamp(0, self.n as isize - 1) as usize;
        self.h[c(z) * self.n + c(x)]
    }

    /// Bilinear height at world (x, z).
    pub fn sample(&self, wx: f32, wz: f32) -> f32 {
        let gx = (wx - self.origin[0]) / self.cell;
        let gz = (wz - self.origin[1]) / self.cell;
        let (fx, fz) = (gx.floor(), gz.floor());
        let (ix, iz) = (fx as isize, fz as isize);
        let (tx, tz) = (gx - fx, gz - fz);
        let a = lerp(self.at(ix, iz), self.at(ix + 1, iz), tx);
        let b = lerp(self.at(ix, iz + 1), self.at(ix + 1, iz + 1), tx);
        lerp(a, b, tz)
    }

    /// Smooth normal from central differences.
    pub fn normal(&self, x: isize, z: isize) -> Vec3 {
        let dx = self.at(x + 1, z) - self.at(x - 1, z);
        let dz = self.at(x, z + 1) - self.at(x, z - 1);
        Vec3::new(-dx, 2.0 * self.cell, -dz).normalize()
    }

    /// Laplacian (positive in valleys/concave areas).
    pub fn curvature(&self, x: isize, z: isize) -> f32 {
        let c = self.at(x, z);
        (self.at(x + 1, z) + self.at(x - 1, z) + self.at(x, z + 1) + self.at(x, z - 1) - 4.0 * c)
            / (self.cell * self.cell)
    }

    /// Build the render mesh with procedural vertex colouring.
    pub fn mesh(&self, spec: &TerrainSpec) -> MeshData {
        let n = self.n;
        let mut m = MeshData { vertices: Vec::with_capacity(n * n), indices: Vec::new() };
        for z in 0..n {
            for x in 0..n {
                let (xi, zi) = (x as isize, z as isize);
                let wx = self.origin[0] + x as f32 * self.cell;
                let wz = self.origin[1] + z as f32 * self.cell;
                let nrm = self.normal(xi, zi);
                let slope = 1.0 - nrm.y;
                let curv = self.curvature(xi, zi);
                let c = terrain_color(spec, wx, wz, slope);
                // Concave areas collect a little painted-in occlusion.
                let ao = (1.0 - (curv * 2.0).clamp(0.0, 0.35)).clamp(0.0, 1.0);
                m.vertices.push(Vertex {
                    pos: [wx, self.h[z * n + x], wz],
                    normal: nrm.to_array(),
                    color: [unorm8(c[0]), unorm8(c[1]), unorm8(c[2]), unorm8(ao)],
                    params: [230, 40, 1, 0],
                });
            }
        }
        for z in 0..n - 1 {
            for x in 0..n - 1 {
                let i = (z * n + x) as u32;
                let (a, b, c, d) = (i, i + 1, i + n as u32, i + n as u32 + 1);
                // Alternate diagonals to avoid directional faceting.
                if (x + z) % 2 == 0 {
                    m.indices.extend_from_slice(&[a, c, b, b, c, d]);
                } else {
                    m.indices.extend_from_slice(&[a, c, d, a, d, b]);
                }
            }
        }
        m
    }
}

fn base_height(spec: &TerrainSpec, x: f32, z: f32) -> f32 {
    let f = 1.0 / spec.feature_m;
    let mut h = fbm2(x * f, z * f, spec.seed, 5, 2.0, 0.45) * spec.amplitude_m * 2.0;
    // Soft swell so the diorama reads as a raised island of land.
    let r = (x * x + z * z).sqrt();
    let half = spec.size_m * 0.5;
    h -= smoothstep(half * 0.55, half, r) * spec.amplitude_m * 0.8;
    if spec.flat_radius_m > 0.0 {
        let t = smoothstep(spec.flat_radius_m, spec.flat_radius_m * 1.8, r);
        let centre = fbm2(0.0, 0.0, spec.seed, 5, 2.0, 0.45) * spec.amplitude_m * 2.0;
        h = lerp(centre + (h - centre) * 0.08, h, t);
    }
    h
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}

/// Procedural albedo: soft low-frequency blotches of fresh/dry grass, dirt in patches, rock on slopes.
fn terrain_color(spec: &TerrainSpec, x: f32, z: f32, slope: f32) -> [f32; 3] {
    let blotch = fbm2(x * 0.045, z * 0.045, spec.seed ^ 0xabc, 4, 2.0, 0.5) * 0.5 + 0.5;
    let fine = fbm2(x * 0.35, z * 0.35, spec.seed ^ 0x123, 3, 2.0, 0.5) * 0.5 + 0.5;
    let mut c = mix3(spec.grass, spec.grass_dry, smoothstep(0.35, 0.75, blotch));
    let dirt_mask =
        smoothstep(0.62, 0.78, fbm2(x * 0.03, z * 0.03, spec.seed ^ 0x77, 3, 2.0, 0.5) * 0.5 + 0.5);
    c = mix3(c, spec.dirt, dirt_mask * 0.7);
    c = mix3(c, spec.rock, smoothstep(0.25, 0.45, slope));
    let v = 0.92 + fine * 0.16;
    [c[0] * v, c[1] * v, c[2] * v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_valid_and_deterministic() {
        let spec = TerrainSpec { size_m: 20.0, cell_m: 1.0, ..Default::default() };
        let hf = Heightfield::generate(&spec);
        let m = hf.mesh(&spec);
        m.validate().unwrap();
        assert_eq!(m.triangles(), 20 * 20 * 2);
        assert_eq!(m.content_hash(), Heightfield::generate(&spec).mesh(&spec).content_hash());
    }

    #[test]
    fn sample_matches_grid() {
        let spec = TerrainSpec { size_m: 10.0, cell_m: 1.0, ..Default::default() };
        let hf = Heightfield::generate(&spec);
        assert!((hf.sample(-5.0, -5.0) - hf.at(0, 0)).abs() < 1e-5);
    }
}
