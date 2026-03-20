// Vignette effect

struct Params {
    amount: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
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

    if (abs(params.amount) < 0.01) {
        textureStore(output_tex, coord, pixel);
        return;
    }

    // Normalized coordinates centered at (0,0)
    let uv = vec2<f32>(f32(gid.x) / f32(dims.x), f32(gid.y) / f32(dims.y)) * 2.0 - 1.0;

    // Distance from center (elliptical)
    let dist = length(uv);
    let strength = params.amount / 100.0;

    // Smooth falloff
    let vignette = 1.0 - strength * smoothstep(0.3, 1.2, dist);

    pixel = vec4<f32>(pixel.rgb * max(vignette, 0.0), pixel.a);
    textureStore(output_tex, coord, pixel);
}
