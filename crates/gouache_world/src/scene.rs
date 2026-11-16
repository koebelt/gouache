//! Scene documents (JSON) and camera bookmarks.

use gouache_procgen::TerrainSpec;
use serde::{Deserialize, Serialize};

use crate::Look;

/// A parametric primitive placed in a test scene. Phase 1 replaces these with tool intents.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Primitive {
    /// Beveled box resting on the terrain (centre x/z, base offset from ground).
    RoundedBox {
        /// Centre x, z (m).
        at: [f32; 2],
        /// Lift above the terrain at the centre (m).
        #[serde(default)]
        lift: f32,
        /// Full size x, y, z (m).
        size: [f32; 3],
        /// Bevel radius (m).
        #[serde(default = "default_radius")]
        radius: f32,
        /// Yaw (degrees).
        #[serde(default)]
        yaw_deg: f32,
        /// sRGB colour.
        color: [f32; 3],
    },
    /// Stone wall along a curve (experiment b; becomes the G7 wall tool intent).
    StoneWall {
        /// Control points x, z (m).
        path: Vec<[f32; 2]>,
        /// Wall parameters.
        #[serde(default)]
        spec: WallSpec,
    },
}

fn default_radius() -> f32 {
    0.08
}

/// Wall parameters (intent).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct WallSpec {
    /// Wall height above local ground (m).
    pub height: f32,
    /// Thickness (m).
    pub thickness: f32,
    /// Course height range (m).
    pub course_h: [f32; 2],
    /// Stone length range (m).
    pub stone_len: [f32; 2],
    /// Mortar gap (m).
    pub gap: f32,
    /// Base stone colour (sRGB).
    pub color: [f32; 3],
    /// Colour jitter (fraction).
    pub color_jitter: f32,
    /// Generator seed.
    pub seed: u64,
}

impl Default for WallSpec {
    fn default() -> Self {
        Self {
            height: 2.4,
            thickness: 0.7,
            course_h: [0.24, 0.36],
            stone_len: [0.35, 0.8],
            gap: 0.035,
            color: [0.86, 0.78, 0.60],
            color_jitter: 0.09,
            seed: 1,
        }
    }
}

/// A camera.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraSpec {
    /// Eye position (m).
    pub eye: [f32; 3],
    /// Look-at target (m).
    pub target: [f32; 3],
    /// Vertical field of view (degrees).
    pub fov_y_deg: f32,
}

/// A named camera + optional look overrides, committed as JSON under `bookmarks/`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bookmark {
    /// Name.
    pub name: String,
    /// Scene it belongs to.
    pub scene: String,
    /// Camera.
    pub camera: CameraSpec,
    /// Look overrides (falls back to the scene's look).
    #[serde(default)]
    pub look: Option<Look>,
}

/// A scene document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneDoc {
    /// Scene name.
    pub name: String,
    /// Terrain base parameters.
    #[serde(default)]
    pub terrain: TerrainSpec,
    /// Placed primitives.
    #[serde(default)]
    pub primitives: Vec<Primitive>,
    /// Default look.
    #[serde(default)]
    pub look: Look,
}
