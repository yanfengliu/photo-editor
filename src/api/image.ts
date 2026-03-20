import { invoke } from "@tauri-apps/api/core";
import type { ExifData } from "../types/metadata";

export async function loadThumbnail(imageId: string): Promise<number[]> {
  return invoke("load_thumbnail", { imageId });
}

export async function loadPreview(imageId: string): Promise<number[]> {
  return invoke("load_preview", { imageId });
}

export async function loadFullResolution(imageId: string): Promise<number[]> {
  return invoke("load_full_resolution", { imageId });
}

export async function getExifData(imageId: string): Promise<ExifData> {
  return invoke("get_exif_data", { imageId });
}
