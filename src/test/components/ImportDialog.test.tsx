import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { open } from "@tauri-apps/plugin-dialog";
import { ImportDialog } from "../../components/library/ImportDialog";
import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";

describe("ImportDialog", () => {
  const importPaths = vi.fn();
  const setShowImportDialog = vi.fn();
  const setStatusMessage = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();

    useCatalogStore.setState({
      importPaths,
      images: [],
      collections: [],
      totalImages: 0,
      loading: false,
      filter: { query: "", ratingMin: 0, colorLabel: null, flag: null, collectionId: null },
      folders: [],
    });

    useUiStore.setState({
      setShowImportDialog,
      setStatusMessage,
    });
  });

  it("should allow selecting individual files and import them", async () => {
    importPaths.mockResolvedValueOnce([
      {
        id: "img-1",
        file_path: "/photos/raw.cr3",
        file_name: "raw.cr3",
        format: "raw",
        width: 0,
        height: 0,
        date_taken: null,
        rating: 0,
        color_label: "none",
        flag: "none",
        camera: null,
        lens: null,
        iso: null,
        focal_length: null,
        aperture: null,
        shutter_speed: null,
        edit_params: null,
        tags: [],
        created_at: "2024-01-01T00:00:00Z",
      },
    ]);

    vi.mocked(open).mockResolvedValueOnce([
      "C:/photos/raw.cr3",
      "C:/photos/preview.jpg",
    ]);

    render(<ImportDialog />);

    fireEvent.click(screen.getByText("Choose Files"));

    await waitFor(() => {
      expect(open).toHaveBeenCalledWith({
        multiple: true,
        filters: [
          {
            name: "Images",
            extensions: [
              "jpg", "jpeg", "png", "tiff", "tif", "bmp", "webp",
              "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf", "pef",
            ],
          },
        ],
      });
    });

    expect(screen.getByDisplayValue("2 files selected")).toBeTruthy();

    fireEvent.click(screen.getByText("Import"));

    await waitFor(() => {
      expect(importPaths).toHaveBeenCalledWith([
        "C:/photos/raw.cr3",
        "C:/photos/preview.jpg",
      ]);
    });

    expect(setStatusMessage).toHaveBeenCalledWith("Importing photos...");
    expect(setStatusMessage).toHaveBeenCalledWith("Imported 1 photo");
    expect(setShowImportDialog).toHaveBeenCalledWith(false);
  });
});
