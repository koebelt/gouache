// Shared frame uniforms and helpers. Must match `FrameUniforms` in frame.rs (std140-ish, vec4 aligned).

struct Frame {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
    sun_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,      // xyz, w = near
    sun_dir: vec4<f32>,         // xyz toward the sun, w = sun angular radius (rad)
    sun_color: vec4<f32>,       // rgb tint multiplier, w = sun intensity
    shadow: vec4<f32>,          // x world extent (m), y depth range (m), z softness, w unused
    resolution: vec4<f32>,      // w, h, 1/w, 1/h
    jitter: vec4<f32>,          // xy current jitter (ndc), z frame index, w GI hysteresis
    params: vec4<f32>,          // x exposure (linear), y gi strength, z ao strength, w sky intensity
    params2: vec4<f32>,         // x bloom, y tonemap contrast, z saturation, w time (s)
    gi_origin: vec4<f32>,       // probe grid min corner xyz, w spacing (m)
    gi_dims: vec4<u32>,         // probe counts x,y,z, w probe update offset (round robin)
    vol_origin: vec4<f32>,      // SDF volume min corner xyz, w voxel size (m)
    vol_dims: vec4<u32>,        // SDF voxel counts xyz, w probes updated per frame
    debug: vec4<u32>,           // x view mode, y accumulated frames, z still camera (1), w unused
};

const PI: f32 = 3.14159265359;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3(2.4)), c / 12.92, c <= vec3(0.04045));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let x = max(c, vec3(0.0));
    return select(1.055 * pow(x, vec3(1.0 / 2.4)) - 0.055, x * 12.92, x <= vec3(0.0031308));
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

// PCG-style integer hash for per-pixel noise.
fn hash_u32(v: u32) -> u32 {
    var x = v * 747796405u + 2891336453u;
    let w = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    return (w >> 22u) ^ w;
}

fn hash_f(v: u32) -> f32 {
    return f32(hash_u32(v) >> 8u) * (1.0 / 16777216.0);
}

// Interleaved gradient noise (Jimenez) for spatial dithering.
fn ign(p: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(p, vec2(0.06711056, 0.00583715))));
}

// R2 low-discrepancy sequence.
fn r2(i: u32) -> vec2<f32> {
    return fract(vec2(0.5) + f32(i) * vec2(0.7548776662, 0.5698402910));
}

fn world_from_depth(uv: vec2<f32>, depth: f32, inv_vp: mat4x4<f32>) -> vec3<f32> {
    let ndc = vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
    let w = inv_vp * ndc;
    return w.xyz / w.w;
}

// Build an orthonormal basis around n (Frisvad / Duff et al.).
fn basis(n: vec3<f32>) -> mat3x3<f32> {
    let s = select(-1.0, 1.0, n.z >= 0.0);
    let a = -1.0 / (s + n.z);
    let b = n.x * n.y * a;
    let t = vec3(1.0 + s * n.x * n.x * a, s * b, -s * n.x);
    let bt = vec3(b, s + n.y * n.y * a, -n.y);
    return mat3x3(t, bt, n);
}

fn cosine_dir(n: vec3<f32>, u: vec2<f32>) -> vec3<f32> {
    let r = sqrt(u.x);
    let phi = 2.0 * PI * u.y;
    let l = vec3(r * cos(phi), r * sin(phi), sqrt(max(0.0, 1.0 - u.x)));
    return normalize(basis(n) * l);
}

fn sphere_dir(u: vec2<f32>) -> vec3<f32> {
    let z = 1.0 - 2.0 * u.x;
    let r = sqrt(max(0.0, 1.0 - z * z));
    let phi = 2.0 * PI * u.y;
    return vec3(r * cos(phi), z, r * sin(phi));
}

// L2 SH irradiance evaluation; coefficients pre-convolved with the cosine lobe.
fn sh9_irradiance(sh: array<vec4<f32>, 9>, n: vec3<f32>) -> vec3<f32> {
    var r = sh[0].rgb * 0.282095;
    r += sh[1].rgb * 0.488603 * n.y;
    r += sh[2].rgb * 0.488603 * n.z;
    r += sh[3].rgb * 0.488603 * n.x;
    r += sh[4].rgb * 1.092548 * n.x * n.y;
    r += sh[5].rgb * 1.092548 * n.y * n.z;
    r += sh[6].rgb * 0.315392 * (3.0 * n.z * n.z - 1.0);
    r += sh[7].rgb * 1.092548 * n.x * n.z;
    r += sh[8].rgb * 0.546274 * (n.x * n.x - n.y * n.y);
    return max(r, vec3(0.0));
}
