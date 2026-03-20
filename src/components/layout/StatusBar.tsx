import { useUiStore } from "../../stores/uiStore";
import { useCatalogStore } from "../../stores/catalogStore";
import styles from "./StatusBar.module.css";

export function StatusBar() {
  const { statusMessage, selectedImageIds, zoomLevel } = useUiStore();
  const { totalImages } = useCatalogStore();

  return (
    <div className={styles.statusBar}>
      <span className={styles.message}>{statusMessage}</span>
      <span className={styles.info}>
        {selectedImageIds.length > 0 && <span>{selectedImageIds.length} selected | </span>}
        {totalImages} photos
        {zoomLevel !== 1 && <span> | {Math.round(zoomLevel * 100)}%</span>}
      </span>
    </div>
  );
}
