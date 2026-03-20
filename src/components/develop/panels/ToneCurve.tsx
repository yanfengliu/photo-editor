import { useState } from "react";
import { useDevelopStore } from "../../../stores/developStore";
import { CollapsibleSection } from "../controls/CollapsibleSection";
import { CurveEditor } from "../controls/CurveEditor";
import styles from "./ToneCurve.module.css";

type Channel = "rgb" | "r" | "g" | "b";
const COLORS: Record<Channel, string> = { rgb: "white", r: "#ff6666", g: "#66ff66", b: "#6666ff" };

export function ToneCurve() {
  const { editParams, updateParam } = useDevelopStore();
  const [channel, setChannel] = useState<Channel>("rgb");
  const curveKey = `curve_${channel}` as keyof typeof editParams;
  const points = editParams[curveKey] as { x: number; y: number }[];

  return (
    <CollapsibleSection title="Tone Curve" defaultOpen={false}>
      <div className={styles.tabs}>
        {(["rgb","r","g","b"] as Channel[]).map((ch) => (
          <button key={ch} className={`${styles.tab} ${channel === ch ? styles.active : ""}`} style={{ color: channel === ch ? COLORS[ch] : undefined }} onClick={() => setChannel(ch)}>{ch.toUpperCase()}</button>
        ))}
      </div>
      <CurveEditor points={points} onChange={(pts) => updateParam(curveKey as any, pts)} color={COLORS[channel]} />
    </CollapsibleSection>
  );
}
