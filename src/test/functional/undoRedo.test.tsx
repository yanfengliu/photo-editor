import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, fireEvent, screen } from "@testing-library/react";
import { useDevelopStore } from "../../stores/developStore";
import { useUiStore } from "../../stores/uiStore";
import { DEFAULT_EDIT_PARAMS } from "../../types/develop";
import { useKeyboardShortcuts } from "../../hooks/useKeyboardShortcuts";
import { BasicAdjustments } from "../../components/develop/panels/BasicAdjustments";

const mockDefaults = vi.hoisted(() => ({
  exposure: 0, contrast: 0, highlights: 0, shadows: 0, whites: 0, blacks: 0,
  temperature: 6500, tint: 0, saturation: 0, vibrance: 0, clarity: 0, dehaze: 0,
  sharpening_amount: 0, sharpening_radius: 1.0, denoise_luminance: 0, denoise_color: 0, denoise_ai: false,
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
}));

vi.mock("../../api/catalog", () => ({
  setRating: vi.fn().mockResolvedValue(undefined),
  setFlag: vi.fn().mockResolvedValue(undefined),
}));

function UndoRedoHarness() {
  useKeyboardShortcuts();
  return <BasicAdjustments />;
}

describe("Undo/Redo integration with slider edits and keyboard shortcuts", () => {
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
      previewData: null,
      previewWidth: 0,
      previewHeight: 0,
    });
    useUiStore.setState({ viewMode: "develop", selectedImageId: "img-1" });
  });

  it("full undo/redo cycle: edit -> Ctrl+Z -> Ctrl+Shift+Z", () => {
    render(<UndoRedoHarness />);
    const sliders = screen.getAllByRole("slider");

    // 1. User drags exposure to 2.0
    fireEvent.change(sliders[0], { target: { value: "2.0" } });
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);

    // 2. User drags contrast to 40
    fireEvent.change(sliders[1], { target: { value: "40" } });
    expect(useDevelopStore.getState().editParams.contrast).toBe(40);
    expect(useDevelopStore.getState().undoStack).toHaveLength(2);

    // 3. User presses Ctrl+Z to undo contrast
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(useDevelopStore.getState().editParams.contrast).toBe(0);
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);
    expect(useDevelopStore.getState().redoStack).toHaveLength(1);

    // 4. User presses Ctrl+Z again to undo exposure
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(0);
    expect(useDevelopStore.getState().redoStack).toHaveLength(2);

    // 5. User presses Ctrl+Shift+Z to redo exposure
    fireEvent.keyDown(window, { key: "z", ctrlKey: true, shiftKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);
    expect(useDevelopStore.getState().editParams.contrast).toBe(0);

    // 6. User presses Ctrl+Shift+Z to redo contrast
    fireEvent.keyDown(window, { key: "z", ctrlKey: true, shiftKey: true });
    expect(useDevelopStore.getState().editParams.contrast).toBe(40);
    expect(useDevelopStore.getState().redoStack).toHaveLength(0);
  });

  it("new edit after undo clears redo stack", () => {
    render(<UndoRedoHarness />);
    const sliders = screen.getAllByRole("slider");

    // Make two edits
    fireEvent.change(sliders[0], { target: { value: "1.0" } });
    fireEvent.change(sliders[1], { target: { value: "20" } });

    // Undo one
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(useDevelopStore.getState().redoStack).toHaveLength(1);

    // New edit should clear redo stack
    fireEvent.change(sliders[2], { target: { value: "-30" } });
    expect(useDevelopStore.getState().redoStack).toHaveLength(0);
    expect(useDevelopStore.getState().editParams.highlights).toBe(-30);
  });

  it("Ctrl+Z does nothing when undo stack is empty", () => {
    render(<UndoRedoHarness />);

    const before = { ...useDevelopStore.getState().editParams };
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(useDevelopStore.getState().editParams).toEqual(before);
  });

  it("Ctrl+Shift+Z does nothing when redo stack is empty", () => {
    render(<UndoRedoHarness />);
    const sliders = screen.getAllByRole("slider");

    fireEvent.change(sliders[0], { target: { value: "1.0" } });
    const before = { ...useDevelopStore.getState().editParams };

    fireEvent.keyDown(window, { key: "z", ctrlKey: true, shiftKey: true });
    expect(useDevelopStore.getState().editParams).toEqual(before);
  });

  it("pointer interaction tracking via slider pointerDown/pointerUp", () => {
    render(<UndoRedoHarness />);
    const sliders = screen.getAllByRole("slider");

    // Simulate pointer down (start adjusting)
    fireEvent.pointerDown(sliders[0]);
    expect(useDevelopStore.getState().isAdjusting).toBe(true);

    // Simulate pointer up (stop adjusting)
    fireEvent.pointerUp(sliders[0]);
    expect(useDevelopStore.getState().isAdjusting).toBe(false);
  });

  it("Ctrl+Shift+Z redo works with uppercase key (real browser behavior)", () => {
    render(<UndoRedoHarness />);
    const sliders = screen.getAllByRole("slider");

    fireEvent.change(sliders[0], { target: { value: "2.0" } });
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);

    // Undo
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(0);

    // Redo with uppercase Z (as browsers actually send with Shift held)
    fireEvent.keyDown(window, { key: "Z", ctrlKey: true, shiftKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);
  });

  it("Ctrl+Y redo works", () => {
    render(<UndoRedoHarness />);
    const sliders = screen.getAllByRole("slider");

    fireEvent.change(sliders[0], { target: { value: "3.0" } });
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(0);

    fireEvent.keyDown(window, { key: "y", ctrlKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(3.0);
  });

  it("slider drag coalesces into single undo step", () => {
    render(<UndoRedoHarness />);
    const sliders = screen.getAllByRole("slider");

    // Simulate full drag: pointerDown → multiple changes → pointerUp
    fireEvent.pointerDown(sliders[0]);
    fireEvent.change(sliders[0], { target: { value: "0.5" } });
    fireEvent.change(sliders[0], { target: { value: "1.0" } });
    fireEvent.change(sliders[0], { target: { value: "1.5" } });
    fireEvent.change(sliders[0], { target: { value: "2.0" } });
    fireEvent.pointerUp(sliders[0]);

    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);
    // Entire drag should be one undo entry
    expect(useDevelopStore.getState().undoStack).toHaveLength(1);

    // Single Ctrl+Z undoes the entire drag
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(0);

    // Single Ctrl+Shift+Z redoes it
    fireEvent.keyDown(window, { key: "Z", ctrlKey: true, shiftKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);
  });
});
