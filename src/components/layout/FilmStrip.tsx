import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";
import { useThumbnail } from "../../hooks/useThumbnail";
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
  const thumbnailUrl = useThumbnail(image.id);

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
