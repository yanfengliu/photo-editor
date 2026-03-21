export interface CurvePoint {
  x: number;
  y: number;
}

export interface EditParams {
  exposure: number;
  contrast: number;
  highlights: number;
  shadows: number;
  whites: number;
  blacks: number;
  temperature: number;
  tint: number;
  saturation: number;
  vibrance: number;
  clarity: number;
  dehaze: number;
  sharpening_amount: number;
  sharpening_radius: number;
  denoise_luminance: number;
  denoise_color: number;
  denoise_ai: boolean;
  vignette_amount: number;
  grain_amount: number;
  grain_size: number;
  curve_rgb: CurvePoint[];
  curve_r: CurvePoint[];
  curve_g: CurvePoint[];
  curve_b: CurvePoint[];
  hsl_hue: number[];
  hsl_saturation: number[];
  hsl_luminance: number[];
  // Crop & Rotation
  crop_x: number;
  crop_y: number;
  crop_width: number;
  crop_height: number;
  rotation: number;       // 0, 90, 180, 270
  rotation_fine: number;  // -45..+45
  // Lens Correction
  enable_lens_correction: boolean;
  lens_profile_id: string | null;
  lens_distortion: number;
  lens_ca_correction: boolean;
  lens_vignette_correction: boolean;
  lens_distortion_amount: number;
}

export const DEFAULT_EDIT_PARAMS: EditParams = {
  exposure: 0,
  contrast: 0,
  highlights: 0,
  shadows: 0,
  whites: 0,
  blacks: 0,
  temperature: 6500,
  tint: 0,
  saturation: 0,
  vibrance: 0,
  clarity: 0,
  dehaze: 0,
  sharpening_amount: 0,
  sharpening_radius: 1.0,
  denoise_luminance: 0,
  denoise_color: 0,
  denoise_ai: false,
  vignette_amount: 0,
  grain_amount: 0,
  grain_size: 25,
  curve_rgb: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
  curve_r: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
  curve_g: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
  curve_b: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
  hsl_hue: [0, 0, 0, 0, 0, 0, 0, 0],
  hsl_saturation: [0, 0, 0, 0, 0, 0, 0, 0],
  hsl_luminance: [0, 0, 0, 0, 0, 0, 0, 0],
  crop_x: 0,
  crop_y: 0,
  crop_width: 1,
  crop_height: 1,
  rotation: 0,
  rotation_fine: 0,
  enable_lens_correction: false,
  lens_profile_id: null,
  lens_distortion: 0,
  lens_ca_correction: true,
  lens_vignette_correction: true,
  lens_distortion_amount: 100,
};

export const HSL_CHANNELS = [
  "Red",
  "Orange",
  "Yellow",
  "Green",
  "Aqua",
  "Blue",
  "Purple",
  "Magenta",
] as const;

export type HslChannel = (typeof HSL_CHANNELS)[number];
