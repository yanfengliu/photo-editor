// Dehaze using simplified dark channel prior

struct Params {
    dehaze: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var input_tex: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: Params;

fn sample_clamped(coord: vec2<i32>, dims: vec2<u32>) -> vec4<f32> {
    let c = clamp(coord, vec2<i32>(0), vec2<i32>(dims) - 1);
    return textureLoad(input_tex, c);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let center = textureLoad(input_tex, coord);

    if (abs(params.dehaze) < 0.01) {
        textureStore(output_tex, coord, center);
        return;
    }

    // Estimate local dark channel (minimum of local neighborhood)
    let radius = 4;
    var dark_channel = 1.0f;
    for (var dy = -radius; dy <= radius; dy += 2) {
        for (var dx = -radius; dx <= radius; dx += 2) {
            let s = sample_clamped(coord + vec2<i32>(dx, dy), dims);
            dark_channel = min(dark_channel, min(s.r, min(s.g, s.b)));
        }
    }

    // Atmospheric light estimate (simplified: assume white)
    let atm = 1.0;

    // Transmission estimate
    let omega = params.dehaze / 100.0;
    let transmission = max(1.0 - omega * dark_channel / atm, 0.1);

    // Recover scene radiance
    let result = (center.rgb - atm * (1.0 - transmission)) / transmission;

    textureStore(output_tex, coord, vec4<f32>(clamp(result, vec3<f32>(0.0), vec3<f32>(1.0)), center.a));
}
