import { useDevelopStore } from "../../../stores/developStore";
import { useUiStore } from "../../../stores/uiStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { AdjustmentSlider } from "../controls/AdjustmentSlider";
import styles from "./CropRotationPanel.module.css";

const ASPECT_PRESETS = [
  { label: "Free", value: null },
  { label: "Original", value: "original" },
  { label: "1:1", value: 1 },
  { label: "4:3", value: 4 / 3 },
  { label: "3:2", value: 3 / 2 },
  { label: "16:9", value: 16 / 9 },
  { label: "5:4", value: 5 / 4 },
  { label: "7:5", value: 7 / 5 },
] as const;

type AspectValue = null | "original" | number;

export function CropRotationPanel() {
  const { editParams, updateParam, previewWidth, previewHeight } =
    useDevelopStore();
  const { cropAspectRatio, setCropAspectRatio } = useUiStore();

  const handleRotate = (degrees: number) => {
    const current = editParams.rotation;
    const next = ((current + degrees) % 360 + 360) % 360;
    updateParam("rotation", next);
  };

  const handleResetCrop = () => {
    updateParam("crop_x", 0);
    updateParam("crop_y", 0);
    updateParam("crop_width", 1);
    updateParam("crop_height", 1);
    setCropAspectRatio(null);
  };

  const handleAspectSelect = (value: AspectValue) => {
    if (value === null) {
      setCropAspectRatio(null);
      return;
    }
    let ratio: number;
    if (value === "original") {
      ratio = previewWidth && previewHeight ? previewWidth / previewHeight : 1;
    } else {
      ratio = value;
    }

    setCropAspectRatio(ratio);

    // Compute normalized crop dimensions that produce the desired pixel aspect ratio.
    // pixel_w / pixel_h = ratio  →  (crop_w * imgW) / (crop_h * imgH) = ratio
    // So:  crop_w / crop_h = ratio * imgH / imgW  (the "normalized ratio")
    const imgW = previewWidth || 1;
    const imgH = previewHeight || 1;
    const normRatio = ratio * imgH / imgW;

    const cx = editParams.crop_x + editParams.crop_width / 2;
    const cy = editParams.crop_y + editParams.crop_height / 2;
    let w: number, h: number;
    if (normRatio >= 1) {
      // Crop is wider (in normalized coords) than tall
      w = Math.min(1, editParams.crop_width);
      h = w / normRatio;
      if (h > 1) { h = 1; w = h * normRatio; }
    } else {
      // Crop is taller than wide
      h = Math.min(1, editParams.crop_height);
      w = h * normRatio;
      if (w > 1) { w = 1; h = w / normRatio; }
    }
    // Clamp so crop stays within [0,1]
    const x = Math.max(0, Math.min(1 - w, cx - w / 2));
    const y = Math.max(0, Math.min(1 - h, cy - h / 2));
    updateParam("crop_x", parseFloat(x.toFixed(4)));
    updateParam("crop_y", parseFloat(y.toFixed(4)));
    updateParam("crop_width", parseFloat(w.toFixed(4)));
    updateParam("crop_height", parseFloat(h.toFixed(4)));
  };

  const isCropped =
    editParams.crop_x !== 0 ||
    editParams.crop_y !== 0 ||
    editParams.crop_width !== 1 ||
    editParams.crop_height !== 1;

  // Determine which preset is active
  const activeRatio = cropAspectRatio;

  return (
    <CollapsibleSection title="Crop & Rotate" defaultOpen={false}>
      <div className={styles.section}>
        <h4 className={styles.sub}>Rotation</h4>
        <div className={styles.rotateButtons}>
          <button
            className={styles.rotateBtn}
            onClick={() => handleRotate(-90)}
            title="Rotate 90° counter-clockwise"
          >
            ↺ 90°
          </button>
          <button
            className={styles.rotateBtn}
            onClick={() => handleRotate(90)}
            title="Rotate 90° clockwise"
          >
            ↻ 90°
          </button>
          <button
            className={styles.rotateBtn}
            onClick={() => handleRotate(180)}
            title="Rotate 180°"
          >
            180°
          </button>
        </div>

        <AdjustmentSlider
          label="Fine Tune"
          value={editParams.rotation_fine}
          min={-45}
          max={45}
          step={0.1}
          defaultValue={0}
          onChange={(v) => updateParam("rotation_fine", v)}
        />
      </div>

      <div className={styles.section}>
        <h4 className={styles.sub}>Aspect Ratio</h4>
        <div className={styles.aspectGrid}>
          {ASPECT_PRESETS.map((preset) => {
            const isActive =
              preset.value === null
                ? activeRatio === null
                : preset.value === "original"
                  ? activeRatio !== null && previewWidth > 0 && Math.abs(activeRatio - previewWidth / previewHeight) < 0.01
                  : activeRatio !== null && Math.abs(activeRatio - (preset.value as number)) < 0.01;
            return (
              <button
                key={preset.label}
                className={`${styles.aspectBtn} ${isActive ? styles.aspectBtnActive : ""}`}
                onClick={() => handleAspectSelect(preset.value as AspectValue)}
              >
                {preset.label}
              </button>
            );
          })}
        </div>
      </div>

      {isCropped && (
        <button className={styles.resetBtn} onClick={handleResetCrop}>
          Reset Crop
        </button>
      )}
    </CollapsibleSection>
  );
}
