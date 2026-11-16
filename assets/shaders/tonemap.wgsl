// Exposure + AgX filmic tonemap (Wrensch's polynomial fit) + gentle grade + vignette + dither.
#import "common.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var t_hdr: texture_2d<f32>;
@group(0) @binding(2) var out_ldr: texture_storage_2d<rgba8unorm, write>;

fn agx_contrast(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x4 = x2 * x2;
    return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

fn agx(c: vec3<f32>) -> vec3<f32> {
    let m = mat3x3<f32>(
        vec3(0.842479062253094, 0.0423282422610123, 0.0423756549057051),
        vec3(0.0784335999999992, 0.878468636469772, 0.0784336),
        vec3(0.0792237451477643, 0.0791661274605434, 0.879142973793104));
    let min_ev = -12.47393;
    let max_ev = 4.026069;
    var v = m * c;
    v = clamp(log2(max(v, vec3(1e-10))), vec3(min_ev), vec3(max_ev));
    v = (v - min_ev) / (max_ev - min_ev);
    return agx_contrast(v);
}

fn agx_eotf(c: vec3<f32>) -> vec3<f32> {
    let inv = mat3x3<f32>(
        vec3(1.19687900512017, -0.0528968517574562, -0.0529716355144438),
        vec3(-0.0980208811401368, 1.15190312990417, -0.0980434501171241),
        vec3(-0.0990297440797205, -0.0989611768448433, 1.15107367264116));
    // AgX output is display-encoded (~gamma 2.2); return linear for our sRGB encode.
    return pow(max(inv * c, vec3(0.0)), vec3(2.2));
}

fn grade(c: vec3<f32>) -> vec3<f32> {
    // Look: contrast around mid grey + saturation (params2.y, params2.z).
    let l = luminance(c);
    var g = mix(vec3(l), c, frame.params2.z);
    g = pow(max(g, vec3(0.0)), vec3(frame.params2.y));
    return g;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let res = vec2<u32>(frame.resolution.xy);
    if (any(gid.xy >= res)) { return; }
    let px = vec2<i32>(gid.xy);
    var c = textureLoad(t_hdr, px, 0).rgb * frame.params.x;
    var o = agx_eotf(agx(c));
    o = grade(o);
    let uv = (vec2<f32>(gid.xy) + 0.5) * frame.resolution.zw;
    let v = uv - 0.5;
    o *= mix(1.0, 1.0 - dot(v, v) * 0.9, 0.5);
    var s = linear_to_srgb(clamp(o, vec3(0.0), vec3(1.0)));
    s += (ign(vec2<f32>(gid.xy)) - 0.5) / 255.0;
    textureStore(out_ldr, px, vec4(s, 1.0));
}
