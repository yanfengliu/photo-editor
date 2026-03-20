// HSL adjustments: 8 color channels (Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta)

struct HslParams {
    hue: array<f32, 8>,
    saturation: array<f32, 8>,
    luminance: array<f32, 8>,
}

@group(0) @binding(0) var input_tex: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: HslParams;

fn rgb_to_hsl(c: vec3<f32>) -> vec3<f32> {
    let max_c = max(c.r, max(c.g, c.b));
    let min_c = min(c.r, min(c.g, c.b));
    let l = (max_c + min_c) * 0.5;
    if (max_c == min_c) { return vec3<f32>(0.0, 0.0, l); }
    let d = max_c - min_c;
    let s = select(d / (2.0 - max_c - min_c), d / (max_c + min_c), l < 0.5);
    var h: f32;
    if (max_c == c.r) { h = (c.g - c.b) / d + select(0.0, 6.0, c.g < c.b); }
    else if (max_c == c.g) { h = (c.b - c.r) / d + 2.0; }
    else { h = (c.r - c.g) / d + 4.0; }
    h /= 6.0;
    return vec3<f32>(h, s, l);
}

fn hue_to_rgb(p: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if (t < 0.0) { t += 1.0; }
    if (t > 1.0) { t -= 1.0; }
    if (t < 1.0/6.0) { return p + (q - p) * 6.0 * t; }
    if (t < 1.0/2.0) { return q; }
    if (t < 2.0/3.0) { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
    return p;
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    if (hsl.y == 0.0) { return vec3<f32>(hsl.z); }
    let q = select(hsl.z + hsl.y - hsl.z * hsl.y, hsl.z * (1.0 + hsl.y), hsl.z < 0.5);
    let p = 2.0 * hsl.z - q;
    return vec3<f32>(
        hue_to_rgb(p, q, hsl.x + 1.0/3.0),
        hue_to_rgb(p, q, hsl.x),
        hue_to_rgb(p, q, hsl.x - 1.0/3.0),
    );
}

fn get_channel_weight(hue: f32, channel: u32) -> f32 {
    // 8 channels evenly spaced: 0=Red, 1=Orange, 2=Yellow, 3=Green, 4=Aqua, 5=Blue, 6=Purple, 7=Magenta
    let center = f32(channel) / 8.0;
    var dist = abs(hue - center);
    dist = min(dist, 1.0 - dist); // wrap around
    let width = 1.0 / 8.0;
    return max(0.0, 1.0 - dist / width);
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    var pixel = textureLoad(input_tex, coord);

    var hsl = rgb_to_hsl(pixel.rgb);

    // Apply weighted adjustments from each channel
    for (var ch = 0u; ch < 8u; ch++) {
        let w = get_channel_weight(hsl.x, ch);
        if (w > 0.0) {
            hsl.x += params.hue[ch] / 360.0 * w;
            hsl.y *= 1.0 + params.saturation[ch] / 100.0 * w;
            hsl.z *= 1.0 + params.luminance[ch] / 100.0 * w;
        }
    }

    // Wrap hue
    hsl.x = fract(hsl.x);
    hsl.y = clamp(hsl.y, 0.0, 1.0);
    hsl.z = clamp(hsl.z, 0.0, 1.0);

    let rgb = hsl_to_rgb(hsl);
    textureStore(output_tex, coord, vec4<f32>(rgb, pixel.a));
}
