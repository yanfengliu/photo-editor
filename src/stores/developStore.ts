import { create } from "zustand";
import type { EditParams } from "../types/develop";
import { DEFAULT_EDIT_PARAMS } from "../types/develop";
import * as processingApi from "../api/processing";

let latestPreviewRequestId = 0;
const serializeParams = (params: EditParams) => JSON.stringify(params);

interface DevelopState {
  currentImageId: string | null;
  editParams: EditParams;
  originalParams: EditParams;
  persistedParams: EditParams;
  undoStack: EditParams[];
  redoStack: EditParams[];
  isProcessing: boolean;
  isAdjusting: boolean;
  adjustStartParams: EditParams | null;
  previewData: Uint8Array | null;
  previewWidth: number;
  previewHeight: number;

  setCurrentImage: (imageId: string) => Promise<void>;
  updateParam: <K extends keyof EditParams>(key: K, value: EditParams[K]) => void;
  applyEdits: (previewSize?: number) => Promise<void>;
  persistEdits: () => Promise<void>;
  resetEdits: () => Promise<void>;
  undo: () => void;
  redo: () => void;
  copyEdits: () => Promise<void>;
  pasteEdits: () => Promise<void>;
  setPreviewData: (data: Uint8Array, width: number, height: number) => void;
  startAdjusting: () => void;
  stopAdjusting: () => void;
}

export const useDevelopStore = create<DevelopState>((set, get) => ({
  currentImageId: null,
  editParams: { ...DEFAULT_EDIT_PARAMS },
  originalParams: { ...DEFAULT_EDIT_PARAMS },
  persistedParams: { ...DEFAULT_EDIT_PARAMS },
  undoStack: [],
  redoStack: [],
  isProcessing: false,
  isAdjusting: false,
  adjustStartParams: null,
  previewData: null,
  previewWidth: 0,
  previewHeight: 0,

  setCurrentImage: async (imageId: string) => {
    latestPreviewRequestId += 1;
    set({
      currentImageId: imageId,
      isProcessing: true,
      previewData: null,
      previewWidth: 0,
      previewHeight: 0,
    });
    try {
      const params = await processingApi.getEditParams(imageId);
      set({
        editParams: params,
        originalParams: { ...params },
        persistedParams: { ...params },
        undoStack: [],
        redoStack: [],
        isAdjusting: false,
        isProcessing: false,
      });
    } catch {
      set({
        editParams: { ...DEFAULT_EDIT_PARAMS },
        originalParams: { ...DEFAULT_EDIT_PARAMS },
        persistedParams: { ...DEFAULT_EDIT_PARAMS },
        isAdjusting: false,
        isProcessing: false,
        previewData: null,
        previewWidth: 0,
        previewHeight: 0,
      });
    }
  },

  updateParam: (key, value) => {
    const current = get().editParams;
    if (Object.is(current[key], value)) return;
    if (get().isAdjusting) {
      // During drag, just update params — undo entry is created in stopAdjusting
      set({ editParams: { ...current, [key]: value } });
    } else {
      // Discrete change (e.g., double-click reset)
      set((s) => ({
        undoStack: [...s.undoStack, { ...current }],
        redoStack: [],
        editParams: { ...current, [key]: value },
      }));
    }
  },

  applyEdits: async (previewSize) => {
    const { currentImageId, editParams } = get();
    if (!currentImageId) return;
    const requestId = ++latestPreviewRequestId;
    set({ isProcessing: true });
    try {
      const result = await processingApi.applyEdits(
        currentImageId,
        editParams,
        previewSize
      );
      if (requestId !== latestPreviewRequestId) return;
      set({
        previewData: result.data,
        previewWidth: result.width,
        previewHeight: result.height,
        isProcessing: false,
      });
    } catch (err) {
      console.error("Failed to apply edits:", err);
      if (requestId === latestPreviewRequestId) set({ isProcessing: false });
    }
  },

  persistEdits: async () => {
    const { currentImageId, editParams, persistedParams } = get();
    if (!currentImageId) return;
    if (serializeParams(editParams) === serializeParams(persistedParams)) return;
    try {
      await processingApi.saveEditParams(currentImageId, editParams);
      set({ persistedParams: { ...editParams } });
    } catch (err) {
      console.error("Failed to save edit params:", err);
    }
  },

  resetEdits: async () => {
    const { currentImageId } = get();
    if (!currentImageId) return;
    try {
      const params = await processingApi.resetEdits(currentImageId);
      set({
        editParams: params,
        persistedParams: { ...params },
        undoStack: [],
        redoStack: [],
        isAdjusting: false,
      });
    } catch (err) {
      console.error("Failed to reset edits:", err);
    }
  },

  undo: () => {
    const { undoStack, editParams } = get();
    if (undoStack.length === 0) return;
    const prev = undoStack[undoStack.length - 1];
    set({
      undoStack: undoStack.slice(0, -1),
      redoStack: [...get().redoStack, { ...editParams }],
      editParams: prev,
    });
  },

  redo: () => {
    const { redoStack, editParams } = get();
    if (redoStack.length === 0) return;
    const next = redoStack[redoStack.length - 1];
    set({
      redoStack: redoStack.slice(0, -1),
      undoStack: [...get().undoStack, { ...editParams }],
      editParams: next,
    });
  },

  copyEdits: async () => {
    const { currentImageId } = get();
    if (!currentImageId) return;
    await processingApi.copyEdits(currentImageId);
  },

  pasteEdits: async () => {
    const { currentImageId } = get();
    if (!currentImageId) return;
    const params = await processingApi.pasteEdits(currentImageId);
    set({ editParams: params, persistedParams: { ...params } });
  },

  setPreviewData: (data, width, height) =>
    set({ previewData: data, previewWidth: width, previewHeight: height }),
  startAdjusting: () =>
    set({ isAdjusting: true, adjustStartParams: { ...get().editParams } }),
  stopAdjusting: () => {
    const { adjustStartParams, editParams } = get();
    if (adjustStartParams && serializeParams(adjustStartParams) !== serializeParams(editParams)) {
      set((s) => ({
        isAdjusting: false,
        adjustStartParams: null,
        undoStack: [...s.undoStack, adjustStartParams],
        redoStack: [],
      }));
    } else {
      set({ isAdjusting: false, adjustStartParams: null });
    }
  },
}));
