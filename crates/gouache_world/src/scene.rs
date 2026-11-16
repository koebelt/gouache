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
}

fn default_radius() -> f32 {
    0.08
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
