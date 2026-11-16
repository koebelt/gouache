// Diffuse GI probe grid: L1 SH irradiance per probe (cosine-convolved, divided by pi).
// sh[0..3].rgb = coefficients (Y00, Y1-1(y), Y10(z), Y11(x)); sh[0].w = validity (0 inside geometry).

struct Probe {
    sh: array<vec4<f32>, 4>,
};

fn probe_index(c: vec3<u32>, dims: vec3<u32>) -> u32 {
    return c.x + dims.x * (c.y + dims.y * c.z);
}

fn probe_pos(c: vec3<u32>, f: Frame) -> vec3<f32> {
    return f.gi_origin.xyz + vec3<f32>(c) * f.gi_origin.w;
}

fn sh1_eval(p: Probe, n: vec3<f32>) -> vec3<f32> {
    let r = p.sh[0].rgb * 0.282095
        + (p.sh[1].rgb * n.y + p.sh[2].rgb * n.z + p.sh[3].rgb * n.x) * 0.488603;
    return max(r, vec3(0.0));
}

// Trilinear probe interpolation with normal-facing and validity weights (DDGI-style, no depth moments yet).
fn gi_sample(world: vec3<f32>, n: vec3<f32>, bent: vec3<f32>, f: Frame) -> vec3<f32> {
    let dims = f.gi_dims.xyz;
    let sp = f.gi_origin.w;
    // Bias the lookup off the surface to reduce leaking through thin walls.
    let p = world + n * (0.35 * sp);
    let g = (p - f.gi_origin.xyz) / sp;
    let base = clamp(vec3<i32>(floor(g)), vec3(0), vec3<i32>(dims) - 2);
    let fr = clamp(g - vec3<f32>(base), vec3(0.0), vec3(1.0));
    var sum = vec3(0.0);
    var wsum = 0.0;
    for (var i = 0u; i < 8u; i++) {
        let o = vec3<u32>(i & 1u, (i >> 1u) & 1u, (i >> 2u) & 1u);
        let c = vec3<u32>(base) + o;
        let pr = probes[probe_index(c, dims)];
        let tri = select(1.0 - fr, fr, vec3<f32>(o) > vec3(0.5));
        var w = tri.x * tri.y * tri.z;
        let to_probe = normalize(probe_pos(c, f) - world + vec3(1e-4));
        let facing = (dot(to_probe, n) + 1.0) * 0.5;
        w *= facing * facing + 0.05;
        w *= max(pr.sh[0].w, 0.0);
        sum += sh1_eval(pr, bent) * w;
        wsum += w;
    }
    return sum / max(wsum, 1e-4);
}
