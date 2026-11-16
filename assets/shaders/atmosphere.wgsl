// Single-scattering Rayleigh + Mie + ozone atmosphere (Hillaire 2020 constants).
// Units: kilometers. Returns radiance relative to a unit-irradiance sun.

const EARTH_R: f32 = 6360.0;
const ATMO_R: f32 = 6460.0;
const RAYLEIGH_SCATTER: vec3<f32> = vec3(5.802e-3, 13.558e-3, 33.1e-3);
const MIE_SCATTER: f32 = 3.996e-3;
const MIE_ABSORB: f32 = 4.40e-3;
const OZONE_ABSORB: vec3<f32> = vec3(0.650e-3, 1.881e-3, 0.085e-3);

fn ray_sphere_far(ro: vec3<f32>, rd: vec3<f32>, r: f32) -> f32 {
    let b = dot(ro, rd);
    let c = dot(ro, ro) - r * r;
    let d = b * b - c;
    if (d < 0.0) { return -1.0; }
    return -b + sqrt(d);
}

fn ray_sphere_near(ro: vec3<f32>, rd: vec3<f32>, r: f32) -> f32 {
    let b = dot(ro, rd);
    let c = dot(ro, ro) - r * r;
    let d = b * b - c;
    if (d < 0.0) { return -1.0; }
    return -b - sqrt(d);
}

struct Medium { scatter_r: vec3<f32>, scatter_m: f32, extinction: vec3<f32> };

fn medium(h: f32) -> Medium {
    let dr = exp(-h / 8.0);
    let dm = exp(-h / 1.2);
    let dozone = max(0.0, 1.0 - abs(h - 25.0) / 15.0);
    var m: Medium;
    m.scatter_r = RAYLEIGH_SCATTER * dr;
    m.scatter_m = MIE_SCATTER * dm;
    m.extinction = m.scatter_r + vec3(m.scatter_m + MIE_ABSORB * dm) + OZONE_ABSORB * dozone;
    return m;
}

fn transmittance_to_space(p: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    let t_max = ray_sphere_far(p, d, ATMO_R);
    let ground = ray_sphere_near(p, d, EARTH_R);
    if (ground > 0.0) { return vec3(0.0); }
    let n = 8;
    let dt = t_max / f32(n);
    var od = vec3(0.0);
    for (var i = 0; i < n; i++) {
        let q = p + d * (f32(i) + 0.5) * dt;
        od += medium(length(q) - EARTH_R).extinction * dt;
    }
    return exp(-od);
}

fn phase_rayleigh(c: f32) -> f32 {
    return 3.0 / (16.0 * PI) * (1.0 + c * c);
}

fn phase_mie(c: f32) -> f32 {
    let g = 0.8;
    let k = 3.0 / (8.0 * PI) * (1.0 - g * g) / (2.0 + g * g);
    return k * (1.0 + c * c) / pow(1.0 + g * g - 2.0 * g * c, 1.5);
}

// Sky radiance seen from 200 m altitude along `dir` (y up), sun direction `sun`.
fn sky_radiance(dir_in: vec3<f32>, sun: vec3<f32>) -> vec3<f32> {
    // Bend the lower hemisphere toward the horizon so the backdrop never goes black.
    let dir = normalize(vec3(dir_in.x, max(dir_in.y, 0.0) + 0.02, dir_in.z));
    let ro = vec3(0.0, EARTH_R + 0.2, 0.0);
    var t_max = ray_sphere_far(ro, dir, ATMO_R);
    let n = 16;
    let dt = t_max / f32(n);
    let c = dot(dir, sun);
    let pr = phase_rayleigh(c);
    let pm = phase_mie(c);
    var trans = vec3(1.0);
    var lum = vec3(0.0);
    for (var i = 0; i < n; i++) {
        let p = ro + dir * (f32(i) + 0.5) * dt;
        let m = medium(length(p) - EARTH_R);
        let sun_t = transmittance_to_space(p, sun);
        let s = (m.scatter_r * pr + vec3(m.scatter_m * pm)) * sun_t;
        let step_t = exp(-m.extinction * dt);
        // Energy-conserving integration (Hillaire).
        lum += trans * (s - s * step_t) / max(m.extinction, vec3(1e-6));
        trans *= step_t;
    }
    return lum;
}

// Sun radiance transmitted to the ground (scaled irradiance color).
fn sun_transmittance(sun: vec3<f32>) -> vec3<f32> {
    let s = normalize(vec3(sun.x, max(sun.y, 0.01), sun.z));
    return transmittance_to_space(vec3(0.0, EARTH_R + 0.2, 0.0), s);
}
