struct Params {
    wb_red_scale: f32,
    wb_green_scale: f32,
    wb_blue_scale: f32,
    exposure: f32,
    contrast: f32,
    highlights: f32,
    shadows: f32,
    whites: f32,
    blacks: f32,
    saturation: f32,
    vibrance: f32,
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

    // White balance (pre-computed Planckian chromaticity multipliers)
    pixel.r *= params.wb_red_scale;
    pixel.g *= params.wb_green_scale;
    pixel.b *= params.wb_blue_scale;

    // Exposure (EV stops)
    let exp_mult = pow(2.0, params.exposure);
    pixel = vec4<f32>(pixel.rgb * exp_mult, pixel.a);

    // Contrast — logistic sigmoid (ImageMagick sigmoidal-contrast)
    // S(x) = (sig(β·(x-0.5)) - sig(-0.5·β)) / (sig(0.5·β) - sig(-0.5·β))
    let c_beta = params.contrast / 100.0 * 10.0;
    if (abs(c_beta) > 0.01) {
        let c_alpha = 0.5;
        let s0 = 1.0 / (1.0 + exp(c_beta * c_alpha));
        let s1 = 1.0 / (1.0 + exp(-c_beta * (1.0 - c_alpha)));
        let sx = 1.0 / (vec3<f32>(1.0) + exp(-c_beta * (pixel.rgb - vec3<f32>(c_alpha))));
        pixel = vec4<f32>((sx - s0) / (s1 - s0), pixel.a);
    }

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

    // Vibrance — selective saturation with skin-tone protection
    let gray_v = dot(pixel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let max_c = max(pixel.r, max(pixel.g, pixel.b));
    let min_c = min(pixel.r, min(pixel.g, pixel.b));
    let cur_sat = select(0.0, (max_c - min_c) / max_c, max_c > 0.0);

    // Skin-tone protection: reduce boost for warm hues (R > G > B)
    var skin_weight = 0.0;
    if (max_c > 0.01 && pixel.r > pixel.g && pixel.g > pixel.b) {
        let rg_ratio = (pixel.r - pixel.g) / max_c;
        let gb_ratio = (pixel.g - pixel.b) / max_c;
        skin_weight = max(1.0 - rg_ratio * 2.0, 0.0) * min(gb_ratio * 3.0, 1.0);
    }
    let protection = 1.0 - skin_weight * 0.7;

    let vib_f = 1.0 + params.vibrance / 100.0 * (1.0 - cur_sat) * protection;
    pixel = vec4<f32>(mix(vec3<f32>(gray_v), pixel.rgb, vib_f), pixel.a);

    // Clamp
    pixel = clamp(pixel, vec4<f32>(0.0), vec4<f32>(1.0));

    textureStore(output_tex, coord, pixel);
}
