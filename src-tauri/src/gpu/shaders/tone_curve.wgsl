// Tone curve via 256-entry LUT (uploaded from CPU-side spline interpolation)

@group(0) @binding(0) var input_tex: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<storage, read> lut_rgb: array<f32, 256>;
@group(0) @binding(3) var<storage, read> lut_r: array<f32, 256>;
@group(0) @binding(4) var<storage, read> lut_g: array<f32, 256>;
@group(0) @binding(5) var<storage, read> lut_b: array<f32, 256>;

fn sample_lut(lut: ptr<storage, array<f32, 256>, read>, val: f32) -> f32 {
    let idx_f = clamp(val, 0.0, 1.0) * 255.0;
    let lo = u32(floor(idx_f));
    let hi = min(lo + 1u, 255u);
    let frac = idx_f - floor(idx_f);
    return mix((*lut)[lo], (*lut)[hi], frac);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    var pixel = textureLoad(input_tex, coord);

    // Apply per-channel curves, then master RGB curve
    pixel.r = sample_lut(&lut_r, pixel.r);
    pixel.g = sample_lut(&lut_g, pixel.g);
    pixel.b = sample_lut(&lut_b, pixel.b);

    pixel.r = sample_lut(&lut_rgb, pixel.r);
    pixel.g = sample_lut(&lut_rgb, pixel.g);
    pixel.b = sample_lut(&lut_rgb, pixel.b);

    textureStore(output_tex, coord, pixel);
}
