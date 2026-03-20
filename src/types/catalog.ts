export interface ImageRecord {
  id: string;
  file_path: string;
  file_name: string;
  format: string;
  width: number;
  height: number;
  date_taken: string | null;
  rating: number;
  color_label: ColorLabel;
  flag: Flag;
  camera: string | null;
  lens: string | null;
  iso: number | null;
  focal_length: number | null;
  aperture: number | null;
  shutter_speed: string | null;
  edit_params: string | null;
  tags: string[];
  created_at: string;
}

export type ColorLabel = "none" | "red" | "yellow" | "green" | "blue" | "purple";
export type Flag = "none" | "picked" | "rejected";

export interface CollectionRecord {
  id: string;
  name: string;
  parent_id: string | null;
  image_count: number;
  created_at: string;
}

export interface ImportProgress {
  total: number;
  processed: number;
  current_file: string;
}

export interface FilterState {
  query: string;
  ratingMin: number;
  colorLabel: ColorLabel | null;
  flag: Flag | null;
  collectionId: string | null;
}
