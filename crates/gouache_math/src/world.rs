//! f64 world positions (meters) and camera-relative conversion.

use std::ops::{Add, Sub};

/// Absolute world position in meters. X/Z horizontal, Y up.
#[derive(Clone, Copy, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct WorldPos {
    /// East.
    pub x: f64,
    /// Up.
    pub y: f64,
    /// South.
    pub z: f64,
}

impl WorldPos {
    /// Construct.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Offset relative to `origin`, narrowed to f32 for rendering.
    /// Subtraction happens in f64 so precision is kept far from the world origin.
    pub fn relative_to(self, origin: WorldPos) -> [f32; 3] {
        [(self.x - origin.x) as f32, (self.y - origin.y) as f32, (self.z - origin.z) as f32]
    }
}

impl Add<[f64; 3]> for WorldPos {
    type Output = WorldPos;
    fn add(self, d: [f64; 3]) -> WorldPos {
        WorldPos::new(self.x + d[0], self.y + d[1], self.z + d[2])
    }
}

impl Sub for WorldPos {
    type Output = [f64; 3];
    fn sub(self, o: WorldPos) -> [f64; 3] {
        [self.x - o.x, self.y - o.y, self.z - o.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_precision_far_from_origin() {
        // 200 km from origin, a 1 mm offset must survive.
        let cam = WorldPos::new(200_000.0, 50.0, 200_000.0);
        let p = cam + [0.001, 0.0, 0.0];
        let r = p.relative_to(cam);
        assert!((r[0] - 0.001).abs() < 1e-6);
    }
}
