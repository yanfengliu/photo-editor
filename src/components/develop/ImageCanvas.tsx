import { useRef, useEffect, useState, useCallback } from "react";
import { useDevelopStore } from "../../stores/developStore";
import { useDebounce } from "../../hooks/useDebounce";
import { useThrottle } from "../../hooks/useThrottle";
import styles from "./ImageCanvas.module.css";

const DEFAULT_PREVIEW_SIZE = 1024;
const MIN_PREVIEW_SIZE = 512;
const LIVE_PREVIEW_MAX_SIZE = 1280;
const SETTLED_PREVIEW_MAX_SIZE = 2048;
const PREVIEW_BUCKET_SIZE = 256;
const MAX_ZOOM = 16;

function quantizePreviewSize(size: number, maxSize: number): number {
  const bounded = Math.max(MIN_PREVIEW_SIZE, Math.min(maxSize, size));
  return Math.ceil(bounded / PREVIEW_BUCKET_SIZE) * PREVIEW_BUCKET_SIZE;
}

interface ViewTransform {
  zoom: number;
  offsetX: number;
  offsetY: number;
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
  const currentImageId = useDevelopStore((s) => s.currentImageId);
  const [previewSize, setPreviewSize] = useState(DEFAULT_PREVIEW_SIZE);
  const [containerSize, setContainerSize] = useState({ w: 0, h: 0 });

  // null = fit-to-container (centered, zoom 1x)
  const [view, setView] = useState<ViewTransform | null>(null);
  const isPanningRef = useRef(false);
  const panLastRef = useRef({ x: 0, y: 0 });

  // Reset view on image change
  useEffect(() => {
    setView(null);
  }, [currentImageId]);

