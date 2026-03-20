import { useCatalogStore } from "../../stores/catalogStore";
import { useUiStore } from "../../stores/uiStore";
import { ThumbnailCard } from "./ThumbnailCard";
import styles from "./ThumbnailGrid.module.css";

export function ThumbnailGrid() {
  const { images, loading } = useCatalogStore();
  const { selectedImageId, selectImage, setViewMode } = useUiStore();

  if (loading && images.length === 0) return <div className={styles.empty}><p>Loading...</p></div>;
  if (images.length === 0) return <div className={styles.empty}><p>No photos in catalog</p><p className={styles.hint}>Import a folder to get started</p></div>;

  return (
    <div className={styles.grid}>
      {images.map((image) => (
        <ThumbnailCard key={image.id} image={image} isSelected={selectedImageId === image.id} onClick={() => selectImage(image.id)} onDoubleClick={() => { selectImage(image.id); setViewMode("develop"); }} />
      ))}
    </div>
  );
}
