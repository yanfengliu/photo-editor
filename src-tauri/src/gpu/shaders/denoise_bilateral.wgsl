// Bilateral filter for noise reduction (preserves edges)

struct Params {
    luminance_strength: f32,
    color_strength: f32,
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

    let strength = max(params.luminance_strength, params.color_strength);
    if (strength < 0.01) {
        textureStore(output_tex, coord, center);
        return;
    }

    let spatial_sigma = 3.0 + strength / 20.0;
    let range_sigma = 0.05 + strength / 500.0;
    let radius = i32(ceil(spatial_sigma * 2.0));

    var sum = vec3<f32>(0.0);
    var weight_sum = 0.0;

    for (var dy = -radius; dy <= radius; dy++) {
        for (var dx = -radius; dx <= radius; dx++) {
            let neighbor = sample_clamped(coord + vec2<i32>(dx, dy), dims);
            let spatial_dist = f32(dx * dx + dy * dy);
            let spatial_w = exp(-spatial_dist / (2.0 * spatial_sigma * spatial_sigma));

            let color_diff = neighbor.rgb - center.rgb;
            let color_dist = dot(color_diff, color_diff);
            let range_w = exp(-color_dist / (2.0 * range_sigma * range_sigma));

            let w = spatial_w * range_w;
            sum += neighbor.rgb * w;
            weight_sum += w;
        }
    }

    let result = sum / weight_sum;
    textureStore(output_tex, coord, vec4<f32>(result, center.a));
}
