import type { ColorLabel as ColorLabelType } from "../../types/catalog";
import styles from "./ColorLabel.module.css";

interface Props { value: ColorLabelType; onChange: (color: ColorLabelType) => void; }
const COLORS: ColorLabelType[] = ["none","red","yellow","green","blue","purple"];

export function ColorLabel({ value, onChange }: Props) {
  return (
    <div className={styles.colors}>
      {COLORS.map((color) => (
        <button key={color} className={`${styles.dot} ${value === color ? styles.active : ""}`} style={{ backgroundColor: color === "none" ? "var(--bg-active)" : `var(--color-${color})` }} onClick={(e) => { e.stopPropagation(); onChange(color); }} title={color} />
      ))}
    </div>
  );
}
