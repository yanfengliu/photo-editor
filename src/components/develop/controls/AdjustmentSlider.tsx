import { useCallback } from "react";
import { useDevelopStore } from "../../../stores/developStore";
import styles from "./AdjustmentSlider.module.css";

interface Props { label: string; value: number; min: number; max: number; step?: number; defaultValue?: number; onChange: (value: number) => void; }

export function AdjustmentSlider({ label, value, min, max, step = 1, defaultValue = 0, onChange }: Props) {
  const { startAdjusting, stopAdjusting } = useDevelopStore();
  const handleDoubleClick = useCallback(() => onChange(defaultValue), [defaultValue, onChange]);
  const handleValueChange = useCallback((nextValue: string) => {
    onChange(parseFloat(nextValue));
  }, [onChange]);
  const handleInteractionStart = useCallback(() => {
    startAdjusting();
  }, [startAdjusting]);
  const handleInteractionEnd = useCallback(() => {
    stopAdjusting();
  }, [stopAdjusting]);

  return (
    <div className={styles.slider}>
      <div className={styles.header}>
        <span className={styles.label}>{label}</span>
        <span className={styles.value} onDoubleClick={handleDoubleClick}>{Number.isInteger(step) ? value.toFixed(0) : value.toFixed(1)}</span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onInput={(e) => handleValueChange(e.currentTarget.value)}
        onChange={(e) => handleValueChange(e.currentTarget.value)}
        onPointerDown={handleInteractionStart}
        onPointerUp={handleInteractionEnd}
        onPointerCancel={handleInteractionEnd}
        onKeyDown={handleInteractionStart}
        onKeyUp={handleInteractionEnd}
        onBlur={handleInteractionEnd}
        onDoubleClick={handleDoubleClick}
      />
    </div>
  );
}
