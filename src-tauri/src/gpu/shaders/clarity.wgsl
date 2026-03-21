// Clarity: large-radius local contrast enhancement with Gaussian blur and halo control

struct Params {
    clarity: f32,
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

    if (abs(params.clarity) < 0.01) {
        textureStore(output_tex, coord, center);
        return;
    }

    // Gaussian blur with sigma=10, sampled every 3 pixels for performance
    // Effective radius ~20px (2*sigma), covers the local contrast range
    let sigma = 10.0;
    let sigma_sq = sigma * sigma;
    let radius = 20;
    let step = 3;
    var blur = vec3<f32>(0.0);
    var wt_sum = 0.0;
    for (var dy = -radius; dy <= radius; dy += step) {
        for (var dx = -radius; dx <= radius; dx += step) {
            let d_sq = f32(dx * dx + dy * dy);
            let wt = exp(-d_sq / (2.0 * sigma_sq));
            blur += sample_clamped(coord + vec2<i32>(dx, dy), dims).rgb * wt;
            wt_sum += wt;
        }
    }
    blur /= wt_sum;

    // Detail (high-frequency component)
    let detail = center.rgb - blur;

    // Halo suppression: clamp detail magnitude to prevent ringing near edges
    let halo_limit = 0.15;
    let clamped_detail = clamp(detail, vec3<f32>(-halo_limit), vec3<f32>(halo_limit));

    // Midtone-weighted application: clarity affects midtones more than extremes
    let lum = dot(center.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let midtone_w = 1.0 - 2.0 * abs(lum - 0.5);
    let effective_strength = (params.clarity / 100.0) * (0.5 + midtone_w * 0.5);

    let result = center.rgb + clamped_detail * effective_strength;
    textureStore(output_tex, coord, vec4<f32>(clamp(result, vec3<f32>(0.0), vec3<f32>(1.0)), center.a));
}
