import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { useUiStore } from "../../stores/uiStore";
import { exportImage } from "../../api/system";
import { Modal } from "../common/Modal";
import styles from "./ExportDialog.module.css";

export function ExportDialog() {
  const { selectedImageId, setShowExportDialog, setStatusMessage } = useUiStore();
  const [format, setFormat] = useState<"jpeg"|"png"|"tiff">("jpeg");
  const [quality, setQuality] = useState(90);
  const [maxDim, setMaxDim] = useState("");
  const [exporting, setExporting] = useState(false);

  const handleExport = async () => {
    if (!selectedImageId) return;
    const path = await save({ filters: [{ name: format.toUpperCase(), extensions: [format === "jpeg" ? "jpg" : format] }] });
    if (!path) return;
    setExporting(true); setStatusMessage("Exporting...");
    try { await exportImage(selectedImageId, { format, quality, output_path: path, max_dimension: maxDim ? parseInt(maxDim) : null }); setStatusMessage(`Exported to ${path}`); setShowExportDialog(false); }
    catch (err) { setStatusMessage(`Export failed: ${err}`); }
    finally { setExporting(false); }
  };

  return (
    <Modal title="Export Image" onClose={() => setShowExportDialog(false)}>
      <div className={styles.content}>
        {!selectedImageId && <p className={styles.warning}>No image selected</p>}
        <div className={styles.field}><label className={styles.label}>Format</label><select className={styles.select} value={format} onChange={(e) => setFormat(e.target.value as any)}><option value="jpeg">JPEG</option><option value="png">PNG</option><option value="tiff">TIFF</option></select></div>
        {format === "jpeg" && <div className={styles.field}><label className={styles.label}>Quality: {quality}%</label><input type="range" min={1} max={100} value={quality} onChange={(e) => setQuality(parseInt(e.target.value))} /></div>}
        <div className={styles.field}><label className={styles.label}>Max Dimension (px)</label><input className={styles.input} type="number" value={maxDim} onChange={(e) => setMaxDim(e.target.value)} placeholder="Original size" /></div>
        <div className={styles.actions}>
          <button className={styles.cancelBtn} onClick={() => setShowExportDialog(false)}>Cancel</button>
          <button className={styles.exportBtn} onClick={handleExport} disabled={!selectedImageId || exporting}>{exporting ? "Exporting..." : "Export"}</button>
        </div>
      </div>
    </Modal>
  );
}
