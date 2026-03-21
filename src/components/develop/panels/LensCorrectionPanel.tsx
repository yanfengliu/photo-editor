import { useEffect, useState } from "react";
import { useDevelopStore } from "../../../stores/developStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { AdjustmentSlider } from "../controls/AdjustmentSlider";
import {
  getLensProfiles,
  detectLensProfile,
  type LensProfileSummary,
} from "../../../api/processing";
import styles from "./LensCorrectionPanel.module.css";

export function LensCorrectionPanel() {
  const { editParams, updateParam, currentImageId } = useDevelopStore();
  const [profiles, setProfiles] = useState<LensProfileSummary[]>([]);
  const [detected, setDetected] = useState<LensProfileSummary | null>(null);

  useEffect(() => {
    getLensProfiles().then(setProfiles).catch(() => {});
  }, []);

  useEffect(() => {
    if (currentImageId) {
      detectLensProfile(currentImageId)
        .then((p) => {
          setDetected(p);
          // Auto-select if no profile is set yet
          if (p && !editParams.lens_profile_id) {
            updateParam("lens_profile_id", p.lens_id);
          }
        })
        .catch(() => {});
    }
  }, [currentImageId]);

  return (
    <CollapsibleSection title="Lens Correction" defaultOpen={false}>
      <div className={styles.toggle}>
        <label className={styles.toggleLabel}>
          <input
            type="checkbox"
            checked={editParams.enable_lens_correction}
            onChange={(e) =>
              updateParam("enable_lens_correction", e.target.checked)
            }
          />
          <span>Enable Profile Corrections</span>
        </label>
      </div>

      {detected && (
        <div className={styles.detected}>Detected: {detected.lens_name}</div>
      )}

      <h4 className={styles.sub}>Lens Profile</h4>
      <select
        className={styles.profileSelect}
        value={editParams.lens_profile_id ?? ""}
        onChange={(e) =>
          updateParam(
            "lens_profile_id",
            e.target.value === "" ? null : e.target.value
          )
        }
      >
        <option value="">None</option>
        {profiles.map((p) => (
          <option key={p.lens_id} value={p.lens_id}>
            {p.lens_name}
          </option>
        ))}
      </select>

      {editParams.enable_lens_correction && (
        <>
          <h4 className={styles.sub}>Distortion</h4>
          <AdjustmentSlider
            label="Distortion"
            value={editParams.lens_distortion}
            min={-100}
            max={100}
            defaultValue={0}
            onChange={(v) => updateParam("lens_distortion", v)}
          />
          <AdjustmentSlider
            label="Amount"
            value={editParams.lens_distortion_amount}
            min={0}
            max={200}
            defaultValue={100}
            onChange={(v) => updateParam("lens_distortion_amount", v)}
          />

          <div className={styles.toggle}>
            <label className={styles.toggleLabel}>
              <input
                type="checkbox"
                checked={editParams.lens_ca_correction}
                onChange={(e) =>
                  updateParam("lens_ca_correction", e.target.checked)
                }
              />
              <span>Remove Chromatic Aberration</span>
            </label>
          </div>

          <div className={styles.toggle}>
            <label className={styles.toggleLabel}>
              <input
                type="checkbox"
                checked={editParams.lens_vignette_correction}
                onChange={(e) =>
                  updateParam("lens_vignette_correction", e.target.checked)
                }
              />
              <span>Remove Lens Vignetting</span>
            </label>
          </div>
        </>
      )}
    </CollapsibleSection>
  );
}
