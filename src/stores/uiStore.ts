import { create } from "zustand";

export type ViewMode = "library" | "develop";

interface UiState {
  viewMode: ViewMode;
  leftPanelOpen: boolean;
  rightPanelOpen: boolean;
  filmStripOpen: boolean;
  selectedImageId: string | null;
  selectedImageIds: string[];
  isImporting: boolean;
  importProgress: { total: number; processed: number } | null;
  showImportDialog: boolean;
  showExportDialog: boolean;
  showDeleteConfirm: boolean;
  showBeforeAfter: boolean;
  zoomLevel: number;
  statusMessage: string;
  cropAspectRatio: number | null; // locked w:h ratio in image pixels, null = free
  cropToolActive: boolean;

  setViewMode: (mode: ViewMode) => void;
  toggleLeftPanel: () => void;
  toggleRightPanel: () => void;
  toggleFilmStrip: () => void;
  selectImage: (id: string | null) => void;
  setSelectedImages: (ids: string[]) => void;
  toggleImageSelection: (id: string) => void;
  setImporting: (importing: boolean) => void;
  setImportProgress: (progress: { total: number; processed: number } | null) => void;
  setShowImportDialog: (show: boolean) => void;
  setShowExportDialog: (show: boolean) => void;
  setShowDeleteConfirm: (show: boolean) => void;
  toggleBeforeAfter: () => void;
  setZoomLevel: (level: number) => void;
  setStatusMessage: (message: string) => void;
  setCropAspectRatio: (ratio: number | null) => void;
  setCropToolActive: (active: boolean) => void;
}

export const useUiStore = create<UiState>((set) => ({
  viewMode: "library",
  leftPanelOpen: true,
  rightPanelOpen: true,
  filmStripOpen: true,
  selectedImageId: null,
  selectedImageIds: [],
  isImporting: false,
  importProgress: null,
  showImportDialog: false,
  showExportDialog: false,
  showDeleteConfirm: false,
  showBeforeAfter: false,
  zoomLevel: 1,
  statusMessage: "Ready",
  cropAspectRatio: null,
  cropToolActive: false,

  setViewMode: (mode) => set({ viewMode: mode }),
  toggleLeftPanel: () => set((s) => ({ leftPanelOpen: !s.leftPanelOpen })),
  toggleRightPanel: () => set((s) => ({ rightPanelOpen: !s.rightPanelOpen })),
  toggleFilmStrip: () => set((s) => ({ filmStripOpen: !s.filmStripOpen })),
  selectImage: (id) =>
    set({ selectedImageId: id, selectedImageIds: id ? [id] : [] }),
  setSelectedImages: (ids) =>
    set({ selectedImageIds: ids, selectedImageId: ids[0] ?? null }),
  toggleImageSelection: (id) =>
    set((s) => {
      const ids = s.selectedImageIds.includes(id)
        ? s.selectedImageIds.filter((i) => i !== id)
        : [...s.selectedImageIds, id];
      return { selectedImageIds: ids, selectedImageId: ids[0] ?? null };
    }),
  setImporting: (importing) => set({ isImporting: importing }),
  setImportProgress: (progress) => set({ importProgress: progress }),
  setShowImportDialog: (show) => set({ showImportDialog: show }),
  setShowExportDialog: (show) => set({ showExportDialog: show }),
  setShowDeleteConfirm: (show) => set({ showDeleteConfirm: show }),
  toggleBeforeAfter: () => set((s) => ({ showBeforeAfter: !s.showBeforeAfter })),
  setZoomLevel: (level) => set({ zoomLevel: level }),
  setStatusMessage: (message) => set({ statusMessage: message }),
  setCropAspectRatio: (ratio) => set({ cropAspectRatio: ratio }),
  setCropToolActive: (active) => set({ cropToolActive: active }),
}));
