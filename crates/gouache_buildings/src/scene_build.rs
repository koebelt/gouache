//! Turn a [`SceneDoc`] into meshes (derived caches of intent).

use glam::Vec3;
use gouache_math::mesh::MeshData;
use gouache_procgen::Heightfield;
use gouache_world::{Primitive, SceneDoc};

use crate::primitives::{RoundedBox, rounded_box};

/// Generated geometry for a scene.
pub struct BuiltScene {
    /// Terrain heightfield (for queries and conforming).
    pub heightfield: Heightfield,
    /// All meshes (terrain first).
    pub meshes: Vec<MeshData>,
}

/// Build every mesh of a scene deterministically.
pub fn build(doc: &SceneDoc) -> BuiltScene {
    let hf = Heightfield::generate(&doc.terrain);
    let mut meshes = vec![hf.mesh(&doc.terrain)];
    for p in &doc.primitives {
        match p {
            Primitive::RoundedBox { at, lift, size, radius, yaw_deg, color } => {
                // Sit on the lowest terrain point under the footprint so it never floats.
                let hx = size[0] * 0.5;
                let hz = size[2] * 0.5;
                let mut ground = f32::MAX;
                for (dx, dz) in [(-hx, -hz), (hx, -hz), (-hx, hz), (hx, hz), (0.0, 0.0)] {
                    ground = ground.min(hf.sample(at[0] + dx, at[1] + dz));
                }
                let base = ground - 0.15 + lift;
                meshes.push(rounded_box(
                    &RoundedBox {
                        center: Vec3::new(at[0], base + size[1] * 0.5, at[1]),
                        half: Vec3::from(*size) * 0.5,
                        radius: *radius,
                        yaw: yaw_deg.to_radians(),
                        color: *color,
                        ground_y: ground,
                    },
                    3,
                ));
            }
            Primitive::StoneWall { path, spec } => {
                let pts: Vec<glam::Vec2> = path.iter().map(|p| glam::Vec2::new(p[0], p[1])).collect();
                if let Some(w) = crate::wall::build_wall(&pts, spec, &|x, z| hf.sample(x, z)) {
                    meshes.push(w.mesh);
                }
            }
        }
    }
    BuiltScene { heightfield: hf, meshes }
}
