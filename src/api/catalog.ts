import { invoke } from "@tauri-apps/api/core";
import type { ImageRecord, CollectionRecord } from "../types/catalog";

export async function importFolder(path: string): Promise<ImageRecord[]> {
  return importPaths([path]);
}

export async function importPaths(paths: string[]): Promise<ImageRecord[]> {
  return invoke("import_paths", { paths });
}

export async function getImages(
  offset = 0,
  limit = 100,
  sortBy = "date_taken",
  sortOrder = "DESC"
): Promise<ImageRecord[]> {
  return invoke("get_images", {
    offset,
    limit,
    sortBy,
    sortOrder,
  });
}

export async function searchImages(
  query: string,
  ratingMin?: number,
  colorLabel?: string,
  flag?: string
): Promise<ImageRecord[]> {
  return invoke("search_images", {
    query,
    ratingMin,
    colorLabel,
    flag,
  });
}

export async function setRating(imageId: string, rating: number): Promise<void> {
  return invoke("set_rating", { imageId, rating });
}

export async function setColorLabel(imageId: string, colorLabel: string): Promise<void> {
  return invoke("set_color_label", { imageId, colorLabel });
}

export async function setFlag(imageId: string, flag: string): Promise<void> {
  return invoke("set_flag", { imageId, flag });
}

export async function addTags(imageId: string, tags: string[]): Promise<void> {
  return invoke("add_tags", { imageId, tags });
}

export async function removeTag(imageId: string, tag: string): Promise<void> {
  return invoke("remove_tag", { imageId, tag });
}

export async function createCollection(
  name: string,
  parentId?: string
): Promise<CollectionRecord> {
  return invoke("create_collection", { name, parentId });
}

export async function addToCollection(
  collectionId: string,
  imageIds: string[]
): Promise<void> {
  return invoke("add_to_collection", { collectionId, imageIds });
}

export async function getCollections(): Promise<CollectionRecord[]> {
  return invoke("get_collections");
}

export async function deleteImages(imageIds: string[]): Promise<void> {
  return invoke("delete_images", { imageIds });
}
