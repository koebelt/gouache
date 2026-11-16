//! Per-frame uniforms (must match `Frame` in common.wgsl) and camera math.

use bytemuck::{Pod, Zeroable};
use glam::camera::rh::proj::directx as proj;
use glam::camera::rh::view::look_at_mat4;
use glam::{Mat4, Vec3, Vec4};
use gouache_world::{CameraSpec, Look};

/// GPU frame uniforms.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FrameUniforms {
    pub(crate) view_proj: [[f32; 4]; 4],
    pub(crate) inv_view_proj: [[f32; 4]; 4],
    pub(crate) prev_view_proj: [[f32; 4]; 4],
    pub(crate) sun_view_proj: [[f32; 4]; 4],
    pub(crate) camera_pos: [f32; 4],
    pub(crate) sun_dir: [f32; 4],
    pub(crate) sun_color: [f32; 4],
    pub(crate) shadow: [f32; 4],
    pub(crate) resolution: [f32; 4],
    pub(crate) jitter: [f32; 4],
    pub(crate) params: [f32; 4],
    pub(crate) params2: [f32; 4],
    pub(crate) gi_origin: [f32; 4],
    pub(crate) gi_dims: [u32; 4],
    pub(crate) vol_origin: [f32; 4],
    pub(crate) vol_dims: [u32; 4],
    pub(crate) debug: [u32; 4],
    pub(crate) ambient_tint: [f32; 4],
    pub(crate) sky_tint: [f32; 4],
    pub(crate) haze: [f32; 4],
}

/// Debug visualisation modes (lighting.wgsl).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    /// Final image.
    #[default]
    Lit = 0,
    /// Indirect diffuse only.
    Indirect = 1,
    /// Ambient occlusion.
    Occlusion = 2,
    /// Sun shadow factor.
    Shadow = 3,
    /// World normals.
    Normals = 4,
}

impl ViewMode {
    /// Parse a CLI name.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "lit" => Self::Lit,
            "indirect" => Self::Indirect,
            "ao" => Self::Occlusion,
            "shadow" => Self::Shadow,
            "normals" => Self::Normals,
            _ => return None,
        })
    }
}

/// Reversed-Z infinite perspective view-projection for a camera.
pub fn camera_view_proj(cam: &CameraSpec, aspect: f32, near: f32) -> Mat4 {
    let eye = Vec3::from(cam.eye);
    let view = look_at_mat4(eye, Vec3::from(cam.target), Vec3::Y);
    let proj = proj::perspective_infinite_reverse(cam.fov_y_deg.to_radians(), aspect, near);
    proj * view
}

/// Orthographic sun matrix fitted to a bounding sphere; returns (matrix, extent m, depth range m).
pub fn sun_view_proj(look: &Look, center: Vec3, radius: f32) -> (Mat4, f32, f32) {
    let dir = Vec3::from(look.sun_dir());
    let eye = center + dir * radius * 2.0;
    let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
    let view = look_at_mat4(eye, center, up);
    let depth = radius * 4.0;
    // wgpu clip z in [0,1]; orthographic_rh maps near->0, far->1.
    let proj = proj::orthographic(-radius, radius, -radius, radius, 0.0, depth);
    (proj * view, radius * 2.0, depth)
}

/// Halton(2,3) subpixel jitter in NDC for accumulation frame `i`.
pub fn jitter(i: u32, w: u32, h: u32) -> [f32; 2] {
    fn halton(mut i: u32, b: u32) -> f32 {
        let (mut f, mut r) = (1.0, 0.0);
        while i > 0 {
            f /= b as f32;
            r += f * (i % b) as f32;
            i /= b;
        }
        r
    }
    let j = i % 64 + 1;
    [(halton(j, 2) - 0.5) * 2.0 / w as f32, (halton(j, 3) - 0.5) * 2.0 / h as f32]
}

pub(crate) fn v4(v: Vec3, w: f32) -> [f32; 4] {
    Vec4::new(v.x, v.y, v.z, w).to_array()
}
