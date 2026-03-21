import { invoke } from "@tauri-apps/api/core";
import type { EditParams } from "../types/develop";

export interface BinaryPreview {
  data: Uint8Array;
  width: number;
  height: number;
}

export async function applyEdits(
  imageId: string,
  params: EditParams,
  previewSize?: number
): Promise<BinaryPreview> {
  // Binary IPC: returns ArrayBuffer with [u32 width][u32 height][RGBA bytes...]
  const buf: ArrayBuffer = await invoke("apply_edits", { imageId, params, previewSize });
  const header = new DataView(buf, 0, 8);
  const width = header.getUint32(0, true);
  const height = header.getUint32(4, true);
  const data = new Uint8Array(buf, 8);
  return { data, width, height };
}

export async function saveEditParams(
  imageId: string,
  params: EditParams
): Promise<void> {
  return invoke("save_edit_params", { imageId, params });
}

export async function getEditParams(imageId: string): Promise<EditParams> {
  return invoke("get_edit_params", { imageId });
}

export async function resetEdits(imageId: string): Promise<EditParams> {
  return invoke("reset_edits", { imageId });
}

export async function copyEdits(imageId: string): Promise<void> {
  return invoke("copy_edits", { imageId });
}

export async function pasteEdits(imageId: string): Promise<EditParams> {
  return invoke("paste_edits", { imageId });
}

export interface LensProfileSummary {
  lens_id: string;
  lens_name: string;
  mount: string;
  focal_range: [number, number];
}

export async function getLensProfiles(): Promise<LensProfileSummary[]> {
  return invoke("get_lens_profiles");
}

export async function detectLensProfile(
  imageId: string
): Promise<LensProfileSummary | null> {
  return invoke("detect_lens_profile", { imageId });
}
