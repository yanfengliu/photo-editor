import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, within } from "@testing-library/react";
import { useDevelopStore } from "../../stores/developStore";
import { useUiStore } from "../../stores/uiStore";
import { DEFAULT_EDIT_PARAMS } from "../../types/develop";
import { BasicAdjustments } from "../../components/develop/panels/BasicAdjustments";
import { WhiteBalance } from "../../components/develop/panels/WhiteBalance";
import { DevelopView } from "../../components/develop/DevelopView";

const mockDefaults = vi.hoisted(() => ({
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
}));

vi.mock("../../api/processing", () => ({
  getEditParams: vi.fn().mockResolvedValue({ ...mockDefaults }),
  applyEdits: vi.fn().mockResolvedValue({ data: new Uint8Array([0, 0, 0, 255]), width: 1, height: 1 }),
  saveEditParams: vi.fn().mockResolvedValue(undefined),
  resetEdits: vi.fn().mockResolvedValue({ ...mockDefaults }),
  copyEdits: vi.fn().mockResolvedValue(undefined),
  pasteEdits: vi.fn().mockResolvedValue({ ...mockDefaults }),
  getLensProfiles: vi.fn().mockResolvedValue([]),
  detectLensProfile: vi.fn().mockResolvedValue(null),
}));

describe("BasicAdjustments panel", () => {
  beforeEach(() => {
    useDevelopStore.setState({
      currentImageId: "img-1",
      editParams: { ...DEFAULT_EDIT_PARAMS },
      undoStack: [],
      redoStack: [],
      isProcessing: false,
      isAdjusting: false,
    });
  });

  it("renders all basic adjustment sliders", () => {
    render(<BasicAdjustments />);
    expect(screen.getByText("Exposure")).toBeTruthy();
    expect(screen.getByText("Contrast")).toBeTruthy();
    expect(screen.getByText("Highlights")).toBeTruthy();
    expect(screen.getByText("Shadows")).toBeTruthy();
    expect(screen.getByText("Whites")).toBeTruthy();
    expect(screen.getByText("Blacks")).toBeTruthy();
    expect(screen.getByText("Saturation")).toBeTruthy();
    expect(screen.getByText("Vibrance")).toBeTruthy();
  });

  it("dragging exposure slider updates store and builds undo stack", () => {
    render(<BasicAdjustments />);
    const sliders = screen.getAllByRole("slider");
    // First slider is exposure
    const exposureSlider = sliders[0];

    fireEvent.change(exposureSlider, { target: { value: "2.5" } });
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.5);
    expect(useDevelopStore.getState().undoStack).toHaveLength(1);
    expect(useDevelopStore.getState().undoStack[0].exposure).toBe(0);
  });

  it("dragging highlights slider updates store", () => {
    render(<BasicAdjustments />);
    const sliders = screen.getAllByRole("slider");
    // highlights is index 2 (exposure, contrast, highlights)
    const highlightsSlider = sliders[2];

    fireEvent.change(highlightsSlider, { target: { value: "-50" } });
    expect(useDevelopStore.getState().editParams.highlights).toBe(-50);
  });

  it("multiple slider changes can be undone sequentially", () => {
    render(<BasicAdjustments />);
    const sliders = screen.getAllByRole("slider");

    // Change exposure
    fireEvent.change(sliders[0], { target: { value: "1.0" } });
    // Change contrast
    fireEvent.change(sliders[1], { target: { value: "30" } });
    // Change shadows
    fireEvent.change(sliders[3], { target: { value: "25" } });

    expect(useDevelopStore.getState().undoStack).toHaveLength(3);
    expect(useDevelopStore.getState().editParams.shadows).toBe(25);

    // Undo shadows
    useDevelopStore.getState().undo();
    expect(useDevelopStore.getState().editParams.shadows).toBe(0);
    expect(useDevelopStore.getState().editParams.contrast).toBe(30);

    // Undo contrast
    useDevelopStore.getState().undo();
    expect(useDevelopStore.getState().editParams.contrast).toBe(0);
    expect(useDevelopStore.getState().editParams.exposure).toBe(1.0);

    // Redo contrast
    useDevelopStore.getState().redo();
    expect(useDevelopStore.getState().editParams.contrast).toBe(30);
  });

  it("double-clicking slider resets to default value", () => {
    useDevelopStore.setState({
      editParams: { ...DEFAULT_EDIT_PARAMS, exposure: 3.0 },
    });
    render(<BasicAdjustments />);
    const sliders = screen.getAllByRole("slider");

    fireEvent.doubleClick(sliders[0]);
    expect(useDevelopStore.getState().editParams.exposure).toBe(0);
  });
});

describe("WhiteBalance panel", () => {
  beforeEach(() => {
    useDevelopStore.setState({
      currentImageId: "img-1",
      editParams: { ...DEFAULT_EDIT_PARAMS },
      undoStack: [],
      redoStack: [],
    });
  });

  it("renders temperature and tint sliders", () => {
    render(<WhiteBalance />);
    expect(screen.getByText("Temperature")).toBeTruthy();
    expect(screen.getByText("Tint")).toBeTruthy();
  });

  it("adjusting temperature updates store", () => {
    render(<WhiteBalance />);
    const sliders = screen.getAllByRole("slider");
    // Temperature is first
    fireEvent.change(sliders[0], { target: { value: "4000" } });
    expect(useDevelopStore.getState().editParams.temperature).toBe(4000);
  });
});

describe("DevelopView", () => {
  beforeEach(() => {
    useDevelopStore.setState({
      currentImageId: null,
      editParams: { ...DEFAULT_EDIT_PARAMS },
      undoStack: [],
      redoStack: [],
      isProcessing: false,
      isAdjusting: false,
      previewData: null,
      previewWidth: 0,
      previewHeight: 0,
    });
    useUiStore.setState({
      selectedImageId: null,
      rightPanelOpen: true,
    });
  });

  it("shows empty state when no image selected", () => {
    render(<DevelopView />);
    expect(screen.getByText("Select an image to edit")).toBeTruthy();
  });

  it("shows canvas and panels when image selected", () => {
    useUiStore.setState({ selectedImageId: "img-1" });
    useDevelopStore.setState({ currentImageId: "img-1" });
    render(<DevelopView />);
    // Canvas container should exist (even if empty preview)
    expect(screen.queryByText("Select an image to edit")).toBeNull();
    // Adjustment panels should be visible
    expect(screen.getByText("Exposure")).toBeTruthy();
    expect(screen.getByText("Temperature")).toBeTruthy();
  });

  it("hides right panel when toggled off", () => {
    useUiStore.setState({ selectedImageId: "img-1", rightPanelOpen: false });
    useDevelopStore.setState({ currentImageId: "img-1" });
    render(<DevelopView />);
    expect(screen.queryByText("Exposure")).toBeNull();
  });
});
