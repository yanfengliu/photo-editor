import { invoke } from "@tauri-apps/api/core";
import type { EditParams, HistoryEntry, PreviewImagePayload } from "../types/develop";

export async function applyEdits(
  imageId: string,
  params: EditParams,
  previewSize?: number
): Promise<PreviewImagePayload> {
  return invoke("apply_edits", { imageId, params, previewSize });
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

export async function saveSnapshot(imageId: string, name: string): Promise<void> {
  return invoke("save_snapshot", { imageId, name });
}

export async function loadSnapshot(
  imageId: string,
  snapshotId: string
): Promise<EditParams> {
  return invoke("load_snapshot", { imageId, snapshotId });
}

export async function getHistory(imageId: string): Promise<HistoryEntry[]> {
  return invoke("get_history", { imageId });
}

export async function copyEdits(imageId: string): Promise<void> {
  return invoke("copy_edits", { imageId });
}

export async function pasteEdits(imageId: string): Promise<EditParams> {
  return invoke("paste_edits", { imageId });
}
