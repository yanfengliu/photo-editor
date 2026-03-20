struct Params {
    temperature: f32,
    tint: f32,
    _pad0: f32,
    _pad1: f32,
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

    // Temperature: shift red/blue balance
    let temp_shift = (params.temperature - 6500.0) / 6500.0;
    pixel.r *= 1.0 + temp_shift * 0.1;
    pixel.b *= 1.0 - temp_shift * 0.1;

    // Tint: shift green/magenta balance
    let tint_shift = params.tint / 150.0;
    pixel.g *= 1.0 + tint_shift * 0.05;

    pixel = clamp(pixel, vec4<f32>(0.0), vec4<f32>(1.0));
    textureStore(output_tex, coord, pixel);
}
