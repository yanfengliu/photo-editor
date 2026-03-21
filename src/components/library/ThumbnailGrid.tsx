import React, { useCallback, useMemo, useRef, useState } from "react";
import { VirtuosoGrid } from "react-virtuoso";
import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";
import { ThumbnailCard } from "./ThumbnailCard";
import { ImageContextMenu } from "./ImageContextMenu";
import type { MenuPosition } from "../common/ContextMenu";
import styles from "./ThumbnailGrid.module.css";

export function ThumbnailGrid() {
  const { images, loading } = useCatalogStore();
  const { selectedImageIds, selectImage, toggleImageSelection, setSelectedImages, setViewMode } = useUiStore();
  const [contextMenu, setContextMenu] = useState<{
    position: MenuPosition;
    imageId: string;
  } | null>(null);
  const lastClickedIndex = useRef<number | null>(null);

  const selectedSet = useMemo(() => new Set(selectedImageIds), [selectedImageIds]);

  const handleClick = useCallback(
    (e: React.MouseEvent, imageId: string, index: number) => {
      if (e.shiftKey && lastClickedIndex.current !== null) {
        const start = Math.min(lastClickedIndex.current, index);
        const end = Math.max(lastClickedIndex.current, index);
        const rangeIds = images.slice(start, end + 1).map((img) => img.id);
        const merged = new Set(selectedImageIds);
        rangeIds.forEach((id) => merged.add(id));
        setSelectedImages([...merged]);
      } else if (e.ctrlKey || e.metaKey) {
        toggleImageSelection(imageId);
        lastClickedIndex.current = index;
      } else {
        selectImage(imageId);
        lastClickedIndex.current = index;
      }
    },
    [images, selectedImageIds, selectImage, toggleImageSelection, setSelectedImages]
  );

  const handleDoubleClick = useCallback(
    (id: string) => {
      selectImage(id);
      setViewMode("develop");
    },
    [selectImage, setViewMode]
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, imageId: string) => {
      e.preventDefault();
      setContextMenu({ position: { x: e.clientX, y: e.clientY }, imageId });
    },
    []
  );

  const handleDragStart = useCallback(
    (e: React.DragEvent, imageId: string) => {
      const dragIds = selectedSet.has(imageId)
        ? selectedImageIds
        : [imageId];
      e.dataTransfer.setData("application/x-photo-ids", JSON.stringify(dragIds));
      e.dataTransfer.effectAllowed = "copy";
    },
    [selectedImageIds, selectedSet]
  );

  const gridComponents = useMemo(
    () => ({
      List: React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
        (props, ref) => (
          <div ref={ref} {...props} className={styles.grid} />
        )
      ),
      Item: ({ children, ...props }: React.HTMLAttributes<HTMLDivElement>) => (
        <div {...props} className={styles.item}>
          {children}
        </div>
      ),
    }),
    []
  );

  if (loading && images.length === 0)
    return (
      <div className={styles.empty}>
        <p>Loading...</p>
      </div>
    );
  if (images.length === 0)
    return (
      <div className={styles.empty}>
        <p>No photos in catalog</p>
        <p className={styles.hint}>Import a folder to get started</p>
      </div>
    );

  return (
    <>
      <VirtuosoGrid
        totalCount={images.length}
        overscan={200}
        components={gridComponents}
        itemContent={(index) => {
          const image = images[index];
          return (
            <ThumbnailCard
              key={image.id}
              image={image}
              isSelected={selectedSet.has(image.id)}
              onClick={(e) => handleClick(e, image.id, index)}
              onDoubleClick={() => handleDoubleClick(image.id)}
              onContextMenu={(e) => handleContextMenu(e, image.id)}
              onDragStart={(e) => handleDragStart(e, image.id)}
            />
          );
        }}
        style={{ flex: 1 }}
      />
      {contextMenu && (
        <ImageContextMenu
          position={contextMenu.position}
          imageId={contextMenu.imageId}
          onClose={() => setContextMenu(null)}
        />
      )}
    </>
  );
}
