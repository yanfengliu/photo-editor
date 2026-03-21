import React, { useCallback, useMemo, useState } from "react";
import { VirtuosoGrid } from "react-virtuoso";
import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";
import { ThumbnailCard } from "./ThumbnailCard";
import { ImageContextMenu } from "./ImageContextMenu";
import type { MenuPosition } from "../common/ContextMenu";
import styles from "./ThumbnailGrid.module.css";

export function ThumbnailGrid() {
  const { images, loading } = useCatalogStore();
  const { selectedImageId, selectImage, setViewMode } = useUiStore();
  const [contextMenu, setContextMenu] = useState<{
    position: MenuPosition;
    imageId: string;
  } | null>(null);

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
              isSelected={selectedImageId === image.id}
              onClick={() => selectImage(image.id)}
              onDoubleClick={() => handleDoubleClick(image.id)}
              onContextMenu={(e) => handleContextMenu(e, image.id)}
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
