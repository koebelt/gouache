//! Beveled primitives. Every shape has rounded edges: nothing is razor sharp.

use glam::{Quat, Vec3};
use gouache_math::mesh::{MeshData, Vertex, unorm8};

/// Grid coordinates along one axis of a rounded box face: dense in the bevels, one flat span.
fn axis_steps(half: f32, r: f32, bevel_segs: u32) -> Vec<f32> {
    let r = r.min(half);
    let mut v = Vec::new();
    for i in 0..=bevel_segs {
        let t = i as f32 / bevel_segs as f32;
        v.push(-half + r * (1.0 - (t * std::f32::consts::FRAC_PI_2).cos()));
    }
    let inner = half - r;
    if inner > 1e-4 {
        for i in 0..=bevel_segs {
            let t = i as f32 / bevel_segs as f32;
            v.push(inner + r * (t * std::f32::consts::FRAC_PI_2).sin());
        }
    } else {
        // Fully round axis: mirror.
        for i in (0..bevel_segs).rev() {
            v.push(-v[i as usize]);
        }
    }
    v
}

/// Parameters of a rounded box.
#[derive(Clone, Copy, Debug)]
pub struct RoundedBox {
    /// Centre of the box.
    pub center: Vec3,
    /// Half extents.
    pub half: Vec3,
    /// Bevel radius (m).
    pub radius: f32,
    /// Rotation about Y (radians).
    pub yaw: f32,
    /// sRGB albedo.
    pub color: [f32; 3],
    /// Ground height under the box for baked contact darkening (m), or NaN.
    pub ground_y: f32,
}

/// Generate a rounded box mesh with smooth normals and baked ground-contact AO.
pub fn rounded_box(b: &RoundedBox, bevel_segs: u32) -> MeshData {
    let mut m = MeshData::default();
    let rot = Quat::from_rotation_y(b.yaw);
    let r = b.radius.min(b.half.min_element());
    let inner = b.half - Vec3::splat(r);
    // Faces as (normal axis, u axis, v axis) with orientation chosen for CCW winding.
    let faces: [(Vec3, Vec3, Vec3); 6] = [
        (Vec3::X, Vec3::NEG_Z, Vec3::Y),
        (Vec3::NEG_X, Vec3::Z, Vec3::Y),
        (Vec3::Y, Vec3::X, Vec3::NEG_Z),
        (Vec3::NEG_Y, Vec3::X, Vec3::Z),
        (Vec3::Z, Vec3::X, Vec3::Y),
        (Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y),
    ];
    for (n, u, v) in faces {
        let hu = (u.abs() * b.half).element_sum();
        let hv = (v.abs() * b.half).element_sum();
        let hn = (n.abs() * b.half).element_sum();
        let su = axis_steps(hu, r, bevel_segs);
        let sv = axis_steps(hv, r, bevel_segs);
        let base = m.vertices.len() as u32;
        for &cv in &sv {
            for &cu in &su {
                let p = n * hn + u * cu + v * cv;
                let q = p.clamp(-inner, inner);
                let d = p - q;
                let nrm = if d.length_squared() > 1e-10 { d.normalize() } else { n };
                let local = q + nrm * r;
                let world = b.center + rot * local;
                let wn = rot * nrm;
                let mut ao = 1.0;
                if b.ground_y.is_finite() {
                    let hgt = world.y - b.ground_y;
                    ao = 0.55 + 0.45 * (hgt / 0.8).clamp(0.0, 1.0).sqrt();
                }
                if nrm.y < -0.5 {
                    ao *= 0.5;
                }
                m.vertices.push(Vertex {
                    pos: world.to_array(),
                    normal: wn.to_array(),
                    color: [unorm8(b.color[0]), unorm8(b.color[1]), unorm8(b.color[2]), unorm8(ao)],
                    params: [220, 0, 2, 0],
                });
            }
        }
        let w = su.len() as u32;
        for j in 0..sv.len() as u32 - 1 {
            for i in 0..w - 1 {
                let a = base + j * w + i;
                m.indices.extend_from_slice(&[a, a + 1, a + w + 1, a, a + w + 1, a + w]);
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_is_valid_and_outward() {
        let b = RoundedBox {
            center: Vec3::new(1.0, 2.0, 3.0),
            half: Vec3::new(1.0, 0.5, 2.0),
            radius: 0.1,
            yaw: 0.3,
            color: [0.5; 3],
            ground_y: 1.5,
        };
        let m = rounded_box(&b, 3);
        m.validate().unwrap();
        // Normals point away from the centre and triangle winding agrees with them.
        for t in m.indices.chunks(3) {
            let p: Vec<Vec3> = t.iter().map(|&i| Vec3::from(m.vertices[i as usize].pos)).collect();
            let fnrm = (p[1] - p[0]).cross(p[2] - p[0]);
            if fnrm.length() < 1e-9 {
                continue;
            }
            let vn = Vec3::from(m.vertices[t[0] as usize].normal);
            assert!(fnrm.dot(vn) > 0.0, "winding disagrees with normal");
        }
    }
}
