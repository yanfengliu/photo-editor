struct Params {
    exposure: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    saturation: f32,
    vibrance: f32,
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

    // Exposure (EV stops)
    let exp_mult = pow(2.0, params.exposure);
    pixel = vec4<f32>(pixel.rgb * exp_mult, pixel.a);

    // Contrast (pivot at 0.5)
    let cf = 1.0 + params.contrast / 100.0;
    pixel = vec4<f32>((pixel.rgb - 0.5) * cf + 0.5, pixel.a);

    // Luminance for highlight/shadow split
    let lum = dot(pixel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

    // Highlights (affect bright areas)
    if (lum > 0.5) {
        let hl_factor = 1.0 + params.highlights / 200.0;
        let blend = (lum - 0.5) * 2.0;
        pixel = vec4<f32>(mix(pixel.rgb, pixel.rgb * hl_factor, blend), pixel.a);
    }

    // Shadows (affect dark areas)
    if (lum <= 0.5) {
        let sh_factor = 1.0 + params.shadows / 200.0;
        let blend = (0.5 - lum) * 2.0;
        pixel = vec4<f32>(mix(pixel.rgb, pixel.rgb * sh_factor, blend), pixel.a);
    }

    // Whites
    if (lum > 0.75) {
        let w_factor = 1.0 + params.whites / 200.0;
        let blend = (lum - 0.75) * 4.0;
        pixel = vec4<f32>(pixel.rgb * (1.0 + (w_factor - 1.0) * blend), pixel.a);
    }

    // Blacks
    if (lum < 0.25) {
        let b_factor = 1.0 + params.blacks / 200.0;
        let blend = (0.25 - lum) * 4.0;
        pixel = vec4<f32>(pixel.rgb * (1.0 + (b_factor - 1.0) * blend), pixel.a);
    }

    // Saturation
    let gray = dot(pixel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let sat_f = 1.0 + params.saturation / 100.0;
    pixel = vec4<f32>(mix(vec3<f32>(gray), pixel.rgb, sat_f), pixel.a);

    // Vibrance (boost less saturated colors more)
    let max_c = max(pixel.r, max(pixel.g, pixel.b));
    let min_c = min(pixel.r, min(pixel.g, pixel.b));
    let cur_sat = select(0.0, (max_c - min_c) / max_c, max_c > 0.0);
    let vib_f = 1.0 + params.vibrance / 100.0 * (1.0 - cur_sat);
    pixel = vec4<f32>(mix(vec3<f32>(gray), pixel.rgb, vib_f), pixel.a);

    // Clamp
    pixel = clamp(pixel, vec4<f32>(0.0), vec4<f32>(1.0));

    textureStore(output_tex, coord, pixel);
}
