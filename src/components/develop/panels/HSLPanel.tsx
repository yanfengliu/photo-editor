import { useState } from "react";
import { useDevelopStore } from "../../../stores/developStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { AdjustmentSlider } from "../controls/AdjustmentSlider";
import { HSL_CHANNELS } from "../../../types/develop";
import styles from "./HSLPanel.module.css";

type Mode = "hue" | "saturation" | "luminance";

export function HSLPanel() {
  const { editParams, updateParam } = useDevelopStore();
  const [mode, setMode] = useState<Mode>("hue");
  const paramKey = `hsl_${mode}` as "hsl_hue" | "hsl_saturation" | "hsl_luminance";
  const values = editParams[paramKey];
  const handleChange = (i: number, v: number) => { const nv = [...values]; nv[i] = v; updateParam(paramKey, nv); };

  return (
    <CollapsibleSection title="HSL / Color" defaultOpen={false}>
      <div className={styles.tabs}>
        {(["hue","saturation","luminance"] as Mode[]).map((m) => (
          <button key={m} className={`${styles.tab} ${mode === m ? styles.active : ""}`} onClick={() => setMode(m)}>{m.charAt(0).toUpperCase() + m.slice(1)}</button>
        ))}
      </div>
      {HSL_CHANNELS.map((ch, i) => <AdjustmentSlider key={ch} label={ch} value={values[i]} min={mode === "hue" ? -180 : -100} max={mode === "hue" ? 180 : 100} defaultValue={0} onChange={(v) => handleChange(i, v)} />)}
    </CollapsibleSection>
  );
}
