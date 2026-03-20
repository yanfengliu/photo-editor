import { useEffect, useCallback, useState } from "react";

interface DragAndDropOptions {
  onDrop: (paths: string[]) => void;
  accept?: string[];
}

export function useDragAndDrop({ onDrop, accept }: DragAndDropOptions) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);

      const files = e.dataTransfer?.files;
      if (!files) return;

      const paths: string[] = [];
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        if (accept) {
          const ext = file.name.split(".").pop()?.toLowerCase() ?? "";
          if (accept.includes(ext)) {
            paths.push(file.name);
          }
        } else {
          paths.push(file.name);
        }
      }

      if (paths.length > 0) {
        onDrop(paths);
      }
    },
    [onDrop, accept]
  );

  useEffect(() => {
    document.addEventListener("dragover", handleDragOver);
    document.addEventListener("dragleave", handleDragLeave);
    document.addEventListener("drop", handleDrop);

    return () => {
      document.removeEventListener("dragover", handleDragOver);
      document.removeEventListener("dragleave", handleDragLeave);
      document.removeEventListener("drop", handleDrop);
    };
  }, [handleDragOver, handleDragLeave, handleDrop]);

  return { isDragging };
}
