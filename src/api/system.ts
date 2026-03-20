import { invoke } from "@tauri-apps/api/core";
import type { GpuInfo, AppConfig } from "../types/metadata";
import type { ExportSettings } from "../types/export";

export async function browseFolder(): Promise<string | null> {
  return invoke("browse_folder");
}

export async function getGpuInfo(): Promise<GpuInfo> {
  return invoke("get_gpu_info");
}

export async function getAppConfig(): Promise<AppConfig> {
  return invoke("get_app_config");
}

export async function setAppConfig(config: AppConfig): Promise<void> {
  return invoke("set_app_config", { config });
}

export async function exportImage(
  imageId: string,
  settings: ExportSettings
): Promise<string> {
  return invoke("export_image", { imageId, settings });
}

export async function batchExport(
  imageIds: string[],
  settings: ExportSettings
): Promise<string[]> {
  return invoke("batch_export", { imageIds, settings });
}

export async function exportXmpSidecar(
  imageId: string,
  outputPath?: string
): Promise<string> {
  return invoke("export_xmp_sidecar", { imageId, outputPath: outputPath ?? null });
}
