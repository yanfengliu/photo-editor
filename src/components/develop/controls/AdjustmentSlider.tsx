import { useCallback } from "react";
import styles from "./AdjustmentSlider.module.css";

interface Props { label: string; value: number; min: number; max: number; step?: number; defaultValue?: number; onChange: (value: number) => void; }

export function AdjustmentSlider({ label, value, min, max, step = 1, defaultValue = 0, onChange }: Props) {
  const handleDoubleClick = useCallback(() => onChange(defaultValue), [defaultValue, onChange]);
  return (
    <div className={styles.slider}>
      <div className={styles.header}>
        <span className={styles.label}>{label}</span>
        <span className={styles.value} onDoubleClick={handleDoubleClick}>{Number.isInteger(step) ? value.toFixed(0) : value.toFixed(1)}</span>
      </div>
      <input type="range" min={min} max={max} step={step} value={value} onChange={(e) => onChange(parseFloat(e.target.value))} onDoubleClick={handleDoubleClick} />
    </div>
  );
}
