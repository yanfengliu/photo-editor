import { describe, it, expect } from "vitest";
import { DEFAULT_EDIT_PARAMS, HSL_CHANNELS } from "../../types/develop";

describe("EditParams defaults", () => {
  it("should have neutral exposure", () => {
    expect(DEFAULT_EDIT_PARAMS.exposure).toBe(0);
  });

  it("should have neutral contrast", () => {
    expect(DEFAULT_EDIT_PARAMS.contrast).toBe(0);
  });

  it("should have daylight white balance", () => {
    expect(DEFAULT_EDIT_PARAMS.temperature).toBe(6500);
  });

  it("should have neutral tint", () => {
    expect(DEFAULT_EDIT_PARAMS.tint).toBe(0);
  });

  it("should have zero saturation adjustment", () => {
    expect(DEFAULT_EDIT_PARAMS.saturation).toBe(0);
    expect(DEFAULT_EDIT_PARAMS.vibrance).toBe(0);
  });

  it("should have zero sharpening with 1.0 radius", () => {
    expect(DEFAULT_EDIT_PARAMS.sharpening_amount).toBe(0);
    expect(DEFAULT_EDIT_PARAMS.sharpening_radius).toBe(1.0);
  });

  it("should have zero denoise", () => {
    expect(DEFAULT_EDIT_PARAMS.denoise_luminance).toBe(0);
    expect(DEFAULT_EDIT_PARAMS.denoise_color).toBe(0);
    expect(DEFAULT_EDIT_PARAMS.denoise_ai).toBe(false);
  });

  it("should have linear identity curve", () => {
    expect(DEFAULT_EDIT_PARAMS.curve_rgb).toEqual([
      { x: 0, y: 0 },
      { x: 1, y: 1 },
    ]);
    expect(DEFAULT_EDIT_PARAMS.curve_r).toEqual(DEFAULT_EDIT_PARAMS.curve_rgb);
    expect(DEFAULT_EDIT_PARAMS.curve_g).toEqual(DEFAULT_EDIT_PARAMS.curve_rgb);
    expect(DEFAULT_EDIT_PARAMS.curve_b).toEqual(DEFAULT_EDIT_PARAMS.curve_rgb);
  });

  it("should have 8 HSL channels all zeroed", () => {
    expect(DEFAULT_EDIT_PARAMS.hsl_hue).toHaveLength(8);
    expect(DEFAULT_EDIT_PARAMS.hsl_saturation).toHaveLength(8);
    expect(DEFAULT_EDIT_PARAMS.hsl_luminance).toHaveLength(8);
    expect(DEFAULT_EDIT_PARAMS.hsl_hue.every((v) => v === 0)).toBe(true);
  });

  it("should have zero vignette and grain", () => {
    expect(DEFAULT_EDIT_PARAMS.vignette_amount).toBe(0);
    expect(DEFAULT_EDIT_PARAMS.grain_amount).toBe(0);
    expect(DEFAULT_EDIT_PARAMS.grain_size).toBe(25);
  });

  it("should have zero dehaze and clarity", () => {
    expect(DEFAULT_EDIT_PARAMS.dehaze).toBe(0);
    expect(DEFAULT_EDIT_PARAMS.clarity).toBe(0);
  });

  it("HSL_CHANNELS should have 8 channels", () => {
    expect(HSL_CHANNELS).toHaveLength(8);
    expect(HSL_CHANNELS[0]).toBe("Red");
    expect(HSL_CHANNELS[7]).toBe("Magenta");
  });
});