  // --- Container resize observer ---
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const update = () => {
      const cw = container.clientWidth;
      const ch = container.clientHeight;
      setContainerSize({ w: cw, h: ch });
      const maxDim = Math.max(cw, ch);
      if (maxDim <= 0) return;
      const deviceScale =
        typeof window === "undefined"
          ? 1
          : Math.min(window.devicePixelRatio || 1, isAdjusting ? 1 : 1.5);
      const overscan = isAdjusting ? 1 : 1.15;
      const maxSize = isAdjusting
        ? LIVE_PREVIEW_MAX_SIZE
        : SETTLED_PREVIEW_MAX_SIZE;
      setPreviewSize((cur) => {
        const next = quantizePreviewSize(
          Math.ceil(maxDim * deviceScale * overscan),
          maxSize
        );
        return cur === next ? cur : next;
      });
    };
    update();
    if (typeof ResizeObserver !== "undefined") {
      const ob = new ResizeObserver(update);
      ob.observe(container);
      return () => ob.disconnect();
    }
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, [isAdjusting]);

  // --- Compute display transform ---
  // fitScale: the scale that makes the image exactly fit the container
  const imageW = previewWidth || 1;
  const imageH = previewHeight || 1;
  const fitScale =
    containerSize.w > 0 && containerSize.h > 0
      ? Math.min(containerSize.w / imageW, containerSize.h / imageH)
      : 1;

  const currentZoom = view?.zoom ?? 1;
  const displayScale = fitScale * currentZoom;

  // Centered offset at zoom=1 (fit view)
  const centeredOffset = {
    x: (containerSize.w - imageW * fitScale) / 2,
    y: (containerSize.h - imageH * fitScale) / 2,
  };
  const offset = view
    ? { x: view.offsetX, y: view.offsetY }
    : centeredOffset;

  // Refs for event handlers to avoid stale closures
  const fitScaleRef = useRef(fitScale);
  const centeredOffsetRef = useRef(centeredOffset);
  const viewRef = useRef(view);
  fitScaleRef.current = fitScale;
  centeredOffsetRef.current = centeredOffset;
  viewRef.current = view;

  // --- Scroll-wheel zoom (native listener for passive:false) ---
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const rect = container.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;

      // Normalize deltaY across input devices
      let delta = e.deltaY;
      if (e.deltaMode === 1) delta *= 33; // line mode → pixels
      if (e.deltaMode === 2) delta *= rect.height; // page mode → pixels

      setView((prev) => {
        const prevZoom = prev?.zoom ?? 1;
        const raw = prevZoom * Math.pow(1.002, -delta);
        const newZoom = Math.min(MAX_ZOOM, Math.max(0.1, raw));

        // Snap back to fit when zooming out through 1x
        if (newZoom <= 1.0 && prevZoom > 1.0 && delta > 0) return null;
        if (Math.abs(newZoom - prevZoom) < 0.001) return prev;

        const fs = fitScaleRef.current;
        const oldScale = fs * prevZoom;
        const newScale = fs * newZoom;

        // Previous offset (centered position when at fit)
        const off = prev
          ? { x: prev.offsetX, y: prev.offsetY }
          : centeredOffsetRef.current;

        // Image-space point under cursor
        const imgX = (mx - off.x) / oldScale;
        const imgY = (my - off.y) / oldScale;

        // New offset keeps that point under cursor
        return {
          zoom: newZoom,
          offsetX: mx - imgX * newScale,
          offsetY: my - imgY * newScale,
        };
      });
    };

    container.addEventListener("wheel", onWheel, { passive: false });
    return () => container.removeEventListener("wheel", onWheel);
  }, []);

  // --- Edit processing ---
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

  // --- Render pixels to canvas ---
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !previewData) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const pixelCount = previewData.length / 4;
    const w = previewWidth || Math.round(Math.sqrt(pixelCount));
    const h = previewHeight || Math.round(pixelCount / w);
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
      imageDataRef.current = new ImageData(w, h);
    }
    if (!imageDataRef.current) {
      imageDataRef.current = new ImageData(w, h);
    }
    imageDataRef.current.data.set(previewData);
    ctx.putImageData(imageDataRef.current, 0, 0);
  }, [previewData, previewWidth, previewHeight]);

  // --- Pan handlers ---
  const handlePointerDown = useCallback((e: React.PointerEvent) => {
    if (!viewRef.current) return; // no pan at fit view
    if (e.button !== 0 && e.button !== 1) return;
    e.preventDefault();
    isPanningRef.current = true;
    panLastRef.current = { x: e.clientX, y: e.clientY };
    containerRef.current?.setPointerCapture(e.pointerId);
  }, []);

  const handlePointerMove = useCallback((e: React.PointerEvent) => {
    if (!isPanningRef.current) return;
    const dx = e.clientX - panLastRef.current.x;
    const dy = e.clientY - panLastRef.current.y;
    panLastRef.current = { x: e.clientX, y: e.clientY };
    setView((prev) =>
      prev
        ? { ...prev, offsetX: prev.offsetX + dx, offsetY: prev.offsetY + dy }
        : prev
    );
  }, []);

  const handlePointerUp = useCallback(() => {
    isPanningRef.current = false;
  }, []);

  // Double-click resets to fit
  const handleDoubleClick = useCallback(() => {
    setView(null);
  }, []);

  const isZoomed = view !== null;

  return (
    <div
      ref={containerRef}
      className={styles.container}
      style={{ cursor: isZoomed ? "grab" : undefined }}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerUp}
      onDoubleClick={handleDoubleClick}
    >
      <canvas
        ref={canvasRef}
        className={styles.canvas}
        style={{
          transform: `translate(${offset.x}px, ${offset.y}px) scale(${displayScale})`,
          imageRendering: displayScale > 2 ? "pixelated" : undefined,
        }}
      />
      {isProcessing && <div className={styles.processing}>Processing...</div>}
      {isZoomed && (
        <div className={styles.zoomBadge}>
          {Math.round(currentZoom * 100)}%
        </div>
      )}
    </div>
  );
}
