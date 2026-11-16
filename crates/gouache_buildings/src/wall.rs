//! Experiment (b): irregular beveled stone courses along a curve, conforming to terrain.
//!
//! Courses are stacked relative to the *local* ground under each stone, so the wall "sticks to the
//! ground" on uneven terrain. Every stone is a unique rounded box (size/rotation/colour/inset jitter);
//! a darker recessed mortar core fills the gaps.

use glam::{Vec2, Vec3};
use gouache_math::mesh::MeshData;
use gouache_math::{CatmullRom, Pcg32};

pub use gouache_world::WallSpec;

use crate::primitives::{RoundedBox, rounded_box};

/// Generated wall.
pub struct WallMesh {
    /// Stones + mortar core.
    pub mesh: MeshData,
    /// Number of stones.
    pub stones: usize,
}

/// Build a wall along `path` (x, z control points) on terrain `ground(x, z)`.
pub fn build_wall(path: &[Vec2], spec: &WallSpec, ground: &dyn Fn(f32, f32) -> f32) -> Option<WallMesh> {
    let pts: Vec<Vec3> = path.iter().map(|p| Vec3::new(p.x, 0.0, p.y)).collect();
    let curve = CatmullRom::new(&pts)?;
    let len = curve.length();
    let mut rng = Pcg32::new(spec.seed, 0x5715);
    let mut mesh = MeshData::default();
    let mut stones = 0;

    // Mortar core: thin dark boxes every ~1 m following the curve and ground.
    let core_step = 0.9f32;
    let mut s = 0.0;
    while s < len {
        let seg = core_step.min(len - s);
        let (p, t) = curve.frame_at_length(s + seg * 0.5);
        let g = ground(p.x, p.z);
        let h = spec.height - 0.08;
        mesh.append(&rounded_box(
            &RoundedBox {
                center: Vec3::new(p.x, g + h * 0.5 - 0.2, p.z),
                half: Vec3::new(seg * 0.5 + 0.06, h * 0.5 + 0.2, spec.thickness * 0.5 - 0.07),
                radius: 0.04,
                yaw: -t.z.atan2(t.x),
                color: [spec.color[0] * 0.55, spec.color[1] * 0.52, spec.color[2] * 0.50],
                ground_y: g,
            },
            1,
        ));
        s += seg;
    }

    // Courses.
    let mut base = -0.12f32; // first course slightly buried
    let mut course = 0;
    while base < spec.height - 0.05 {
        let ch = rng.range(spec.course_h[0], spec.course_h[1]).min(spec.height - base + 0.02);
        let mut s = -rng.range(0.0, spec.stone_len[1]); // running bond offset
        while s < len {
            let mut sl = rng.range(spec.stone_len[0], spec.stone_len[1]);
            // Occasionally a short "closer" or long through-stone.
            if rng.f32() < 0.12 {
                sl *= 0.55;
            }
            let (a, b) = (s.max(0.0), (s + sl).min(len));
            s += sl;
            if b - a < 0.12 {
                continue;
            }
            let mid = (a + b) * 0.5;
            let (p, t) = curve.frame_at_length(mid);
            // Stick to local ground: lowest ground point under the stone footprint.
            let (pa, _) = curve.frame_at_length(a);
            let (pb, _) = curve.frame_at_length(b);
            let g = ground(pa.x, pa.z).min(ground(pb.x, pb.z)).min(ground(p.x, p.z));
            let gap = spec.gap * rng.range(0.7, 1.3);
            let half = Vec3::new(
                (b - a) * 0.5 - gap * 0.5,
                ch * 0.5 - gap * 0.5 * rng.range(0.8, 1.2),
                spec.thickness * 0.5 * rng.range(0.92, 1.0),
            );
            let normal = Vec3::new(-t.z, 0.0, t.x);
            // Slight random inset/outset per face, jittered rotation.
            let inset = rng.jitter(0.025);
            let center = Vec3::new(p.x, g + base + ch * 0.5, p.z) + normal * inset;
            let cj = spec.color_jitter;
            let v = rng.range(1.0 - cj, 1.0 + cj * 0.6);
            let warm = rng.jitter(cj * 0.5);
            let color = [
                (spec.color[0] * v * (1.0 + warm)).min(1.0),
                (spec.color[1] * v).min(1.0),
                (spec.color[2] * v * (1.0 - warm)).min(1.0),
            ];
            mesh.append(&rounded_box(
                &RoundedBox {
                    center,
                    half: half.max(Vec3::splat(0.03)),
                    radius: (half.min_element() * rng.range(0.35, 0.6)).min(0.07),
                    yaw: -t.z.atan2(t.x) + rng.jitter(0.035),
                    color,
                    ground_y: g,
                },
                2,
            ));
            stones += 1;
        }
        base += ch;
        course += 1;
        if course > 64 {
            break;
        }
    }
    Some(WallMesh { mesh, stones })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_valid_bounded_deterministic() {
        let path = [Vec2::new(0.0, 0.0), Vec2::new(5.0, 2.0), Vec2::new(10.0, 0.0)];
        let ground = |x: f32, z: f32| (x * 0.3).sin() * 0.5 + z * 0.05;
        let a = build_wall(&path, &WallSpec::default(), &ground).unwrap();
        a.mesh.validate().unwrap();
        assert!(a.stones > 50);
        assert!(a.mesh.triangles() < 150_000);
        let b = build_wall(&path, &WallSpec::default(), &ground).unwrap();
        assert_eq!(a.mesh.content_hash(), b.mesh.content_hash());
    }

    #[test]
    fn random_paths_never_panic() {
        let mut r = Pcg32::new(9, 9);
        for _ in 0..50 {
            let n = 2 + (r.next_u32() % 5) as usize;
            let path: Vec<Vec2> = (0..n).map(|_| Vec2::new(r.jitter(10.0), r.jitter(10.0))).collect();
            if let Some(w) = build_wall(&path, &WallSpec::default(), &|_, _| 0.0) {
                w.mesh.validate().unwrap();
            }
        }
    }
}
