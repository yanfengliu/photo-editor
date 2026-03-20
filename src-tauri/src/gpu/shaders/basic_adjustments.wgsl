struct Params {
    temperature: f32,
    tint: f32,
    exposure: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    saturation: f32,
    vibrance: f32,
    dehaze: f32,
    _pad0: f32,
}

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    var pixel = textureLoad(input_tex, coord, 0);

    // White balance (temperature + tint)
    let temp_shift = (params.temperature - 6500.0) / 6500.0;
    pixel.r *= 1.0 + temp_shift * 0.1;
    pixel.b *= 1.0 - temp_shift * 0.1;
    pixel.g *= 1.0 + params.tint / 150.0 * 0.05;

    // Exposure (EV stops)
    let exp_mult = pow(2.0, params.exposure);
    pixel = vec4<f32>(pixel.rgb * exp_mult, pixel.a);

    // Contrast (pivot at 0.5)
    let cf = 1.0 + params.contrast / 100.0;
    pixel = vec4<f32>((pixel.rgb - 0.5) * cf + 0.5, pixel.a);

    // Luminance for tone region adjustments
    let lum = dot(pixel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

    // Smooth luminance masks using power curves (no hard thresholds)
    // Highlights: concentrated on bright areas, smooth rolloff
    let hl_mask = lum * lum;
    // Shadows: concentrated on dark areas, smooth rolloff
    let sh_mask = (1.0 - lum) * (1.0 - lum);
    // Whites: tighter focus on brightest values
    let w_mask = hl_mask * hl_mask;
    // Blacks: tighter focus on darkest values
    let b_mask = sh_mask * sh_mask;

    // Accumulate luminance shift from all tone regions
    let lum_shift = hl_mask * params.highlights / 200.0
                  + sh_mask * params.shadows / 200.0
                  + w_mask * params.whites / 200.0
                  + b_mask * params.blacks / 200.0;

    // Apply luminance shift as a ratio to preserve color
    if (abs(lum_shift) > 0.0001) {
        let target_lum = clamp(lum + lum_shift, 0.0, 1.5);
        let ratio = select(target_lum / lum, 1.0 + lum_shift, lum < 0.001);
        pixel = vec4<f32>(pixel.rgb * ratio, pixel.a);
    }

    // Saturation
    let gray_s = dot(pixel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let sat_f = 1.0 + params.saturation / 100.0;
    pixel = vec4<f32>(mix(vec3<f32>(gray_s), pixel.rgb, sat_f), pixel.a);

    // Vibrance (boost less saturated colors more) — recalculate gray after saturation
    let gray_v = dot(pixel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let max_c = max(pixel.r, max(pixel.g, pixel.b));
    let min_c = min(pixel.r, min(pixel.g, pixel.b));
    let cur_sat = select(0.0, (max_c - min_c) / max_c, max_c > 0.0);
    let vib_f = 1.0 + params.vibrance / 100.0 * (1.0 - cur_sat);
    pixel = vec4<f32>(mix(vec3<f32>(gray_v), pixel.rgb, vib_f), pixel.a);

    // Dehaze
    if (abs(params.dehaze) > 0.01) {
        let atmosphere = min(pixel.r, min(pixel.g, pixel.b));
        let strength = params.dehaze / 100.0;
        pixel = vec4<f32>(pixel.rgb + (pixel.rgb - atmosphere) * strength, pixel.a);
    }

    // Clamp
    pixel = clamp(pixel, vec4<f32>(0.0), vec4<f32>(1.0));

    textureStore(output_tex, coord, pixel);
}
