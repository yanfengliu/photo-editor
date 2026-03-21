import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { useUiStore } from "../../stores/uiStore";
import { useCatalogStore } from "../../stores/catalogStore";
import { DeleteConfirmDialog } from "../../components/common/DeleteConfirmDialog";

vi.mock("../../api/catalog", () => ({
  deleteImages: vi.fn().mockResolvedValue(undefined),
  getImages: vi.fn().mockResolvedValue([]),
  getCollections: vi.fn().mockResolvedValue([]),
  searchImages: vi.fn().mockResolvedValue([]),
  setRating: vi.fn().mockResolvedValue(undefined),
  setFlag: vi.fn().mockResolvedValue(undefined),
  setColorLabel: vi.fn().mockResolvedValue(undefined),
  addTags: vi.fn().mockResolvedValue(undefined),
  removeTag: vi.fn().mockResolvedValue(undefined),
  createCollection: vi.fn().mockResolvedValue({ id: "c1", name: "test", parent_id: null, image_count: 0, created_at: "" }),
  addToCollection: vi.fn().mockResolvedValue(undefined),
  importPaths: vi.fn().mockResolvedValue([]),
}));

describe("DeleteConfirmDialog", () => {
  beforeEach(() => {
    useUiStore.setState({
      selectedImageId: "img-1",
      selectedImageIds: ["img-1"],
      showDeleteConfirm: true,
    });
    useCatalogStore.setState({
      images: [
        {
          id: "img-1",
          file_path: "/photos/test.jpg",
          file_name: "test.jpg",
          format: "jpeg",
          width: 1920,
          height: 1080,
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
          created_at: "",
        },
      ],
      totalImages: 1,
    });
  });

  it("should render confirmation message for single image", () => {
    render(<DeleteConfirmDialog />);
    expect(screen.getByText(/this image/)).toBeTruthy();
  });

  it("should render confirmation message for multiple images", () => {
    useUiStore.setState({ selectedImageIds: ["img-1", "img-2"] });
    render(<DeleteConfirmDialog />);
    expect(screen.getByText(/these 2 images/)).toBeTruthy();
  });

  it("should mention original files are not deleted", () => {
    render(<DeleteConfirmDialog />);
    expect(screen.getByText(/original files will not be deleted/)).toBeTruthy();
  });

  it("should render Cancel and Delete buttons", () => {
    render(<DeleteConfirmDialog />);
    expect(screen.getByText("Cancel")).toBeTruthy();
    expect(screen.getByText("Delete")).toBeTruthy();
  });

  it("should close dialog on Cancel click", () => {
    render(<DeleteConfirmDialog />);
    fireEvent.click(screen.getByText("Cancel"));
    expect(useUiStore.getState().showDeleteConfirm).toBe(false);
  });

  it("should close dialog on Escape key", () => {
    render(<DeleteConfirmDialog />);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(useUiStore.getState().showDeleteConfirm).toBe(false);
  });

  it("should delete images and close on Delete click", async () => {
    render(<DeleteConfirmDialog />);
    fireEvent.click(screen.getByText("Delete"));
    await waitFor(() => {
      expect(useUiStore.getState().showDeleteConfirm).toBe(false);
      expect(useUiStore.getState().selectedImageId).toBeNull();
      expect(useCatalogStore.getState().images).toHaveLength(0);
    });
  });

  it("should clear selection after delete", async () => {
    render(<DeleteConfirmDialog />);
    fireEvent.click(screen.getByText("Delete"));
    await waitFor(() => {
      expect(useUiStore.getState().selectedImageId).toBeNull();
      expect(useUiStore.getState().selectedImageIds).toEqual([]);
    });
  });
});
