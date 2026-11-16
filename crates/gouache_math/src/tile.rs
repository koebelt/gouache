//! Power-of-two tile quadtree coordinates.
//!
//! Level 0 is the finest tile ([`TileCoord::LEAF_SIZE_M`] meters). Each level up doubles the size.
//! The root covers `LEAF_SIZE_M << MAX_LEVEL` meters (2^18 m = 262 km by default).

use crate::WorldPos;

/// A tile in the world quadtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct TileCoord {
    /// 0 = finest level.
    pub level: u8,
    /// Tile column (east).
    pub x: i32,
    /// Tile row (south).
    pub z: i32,
}

impl TileCoord {
    /// Leaf tile edge length in meters.
    pub const LEAF_SIZE_M: f64 = 32.0;
    /// Highest level (root). 32 m << 13 = 2^18 m.
    pub const MAX_LEVEL: u8 = 13;

    /// Edge length of this tile in meters.
    pub fn size_m(self) -> f64 {
        Self::LEAF_SIZE_M * f64::from(1u32 << self.level)
    }

    /// Tile containing a world position at `level`.
    pub fn containing(p: WorldPos, level: u8) -> Self {
        let s = Self::LEAF_SIZE_M * f64::from(1u32 << level);
        Self { level, x: (p.x / s).floor() as i32, z: (p.z / s).floor() as i32 }
    }

    /// Minimum (north-west) corner in world meters.
    pub fn origin(self) -> WorldPos {
        let s = self.size_m();
        WorldPos::new(f64::from(self.x) * s, 0.0, f64::from(self.z) * s)
    }

    /// Parent tile, if not the root level.
    pub fn parent(self) -> Option<Self> {
        (self.level < Self::MAX_LEVEL).then(|| Self {
            level: self.level + 1,
            x: self.x.div_euclid(2),
            z: self.z.div_euclid(2),
        })
    }

    /// The four children (NW, NE, SW, SE), if not a leaf.
    pub fn children(self) -> Option<[Self; 4]> {
        (self.level > 0).then(|| {
            let (l, x, z) = (self.level - 1, self.x * 2, self.z * 2);
            [
                Self { level: l, x, z },
                Self { level: l, x: x + 1, z },
                Self { level: l, x, z: z + 1 },
                Self { level: l, x: x + 1, z: z + 1 },
            ]
        })
    }

    /// Edge neighbor at the same level.
    pub fn neighbor(self, dx: i32, dz: i32) -> Self {
        Self { level: self.level, x: self.x + dx, z: self.z + dz }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_2_pow_18() {
        let r = TileCoord { level: TileCoord::MAX_LEVEL, x: 0, z: 0 };
        assert_eq!(r.size_m(), 262_144.0);
    }

    #[test]
    fn parent_child_roundtrip_incl_negative() {
        for &(x, z) in &[(0, 0), (5, -3), (-1, -1), (-8, 7)] {
            let t = TileCoord { level: 2, x, z };
            let p = t.parent().unwrap();
            assert!(p.children().unwrap().contains(&t));
        }
    }

    #[test]
    fn containing_matches_origin() {
        let p = WorldPos::new(-0.5, 0.0, 33.0);
        let t = TileCoord::containing(p, 0);
        assert_eq!((t.x, t.z), (-1, 1));
        let o = t.origin();
        assert!(o.x <= p.x && p.x < o.x + 32.0 && o.z <= p.z && p.z < o.z + 32.0);
    }

    #[test]
    fn neighbors_share_edges() {
        let t = TileCoord { level: 3, x: 2, z: -4 };
        let e = t.neighbor(1, 0);
        assert_eq!(t.origin().x + t.size_m(), e.origin().x);
    }
}
