import { create } from "zustand";
import type { ImageRecord, CollectionRecord, FilterState } from "../types/catalog";
import * as catalogApi from "../api/catalog";

interface CatalogState {
  images: ImageRecord[];
  collections: CollectionRecord[];
  totalImages: number;
  loading: boolean;
  filter: FilterState;
  folders: string[];

  loadImages: (offset?: number, limit?: number) => Promise<void>;
  importFolder: (path: string) => Promise<ImageRecord[]>;
  searchImages: () => Promise<void>;
  setFilter: (filter: Partial<FilterState>) => void;
  clearFilter: () => void;
  setRating: (imageId: string, rating: number) => Promise<void>;
  setColorLabel: (imageId: string, colorLabel: string) => Promise<void>;
  setFlag: (imageId: string, flag: string) => Promise<void>;
  addTags: (imageId: string, tags: string[]) => Promise<void>;
  removeTag: (imageId: string, tag: string) => Promise<void>;
  loadCollections: () => Promise<void>;
  createCollection: (name: string, parentId?: string) => Promise<void>;
  addToCollection: (collectionId: string, imageIds: string[]) => Promise<void>;
  deleteImages: (imageIds: string[]) => Promise<void>;
  updateImageInList: (imageId: string, updates: Partial<ImageRecord>) => void;
}

const defaultFilter: FilterState = {
  query: "",
  ratingMin: 0,
  colorLabel: null,
  flag: null,
  collectionId: null,
};

export const useCatalogStore = create<CatalogState>((set, get) => ({
  images: [],
  collections: [],
  totalImages: 0,
  loading: false,
  filter: { ...defaultFilter },
  folders: [],

  loadImages: async (offset = 0, limit = 200) => {
    set({ loading: true });
    try {
      const images = await catalogApi.getImages(offset, limit);
      if (offset === 0) {
        set({ images, totalImages: images.length, loading: false });
      } else {
        set((s) => ({
          images: [...s.images, ...images],
          totalImages: s.totalImages + images.length,
          loading: false,
        }));
      }
    } catch (err) {
      console.error("Failed to load images:", err);
      set({ loading: false });
    }
  },

  importFolder: async (path: string) => {
    set({ loading: true });
    try {
      const imported = await catalogApi.importFolder(path);
      set((s) => ({
        images: [...imported, ...s.images],
        totalImages: s.totalImages + imported.length,
        loading: false,
      }));
      return imported;
    } catch (err) {
      console.error("Failed to import folder:", err);
      set({ loading: false });
      return [];
    }
  },

  searchImages: async () => {
    const { filter } = get();
    set({ loading: true });
    try {
      const images = await catalogApi.searchImages(
        filter.query,
        filter.ratingMin > 0 ? filter.ratingMin : undefined,
        filter.colorLabel ?? undefined,
        filter.flag ?? undefined
      );
      set({ images, totalImages: images.length, loading: false });
    } catch (err) {
      console.error("Failed to search images:", err);
      set({ loading: false });
    }
  },

  setFilter: (partial) =>
    set((s) => ({ filter: { ...s.filter, ...partial } })),

  clearFilter: () => set({ filter: { ...defaultFilter } }),

  setRating: async (imageId, rating) => {
    await catalogApi.setRating(imageId, rating);
    get().updateImageInList(imageId, { rating });
  },

  setColorLabel: async (imageId, colorLabel) => {
    await catalogApi.setColorLabel(imageId, colorLabel);
    get().updateImageInList(imageId, { color_label: colorLabel as ImageRecord["color_label"] });
  },

  setFlag: async (imageId, flag) => {
    await catalogApi.setFlag(imageId, flag);
    get().updateImageInList(imageId, { flag: flag as ImageRecord["flag"] });
  },

  addTags: async (imageId, tags) => {
    await catalogApi.addTags(imageId, tags);
    const img = get().images.find((i) => i.id === imageId);
    if (img) {
      get().updateImageInList(imageId, {
        tags: [...new Set([...img.tags, ...tags])],
      });
    }
  },

  removeTag: async (imageId, tag) => {
    await catalogApi.removeTag(imageId, tag);
    const img = get().images.find((i) => i.id === imageId);
    if (img) {
      get().updateImageInList(imageId, {
        tags: img.tags.filter((t) => t !== tag),
      });
    }
  },

  loadCollections: async () => {
    try {
      const collections = await catalogApi.getCollections();
      set({ collections });
    } catch (err) {
      console.error("Failed to load collections:", err);
    }
  },

  createCollection: async (name, parentId) => {
    const collection = await catalogApi.createCollection(name, parentId);
    set((s) => ({ collections: [...s.collections, collection] }));
  },

  addToCollection: async (collectionId, imageIds) => {
    await catalogApi.addToCollection(collectionId, imageIds);
    await get().loadCollections();
  },

  deleteImages: async (imageIds) => {
    await catalogApi.deleteImages(imageIds);
    set((s) => ({
      images: s.images.filter((i) => !imageIds.includes(i.id)),
      totalImages: s.totalImages - imageIds.length,
    }));
  },

  updateImageInList: (imageId, updates) =>
    set((s) => ({
      images: s.images.map((img) =>
        img.id === imageId ? { ...img, ...updates } : img
      ),
    })),
}));
