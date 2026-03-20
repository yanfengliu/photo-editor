// Unsharp mask sharpening

struct Params {
    amount: f32,
    radius: f32,
    _pad0: f32,
    _pad1: f32,
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

    if (params.amount < 0.01) {
        textureStore(output_tex, coord, center);
        return;
    }

    let r = i32(ceil(params.radius));
    var blur = vec3<f32>(0.0);
    var weight_sum = 0.0;

    for (var dy = -r; dy <= r; dy++) {
        for (var dx = -r; dx <= r; dx++) {
            let d = sqrt(f32(dx * dx + dy * dy));
            if (d > params.radius) { continue; }
            let w = exp(-d * d / (2.0 * params.radius * params.radius));
            blur += sample_clamped(coord + vec2<i32>(dx, dy), dims).rgb * w;
            weight_sum += w;
        }
    }
    blur /= weight_sum;

    // Unsharp mask: original + amount * (original - blur)
    let strength = params.amount / 100.0;
    let sharp = center.rgb + (center.rgb - blur) * strength;

    textureStore(output_tex, coord, vec4<f32>(clamp(sharp, vec3<f32>(0.0), vec3<f32>(1.0)), center.a));
}
