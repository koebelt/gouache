//! GPU-agnostic triangle mesh data shared by generators and the renderer.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Interleaved vertex, 32 bytes. Must match `VsIn` in `gbuffer.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    /// Position (m), chunk-relative.
    pub pos: [f32; 3],
    /// Unit normal.
    pub normal: [f32; 3],
    /// sRGB albedo (rgb) + baked ambient occlusion (a, 255 = unoccluded).
    pub color: [u8; 4],
    /// Roughness, translucency, material id, wind weight.
    pub params: [u8; 4],
}

/// Indexed triangle list.
#[derive(Clone, Debug, Default)]
pub struct MeshData {
    /// Vertices.
    pub vertices: Vec<Vertex>,
    /// Triangle indices (CCW front faces, Y up, right handed).
    pub indices: Vec<u32>,
}

/// Errors from [`MeshData::validate`].
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum MeshError {
    /// A position or normal is NaN/inf.
    #[error("non-finite vertex {0}")]
    NonFinite(usize),
    /// Index out of range.
    #[error("index {0} out of range")]
    BadIndex(usize),
    /// Index count not a multiple of three.
    #[error("index count not a multiple of 3")]
    NotTriangles,
}

impl MeshData {
    /// Append another mesh.
    pub fn append(&mut self, o: &MeshData) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&o.vertices);
        self.indices.extend(o.indices.iter().map(|i| i + base));
    }

    /// Triangle count.
    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }

    /// Axis-aligned bounds, or `None` if empty.
    pub fn aabb(&self) -> Option<(Vec3, Vec3)> {
        let mut it = self.vertices.iter().map(|v| Vec3::from(v.pos));
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }

    /// Check for NaNs and bad indices.
    pub fn validate(&self) -> Result<(), MeshError> {
        if !self.indices.len().is_multiple_of(3) {
            return Err(MeshError::NotTriangles);
        }
        for (i, v) in self.vertices.iter().enumerate() {
            if v.pos.iter().chain(v.normal.iter()).any(|x| !x.is_finite()) {
                return Err(MeshError::NonFinite(i));
            }
        }
        let n = self.vertices.len() as u32;
        if let Some(i) = self.indices.iter().position(|&i| i >= n) {
            return Err(MeshError::BadIndex(i));
        }
        Ok(())
    }

    /// Deterministic content hash (cross-OS determinism checks).
    pub fn content_hash(&self) -> u64 {
        let mut h = 0x9e37_79b9_7f4a_7c15u64;
        for w in bytemuck::cast_slice::<Vertex, u32>(&self.vertices) {
            h = crate::hash_u64(h ^ u64::from(*w));
        }
        for &i in &self.indices {
            h = crate::hash_u64(h ^ u64::from(i).rotate_left(32));
        }
        h
    }
}

/// Pack a linear [0,1] float into u8.
#[inline]
pub fn unorm8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}
