import { useEffect } from "react";
import { useUiStore } from "../stores/uiStore";
import { useCatalogStore } from "../stores/catalogStore";
import { useDevelopStore } from "../stores/developStore";

export function useKeyboardShortcuts() {
  const { viewMode, setViewMode, selectedImageId } = useUiStore();
  const { setRating, setFlag } = useCatalogStore();
  const { undo, redo } = useDevelopStore();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Ignore if typing in an input
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      const ctrl = e.ctrlKey || e.metaKey;

      // View toggles
      if (e.key === "g" || e.key === "G") {
        e.preventDefault();
        setViewMode("library");
        return;
      }
      if (e.key === "d" || e.key === "D") {
        e.preventDefault();
        setViewMode("develop");
        return;
      }

      // Ratings (1-5, 0 to clear)
      if (selectedImageId && /^[0-5]$/.test(e.key) && !ctrl) {
        e.preventDefault();
        setRating(selectedImageId, parseInt(e.key));
        return;
      }

      // Flags
      if (selectedImageId && e.key === "p") {
        e.preventDefault();
        setFlag(selectedImageId, "picked");
        return;
      }
      if (selectedImageId && e.key === "x") {
        e.preventDefault();
        setFlag(selectedImageId, "rejected");
        return;
      }
      if (selectedImageId && e.key === "u") {
        e.preventDefault();
        setFlag(selectedImageId, "none");
        return;
      }

      // Undo/Redo
      if (ctrl && e.key.toLowerCase() === "z" && !e.shiftKey) {
        e.preventDefault();
        undo();
        return;
      }
      if (ctrl && (e.key.toLowerCase() === "y" || (e.key.toLowerCase() === "z" && e.shiftKey))) {
        e.preventDefault();
        redo();
        return;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [viewMode, selectedImageId, setViewMode, setRating, setFlag, undo, redo]);
}
