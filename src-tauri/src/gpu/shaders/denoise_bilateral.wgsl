// Bilateral filter for noise reduction with separate luma/chroma processing.
//
// Professional noise reduction processes luminance and chrominance
// independently (Tomasi & Manduchi 1998, ICCV). The eye is less sensitive
// to color noise, so chroma channels are filtered more aggressively.

struct Params {
    luminance_strength: f32,
    color_strength: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var input_tex: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output_tex: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<uniform> params: Params;

fn sample_clamped(coord: vec2<i32>, dims: vec2<u32>) -> vec4<f32> {
    let c = clamp(coord, vec2<i32>(0), vec2<i32>(dims) - 1);
    return textureLoad(input_tex, c);
}

// BT.709 luminance
fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(input_tex);
    if (gid.x >= dims.x || gid.y >= dims.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let center = textureLoad(input_tex, coord);

    let strength = max(params.luminance_strength, params.color_strength);
    if (strength < 0.01) {
        textureStore(output_tex, coord, center);
        return;
    }

    let center_y = luma(center.rgb);
    let center_cb = center.b - center_y;
    let center_cr = center.r - center_y;

    // Luma bilateral: spatial sigma scales with strength, range sigma is tight
    let lum_spatial_sigma = 3.0 + params.luminance_strength / 20.0;
    let lum_range_sigma = 0.05 + params.luminance_strength / 500.0;
    // Chroma bilateral: wider spatial + range sigma (eye less sensitive)
    let chr_spatial_sigma = 4.0 + params.color_strength / 15.0;
    let chr_range_sigma = 0.10 + params.color_strength / 200.0;

    let radius = i32(ceil(max(lum_spatial_sigma, chr_spatial_sigma) * 2.0));

    var sum_y = 0.0;
    var sum_cb = 0.0;
    var sum_cr = 0.0;
    var wt_y = 0.0;
    var wt_c = 0.0;

    for (var dy = -radius; dy <= radius; dy++) {
        for (var dx = -radius; dx <= radius; dx++) {
            let neighbor = sample_clamped(coord + vec2<i32>(dx, dy), dims);
            let ny = luma(neighbor.rgb);
            let ncb = neighbor.b - ny;
            let ncr = neighbor.r - ny;
            let spatial_dist = f32(dx * dx + dy * dy);

            // Luma weight
            if (params.luminance_strength > 0.5) {
                let sw_l = exp(-spatial_dist / (2.0 * lum_spatial_sigma * lum_spatial_sigma));
                let yd = ny - center_y;
                let rw_l = exp(-yd * yd / (2.0 * lum_range_sigma * lum_range_sigma));
                let w_l = sw_l * rw_l;
                sum_y += ny * w_l;
                wt_y += w_l;
            }

            // Chroma weight
            if (params.color_strength > 0.5) {
                let sw_c = exp(-spatial_dist / (2.0 * chr_spatial_sigma * chr_spatial_sigma));
                let dcb = ncb - center_cb;
                let dcr = ncr - center_cr;
                let chroma_dist = dcb * dcb + dcr * dcr;
                let rw_c = exp(-chroma_dist / (2.0 * chr_range_sigma * chr_range_sigma));
                let w_c = sw_c * rw_c;
                sum_cb += ncb * w_c;
                sum_cr += ncr * w_c;
                wt_c += w_c;
            }
        }
    }

    // Reconstruct: filtered luma + filtered chroma → RGB
    var out_y = center_y;
    if (params.luminance_strength > 0.5 && wt_y > 0.0) {
        out_y = sum_y / wt_y;
    }
    var out_cb = center_cb;
    var out_cr = center_cr;
    if (params.color_strength > 0.5 && wt_c > 0.0) {
        out_cb = sum_cb / wt_c;
        out_cr = sum_cr / wt_c;
    }

    // Y + Cr = R, Y + Cb = B, derive G from Y definition
    let out_r = out_y + out_cr;
    let out_b = out_y + out_cb;
    let out_g = (out_y - 0.2126 * out_r - 0.0722 * out_b) / 0.7152;

    textureStore(output_tex, coord, vec4<f32>(
        clamp(vec3<f32>(out_r, out_g, out_b), vec3<f32>(0.0), vec3<f32>(1.0)),
        center.a
    ));
}
