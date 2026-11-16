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
    let pen_m = clamp(dist_m * tan(frame.sun_dir.w) * frame.shadow.z, texel * 1.5, 1.2);
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
        var c = sky_radiance(dir, sun) * frame.params.w * frame.sun_color.w;
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
        indirect = sh9_irradiance(sh, bent) * frame.params.w * frame.sun_color.w;
    }
    let occl = ssao * baked_ao;
    var color = albedo * (direct + back + indirect * occl);

    if (frame.debug.x == 1u) { color = indirect * occl; }
    else if (frame.debug.x == 2u) { color = vec3(occl); }
    else if (frame.debug.x == 3u) { color = vec3(shadow); }
    else if (frame.debug.x == 4u) { color = n * 0.5 + 0.5; }
    textureStore(out_hdr, px, vec4(color, 1.0));
}
