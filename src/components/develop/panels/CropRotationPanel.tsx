import { useDevelopStore } from "../../../stores/developStore";
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
  };

  const handleAspectSelect = (value: AspectValue) => {
    if (value === null) return; // Free — no constraint applied
    let ratio: number;
    if (value === "original") {
      ratio = previewWidth && previewHeight ? previewWidth / previewHeight : 1;
    } else {
      ratio = value;
    }
    // Fit the largest rect with this ratio inside the current image
    const cx = editParams.crop_x + editParams.crop_width / 2;
    const cy = editParams.crop_y + editParams.crop_height / 2;
    let w: number, h: number;
    if (ratio >= 1) {
      w = Math.min(1, editParams.crop_width);
      h = w / ratio;
      if (h > 1) { h = 1; w = h * ratio; }
    } else {
      h = Math.min(1, editParams.crop_height);
      w = h * ratio;
      if (w > 1) { w = 1; h = w / ratio; }
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
          {ASPECT_PRESETS.map((preset) => (
            <button
              key={preset.label}
              className={styles.aspectBtn}
              onClick={() => handleAspectSelect(preset.value as AspectValue)}
            >
              {preset.label}
            </button>
          ))}
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
