import { useDevelopStore } from "../../../stores/developStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { AdjustmentSlider } from "../controls/AdjustmentSlider";
import styles from "./DetailPanel.module.css";

export function DetailPanel() {
  const { editParams, updateParam } = useDevelopStore();
  return (
    <CollapsibleSection title="Detail" defaultOpen={false}>
      <h4 className={styles.sub}>Sharpening</h4>
      <AdjustmentSlider label="Amount" value={editParams.sharpening_amount} min={0} max={150} defaultValue={0} onChange={(v) => updateParam("sharpening_amount", v)} />
      <AdjustmentSlider label="Radius" value={editParams.sharpening_radius} min={0.5} max={3.0} step={0.1} defaultValue={1.0} onChange={(v) => updateParam("sharpening_radius", v)} />
      <AdjustmentSlider label="Detail" value={editParams.sharpening_detail} min={0} max={100} defaultValue={25} onChange={(v) => updateParam("sharpening_detail", v)} />
      <AdjustmentSlider label="Clarity" value={editParams.clarity} min={-100} max={100} defaultValue={0} onChange={(v) => updateParam("clarity", v)} />
      <h4 className={styles.sub}>Noise Reduction</h4>
      <AdjustmentSlider label="Luminance" value={editParams.denoise_luminance} min={0} max={100} defaultValue={0} onChange={(v) => updateParam("denoise_luminance", v)} />
      <AdjustmentSlider label="Color" value={editParams.denoise_color} min={0} max={100} defaultValue={0} onChange={(v) => updateParam("denoise_color", v)} />
      <div className={styles.toggle}><label className={styles.toggleLabel}><input type="checkbox" checked={editParams.denoise_ai} onChange={(e) => updateParam("denoise_ai", e.target.checked)} disabled /><span>AI Denoise (coming soon)</span></label></div>
    </CollapsibleSection>
  );
}
