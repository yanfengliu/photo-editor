import { useRef, useEffect } from "react";
import { useDevelopStore } from "../../stores/developStore";
import { useDebounce } from "../../hooks/useDebounce";
import styles from "./ImageCanvas.module.css";

export function ImageCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { previewData, previewWidth, previewHeight, editParams, applyEdits, isProcessing } = useDevelopStore();

  const debouncedApply = useDebounce(() => { applyEdits(2048); }, 30);
  useEffect(() => { debouncedApply(); }, [editParams]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !previewData) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const pixelCount = previewData.length / 4;
    const width = previewWidth || Math.round(Math.sqrt(pixelCount));
    const height = previewHeight || Math.round(pixelCount / width);
    canvas.width = width; canvas.height = height;
    const clampedData = new Uint8ClampedArray(previewData.length);
    clampedData.set(previewData);
    const imageData = new ImageData(clampedData, width, height);
    ctx.putImageData(imageData, 0, 0);
  }, [previewData, previewWidth, previewHeight]);

  return (
    <div className={styles.container}>
      <canvas ref={canvasRef} className={styles.canvas} />
      {isProcessing && <div className={styles.processing}>Processing...</div>}
    </div>
  );
}
