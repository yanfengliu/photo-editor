import { useState, useCallback, useEffect } from "react";
import { useCatalogStore } from "../../stores/catalogStore";
import styles from "./FolderTree.module.css";

export function FolderTree() {
  const { collections, loadCollections, createCollection, filter, setFilter, loadImages, searchImages } = useCatalogStore();
  const [showNewInput, setShowNewInput] = useState(false);
  const [newName, setNewName] = useState("");

  useEffect(() => {
    loadCollections();
  }, [loadCollections]);

  const handleCreate = useCallback(async () => {
    const name = newName.trim();
    if (!name) return;
    await createCollection(name);
    setNewName("");
    setShowNewInput(false);
  }, [newName, createCollection]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") handleCreate();
      if (e.key === "Escape") {
        setShowNewInput(false);
        setNewName("");
      }
    },
    [handleCreate]
  );

  const handleAllPhotos = useCallback(() => {
    setFilter({ collectionId: null });
    loadImages();
  }, [setFilter, loadImages]);

  const handleCollectionClick = useCallback(
    (collectionId: string) => {
      setFilter({ collectionId });
      searchImages();
    },
    [setFilter, searchImages]
  );

  const activeCollectionId = filter.collectionId;

  return (
    <div className={styles.tree}>
      <div className={styles.section}>
        <h3 className={styles.sectionTitle}>Catalog</h3>
        <div
          className={`${styles.item} ${activeCollectionId === null ? styles.active : ""}`}
          onClick={handleAllPhotos}
        >
          <span>All Photos</span>
        </div>
        <div className={styles.item}>
          <span>Recent Imports</span>
        </div>
      </div>
      <div className={styles.section}>
        <div className={styles.sectionHeader}>
          <h3 className={styles.sectionTitle}>Collections</h3>
          <button
            className={styles.addBtn}
            onClick={() => setShowNewInput(true)}
            title="Create new collection"
          >
            +
          </button>
        </div>
        {showNewInput && (
          <input
            className={styles.newCollInput}
            type="text"
            placeholder="Collection name..."
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={handleKeyDown}
            onBlur={handleCreate}
            autoFocus
          />
        )}
        {collections.map((col) => (
          <div
            key={col.id}
            className={`${styles.item} ${activeCollectionId === col.id ? styles.active : ""}`}
            onClick={() => handleCollectionClick(col.id)}
          >
            <span>{col.name}</span>
            <span className={styles.count}>{col.image_count}</span>
          </div>
        ))}
        {collections.length === 0 && !showNewInput && (
          <div className={styles.empty}>No collections yet</div>
        )}
      </div>
    </div>
  );
}
