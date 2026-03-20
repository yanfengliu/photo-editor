import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";
import styles from "./FilmStrip.module.css";

export function FilmStrip() {
  const { images } = useCatalogStore();
  const { selectedImageId, selectImage, setViewMode } = useUiStore();

  return (
    <div className={styles.filmStrip}>
      <div className={styles.track}>
        {images.map((image) => (
          <div key={image.id} className={`${styles.thumb} ${selectedImageId === image.id ? styles.selected : ""}`} onClick={() => selectImage(image.id)} onDoubleClick={() => { selectImage(image.id); setViewMode("develop"); }}>
            <div className={styles.thumbInner}>
              <span className={styles.thumbLabel}>{image.file_name}</span>
            </div>
            {image.rating > 0 && <span className={styles.rating}>{"★".repeat(image.rating)}</span>}
          </div>
        ))}
      </div>
    </div>
  );
}
