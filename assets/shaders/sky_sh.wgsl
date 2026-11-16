// Projects sky radiance into cosine-convolved L2 SH and computes sun color.
// One workgroup of 64 threads, 32 directions each (9 KB shared memory). Output: 9 SH coefficients + sun color.
#import "common.wgsl"
#import "atmosphere.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read_write> sky_out: array<vec4<f32>, 10>;

var<workgroup> partial: array<array<vec3<f32>, 9>, 64>;

@compute @workgroup_size(64)
fn main(@builtin(local_invocation_index) li: u32) {
    let sun = frame.sun_dir.xyz;
    var acc: array<vec3<f32>, 9>;
    for (var k = 0u; k < 9u; k++) { acc[k] = vec3(0.0); }
    let per = 32u;
    for (var s = 0u; s < per; s++) {
        let idx = li * per + s;
        let d = sphere_dir(r2(idx));
        let l = sky_radiance(d, sun);
        acc[0] += l * 0.282095;
        acc[1] += l * 0.488603 * d.y;
        acc[2] += l * 0.488603 * d.z;
        acc[3] += l * 0.488603 * d.x;
        acc[4] += l * 1.092548 * d.x * d.y;
        acc[5] += l * 1.092548 * d.y * d.z;
        acc[6] += l * 0.315392 * (3.0 * d.z * d.z - 1.0);
        acc[7] += l * 1.092548 * d.x * d.z;
        acc[8] += l * 0.546274 * (d.x * d.x - d.y * d.y);
    }
    partial[li] = acc;
    workgroupBarrier();
    var stride = 32u;
    loop {
        if (stride == 0u) { break; }
        if (li < stride) {
            for (var k = 0u; k < 9u; k++) {
                partial[li][k] += partial[li + stride][k];
            }
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }
    if (li == 0u) {
        let n = f32(64u * per);
        let w = 4.0 * PI / n;
        // Cosine-lobe convolution: band 0 * pi, band 1 * 2pi/3, band 2 * pi/4; then /pi for radiance->irradiance/pi.
        let band = array<f32, 9>(1.0, 2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 0.25, 0.25, 0.25, 0.25, 0.25);
        for (var k = 0u; k < 9u; k++) {
            sky_out[k] = vec4(partial[0][k] * w * band[k], 0.0);
        }
        sky_out[9] = vec4(sun_transmittance(sun), 0.0);
    }
}
