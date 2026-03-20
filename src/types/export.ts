export interface ExportSettings {
  format: "jpeg" | "png" | "tiff";
  quality: number;
  output_path: string;
  max_dimension: number | null;
}

export interface BatchExportSettings extends ExportSettings {
  image_ids: string[];
}
