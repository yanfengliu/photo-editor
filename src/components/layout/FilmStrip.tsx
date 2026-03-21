import { useEffect, useState } from "react";
import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";
import { loadThumbnail } from "../../api/image";
import type { ImageRecord } from "../../types/catalog";
import styles from "./FilmStrip.module.css";

function FilmStripThumb({
  image,
  isSelected,
  onClick,
  onDoubleClick,
}: {
  image: ImageRecord;
  isSelected: boolean;
  onClick: () => void;
  onDoubleClick: () => void;
}) {
  const [thumbnailUrl, setThumbnailUrl] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    let objectUrl: string | null = null;

    setThumbnailUrl(null);

    loadThumbnail(image.id)
      .then((bytes) => {
        if (!active || bytes.length === 0) return;
        const buffer = new ArrayBuffer(bytes.byteLength);
        new Uint8Array(buffer).set(bytes);
        objectUrl = URL.createObjectURL(
          new Blob([buffer], { type: "image/jpeg" })
        );
        setThumbnailUrl(objectUrl);
      })
      .catch(() => {
        setThumbnailUrl(null);
      });

    return () => {
      active = false;
      if (objectUrl) {
        URL.revokeObjectURL(objectUrl);
      }
    };
  }, [image.id]);

  return (
    <div
      className={`${styles.thumb} ${isSelected ? styles.selected : ""}`}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
    >
      <div className={styles.thumbInner}>
        {thumbnailUrl ? (
          <img
            className={styles.thumbImage}
            src={thumbnailUrl}
            alt={image.file_name}
          />
        ) : (
          <span className={styles.thumbLabel}>{image.file_name}</span>
        )}
      </div>
      {image.rating > 0 && (
        <span className={styles.rating}>{"★".repeat(image.rating)}</span>
      )}
    </div>
  );
}

export function FilmStrip() {
  const { images } = useCatalogStore();
  const { selectedImageId, selectImage, setViewMode } = useUiStore();

  return (
    <div className={styles.filmStrip}>
      <div className={styles.track}>
        {images.map((image) => (
          <FilmStripThumb
            key={image.id}
            image={image}
            isSelected={selectedImageId === image.id}
            onClick={() => selectImage(image.id)}
            onDoubleClick={() => {
              selectImage(image.id);
              setViewMode("develop");
            }}
          />
        ))}
      </div>
    </div>
  );
}
