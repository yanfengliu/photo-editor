import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "../../stores/uiStore";
import { useCatalogStore } from "../../stores/catalogStore";
import { Modal } from "../common/Modal";
import styles from "./ImportDialog.module.css";

const IMAGE_EXTENSIONS = [
  "jpg", "jpeg", "png", "tiff", "tif", "bmp", "webp",
  "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "raf", "pef",
];

export function ImportDialog() {
  const { setShowImportDialog, setStatusMessage } = useUiStore();
  const { importPaths } = useCatalogStore();
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [importing, setImporting] = useState(false);

  const selectionLabel =
    selectedPaths.length === 0
      ? ""
      : selectedPaths.length === 1
        ? selectedPaths[0]
        : `${selectedPaths.length} files selected`;

  const handleBrowseFolder = async () => {
    const selection = await open({ directory: true });
    if (typeof selection === "string") {
      setSelectedPaths([selection]);
    }
  };

  const handleBrowseFiles = async () => {
    const selection = await open({
      multiple: true,
      filters: [{ name: "Images", extensions: IMAGE_EXTENSIONS }],
    });

    if (Array.isArray(selection) && selection.length > 0) {
      setSelectedPaths(selection);
    } else if (typeof selection === "string") {
      setSelectedPaths([selection]);
    }
  };

  const handleImport = async () => {
    if (selectedPaths.length === 0) return;
    setImporting(true); setStatusMessage("Importing photos...");
    try {
      const imported = await importPaths(selectedPaths);
      const photoLabel = imported.length === 1 ? "photo" : "photos";
      setStatusMessage(`Imported ${imported.length} ${photoLabel}`);
      setShowImportDialog(false);
    }
    catch (err) { setStatusMessage(`Import failed: ${err}`); }
    finally { setImporting(false); }
  };

  return (
    <Modal title="Import Photos" onClose={() => setShowImportDialog(false)}>
      <div className={styles.content}>
        <div className={styles.field}>
          <label className={styles.label}>Import Source</label>
          <div className={styles.pathRow}>
            <input className={styles.input} value={selectionLabel} readOnly placeholder="Select a folder or one or more files..." />
          </div>
          <div className={styles.pickerRow}>
            <button className={styles.browseBtn} onClick={handleBrowseFolder}>Choose Folder</button>
            <button className={styles.browseBtn} onClick={handleBrowseFiles}>Choose Files</button>
          </div>
          {selectedPaths.length > 1 && <p className={styles.hint}>{selectedPaths.length} files selected for import</p>}
        </div>
        <div className={styles.actions}>
          <button className={styles.cancelBtn} onClick={() => setShowImportDialog(false)}>Cancel</button>
          <button className={styles.importBtn} onClick={handleImport} disabled={selectedPaths.length === 0 || importing}>{importing ? "Importing..." : "Import"}</button>
        </div>
      </div>
    </Modal>
  );
}
