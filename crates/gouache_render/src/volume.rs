//! CPU scene voxelisation -> signed distance field + albedo volume for compute GI tracing.
//!
//! Terrain is treated as solid below the heightfield; closed meshes are surface-rasterised and
//! their interiors found by flood-filling "outside" from the top layer. Distances use the exact
//! separable squared EDT of Felzenszwalb & Huttenlocher (2012).

use glam::Vec3;
use gouache_math::mesh::MeshData;
use rayon::prelude::*;

/// Voxelised scene ready for upload.
pub struct SceneVolume {
    /// Min corner (m).
    pub origin: Vec3,
    /// Voxel edge (m).
    pub voxel: f32,
    /// Voxel counts.
    pub dims: [u32; 3],
    /// Signed distance per voxel (m), negative inside. x fastest, then y, then z.
    pub sdf: Vec<f32>,
    /// Linear albedo (rgba8) per voxel, dilated off the surface.
    pub albedo: Vec<[u8; 4]>,
}

/// Height query used to mark solid terrain columns.
pub trait HeightQuery: Sync {
    /// Terrain height at world (x, z).
    fn height(&self, x: f32, z: f32) -> f32;
}

impl HeightQuery for gouache_procgen::Heightfield {
    fn height(&self, x: f32, z: f32) -> f32 {
        self.sample(x, z)
    }
}

