// Temporal accumulation (TAA core). Reprojects history with the previous view-projection,
// clamps it to the current 3x3 neighbourhood in YCoCg, then blends.
// For still headless shots (debug.y = accumulated frame count) it converges to a true average.
#import "common.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var t_cur: texture_2d<f32>;
@group(0) @binding(2) var t_hist: texture_2d<f32>;
@group(0) @binding(3) var t_depth: texture_depth_2d;
@group(0) @binding(4) var s_lin: sampler;
@group(0) @binding(5) var out_hist: texture_storage_2d<rgba16float, write>;

fn to_ycocg(c: vec3<f32>) -> vec3<f32> {
    return vec3(c.x * 0.25 + c.y * 0.5 + c.z * 0.25, c.x * 0.5 - c.z * 0.5, -c.x * 0.25 + c.y * 0.5 - c.z * 0.25);
}

fn from_ycocg(c: vec3<f32>) -> vec3<f32> {
    return vec3(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let res = vec2<u32>(frame.resolution.xy);
    if (any(gid.xy >= res)) { return; }
    let px = vec2<i32>(gid.xy);
    let cur = textureLoad(t_cur, px, 0).rgb;
    let n = frame.debug.y;
    if (n == 0u) { textureStore(out_hist, px, vec4(cur, 1.0)); return; }

    let uv = (vec2<f32>(gid.xy) + 0.5) * frame.resolution.zw;
    let depth = textureLoad(t_depth, px, 0);
    let world = world_from_depth(uv, max(depth, 1e-7), frame.inv_view_proj);
    let pc = frame.prev_view_proj * vec4(world, 1.0);
    let puv = vec2(pc.x / pc.w * 0.5 + 0.5, 0.5 - pc.y / pc.w * 0.5);
    let still_cam = frame.debug.z == 1u;
    // Still camera: no reprojection (re-sampling every frame would blur cumulatively).
    let hist = select(textureSampleLevel(t_hist, s_lin, puv, 0.0).rgb, textureLoad(t_hist, px, 0).rgb, still_cam);

    // Neighbourhood clamp (variance clipping).
    var m1 = vec3(0.0);
    var m2 = vec3(0.0);
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let q = clamp(px + vec2(x, y), vec2(0), vec2<i32>(res) - 1);
            let c = to_ycocg(textureLoad(t_cur, q, 0).rgb);
            m1 += c;
            m2 += c * c;
        }
    }
    m1 /= 9.0;
    let sigma = sqrt(max(m2 / 9.0 - m1 * m1, vec3(0.0)));
    // Still camera accumulates freely (true average); moving camera clamps tightly.
    let still = still_cam;
    let k = select(1.25, 64.0, still);
    let hc = clamp(to_ycocg(hist), m1 - k * sigma, m1 + k * sigma);
    let off = !still && (any(puv < vec2(0.0)) || any(puv > vec2(1.0)));
    var alpha = select(0.1, 1.0 / f32(n + 1u), still);
    if (off) { alpha = 1.0; }
    let outc = mix(from_ycocg(hc), cur, alpha);
    textureStore(out_hist, px, vec4(outc, 1.0));
}
