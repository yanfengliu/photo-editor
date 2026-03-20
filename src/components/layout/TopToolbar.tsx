import { useUiStore } from "../../stores/uiStore";
import styles from "./TopToolbar.module.css";

export function TopToolbar() {
  const { viewMode, setViewMode, setShowImportDialog, setShowExportDialog } = useUiStore();

  return (
    <div className={styles.toolbar}>
      <div className={styles.left}>
        <span className={styles.logo}>Photo Editor</span>
      </div>
      <div className={styles.center}>
        <button className={`${styles.viewBtn} ${viewMode === "library" ? styles.active : ""}`} onClick={() => setViewMode("library")}>Library</button>
        <button className={`${styles.viewBtn} ${viewMode === "develop" ? styles.active : ""}`} onClick={() => setViewMode("develop")}>Develop</button>
      </div>
      <div className={styles.right}>
        <button className={styles.actionBtn} onClick={() => setShowImportDialog(true)}>Import</button>
        <button className={styles.actionBtn} onClick={() => setShowExportDialog(true)}>Export</button>
      </div>
    </div>
  );
}
