import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { useDevelopStore } from "../../stores/developStore";
import { DEFAULT_EDIT_PARAMS } from "../../types/develop";
import { CropRotationPanel } from "../../components/develop/panels/CropRotationPanel";

vi.mock("../../api/processing", () => {
  const d = {
    exposure: 0, contrast: 0, highlights: 0, shadows: 0, whites: 0, blacks: 0,
    temperature: 6500, tint: 0, saturation: 0, vibrance: 0, clarity: 0, dehaze: 0,
    sharpening_amount: 0, sharpening_radius: 1.0, sharpening_detail: 25, denoise_luminance: 0, denoise_color: 0, denoise_ai: false,
    vignette_amount: 0, grain_amount: 0, grain_size: 25,
    curve_rgb: [{ x: 0, y: 0 }, { x: 1, y: 1 }], curve_r: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    curve_g: [{ x: 0, y: 0 }, { x: 1, y: 1 }], curve_b: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    hsl_hue: [0, 0, 0, 0, 0, 0, 0, 0], hsl_saturation: [0, 0, 0, 0, 0, 0, 0, 0], hsl_luminance: [0, 0, 0, 0, 0, 0, 0, 0],
    crop_x: 0, crop_y: 0, crop_width: 1, crop_height: 1, rotation: 0, rotation_fine: 0,
    enable_lens_correction: false, lens_profile_id: null, lens_distortion: 0,
    lens_ca_correction: true, lens_vignette_correction: true, lens_distortion_amount: 100,
  };
  return {
    getEditParams: vi.fn().mockResolvedValue({ ...d }),
    applyEdits: vi.fn().mockResolvedValue({ data: new Uint8Array([0, 0, 0, 255]), width: 1, height: 1 }),
    saveEditParams: vi.fn().mockResolvedValue(undefined),
    resetEdits: vi.fn().mockResolvedValue({ ...d }),
    copyEdits: vi.fn().mockResolvedValue(undefined),
    pasteEdits: vi.fn().mockResolvedValue({ ...d }),
  };
});

function renderExpanded() {
  render(<CropRotationPanel />);
  // Expand the collapsed section
  fireEvent.click(screen.getByText("Crop & Rotate"));
}

