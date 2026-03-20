import { describe, it, expect } from "vitest";
import type { ImageRecord, ColorLabel, Flag, FilterState } from "../../types/catalog";

describe("Catalog types", () => {
  it("should allow valid color labels", () => {
    const labels: ColorLabel[] = ["none", "red", "yellow", "green", "blue", "purple"];
    expect(labels).toHaveLength(6);
  });

  it("should allow valid flag values", () => {
    const flags: Flag[] = ["none", "picked", "rejected"];
    expect(flags).toHaveLength(3);
  });

  it("should create a valid ImageRecord", () => {
    const record: ImageRecord = {
      id: "test-id",
      file_path: "/path/to/image.jpg",
      file_name: "image.jpg",
      format: "jpeg",
      width: 1920,
      height: 1080,
      date_taken: null,
      rating: 3,
      color_label: "blue",
      flag: "picked",
      camera: null,
      lens: null,
      iso: null,
      focal_length: null,
      aperture: null,
      shutter_speed: null,
      edit_params: null,
      tags: ["landscape", "sunset"],
      created_at: "2024-01-01T00:00:00Z",
    };
    expect(record.id).toBe("test-id");
    expect(record.rating).toBe(3);
    expect(record.tags).toContain("landscape");
  });

  it("should create a valid FilterState", () => {
    const filter: FilterState = {
      query: "sunset",
      ratingMin: 3,
      colorLabel: "red",
      flag: "picked",
      collectionId: null,
    };
    expect(filter.query).toBe("sunset");
    expect(filter.ratingMin).toBe(3);
  });
});
