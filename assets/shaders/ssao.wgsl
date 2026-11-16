// Screen-space AO + bent normal. Normal-oriented hemisphere samples in world space,
// projected to screen and compared with the depth buffer. Range-checked, temporally rotated.
#import "common.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var t_depth: texture_depth_2d;
@group(0) @binding(2) var t_normal: texture_2d<f32>;
@group(0) @binding(3) var out_ao: texture_storage_2d<rgba16float, write>;

const SAMPLES: u32 = 16u;

fn view_depth(world: vec3<f32>) -> f32 {
    return distance(world, frame.camera_pos.xyz);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let res = vec2<u32>(frame.resolution.xy);
    if (any(gid.xy >= res)) { return; }
    let px = vec2<i32>(gid.xy);
    let depth = textureLoad(t_depth, px, 0);
    if (depth <= 0.0) { textureStore(out_ao, px, vec4(1.0, 0.0, 1.0, 0.0)); return; }
    let uv = (vec2<f32>(gid.xy) + 0.5) * frame.resolution.zw;
    let p = world_from_depth(uv, depth, frame.inv_view_proj);
    let n = normalize(textureLoad(t_normal, px, 0).xyz);
    let dist = view_depth(p);
    // Radius: 0.9 m near the camera, grows slowly with distance so far objects keep contact AO.
    let radius = 0.9 + dist * 0.01;
    let tbn = basis(n);
    let noise = ign(vec2<f32>(gid.xy) + frame.jitter.z * 5.588238);
    var occ = 0.0;
    var bent = vec3(0.0);
    for (var i = 0u; i < SAMPLES; i++) {
        let u = fract(r2(i) + vec2(noise, hash_f(gid.x * 73856093u ^ gid.y * 19349663u ^ u32(frame.jitter.z))));
        let rz = sqrt(u.x);
        let phi = 2.0 * PI * u.y;
        let l = vec3(sqrt(1.0 - rz * rz) * cos(phi), sqrt(1.0 - rz * rz) * sin(phi), rz);
        let dir = tbn * l;
        let scale = mix(0.1, 1.0, (f32(i) + noise) / f32(SAMPLES));
        let s = p + n * 0.03 + dir * radius * scale * scale;
        let clip = frame.view_proj * vec4(s, 1.0);
        let suv = vec2(clip.x / clip.w * 0.5 + 0.5, 0.5 - clip.y / clip.w * 0.5);
        var blocked = 0.0;
        if (all(suv > vec2(0.0)) && all(suv < vec2(1.0))) {
            let sp = vec2<i32>(suv * frame.resolution.xy);
            let sd = textureLoad(t_depth, sp, 0);
            let sw = world_from_depth(suv, sd, frame.inv_view_proj);
            let scene_d = view_depth(sw);
            let sample_d = view_depth(s);
            if (scene_d < sample_d - 0.02) {
                let range = smoothstep(0.0, 1.0, radius / max(abs(dist - scene_d), 1e-3));
                blocked = range;
            }
        }
        occ += blocked;
        bent += dir * (1.0 - blocked);
    }
    let ao = 1.0 - occ / f32(SAMPLES);
    let bn = select(n, normalize(bent), length(bent) > 1e-3);
    textureStore(out_ao, px, vec4(pow(ao, 2.0), bn));
}