describe("CropRotationPanel", () => {
  beforeEach(() => {
    useDevelopStore.setState({
      currentImageId: "img-1",
      editParams: { ...DEFAULT_EDIT_PARAMS },
      originalParams: { ...DEFAULT_EDIT_PARAMS },
      persistedParams: { ...DEFAULT_EDIT_PARAMS },
      undoStack: [],
      redoStack: [],
      isProcessing: false,
      isAdjusting: false,
      adjustStartParams: null,
      previewData: null,
      previewWidth: 1920,
      previewHeight: 1080,
    });
  });

  it("should render rotation buttons", () => {
    renderExpanded();
    expect(screen.getByTitle("Rotate 90° counter-clockwise")).toBeTruthy();
    expect(screen.getByTitle("Rotate 90° clockwise")).toBeTruthy();
    expect(screen.getByTitle("Rotate 180°")).toBeTruthy();
  });

  it("should render aspect ratio presets", () => {
    renderExpanded();
    expect(screen.getByText("Free")).toBeTruthy();
    expect(screen.getByText("Original")).toBeTruthy();
    expect(screen.getByText("1:1")).toBeTruthy();
    expect(screen.getByText("4:3")).toBeTruthy();
    expect(screen.getByText("16:9")).toBeTruthy();
  });

  it("should rotate 90° clockwise on button click", () => {
    renderExpanded();
    fireEvent.click(screen.getByTitle("Rotate 90° clockwise"));
    expect(useDevelopStore.getState().editParams.rotation).toBe(90);
  });

  it("should rotate 90° counter-clockwise on button click", () => {
    renderExpanded();
    fireEvent.click(screen.getByTitle("Rotate 90° counter-clockwise"));
    expect(useDevelopStore.getState().editParams.rotation).toBe(270);
  });

  it("should rotate 180° on button click", () => {
    renderExpanded();
    fireEvent.click(screen.getByTitle("Rotate 180°"));
    expect(useDevelopStore.getState().editParams.rotation).toBe(180);
  });

  it("should accumulate rotation", () => {
    renderExpanded();
    fireEvent.click(screen.getByTitle("Rotate 90° clockwise"));
    fireEvent.click(screen.getByTitle("Rotate 90° clockwise"));
    expect(useDevelopStore.getState().editParams.rotation).toBe(180);
  });

  it("should wrap rotation around 360°", () => {
    renderExpanded();
    fireEvent.click(screen.getByTitle("Rotate 90° clockwise"));
    fireEvent.click(screen.getByTitle("Rotate 90° clockwise"));
    fireEvent.click(screen.getByTitle("Rotate 90° clockwise"));
    fireEvent.click(screen.getByTitle("Rotate 90° clockwise"));
    expect(useDevelopStore.getState().editParams.rotation).toBe(0);
  });

  it("should apply 1:1 aspect ratio crop", () => {
    renderExpanded();
    fireEvent.click(screen.getByText("1:1"));
    const { crop_width, crop_height } = useDevelopStore.getState().editParams;
    // Check pixel aspect ratio: (crop_width * imgW) / (crop_height * imgH) ≈ 1
    const pixelRatio = (crop_width * 1920) / (crop_height * 1080);
    expect(Math.abs(pixelRatio - 1)).toBeLessThan(0.01);
  });

  it("should apply 16:9 aspect ratio crop", () => {
    renderExpanded();
    fireEvent.click(screen.getByText("16:9"));
    const { crop_width, crop_height } = useDevelopStore.getState().editParams;
    // Check pixel aspect ratio: (crop_width * imgW) / (crop_height * imgH) ≈ 16/9
    const pixelRatio = (crop_width * 1920) / (crop_height * 1080);
    expect(Math.abs(pixelRatio - 16 / 9)).toBeLessThan(0.02);
  });

  it("should not modify crop on Free preset", () => {
    renderExpanded();
    fireEvent.click(screen.getByText("Free"));
    const { crop_x, crop_y, crop_width, crop_height } = useDevelopStore.getState().editParams;
    expect(crop_x).toBe(0);
    expect(crop_y).toBe(0);
    expect(crop_width).toBe(1);
    expect(crop_height).toBe(1);
  });

  it("should show Reset Crop button when cropped", () => {
    useDevelopStore.setState({
      editParams: { ...DEFAULT_EDIT_PARAMS, crop_x: 0.1, crop_width: 0.8 },
    });
    render(<CropRotationPanel />);
    fireEvent.click(screen.getByText("Crop & Rotate"));
    expect(screen.getByText("Reset Crop")).toBeTruthy();
  });

  it("should not show Reset Crop button when not cropped", () => {
    renderExpanded();
    expect(screen.queryByText("Reset Crop")).toBeNull();
  });

  it("should reset crop on Reset Crop click", () => {
    useDevelopStore.setState({
      editParams: { ...DEFAULT_EDIT_PARAMS, crop_x: 0.1, crop_y: 0.1, crop_width: 0.5, crop_height: 0.5 },
    });
    render(<CropRotationPanel />);
    fireEvent.click(screen.getByText("Crop & Rotate"));
    fireEvent.click(screen.getByText("Reset Crop"));
    const { crop_x, crop_y, crop_width, crop_height } = useDevelopStore.getState().editParams;
    expect(crop_x).toBe(0);
    expect(crop_y).toBe(0);
    expect(crop_width).toBe(1);
    expect(crop_height).toBe(1);
  });

  it("should keep crop within [0,1] bounds for any aspect ratio", () => {
    renderExpanded();
    // Apply all aspect ratios and verify bounds
    for (const label of ["1:1", "4:3", "3:2", "16:9", "5:4", "7:5"]) {
      fireEvent.click(screen.getByText(label));
      const { crop_x, crop_y, crop_width, crop_height } = useDevelopStore.getState().editParams;
      expect(crop_x).toBeGreaterThanOrEqual(0);
      expect(crop_y).toBeGreaterThanOrEqual(0);
      expect(crop_x + crop_width).toBeLessThanOrEqual(1.001);
      expect(crop_y + crop_height).toBeLessThanOrEqual(1.001);
    }
  });
});
