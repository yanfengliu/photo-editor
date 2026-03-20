import { describe, it, expect, beforeEach } from "vitest";
import { useUiStore } from "../../stores/uiStore";

describe("uiStore", () => {
  beforeEach(() => {
    useUiStore.setState({
      viewMode: "library",
      leftPanelOpen: true,
      rightPanelOpen: true,
      filmStripOpen: true,
      selectedImageId: null,
      selectedImageIds: [],
      isImporting: false,
      importProgress: null,
      showImportDialog: false,
      showExportDialog: false,
      showBeforeAfter: false,
      zoomLevel: 1,
      statusMessage: "Ready",
    });
  });

  it("should have correct initial state", () => {
    const state = useUiStore.getState();
    expect(state.viewMode).toBe("library");
    expect(state.leftPanelOpen).toBe(true);
    expect(state.rightPanelOpen).toBe(true);
    expect(state.selectedImageId).toBeNull();
    expect(state.statusMessage).toBe("Ready");
  });

  it("should toggle view mode", () => {
    useUiStore.getState().setViewMode("develop");
    expect(useUiStore.getState().viewMode).toBe("develop");
    useUiStore.getState().setViewMode("library");
    expect(useUiStore.getState().viewMode).toBe("library");
  });

  it("should toggle panels", () => {
    useUiStore.getState().toggleLeftPanel();
    expect(useUiStore.getState().leftPanelOpen).toBe(false);
    useUiStore.getState().toggleLeftPanel();
    expect(useUiStore.getState().leftPanelOpen).toBe(true);

    useUiStore.getState().toggleRightPanel();
    expect(useUiStore.getState().rightPanelOpen).toBe(false);

    useUiStore.getState().toggleFilmStrip();
    expect(useUiStore.getState().filmStripOpen).toBe(false);
  });

  it("should select an image", () => {
    useUiStore.getState().selectImage("img-1");
    expect(useUiStore.getState().selectedImageId).toBe("img-1");
    expect(useUiStore.getState().selectedImageIds).toEqual(["img-1"]);
  });

  it("should clear selection when selecting null", () => {
    useUiStore.getState().selectImage("img-1");
    useUiStore.getState().selectImage(null);
    expect(useUiStore.getState().selectedImageId).toBeNull();
    expect(useUiStore.getState().selectedImageIds).toEqual([]);
  });

  it("should toggle image selection", () => {
    useUiStore.getState().toggleImageSelection("img-1");
    expect(useUiStore.getState().selectedImageIds).toEqual(["img-1"]);
    useUiStore.getState().toggleImageSelection("img-2");
    expect(useUiStore.getState().selectedImageIds).toEqual(["img-1", "img-2"]);
    useUiStore.getState().toggleImageSelection("img-1");
    expect(useUiStore.getState().selectedImageIds).toEqual(["img-2"]);
  });

  it("should set multiple selected images", () => {
    useUiStore.getState().setSelectedImages(["a", "b", "c"]);
    expect(useUiStore.getState().selectedImageIds).toEqual(["a", "b", "c"]);
    expect(useUiStore.getState().selectedImageId).toBe("a");
  });

  it("should manage import state", () => {
    useUiStore.getState().setImporting(true);
    expect(useUiStore.getState().isImporting).toBe(true);
    useUiStore.getState().setImportProgress({ total: 100, processed: 50 });
    expect(useUiStore.getState().importProgress).toEqual({ total: 100, processed: 50 });
  });

  it("should manage dialog visibility", () => {
    useUiStore.getState().setShowImportDialog(true);
    expect(useUiStore.getState().showImportDialog).toBe(true);
    useUiStore.getState().setShowExportDialog(true);
    expect(useUiStore.getState().showExportDialog).toBe(true);
  });

  it("should toggle before/after", () => {
    useUiStore.getState().toggleBeforeAfter();
    expect(useUiStore.getState().showBeforeAfter).toBe(true);
    useUiStore.getState().toggleBeforeAfter();
    expect(useUiStore.getState().showBeforeAfter).toBe(false);
  });

  it("should set zoom level", () => {
    useUiStore.getState().setZoomLevel(2.5);
    expect(useUiStore.getState().zoomLevel).toBe(2.5);
  });

  it("should set status message", () => {
    useUiStore.getState().setStatusMessage("Importing...");
    expect(useUiStore.getState().statusMessage).toBe("Importing...");
  });
});
