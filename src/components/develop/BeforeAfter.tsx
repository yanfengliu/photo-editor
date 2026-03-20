import { useState, useRef } from "react";
import styles from "./BeforeAfter.module.css";

interface Props { beforeData: Uint8Array | null; afterData: Uint8Array | null; width: number; height: number; }

export function BeforeAfter({ }: Props) {
  const [splitPos, setSplitPos] = useState(50);
  const containerRef = useRef<HTMLDivElement>(null);
  const handleMouseMove = (e: React.MouseEvent) => {
    if (!containerRef.current || e.buttons !== 1) return;
    const rect = containerRef.current.getBoundingClientRect();
    setSplitPos(Math.max(0, Math.min(100, ((e.clientX - rect.left) / rect.width) * 100)));
  };

  return (
    <div ref={containerRef} className={styles.container} onMouseMove={handleMouseMove}>
      <div className={styles.before} style={{ clipPath: `inset(0 ${100 - splitPos}% 0 0)` }}><span className={styles.label}>Before</span></div>
      <div className={styles.after} style={{ clipPath: `inset(0 0 0 ${splitPos}%)` }}><span className={styles.label}>After</span></div>
      <div className={styles.divider} style={{ left: `${splitPos}%` }} />
    </div>
  );
}
