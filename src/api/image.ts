import { invoke } from "@tauri-apps/api/core";
import type { ExifData } from "../types/metadata";

export async function loadThumbnail(imageId: string): Promise<Uint8Array> {
  const bytes = await invoke<number[]>("load_thumbnail", { imageId });
  return new Uint8Array(bytes);
}

export async function loadPreview(imageId: string): Promise<Uint8Array> {
  const bytes = await invoke<number[]>("load_preview", { imageId });
  return new Uint8Array(bytes);
}

export async function loadFullResolution(imageId: string): Promise<Uint8Array> {
  const bytes = await invoke<number[]>("load_full_resolution", { imageId });
  return new Uint8Array(bytes);
}

export async function getExifData(imageId: string): Promise<ExifData> {
  return invoke("get_exif_data", { imageId });
}
