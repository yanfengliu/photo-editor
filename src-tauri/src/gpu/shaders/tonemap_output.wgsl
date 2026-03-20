// Final output pass: linear to sRGB conversion

@group(0) @binding(0) var input_tex: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;

fn linear_to_srgb(c: f32) -> f32 {
    if (c <= 0.0031308) {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let pixel = textureLoad(input_tex, coord);

    let srgb = vec4<f32>(
        linear_to_srgb(pixel.r),
        linear_to_srgb(pixel.g),
        linear_to_srgb(pixel.b),
        pixel.a,
    );

    textureStore(output_tex, coord, clamp(srgb, vec4<f32>(0.0), vec4<f32>(1.0)));
}
