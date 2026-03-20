import { useEffect } from "react";
import { FolderTree } from "./FolderTree";
import { ThumbnailGrid } from "./ThumbnailGrid";
import { FilterBar } from "./FilterBar";
import { MetadataPanel } from "./MetadataPanel";
import { useUiStore } from "../../stores/uiStore";
import { useCatalogStore } from "../../stores/catalogStore";
import styles from "./LibraryView.module.css";

export function LibraryView() {
  const { leftPanelOpen, rightPanelOpen, selectedImageId } = useUiStore();
  const { loadImages, images } = useCatalogStore();
  useEffect(() => { if (images.length === 0) loadImages(); }, []);
  const selectedImage = images.find((i) => i.id === selectedImageId) ?? null;

  return (
    <div className={styles.library}>
      {leftPanelOpen && <div className={styles.leftPanel}><FolderTree /></div>}
      <div className={styles.center}><FilterBar /><ThumbnailGrid /></div>
      {rightPanelOpen && <div className={styles.rightPanel}><MetadataPanel image={selectedImage} /></div>}
    </div>
  );
}
