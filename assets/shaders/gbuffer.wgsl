// G-buffer: albedo (sRGB target, a = baked AO) + world normal (a = translucency).
#import "common.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,   // unorm8x4: sRGB albedo + baked AO
    @location(3) params: vec4<f32>,  // unorm8x4: roughness, translucency, material, wind
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) params: vec4<f32>,
    @location(3) world: vec3<f32>,
};

@vertex
fn vs(v: VsIn) -> VsOut {
    var o: VsOut;
    var clip = frame.view_proj * vec4(v.pos, 1.0);
    clip = vec4(clip.xy + frame.jitter.xy * clip.w, clip.zw);
    o.clip = clip;
    o.normal = v.normal;
    o.color = v.color;
    o.params = v.params;
    o.world = v.pos;
    return o;
}

struct GOut {
    @location(0) albedo: vec4<f32>,
    @location(1) normal: vec4<f32>,
};

@fragment
fn fs(i: VsOut, @builtin(front_facing) ff: bool) -> GOut {
    var n = normalize(i.normal);
    if (!ff) { n = -n; }
    var o: GOut;
    // Target is *Srgb: write linear, hardware encodes.
    o.albedo = vec4(srgb_to_linear(i.color.rgb), i.color.a);
    o.normal = vec4(n, i.params.y);
    return o;
}
