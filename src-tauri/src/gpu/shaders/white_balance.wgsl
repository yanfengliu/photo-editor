// White balance using pre-computed Planckian chromaticity multipliers.
// The Rust backend computes (red_scale, green_scale, blue_scale) from the
// Hernandez-Andres (1999) CIE xy approximation of the Planckian locus,
// then passes them as uniforms to avoid duplicating the polynomial in WGSL.

struct Params {
    red_scale: f32,
    green_scale: f32,
    blue_scale: f32,
    _pad0: f32,
}

@group(0) @binding(0) var input_tex: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    var pixel = textureLoad(input_tex, coord);

    pixel.r *= params.red_scale;
    pixel.g *= params.green_scale;
    pixel.b *= params.blue_scale;

    pixel = clamp(pixel, vec4<f32>(0.0), vec4<f32>(1.0));
    textureStore(output_tex, coord, pixel);
}