fn srgb_to_linear(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// 1D squared EDT in place (`f` holds 0 at sites, a huge value elsewhere). f64 for precision.
fn edt_1d(f: &mut [f64], v: &mut [usize], z: &mut [f64], d: &mut [f64]) {
    let n = f.len();
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f64::NEG_INFINITY;
    z[1] = f64::INFINITY;
    for q in 1..n {
        let fq = f[q] + (q * q) as f64;
        let mut s;
        loop {
            let p = v[k];
            s = (fq - (f[p] + (p * p) as f64)) / (2.0 * (q as f64 - p as f64));
            if s <= z[k] && k > 0 {
                k -= 1;
            } else {
                break;
            }
        }
        if s <= z[k] {
            // k == 0 and q dominates the whole envelope.
            v[0] = q;
            z[1] = f64::INFINITY;
            continue;
        }
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = f64::INFINITY;
    }
    k = 0;
    for (q, dq_out) in d.iter_mut().enumerate().take(n) {
        while z[k + 1] < q as f64 {
            k += 1;
        }
        let p = v[k];
        let dq = q as f64 - p as f64;
        *dq_out = dq * dq + f[p];
    }
    f.copy_from_slice(&d[..n]);
}

/// Squared Euclidean distance (in voxels) to the nearest `true` voxel.
fn edt_3d(sites: &[bool], dims: [usize; 3]) -> Vec<f64> {
    let [nx, ny, nz] = dims;
    let big = 1e20f64;
    let mut g: Vec<f64> = sites.iter().map(|&s| if s { 0.0 } else { big }).collect();
    let pass = |g: &mut Vec<f64>, len: usize, count: usize, idx: &(dyn Fn(usize, usize) -> usize + Sync)| {
        let lines: Vec<Vec<f64>> = (0..count)
            .into_par_iter()
            .map(|l| {
                let mut f: Vec<f64> = (0..len).map(|i| g[idx(l, i)]).collect();
                let (mut v, mut z, mut d) = (vec![0usize; len], vec![0f64; len + 1], vec![0f64; len]);
                edt_1d(&mut f, &mut v, &mut z, &mut d);
                f
            })
            .collect();
        for (l, f) in lines.into_iter().enumerate() {
            for (i, x) in f.into_iter().enumerate() {
                g[idx(l, i)] = x;
            }
        }
    };
    pass(&mut g, nx, ny * nz, &|l, i| l * nx + i);
    pass(&mut g, ny, nx * nz, &|l, i| (l / nx) * nx * ny + i * nx + (l % nx));
    pass(&mut g, nz, nx * ny, &|l, i| i * nx * ny + l);
    g
}

/// Voxelise `meshes` (+ optional solid terrain) into a signed distance field.
pub fn build_volume(
    terrain: Option<&dyn HeightQuery>,
    meshes: &[MeshData],
    target_voxel: f32,
    max_dim: u32,
) -> SceneVolume {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for m in meshes {
        if let Some((a, b)) = m.aabb() {
            lo = lo.min(a);
            hi = hi.max(b);
        }
    }
    // Pad only vertically: x/z bounds follow the terrain so the outside flood cannot leak under it.
    lo.y -= 2.0;
    hi.y += 8.0;
    let size = hi - lo;
    let voxel = target_voxel.max(size.max_element() / max_dim as f32);
    let dims = [
        (size.x / voxel).ceil() as usize + 1,
        (size.y / voxel).ceil() as usize + 1,
        (size.z / voxel).ceil() as usize + 1,
    ];
    let [nx, ny, nz] = dims;
    let n = nx * ny * nz;
    let at = |x: usize, y: usize, z: usize| (z * ny + y) * nx + x;
    let center = |x: usize, y: usize, z: usize| lo + (Vec3::new(x as f32, y as f32, z as f32) + 0.5) * voxel;

    let mut solid = vec![false; n];
    if let Some(t) = terrain {
        let cols: Vec<(usize, usize, f32)> = (0..nz)
            .into_par_iter()
            .flat_map_iter(|z| (0..nx).map(move |x| (x, z)))
            .map(|(x, z)| {
                let c = center(x, 0, z);
                (x, z, t.height(c.x, c.z))
            })
            .collect();
        for (x, z, h) in cols {
            for y in 0..ny {
                if center(x, y, z).y < h {
                    solid[at(x, y, z)] = true;
                }
            }
        }
    }

    // Surface rasterisation (colour for everything, occupancy for non-terrain meshes).
    let mut surf = vec![false; n];
    let mut col_sum = vec![[0f32; 4]; n];
    let skip_occupancy = usize::from(terrain.is_some());
    for (mi, m) in meshes.iter().enumerate() {
        for t in m.indices.as_chunks::<3>().0 {
            let v = [&m.vertices[t[0] as usize], &m.vertices[t[1] as usize], &m.vertices[t[2] as usize]];
            let p: [Vec3; 3] = [v[0].pos.into(), v[1].pos.into(), v[2].pos.into()];
            let edge = (p[1] - p[0]).length().max((p[2] - p[1]).length()).max((p[0] - p[2]).length());
            let steps = ((edge / (voxel * 0.3)).ceil() as usize).max(1);
            for i in 0..=steps {
                for j in 0..=steps - i {
                    let (a, b) = (i as f32 / steps as f32, j as f32 / steps as f32);
                    let q = p[0] + (p[1] - p[0]) * a + (p[2] - p[0]) * b;
                    let g = (q - lo) / voxel;
                    if g.min_element() < 0.0 {
                        continue;
                    }
                    let (x, y, z) = (g.x as usize, g.y as usize, g.z as usize);
                    if x >= nx || y >= ny || z >= nz {
                        continue;
                    }
                    let k = at(x, y, z);
                    if mi >= skip_occupancy {
                        surf[k] = true;
                    }
                    let vc = v[0].color;
                    let c = &mut col_sum[k];
                    c[0] += srgb_to_linear(vc[0]);
                    c[1] += srgb_to_linear(vc[1]);
                    c[2] += srgb_to_linear(vc[2]);
                    c[3] += 1.0;
                }
            }
        }
    }

    // Flood "outside" from the top layer through non-solid, non-surface voxels.
    let mut outside = vec![false; n];
    let mut stack: Vec<(usize, usize, usize)> = Vec::new();
    for z in 0..nz {
        for x in 0..nx {
            let k = at(x, ny - 1, z);
            if !solid[k] && !surf[k] {
                outside[k] = true;
                stack.push((x, ny - 1, z));
            }
        }
    }
    while let Some((x, y, z)) = stack.pop() {
        let nb = [
            (x.wrapping_sub(1), y, z),
            (x + 1, y, z),
            (x, y.wrapping_sub(1), z),
            (x, y + 1, z),
            (x, y, z.wrapping_sub(1)),
            (x, y, z + 1),
        ];
        for (a, b, c) in nb {
            if a >= nx || b >= ny || c >= nz {
                continue;
            }
            let k = at(a, b, c);
            if !outside[k] && !solid[k] && !surf[k] {
                outside[k] = true;
                stack.push((a, b, c));
            }
        }
    }
    let inside: Vec<bool> = outside.iter().map(|o| !o).collect();
    let d_in = edt_3d(&inside, dims);
    let d_out = edt_3d(&outside, dims);
    let sdf: Vec<f32> = (0..n)
        .into_par_iter()
        .map(|k| {
            let d = if outside[k] { d_in[k].sqrt() - 0.5 } else { -(d_out[k].sqrt() - 0.5) };
            (d as f32 * voxel).clamp(-8.0, 30.0)
        })
        .collect();

    // Albedo: average, then dilate twice so trilinear lookups near the surface stay coloured.
    let mut alb: Vec<[f32; 4]> = col_sum
        .iter()
        .map(|c| if c[3] > 0.0 { [c[0] / c[3], c[1] / c[3], c[2] / c[3], 1.0] } else { [0.0; 4] })
        .collect();
    for _ in 0..3 {
        let prev = alb.clone();
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let k = at(x, y, z);
                    if prev[k][3] > 0.0 {
                        continue;
                    }
                    let mut s = [0f32; 4];
                    for (a, b, c) in [
                        (x.wrapping_sub(1), y, z),
                        (x + 1, y, z),
                        (x, y.wrapping_sub(1), z),
                        (x, y + 1, z),
                        (x, y, z.wrapping_sub(1)),
                        (x, y, z + 1),
                    ] {
                        if a < nx && b < ny && c < nz {
                            let p = prev[at(a, b, c)];
                            if p[3] > 0.0 {
                                s = [s[0] + p[0], s[1] + p[1], s[2] + p[2], s[3] + 1.0];
                            }
                        }
                    }
                    if s[3] > 0.0 {
                        alb[k] = [s[0] / s[3], s[1] / s[3], s[2] / s[3], 1.0];
                    }
                }
            }
        }
    }
    let albedo = alb
        .iter()
        .map(|c| {
            let q = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            [q(c[0]), q(c[1]), q(c[2]), q(c[3])]
        })
        .collect();
    SceneVolume { origin: lo, voxel, dims: [nx as u32, ny as u32, nz as u32], sdf, albedo }
}

