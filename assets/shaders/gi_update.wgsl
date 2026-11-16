// Probe update: each workgroup owns one probe, 64 threads each trace one ray through the
// scene SDF. Hits are lit by the sun (shadow map) + previous probe irradiance (multi-bounce).
// Misses take sky irradiance. Results are projected to L1 SH and blended with hysteresis.
#import "common.wgsl"
#import "gi_common.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var t_sdf: texture_3d<f32>;      // r = signed distance (m), clamped
@group(0) @binding(2) var t_vol_albedo: texture_3d<f32>; // linear albedo of nearest surface
@group(0) @binding(3) var s_lin: sampler;
@group(0) @binding(4) var t_shadow: texture_depth_2d;
@group(0) @binding(5) var s_shadow: sampler_comparison;
@group(0) @binding(6) var<storage, read> sky: array<vec4<f32>, 10>;
@group(0) @binding(7) var<storage, read_write> probes: array<Probe>;

const RAYS: u32 = 64u;

var<workgroup> sh_acc: array<array<vec3<f32>, 4>, 64>;
var<workgroup> inside: array<u32, 64>;

fn vol_uvw(p: vec3<f32>) -> vec3<f32> {
    return (p - frame.vol_origin.xyz) / (vec3<f32>(frame.vol_dims.xyz) * frame.vol_origin.w);
}

fn sdf(p: vec3<f32>) -> f32 {
    let uvw = vol_uvw(p);
    if (any(uvw < vec3(0.0)) || any(uvw > vec3(1.0))) { return 4.0; }
    return textureSampleLevel(t_sdf, s_lin, uvw, 0.0).r;
}

fn sdf_normal(p: vec3<f32>) -> vec3<f32> {
    let e = frame.vol_origin.w * 0.75;
    let d = vec3(
        sdf(p + vec3(e, 0.0, 0.0)) - sdf(p - vec3(e, 0.0, 0.0)),
        sdf(p + vec3(0.0, e, 0.0)) - sdf(p - vec3(0.0, e, 0.0)),
        sdf(p + vec3(0.0, 0.0, e)) - sdf(p - vec3(0.0, 0.0, e)));
    return normalize(d + vec3(0.0, 1e-5, 0.0));
}

fn sun_vis(p: vec3<f32>) -> f32 {
    let sp = frame.sun_view_proj * vec4(p, 1.0);
    let uv = vec2(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
    if (any(uv < vec2(0.0)) || any(uv > vec2(1.0))) { return 1.0; }
    return textureSampleCompareLevel(t_shadow, s_shadow, uv, sp.z - 0.002);
}

fn sky_irr(d: vec3<f32>) -> vec3<f32> {
    var sh: array<vec4<f32>, 9>;
    for (var k = 0u; k < 9u; k++) { sh[k] = sky[k]; }
    return sh9_irradiance(sh, d) * frame.params.w * frame.sun_color.w * frame.ambient_tint.rgb;
}

@compute @workgroup_size(64)
fn main(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) li: u32) {
    let dims = frame.gi_dims.xyz;
    let total = dims.x * dims.y * dims.z;
    let pid = (frame.gi_dims.w + wg.x) % total;
    let c = vec3<u32>(pid % dims.x, (pid / dims.x) % dims.y, pid / (dims.x * dims.y));
    let origin = probe_pos(c, frame);

    // Randomly rotated spherical Fibonacci directions.
    let fi = f32(li) + 0.5;
    let rnd = vec2(hash_f(pid * 7919u + u32(frame.jitter.z)), hash_f(pid * 104729u + u32(frame.jitter.z) * 31u));
    let z = 1.0 - 2.0 * fract(fi / f32(RAYS) + rnd.x * 0.37);
    let phi = 2.0 * PI * fract(fi * 0.6180339887 + rnd.y);
    let rr = sqrt(max(0.0, 1.0 - z * z));
    let dir = vec3(rr * cos(phi), z, rr * sin(phi));

    let d0 = sdf(origin);
    inside[li] = select(0u, 1u, d0 < 0.0);

    var radiance = vec3(0.0);
    var t = max(d0, 0.0) + frame.vol_origin.w * 0.5;
    var hit = false;
    let t_max = 60.0;
    for (var i = 0; i < 96; i++) {
        let p = origin + dir * t;
        let d = sdf(p);
        if (d < frame.vol_origin.w * 0.3) { hit = true; break; }
        t += max(d, frame.vol_origin.w * 0.4);
        if (t > t_max) { break; }
    }
    if (hit) {
        let p = origin + dir * t;
        let n = sdf_normal(p);
        let alb = textureSampleLevel(t_vol_albedo, s_lin, vol_uvw(p), 0.0).rgb;
        let sun = frame.sun_dir.xyz;
        let sun_rad = sky[9].rgb * frame.sun_color.rgb * frame.sun_color.w;
        let surf = p + n * frame.vol_origin.w;
        let direct = sun_rad / PI * max(dot(n, sun), 0.0) * sun_vis(surf);
        // Multi-bounce feedback: read neighbouring probe irradiance at the hit.
        let indirect = gi_sample(surf, n, n, frame);
        radiance = alb * (direct + indirect);
        if (dot(dir, n) > 0.2 && t < frame.vol_origin.w * 2.0) { radiance = vec3(0.0); }
    } else {
        // Sky radiance approximated by its irradiance / pi (smooth, adequate for diffuse).
        radiance = sky_irr(dir);
    }

    // Project to L1 SH (Monte Carlo over the sphere: weight 4pi/N), cosine-convolve, /pi.
    let w = 4.0 * PI / f32(RAYS);
    sh_acc[li][0] = radiance * 0.282095 * w;
    sh_acc[li][1] = radiance * 0.488603 * dir.y * w * (2.0 / 3.0);
    sh_acc[li][2] = radiance * 0.488603 * dir.z * w * (2.0 / 3.0);
    sh_acc[li][3] = radiance * 0.488603 * dir.x * w * (2.0 / 3.0);
    workgroupBarrier();
    var stride = 32u;
    loop {
        if (stride == 0u) { break; }
        if (li < stride) {
            for (var k = 0u; k < 4u; k++) { sh_acc[li][k] += sh_acc[li + stride][k]; }
            inside[li] += inside[li + stride];
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }
    if (li == 0u) {
        var p = probes[pid];
        let first = p.sh[0].w < 0.0;   // -1 marks an uninitialised probe
        let hyst = select(frame.jitter.w, 0.0, first);
        let valid = select(1.0, 0.0, inside[0] > 0u);
        for (var k = 0u; k < 4u; k++) {
            let nv = mix(sh_acc[0][k], p.sh[k].rgb, hyst);
            p.sh[k] = vec4(nv, p.sh[k].w);
        }
        p.sh[0].w = valid;
        probes[pid] = p;
    }
}
