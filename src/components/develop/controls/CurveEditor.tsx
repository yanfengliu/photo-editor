import { useState, useRef, useCallback } from "react";
import type { CurvePoint } from "../../../types/develop";
import styles from "./CurveEditor.module.css";

interface Props { points: CurvePoint[]; onChange: (points: CurvePoint[]) => void; color?: string; }
const SIZE = 200, PAD = 8;

export function CurveEditor({ points, onChange, color = "white" }: Props) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [dragging, setDragging] = useState<number | null>(null);
  const toScreen = (p: CurvePoint) => ({ x: PAD + p.x * (SIZE - 2 * PAD), y: PAD + (1 - p.y) * (SIZE - 2 * PAD) });
  const fromScreen = (sx: number, sy: number): CurvePoint => ({ x: Math.max(0, Math.min(1, (sx - PAD) / (SIZE - 2 * PAD))), y: Math.max(0, Math.min(1, 1 - (sy - PAD) / (SIZE - 2 * PAD))) });
  const pathD = points.map((p, i) => { const s = toScreen(p); return `${i === 0 ? "M" : "L"} ${s.x} ${s.y}`; }).join(" ");

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (dragging === null || !svgRef.current) return;
    const rect = svgRef.current.getBoundingClientRect();
    const pt = fromScreen(e.clientX - rect.left, e.clientY - rect.top);
    const newPoints = [...points];
    if (dragging === 0) pt.x = 0; else if (dragging === points.length - 1) pt.x = 1;
    newPoints[dragging] = pt; onChange(newPoints);
  }, [dragging, points, onChange]);

  const handleDoubleClick = (e: React.MouseEvent) => {
    if (!svgRef.current) return;
    const rect = svgRef.current.getBoundingClientRect();
    const pt = fromScreen(e.clientX - rect.left, e.clientY - rect.top);
    onChange([...points, pt].sort((a, b) => a.x - b.x));
  };

  return (
    <svg ref={svgRef} width={SIZE} height={SIZE} className={styles.svg} onMouseMove={handleMouseMove} onMouseUp={() => setDragging(null)} onMouseLeave={() => setDragging(null)} onDoubleClick={handleDoubleClick}>
      <line x1={PAD} y1={SIZE - PAD} x2={SIZE - PAD} y2={PAD} className={styles.diagonal} />
      <path d={pathD} fill="none" stroke={color} strokeWidth={2} />
      {points.map((p, i) => { const s = toScreen(p); return <circle key={i} cx={s.x} cy={s.y} r={5} fill={dragging === i ? color : "transparent"} stroke={color} strokeWidth={2} className={styles.point} onMouseDown={(e) => { e.preventDefault(); setDragging(i); }} />; })}
    </svg>
  );
}
