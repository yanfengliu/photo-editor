import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { useCatalogStore } from "../../stores/catalogStore";
import { FolderTree } from "../../components/library/FolderTree";

vi.mock("../../api/catalog", () => ({
  getCollections: vi.fn().mockResolvedValue([]),
  createCollection: vi.fn().mockImplementation((name: string) =>
    Promise.resolve({ id: `col-${Date.now()}`, name, parent_id: null, image_count: 0, created_at: "" })
  ),
  getImages: vi.fn().mockResolvedValue([]),
  searchImages: vi.fn().mockResolvedValue([]),
  setRating: vi.fn().mockResolvedValue(undefined),
  setFlag: vi.fn().mockResolvedValue(undefined),
  setColorLabel: vi.fn().mockResolvedValue(undefined),
  addTags: vi.fn().mockResolvedValue(undefined),
  removeTag: vi.fn().mockResolvedValue(undefined),
  addToCollection: vi.fn().mockResolvedValue(undefined),
  deleteImages: vi.fn().mockResolvedValue(undefined),
  importPaths: vi.fn().mockResolvedValue([]),
}));

describe("FolderTree", () => {
  beforeEach(() => {
    useCatalogStore.setState({
      collections: [],
      images: [],
      totalImages: 0,
      loading: false,
    });
  });

  it("should render catalog sections", () => {
    render(<FolderTree />);
    expect(screen.getByText("All Photos")).toBeTruthy();
    expect(screen.getByText("Recent Imports")).toBeTruthy();
    expect(screen.getByText("Collections")).toBeTruthy();
  });

  it("should show empty state when no collections", () => {
    render(<FolderTree />);
    expect(screen.getByText("No collections yet")).toBeTruthy();
  });

  it("should render existing collections", () => {
    useCatalogStore.setState({
      collections: [
        { id: "c1", name: "Favorites", parent_id: null, image_count: 5, created_at: "" },
        { id: "c2", name: "Portfolio", parent_id: null, image_count: 12, created_at: "" },
      ],
    });
    render(<FolderTree />);
    expect(screen.getByText("Favorites")).toBeTruthy();
    expect(screen.getByText("Portfolio")).toBeTruthy();
    expect(screen.getByText("5")).toBeTruthy();
    expect(screen.getByText("12")).toBeTruthy();
  });

  it("should show create collection button", () => {
    render(<FolderTree />);
    expect(screen.getByTitle("Create new collection")).toBeTruthy();
  });

  it("should show input when + button is clicked", () => {
    render(<FolderTree />);
    fireEvent.click(screen.getByTitle("Create new collection"));
    expect(screen.getByPlaceholderText("Collection name...")).toBeTruthy();
  });

  it("should create collection on Enter", async () => {
    render(<FolderTree />);
    fireEvent.click(screen.getByTitle("Create new collection"));
    const input = screen.getByPlaceholderText("Collection name...");
    fireEvent.change(input, { target: { value: "Travel" } });
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => {
      expect(useCatalogStore.getState().collections.some(c => c.name === "Travel")).toBe(true);
    });
  });

  it("should cancel input on Escape", () => {
    render(<FolderTree />);
    fireEvent.click(screen.getByTitle("Create new collection"));
    const input = screen.getByPlaceholderText("Collection name...");
    fireEvent.keyDown(input, { key: "Escape" });
    expect(screen.queryByPlaceholderText("Collection name...")).toBeNull();
  });

  it("should not create collection with empty name", async () => {
    render(<FolderTree />);
    fireEvent.click(screen.getByTitle("Create new collection"));
    const input = screen.getByPlaceholderText("Collection name...");
    fireEvent.keyDown(input, { key: "Enter" });
    // Input should still be visible since name was empty
    await waitFor(() => {
      expect(useCatalogStore.getState().collections).toHaveLength(0);
    });
  });
});
