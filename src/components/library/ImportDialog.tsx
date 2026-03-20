import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "../../stores/uiStore";
import { useCatalogStore } from "../../stores/catalogStore";
import { Modal } from "../common/Modal";
import styles from "./ImportDialog.module.css";

export function ImportDialog() {
  const { setShowImportDialog, setStatusMessage } = useUiStore();
  const { importFolder } = useCatalogStore();
  const [folderPath, setFolderPath] = useState("");
  const [importing, setImporting] = useState(false);

  const handleBrowse = async () => { const s = await open({ directory: true }); if (s) setFolderPath(s as string); };
  const handleImport = async () => {
    if (!folderPath) return;
    setImporting(true); setStatusMessage("Importing photos...");
    try { const imported = await importFolder(folderPath); setStatusMessage(`Imported ${imported.length} photos`); setShowImportDialog(false); }
    catch (err) { setStatusMessage(`Import failed: ${err}`); }
    finally { setImporting(false); }
  };

  return (
    <Modal title="Import Photos" onClose={() => setShowImportDialog(false)}>
      <div className={styles.content}>
        <div className={styles.field}>
          <label className={styles.label}>Source Folder</label>
          <div className={styles.pathRow}>
            <input className={styles.input} value={folderPath} readOnly placeholder="Select a folder..." />
            <button className={styles.browseBtn} onClick={handleBrowse}>Browse</button>
          </div>
        </div>
        <div className={styles.actions}>
          <button className={styles.cancelBtn} onClick={() => setShowImportDialog(false)}>Cancel</button>
          <button className={styles.importBtn} onClick={handleImport} disabled={!folderPath || importing}>{importing ? "Importing..." : "Import"}</button>
        </div>
      </div>
    </Modal>
  );
}
