import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "../../stores/uiStore";
import { batchExport } from "../../api/system";
import { Modal } from "../common/Modal";
import styles from "./ExportDialog.module.css";

export function BatchExportDialog() {
  const { selectedImageIds, setStatusMessage } = useUiStore();
  const [format, setFormat] = useState<"jpeg"|"png"|"tiff">("jpeg");
  const [quality] = useState(90);
  const [outputDir, setOutputDir] = useState("");
  const [exporting, setExporting] = useState(false);
  const [isOpen, setIsOpen] = useState(true);

  const handleBrowse = async () => { const s = await open({ directory: true }); if (s) setOutputDir(s as string); };
  const handleExport = async () => {
    if (!selectedImageIds.length || !outputDir) return;
    setExporting(true); setStatusMessage(`Exporting ${selectedImageIds.length} images...`);
    try { const res = await batchExport(selectedImageIds, { format, quality, output_path: outputDir, max_dimension: null }); setStatusMessage(`Exported ${res.length} images`); setIsOpen(false); }
    catch (err) { setStatusMessage(`Batch export failed: ${err}`); }
    finally { setExporting(false); }
  };
  if (!isOpen) return null;

  return (
    <Modal title="Batch Export" onClose={() => setIsOpen(false)}>
      <div className={styles.content}>
        <p>{selectedImageIds.length} images selected</p>
        <div className={styles.field}><label className={styles.label}>Output Folder</label><div style={{display:"flex",gap:8}}><input className={styles.input} value={outputDir} readOnly placeholder="Select folder..." /><button onClick={handleBrowse}>Browse</button></div></div>
        <div className={styles.field}><label className={styles.label}>Format</label><select className={styles.select} value={format} onChange={(e) => setFormat(e.target.value as any)}><option value="jpeg">JPEG</option><option value="png">PNG</option><option value="tiff">TIFF</option></select></div>
        <div className={styles.actions}><button className={styles.cancelBtn} onClick={() => setIsOpen(false)}>Cancel</button><button className={styles.exportBtn} onClick={handleExport} disabled={exporting || !outputDir}>{exporting ? "Exporting..." : "Export All"}</button></div>
      </div>
    </Modal>
  );
}
