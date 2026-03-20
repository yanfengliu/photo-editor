import { describe, it, expect, beforeEach, vi } from "vitest";

vi.mock("../../api/processing", () => {
  const d = {
    exposure: 0, contrast: 0, highlights: 0, shadows: 0, whites: 0, blacks: 0,
    temperature: 6500, tint: 0, saturation: 0, vibrance: 0,
    curve_rgb: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    curve_r: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    curve_g: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    curve_b: [{ x: 0, y: 0 }, { x: 1, y: 1 }],
    hsl_hue: [0, 0, 0, 0, 0, 0, 0, 0],
    hsl_saturation: [0, 0, 0, 0, 0, 0, 0, 0],
    hsl_luminance: [0, 0, 0, 0, 0, 0, 0, 0],
    sharpening_amount: 0, sharpening_radius: 1.0, clarity: 0,
    denoise_luminance: 0, denoise_color: 0, denoise_ai: false,
    dehaze: 0, vignette_amount: 0, grain_amount: 0, grain_size: 25,
  };
  return {
    getEditParams: vi.fn().mockResolvedValue({ ...d }),
    applyEdits: vi.fn().mockResolvedValue({ data: [0, 0, 0, 255], width: 1, height: 1 }),
    resetEdits: vi.fn().mockResolvedValue({ ...d }),
    saveSnapshot: vi.fn().mockResolvedValue(undefined),
    loadSnapshot: vi.fn().mockResolvedValue({ ...d }),
    getHistory: vi.fn().mockResolvedValue([]),
    copyEdits: vi.fn().mockResolvedValue(undefined),
    pasteEdits: vi.fn().mockResolvedValue({ ...d }),
  };
});

import { useDevelopStore } from "../../stores/developStore";
import { DEFAULT_EDIT_PARAMS } from "../../types/develop";

describe("developStore", () => {
  beforeEach(() => {
    useDevelopStore.setState({
      currentImageId: null,
      editParams: { ...DEFAULT_EDIT_PARAMS },
      originalParams: { ...DEFAULT_EDIT_PARAMS },
      history: [],
      snapshots: [],
      undoStack: [],
      redoStack: [],
      isProcessing: false,
      previewData: null,
      previewWidth: 0,
      previewHeight: 0,
    });
  });

  it("should have correct default edit params", () => {
    const { editParams } = useDevelopStore.getState();
    expect(editParams.exposure).toBe(0);
    expect(editParams.contrast).toBe(0);
    expect(editParams.temperature).toBe(6500);
    expect(editParams.tint).toBe(0);
    expect(editParams.saturation).toBe(0);
    expect(editParams.vibrance).toBe(0);
    expect(editParams.sharpening_radius).toBe(1.0);
    expect(editParams.grain_size).toBe(25);
    expect(editParams.denoise_ai).toBe(false);
    expect(editParams.hsl_hue).toEqual([0, 0, 0, 0, 0, 0, 0, 0]);
    expect(editParams.curve_rgb).toEqual([{ x: 0, y: 0 }, { x: 1, y: 1 }]);
  });

  it("should update a single parameter", () => {
    useDevelopStore.getState().updateParam("exposure", 1.5);
    expect(useDevelopStore.getState().editParams.exposure).toBe(1.5);
  });

  it("should add to undo stack on param update", () => {
    useDevelopStore.getState().updateParam("exposure", 1.0);
    expect(useDevelopStore.getState().undoStack).toHaveLength(1);
    expect(useDevelopStore.getState().undoStack[0].exposure).toBe(0);
  });

  it("should clear redo stack on new param update", () => {
    useDevelopStore.getState().updateParam("exposure", 1.0);
    useDevelopStore.getState().updateParam("exposure", 2.0);
    useDevelopStore.getState().undo();
    expect(useDevelopStore.getState().redoStack).toHaveLength(1);
    useDevelopStore.getState().updateParam("contrast", 50);
    expect(useDevelopStore.getState().redoStack).toHaveLength(0);
  });

  it("should undo parameter changes", () => {
    useDevelopStore.getState().updateParam("exposure", 1.0);
    useDevelopStore.getState().updateParam("exposure", 2.0);
    useDevelopStore.getState().undo();
    expect(useDevelopStore.getState().editParams.exposure).toBe(1.0);
    expect(useDevelopStore.getState().redoStack).toHaveLength(1);
  });

  it("should redo parameter changes", () => {
    useDevelopStore.getState().updateParam("exposure", 1.0);
    useDevelopStore.getState().undo();
    useDevelopStore.getState().redo();
    expect(useDevelopStore.getState().editParams.exposure).toBe(1.0);
  });

  it("should not undo when stack is empty", () => {
    const before = useDevelopStore.getState().editParams.exposure;
    useDevelopStore.getState().undo();
    expect(useDevelopStore.getState().editParams.exposure).toBe(before);
  });

  it("should not redo when stack is empty", () => {
    useDevelopStore.getState().updateParam("exposure", 1.0);
    const before = useDevelopStore.getState().editParams.exposure;
    useDevelopStore.getState().redo();
    expect(useDevelopStore.getState().editParams.exposure).toBe(before);
  });

  it("should set current image", async () => {
    await useDevelopStore.getState().setCurrentImage("img-1");
    expect(useDevelopStore.getState().currentImageId).toBe("img-1");
    expect(useDevelopStore.getState().undoStack).toHaveLength(0);
    expect(useDevelopStore.getState().redoStack).toHaveLength(0);
  });

  it("should set preview data", () => {
    const data = new Uint8Array([255, 0, 0, 255]);
    useDevelopStore.getState().setPreviewData(data, 1, 1);
    expect(useDevelopStore.getState().previewData).toBe(data);
    expect(useDevelopStore.getState().previewWidth).toBe(1);
    expect(useDevelopStore.getState().previewHeight).toBe(1);
  });

  it("should populate preview dimensions after applying edits", async () => {
    await useDevelopStore.getState().setCurrentImage("img-1");
    await useDevelopStore.getState().applyEdits(2048);
    expect(useDevelopStore.getState().previewData).toEqual(new Uint8Array([0, 0, 0, 255]));
    expect(useDevelopStore.getState().previewWidth).toBe(1);
    expect(useDevelopStore.getState().previewHeight).toBe(1);
  });

  it("should update HSL array parameters", () => {
    const newHue = [10, 20, 30, 40, 50, 60, 70, 80];
    useDevelopStore.getState().updateParam("hsl_hue", newHue);
    expect(useDevelopStore.getState().editParams.hsl_hue).toEqual(newHue);
  });

  it("should update curve points", () => {
    const newCurve = [{ x: 0, y: 0 }, { x: 0.5, y: 0.7 }, { x: 1, y: 1 }];
    useDevelopStore.getState().updateParam("curve_rgb", newCurve);
    expect(useDevelopStore.getState().editParams.curve_rgb).toEqual(newCurve);
  });
});
