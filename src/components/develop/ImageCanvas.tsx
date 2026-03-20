import { useRef, useEffect, useState } from "react";
import { useDevelopStore } from "../../stores/developStore";
import { useDebounce } from "../../hooks/useDebounce";
import { useThrottle } from "../../hooks/useThrottle";
import styles from "./ImageCanvas.module.css";

const DEFAULT_PREVIEW_SIZE = 1024;
const MIN_PREVIEW_SIZE = 512;
const LIVE_PREVIEW_MAX_SIZE = 1280;
const SETTLED_PREVIEW_MAX_SIZE = 2048;
const PREVIEW_BUCKET_SIZE = 256;

function quantizePreviewSize(size: number, maxSize: number): number {
  const bounded = Math.max(MIN_PREVIEW_SIZE, Math.min(maxSize, size));
  return Math.ceil(bounded / PREVIEW_BUCKET_SIZE) * PREVIEW_BUCKET_SIZE;
}

export function ImageCanvas() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const imageDataRef = useRef<ImageData | null>(null);
  const {
    previewData,
    previewWidth,
    previewHeight,
    editParams,
    applyEdits,
    persistEdits,
    isProcessing,
    isAdjusting,
  } = useDevelopStore();
  const [previewSize, setPreviewSize] = useState(DEFAULT_PREVIEW_SIZE);

  useEffect(() => {
    const updatePreviewSize = () => {
      const container = containerRef.current;
      if (!container) return;

      const maxDimension = Math.max(
        container.clientWidth,
        container.clientHeight
      );
      if (maxDimension <= 0) return;

      const deviceScale =
        typeof window === "undefined"
          ? 1
          : Math.min(window.devicePixelRatio || 1, isAdjusting ? 1 : 1.5);
      const overscan = isAdjusting ? 1 : 1.15;
      const maxSize = isAdjusting
        ? LIVE_PREVIEW_MAX_SIZE
        : SETTLED_PREVIEW_MAX_SIZE;
      const nextSize = quantizePreviewSize(
        Math.ceil(maxDimension * deviceScale * overscan),
        maxSize
      );

      setPreviewSize((current) => (current === nextSize ? current : nextSize));
    };

    updatePreviewSize();

    const container = containerRef.current;
    if (!container) return;

    if (typeof ResizeObserver !== "undefined") {
      const observer = new ResizeObserver(updatePreviewSize);
      observer.observe(container);
      return () => observer.disconnect();
    }

    window.addEventListener("resize", updatePreviewSize);
    return () => window.removeEventListener("resize", updatePreviewSize);
  }, [isAdjusting]);

  const throttledApply = useThrottle(() => {
    applyEdits(previewSize);
  }, 30);
  const debouncedPersist = useDebounce(() => {
    persistEdits();
  }, 250);

  useEffect(() => {
    throttledApply();
    if (!isAdjusting) debouncedPersist();
  }, [editParams, isAdjusting, previewSize, throttledApply, debouncedPersist]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !previewData) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const pixelCount = previewData.length / 4;
    const width = previewWidth || Math.round(Math.sqrt(pixelCount));
    const height = previewHeight || Math.round(pixelCount / width);
    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
      imageDataRef.current = new ImageData(width, height);
    }
    if (!imageDataRef.current) {
      imageDataRef.current = new ImageData(width, height);
    }
    const imageData = imageDataRef.current;
    imageData.data.set(previewData);
    ctx.putImageData(imageData, 0, 0);
  }, [previewData, previewWidth, previewHeight]);

  return (
    <div ref={containerRef} className={styles.container}>
      <canvas ref={canvasRef} className={styles.canvas} />
      {isProcessing && <div className={styles.processing}>Processing...</div>}
    </div>
  );
}
