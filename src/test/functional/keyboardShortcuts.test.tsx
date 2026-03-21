import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, fireEvent, screen } from "@testing-library/react";
import { useUiStore } from "../../stores/uiStore";
import { useDevelopStore } from "../../stores/developStore";
import { useCatalogStore } from "../../stores/catalogStore";
import { DEFAULT_EDIT_PARAMS } from "../../types/develop";
import { useKeyboardShortcuts } from "../../hooks/useKeyboardShortcuts";
import * as catalogApi from "../../api/catalog";

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
}));

vi.mock("../../api/catalog", () => ({
  setRating: vi.fn().mockResolvedValue(undefined),
  setFlag: vi.fn().mockResolvedValue(undefined),
}));

function ShortcutHarness() {
  useKeyboardShortcuts();
  return <div data-testid="harness" />;
}

function pressKey(key: string, opts: Partial<KeyboardEventInit> = {}) {
  fireEvent.keyDown(window, { key, ...opts });
}

const testImage = {
  id: "img-1", file_path: "/a.jpg", file_name: "a.jpg", format: "jpeg",
  width: 100, height: 100, date_taken: null, rating: 0, color_label: "none",
  flag: "none", camera: null, lens: null, iso: null, focal_length: null,
  aperture: null, shutter_speed: null, edit_params: null, tags: [], created_at: "",
};

function resetStores(opts: { selectedImageId?: string | null } = {}) {
  useUiStore.setState({ viewMode: "library", selectedImageId: opts.selectedImageId ?? null, selectedImageIds: [] });
  useDevelopStore.setState({
    currentImageId: null,
    editParams: { ...DEFAULT_EDIT_PARAMS },
    undoStack: [],
    redoStack: [],
  });
  if (opts.selectedImageId) {
    useCatalogStore.setState({ images: [testImage] });
  }
}

describe("Keyboard shortcuts", () => {
  it("G switches to library view", () => {
    resetStores();
    render(<ShortcutHarness />);
    useUiStore.setState({ viewMode: "develop" });
    pressKey("g");
    expect(useUiStore.getState().viewMode).toBe("library");
  });

  it("D switches to develop view", () => {
    resetStores();
    render(<ShortcutHarness />);
    pressKey("d");
    expect(useUiStore.getState().viewMode).toBe("develop");
  });

  it("number keys 1-5 set rating when image selected", () => {
    resetStores({ selectedImageId: "img-1" });
    render(<ShortcutHarness />);
    pressKey("3");
    expect(vi.mocked(catalogApi.setRating)).toHaveBeenCalledWith("img-1", 3);
  });

  it("P sets picked flag when image selected", () => {
    resetStores({ selectedImageId: "img-1" });
    render(<ShortcutHarness />);
    pressKey("p");
    expect(vi.mocked(catalogApi.setFlag)).toHaveBeenCalledWith("img-1", "picked");
  });

  it("X sets rejected flag when image selected", () => {
    resetStores({ selectedImageId: "img-1" });
    render(<ShortcutHarness />);
    pressKey("x");
    expect(vi.mocked(catalogApi.setFlag)).toHaveBeenCalledWith("img-1", "rejected");
  });

  it("Ctrl+Z triggers undo", () => {
    resetStores();
    render(<ShortcutHarness />);
    useDevelopStore.getState().updateParam("exposure", 1.0);
    expect(useDevelopStore.getState().editParams.exposure).toBe(1.0);

    pressKey("z", { ctrlKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(0);
  });

  it("Ctrl+Shift+Z triggers redo", () => {
    resetStores();
    render(<ShortcutHarness />);
    useDevelopStore.getState().updateParam("exposure", 1.0);
    useDevelopStore.getState().undo();
    expect(useDevelopStore.getState().editParams.exposure).toBe(0);

    pressKey("z", { ctrlKey: true, shiftKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(1.0);
  });

  it("Ctrl+Y also triggers redo", () => {
    resetStores();
    render(<ShortcutHarness />);
    useDevelopStore.getState().updateParam("exposure", 2.0);
    useDevelopStore.getState().undo();

    pressKey("y", { ctrlKey: true });
    expect(useDevelopStore.getState().editParams.exposure).toBe(2.0);
  });

  it("does not fire shortcuts when typing in an input", () => {
    resetStores();
    render(<ShortcutHarness />);
    useUiStore.setState({ viewMode: "library" });
    const input = document.createElement("input");
    document.body.appendChild(input);
    input.focus();

    fireEvent.keyDown(input, { key: "d" });
    expect(useUiStore.getState().viewMode).toBe("library");

    document.body.removeChild(input);
  });

  it("rating keys are ignored when no image is selected", () => {
    resetStores();
    render(<ShortcutHarness />);
    const callsBefore = vi.mocked(catalogApi.setRating).mock.calls.length;
    pressKey("3");
    expect(vi.mocked(catalogApi.setRating).mock.calls.length).toBe(callsBefore);
  });
});
