// Film grain effect

struct Params {
    amount: f32,
    size: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var input_tex: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: Params;

// Simple hash function for pseudo-random noise
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    var pixel = textureLoad(input_tex, coord);

    if (params.amount < 0.01) {
        textureStore(output_tex, coord, pixel);
        return;
    }

    // Scale coordinates by grain size
    let grain_scale = max(params.size / 25.0, 0.5);
    let grain_coord = vec2<f32>(f32(gid.x), f32(gid.y)) / grain_scale;

    // Generate noise
    let noise = hash(grain_coord) * 2.0 - 1.0;
    let strength = params.amount / 200.0;

    // Apply grain (luminance-weighted: more grain in midtones)
    let lum = dot(pixel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let midtone_weight = 1.0 - abs(lum - 0.5) * 2.0;
    let grain = noise * strength * (0.5 + midtone_weight * 0.5);

    pixel = vec4<f32>(pixel.rgb + grain, pixel.a);
    pixel = clamp(pixel, vec4<f32>(0.0), vec4<f32>(1.0));

    textureStore(output_tex, coord, pixel);
}
