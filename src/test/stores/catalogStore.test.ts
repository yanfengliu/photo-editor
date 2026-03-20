import { describe, it, expect, beforeEach, vi } from "vitest";
import { useCatalogStore } from "../../stores/catalogStore";
import type { ImageRecord } from "../../types/catalog";

const mockImage: ImageRecord = {
  id: "img-1",
  file_path: "/photos/test.jpg",
  file_name: "test.jpg",
  format: "jpeg",
  width: 1920,
  height: 1080,
  date_taken: "2024-01-15",
  rating: 0,
  color_label: "none",
  flag: "none",
  camera: "Canon EOS R5",
  lens: "RF 24-70mm",
  iso: 400,
  focal_length: 50,
  aperture: 2.8,
  shutter_speed: "1/250",
  edit_params: null,
  tags: [],
  created_at: "2024-01-15T10:00:00Z",
};

vi.mock("../../api/catalog", () => ({
  getImages: vi.fn().mockResolvedValue([]),
  importFolder: vi.fn().mockResolvedValue([]),
  searchImages: vi.fn().mockResolvedValue([]),
  setRating: vi.fn().mockResolvedValue(undefined),
  setColorLabel: vi.fn().mockResolvedValue(undefined),
  setFlag: vi.fn().mockResolvedValue(undefined),
  addTags: vi.fn().mockResolvedValue(undefined),
  removeTag: vi.fn().mockResolvedValue(undefined),
  getCollections: vi.fn().mockResolvedValue([]),
  createCollection: vi.fn().mockResolvedValue({ id: "col-1", name: "Test", parent_id: null, image_count: 0, created_at: "" }),
  addToCollection: vi.fn().mockResolvedValue(undefined),
  deleteImages: vi.fn().mockResolvedValue(undefined),
}));

describe("catalogStore", () => {
  beforeEach(() => {
    useCatalogStore.setState({
      images: [],
      collections: [],
      totalImages: 0,
      loading: false,
      filter: { query: "", ratingMin: 0, colorLabel: null, flag: null, collectionId: null },
      folders: [],
    });
  });

  it("should have empty initial state", () => {
    const state = useCatalogStore.getState();
    expect(state.images).toEqual([]);
    expect(state.collections).toEqual([]);
    expect(state.totalImages).toBe(0);
    expect(state.loading).toBe(false);
  });

  it("should set filter", () => {
    useCatalogStore.getState().setFilter({ query: "sunset" });
    expect(useCatalogStore.getState().filter.query).toBe("sunset");
    expect(useCatalogStore.getState().filter.ratingMin).toBe(0);
  });

  it("should set partial filter without overwriting other fields", () => {
    useCatalogStore.getState().setFilter({ query: "sunset" });
    useCatalogStore.getState().setFilter({ ratingMin: 3 });
    const f = useCatalogStore.getState().filter;
    expect(f.query).toBe("sunset");
    expect(f.ratingMin).toBe(3);
  });

  it("should clear filter", () => {
    useCatalogStore.getState().setFilter({ query: "test", ratingMin: 4, flag: "picked" });
    useCatalogStore.getState().clearFilter();
    const f = useCatalogStore.getState().filter;
    expect(f.query).toBe("");
    expect(f.ratingMin).toBe(0);
    expect(f.flag).toBeNull();
  });

  it("should update image in list", () => {
    useCatalogStore.setState({ images: [mockImage], totalImages: 1 });
    useCatalogStore.getState().updateImageInList("img-1", { rating: 5 });
    expect(useCatalogStore.getState().images[0].rating).toBe(5);
  });

  it("should not modify other images when updating", () => {
    const img2: ImageRecord = { ...mockImage, id: "img-2", file_name: "test2.jpg" };
    useCatalogStore.setState({ images: [mockImage, img2], totalImages: 2 });
    useCatalogStore.getState().updateImageInList("img-1", { rating: 5 });
    expect(useCatalogStore.getState().images[1].rating).toBe(0);
  });

  it("should delete images from list", async () => {
    useCatalogStore.setState({ images: [mockImage], totalImages: 1 });
    await useCatalogStore.getState().deleteImages(["img-1"]);
    expect(useCatalogStore.getState().images).toHaveLength(0);
    expect(useCatalogStore.getState().totalImages).toBe(0);
  });

  it("should set rating via API and update locally", async () => {
    useCatalogStore.setState({ images: [mockImage], totalImages: 1 });
    await useCatalogStore.getState().setRating("img-1", 4);
    expect(useCatalogStore.getState().images[0].rating).toBe(4);
  });

  it("should set color label via API and update locally", async () => {
    useCatalogStore.setState({ images: [mockImage], totalImages: 1 });
    await useCatalogStore.getState().setColorLabel("img-1", "red");
    expect(useCatalogStore.getState().images[0].color_label).toBe("red");
  });

  it("should set flag via API and update locally", async () => {
    useCatalogStore.setState({ images: [mockImage], totalImages: 1 });
    await useCatalogStore.getState().setFlag("img-1", "picked");
    expect(useCatalogStore.getState().images[0].flag).toBe("picked");
  });
});
