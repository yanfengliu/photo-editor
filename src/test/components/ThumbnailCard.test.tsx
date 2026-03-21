import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { ThumbnailCard } from "../../components/library/ThumbnailCard";

vi.mock("../../api/image", () => ({
  loadThumbnail: vi.fn(),
}));

describe("ThumbnailCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    URL.createObjectURL = vi.fn(() => "blob:thumb");
    URL.revokeObjectURL = vi.fn();
  });

  it("should render the loaded thumbnail image", async () => {
    const imageApi = await import("../../api/image");
    vi.mocked(imageApi.loadThumbnail).mockResolvedValueOnce(new Uint8Array([255, 216, 255]));

    render(
      <ThumbnailCard
        image={{
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
          created_at: "2024-01-01T00:00:00Z",
        }}
        isSelected={false}
        onClick={() => undefined}
        onDoubleClick={() => undefined}
      />
    );

    await waitFor(() => {
      expect(screen.getByRole("img", { name: "test.jpg" })).toHaveAttribute("src", "blob:thumb");
    });
  });
});
