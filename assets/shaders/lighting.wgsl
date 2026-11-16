// Deferred lighting: sun (PCSS soft shadows) + diffuse GI (probe grid) * AO + sky backdrop.
#import "common.wgsl"
#import "atmosphere.wgsl"
#import "gi_common.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var t_depth: texture_depth_2d;
@group(0) @binding(2) var t_albedo: texture_2d<f32>;
@group(0) @binding(3) var t_normal: texture_2d<f32>;
@group(0) @binding(4) var t_shadow: texture_depth_2d;
@group(0) @binding(5) var s_shadow: sampler_comparison;
@group(0) @binding(6) var t_ao: texture_2d<f32>;
@group(0) @binding(7) var<storage, read> sky: array<vec4<f32>, 10>;
@group(0) @binding(8) var<storage, read> probes: array<Probe>;
@group(0) @binding(9) var out_hdr: texture_storage_2d<rgba16float, write>;

fn vogel(i: u32, n: u32, rot: f32) -> vec2<f32> {
    let r = sqrt((f32(i) + 0.5) / f32(n));
    let th = f32(i) * 2.39996323 + rot;
    return r * vec2(cos(th), sin(th));
}

// Contact-hardening soft shadow. Shadow space is orthographic, so depth is linear in meters.
fn sun_shadow(world: vec3<f32>, n: vec3<f32>, noise: f32) -> f32 {
    let size = frame.shadow.x;                    // world extent of the shadow map (m)
    let dims = vec2<f32>(textureDimensions(t_shadow));
    let texel = size / dims.x;
    let ndl = dot(n, frame.sun_dir.xyz);
    let offset = n * texel * (1.5 + 2.0 * (1.0 - abs(ndl)));
    let sp = frame.sun_view_proj * vec4(world + offset, 1.0);
    let uv = vec2(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
    if (any(uv < vec2(0.0)) || any(uv > vec2(1.0)) || sp.z > 1.0) { return 1.0; }
    let range_m = frame.shadow.y;
    let z = sp.z;
    let rot = noise * 2.0 * PI;
    // Blocker search within 0.6 m.
    let search_uv = 0.6 / size;
    var blockers = 0.0;
    var bsum = 0.0;
    for (var i = 0u; i < 12u; i++) {
        let o = vogel(i, 12u, rot) * search_uv;
        let c = clamp(vec2<i32>((uv + o) * dims), vec2(0), vec2<i32>(dims) - 1);
        let d = textureLoad(t_shadow, c, 0);
        if (d < z - 0.0005) { blockers += 1.0; bsum += d; }
    }
    if (blockers < 0.5) { return 1.0; }
    let avg = bsum / blockers;
    let dist_m = (z - avg) * range_m;
    // Penumbra grows with blocker distance (sun angular radius scaled by artistic softness).
    let pen_m = clamp(dist_m * tan(frame.sun_dir.w) * frame.shadow.z, texel * 3.0, 1.2);
    let filt_uv = pen_m / size;
    var lit = 0.0;
    for (var i = 0u; i < 16u; i++) {
        let o = vogel(i, 16u, rot + 1.3) * filt_uv;
        lit += textureSampleCompareLevel(t_shadow, s_shadow, uv + o, z - 0.0003);
    }
    return lit / 16.0;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let res = vec2<u32>(frame.resolution.xy);
    if (any(gid.xy >= res)) { return; }
    let px = vec2<i32>(gid.xy);
    let uv = (vec2<f32>(gid.xy) + 0.5) * frame.resolution.zw;
    let depth = textureLoad(t_depth, px, 0);
    let sun = frame.sun_dir.xyz;
    let sun_rad = sky[9].rgb * frame.sun_color.rgb * frame.sun_color.w;

    if (depth <= 0.0) {
        // Sky (reversed Z: far plane at 0).
        let far = world_from_depth(uv, 1e-7, frame.inv_view_proj);
        let dir = normalize(far - frame.camera_pos.xyz);
        var c = sky_radiance(dir, sun) * frame.params.w * frame.sun_color.w * frame.sky_tint.rgb;
        let cs = dot(dir, sun);
        let disk = smoothstep(cos(frame.sun_dir.w * 1.6), cos(frame.sun_dir.w), cs);
        c += sun_rad * disk * 40.0;
        textureStore(out_hdr, px, vec4(c, 1.0));
        return;
    }

    let world = world_from_depth(uv, depth, frame.inv_view_proj);
    let a = textureLoad(t_albedo, px, 0);
    let nt = textureLoad(t_normal, px, 0);
    let n = normalize(nt.xyz);
    let albedo = a.rgb;
    let baked_ao = a.a;
    let translucency = nt.a;
    let noise = fract(ign(vec2<f32>(gid.xy)) + frame.jitter.z * 0.618034);

    let ndl = dot(n, sun);
    let shadow = sun_shadow(world, n, noise);
    let direct = sun_rad / PI * max(ndl, 0.0) * shadow;
    let back = sun_rad / PI * max(-ndl, 0.0) * shadow * translucency * 0.8;

    let ao_s = textureLoad(t_ao, px, 0);
    let ssao = mix(1.0, ao_s.r, frame.params.z);
    let bent = normalize(mix(n, ao_s.gba, 0.5));
    var indirect: vec3<f32>;
    if (frame.params.y > 0.0) {
        indirect = gi_sample(world, n, bent, frame) * frame.params.y;
    } else {
        var sh: array<vec4<f32>, 9>;
        for (var k = 0u; k < 9u; k++) { sh[k] = sky[k]; }
        indirect = sh9_irradiance(sh, bent) * frame.params.w * frame.sun_color.w * frame.ambient_tint.rgb;
    }
    let occl = ssao * baked_ao;
    // Warm sun / cool shade: ambient reaching surfaces the sun does not light is cooled (stylised, Tiny Glade).
    let sunlit = shadow * smoothstep(0.0, 0.25, ndl);
    indirect *= mix(frame.shade_tint.rgb, vec3(1.0), sunlit);
    var color = albedo * (direct + back + indirect * occl);

    // Aerial perspective + diorama edge fade into a soft painted backdrop (MASTER_SPEC Phase 1).
    if (frame.debug.x == 0u) {
        let to_cam = world - frame.camera_pos.xyz;
        let dist = length(to_cam);
        let vdir = to_cam / max(dist, 1e-3);
        var shh: array<vec4<f32>, 9>;
        for (var k = 0u; k < 9u; k++) { shh[k] = sky[k]; }
        let hd = normalize(vec3(vdir.x, 0.25, vdir.z));
        // Haze = sky light + sun in-scattering (isotropic part + forward Mie lobe): luminous and warm.
        let glow = 0.35 + pow(max(dot(vdir, sun), 0.0), 6.0) * 0.8;
        let haze_col = (sh9_irradiance(shh, hd) * frame.sun_color.w * frame.params.w * 0.6 + sun_rad * glow / PI)
            * frame.sky_tint.rgb;
        let edge_r = length(world.xz - frame.haze.zw);
        let edge = smoothstep(frame.haze.y * 0.55, frame.haze.y, edge_r);
        let fog = 1.0 - exp(-max(dist - 30.0, 0.0) * frame.haze.x);
        color = mix(color, haze_col, clamp(max(fog, edge * 0.85), 0.0, 1.0));
    }

    if (frame.debug.x == 1u) { color = indirect * occl; }
    else if (frame.debug.x == 2u) { color = vec3(occl); }
    else if (frame.debug.x == 3u) { color = vec3(shadow); }
    else if (frame.debug.x == 4u) { color = n * 0.5 + 0.5; }
    textureStore(out_hdr, px, vec4(color, 1.0));
}
