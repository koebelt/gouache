//! Look parameters: sun, sky, exposure, GI and grading. Serialised in scenes and bookmarks.

use serde::{Deserialize, Serialize};

/// All tunable lighting/grading parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Look {
    /// Sun azimuth, degrees clockwise from north (-Z) toward east (+X).
    pub sun_azimuth_deg: f32,
    /// Sun elevation above the horizon (degrees).
    pub sun_elevation_deg: f32,
    /// Sun angular radius (degrees); drives penumbra width.
    pub sun_radius_deg: f32,
    /// Multiplier on the physical penumbra (stylised softness).
    pub shadow_softness: f32,
    /// Sun intensity multiplier.
    pub sun_intensity: f32,
    /// Sun tint (linear multiplier on the atmospheric sun colour).
    pub sun_tint: [f32; 3],
    /// Sky intensity multiplier (ambient + backdrop).
    pub sky_intensity: f32,
    /// Tint on sky light reaching surfaces (stylised: violet shadows).
    pub ambient_tint: [f32; 3],
    /// Tint on the visible sky/backdrop.
    pub sky_tint: [f32; 3],
    /// Multiplier on ambient light where the sun does not reach (cool violet shade).
    pub shade_tint: [f32; 3],
    /// Aerial perspective density (1/m).
    pub haze_density: f32,
    /// Linear exposure multiplier.
    pub exposure: f32,
    /// Diffuse GI strength (0 = sky ambient only).
    pub gi_strength: f32,
    /// Screen-space AO strength.
    pub ao_strength: f32,
    /// Post contrast (gamma around display values).
    pub contrast: f32,
    /// Post saturation.
    pub saturation: f32,
}

impl Default for Look {
    fn default() -> Self {
        Self {
            sun_azimuth_deg: 225.0,
            sun_elevation_deg: 28.0,
            sun_radius_deg: 0.27,
            shadow_softness: 6.0,
            sun_intensity: 3.0,
            sun_tint: [1.0, 0.95, 0.85],
            sky_intensity: 1.0,
            ambient_tint: [1.0, 1.0, 1.0],
            sky_tint: [1.0, 1.0, 1.0],
            haze_density: 0.004,
            shade_tint: [1.0, 1.0, 1.0],
            exposure: 1.6,
            gi_strength: 1.0,
            ao_strength: 1.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

impl Look {
    /// Unit vector pointing toward the sun (Y up, north = -Z).
    pub fn sun_dir(&self) -> [f32; 3] {
        let az = self.sun_azimuth_deg.to_radians();
        let el = self.sun_elevation_deg.to_radians();
        let (sa, ca) = (libm::sinf(az), libm::cosf(az));
        let (se, ce) = (libm::sinf(el), libm::cosf(el));
        [ce * sa, se, -ce * ca]
    }
}