/// IEEE f32 -> f16 bits (round to nearest even, no NaN payload care).
pub fn f32_to_f16(x: f32) -> u16 {
    let b = x.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let mant = b & 0x7f_ffff;
    if exp >= 31 {
        return sign | 0x7c00;
    }
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        let m = (mant | 0x80_0000) >> (1 - exp);
        return sign | ((m + 0x1000) >> 13) as u16;
    }
    let h = (sign as u32) | ((exp as u32) << 10) | (mant >> 13);
    let round = (mant >> 12) & 1;
    (h + round) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edt_line() {
        let mut f = vec![1e20, 0.0, 1e20, 1e20, 0.0, 1e20];
        let (mut v, mut z, mut d) = (vec![0; 6], vec![0.0; 7], vec![0.0; 6]);
        edt_1d(&mut f, &mut v, &mut z, &mut d);
        assert_eq!(f, vec![1.0, 0.0, 1.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn half_conversion() {
        assert_eq!(f32_to_f16(1.0), 0x3c00);
        assert_eq!(f32_to_f16(-2.0), 0xc000);
        assert_eq!(f32_to_f16(0.0), 0);
    }

    struct Flat;
    impl HeightQuery for Flat {
        fn height(&self, _: f32, _: f32) -> f32 {
            0.0
        }
    }

    #[test]
    fn flat_terrain_sdf_sign() {
        let mut m = MeshData::default();
        use gouache_math::mesh::Vertex;
        for p in [[-5.0, 0.0, -5.0], [5.0, 0.0, -5.0], [-5.0, 0.0, 5.0]] {
            m.vertices.push(Vertex { pos: p, normal: [0.0, 1.0, 0.0], ..Default::default() });
        }
        m.indices = vec![0, 2, 1];
        let v = build_volume(Some(&Flat), &[m], 0.5, 256);
        let probe = |y: f32| {
            let g = ((Vec3::new(0.0, y, 0.0) - v.origin) / v.voxel).as_uvec3();
            v.sdf[((g.z * v.dims[1] + g.y) * v.dims[0] + g.x) as usize]
        };
        assert!(probe(3.0) > 2.0);
        assert!(probe(-1.0) < 0.0);
    }
}
