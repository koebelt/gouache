// Sun shadow depth pass (orthographic, standard Z).
#import "common.wgsl"

@group(0) @binding(0) var<uniform> frame: Frame;

@vertex
fn vs(@location(0) pos: vec3<f32>) -> @builtin(position) vec4<f32> {
    return frame.sun_view_proj * vec4(pos, 1.0);
}
