import type { Flag } from "../../types/catalog";
import styles from "./FlagToggle.module.css";

interface Props { value: Flag; onChange: (flag: Flag) => void; size?: "small" | "normal"; }

export function FlagToggle({ value, onChange, size = "normal" }: Props) {
  return (
    <div className={`${styles.flags} ${size === "small" ? styles.small : ""}`}>
      <button className={`${styles.flag} ${value === "picked" ? styles.picked : ""}`} onClick={(e) => { e.stopPropagation(); onChange(value === "picked" ? "none" : "picked"); }} title="Pick (P)">▲</button>
      <button className={`${styles.flag} ${value === "rejected" ? styles.rejected : ""}`} onClick={(e) => { e.stopPropagation(); onChange(value === "rejected" ? "none" : "rejected"); }} title="Reject (X)">▼</button>
    </div>
  );
}
