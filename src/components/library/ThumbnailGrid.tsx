import React, { useCallback, useMemo } from "react";
import { VirtuosoGrid } from "react-virtuoso";
import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";
import { ThumbnailCard } from "./ThumbnailCard";
import styles from "./ThumbnailGrid.module.css";

export function ThumbnailGrid() {
  const { images, loading } = useCatalogStore();
  const { selectedImageId, selectImage, setViewMode } = useUiStore();

  const handleDoubleClick = useCallback(
    (id: string) => {
      selectImage(id);
      setViewMode("develop");
    },
    [selectImage, setViewMode]
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
          />
        );
      }}
      style={{ flex: 1 }}
    />
  );
}
