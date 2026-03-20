import { useCatalogStore } from "../../stores/catalogStore";
import styles from "./FolderTree.module.css";

export function FolderTree() {
  const { collections } = useCatalogStore();
  return (
    <div className={styles.tree}>
      <div className={styles.section}>
        <h3 className={styles.sectionTitle}>Catalog</h3>
        <div className={styles.item}><span>All Photos</span></div>
        <div className={styles.item}><span>Recent Imports</span></div>
      </div>
      <div className={styles.section}>
        <h3 className={styles.sectionTitle}>Collections</h3>
        {collections.map((col) => (
          <div key={col.id} className={styles.item}>
            <span>{col.name}</span>
            <span className={styles.count}>{col.image_count}</span>
          </div>
        ))}
        {collections.length === 0 && <div className={styles.empty}>No collections yet</div>}
      </div>
    </div>
  );
}
