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
const MIN_CROP_SIZE = 0.02; // Minimum crop dimension (2% of image)

function quantizePreviewSize(size: number, maxSize: number): number {
  const bounded = Math.max(MIN_PREVIEW_SIZE, Math.min(maxSize, size));
  return Math.ceil(bounded / PREVIEW_BUCKET_SIZE) * PREVIEW_BUCKET_SIZE;
}

interface ViewTransform {
  zoom: number;
  offsetX: number;
  offsetY: number;
}

type DragHandle = "n" | "s" | "e" | "w" | "nw" | "ne" | "sw" | "se" | "move";

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
    updateParam,
    startAdjusting,
    stopAdjusting,
  } = useDevelopStore();
  const currentImageId = useDevelopStore((s) => s.currentImageId);
  const [previewSize, setPreviewSize] = useState(DEFAULT_PREVIEW_SIZE);
  const [containerSize, setContainerSize] = useState({ w: 0, h: 0 });

  // null = fit-to-container (centered, zoom 1x)
  const [view, setView] = useState<ViewTransform | null>(null);
  const isPanningRef = useRef(false);
  const panLastRef = useRef({ x: 0, y: 0 });

  // Crop drag state
  const [cropDrag, setCropDrag] = useState<{
    handle: DragHandle;
    startX: number;
    startY: number;
    startCrop: { x: number; y: number; w: number; h: number };
  } | null>(null);

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
  const imageW = previewWidth || 1;
  const imageH = previewHeight || 1;
  const fitScale =
    containerSize.w > 0 && containerSize.h > 0
      ? Math.min(containerSize.w / imageW, containerSize.h / imageH)
      : 1;

  const currentZoom = view?.zoom ?? 1;
  const displayScale = fitScale * currentZoom;

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

      let delta = e.deltaY;
      if (e.deltaMode === 1) delta *= 33;
      if (e.deltaMode === 2) delta *= rect.height;

      setView((prev) => {
        const prevZoom = prev?.zoom ?? 1;
        const raw = prevZoom * Math.pow(1.002, -delta);
        const newZoom = Math.min(MAX_ZOOM, Math.max(0.1, raw));

        if (newZoom <= 1.0 && prevZoom > 1.0 && delta > 0) return null;
        if (Math.abs(newZoom - prevZoom) < 0.001) return prev;

        const fs = fitScaleRef.current;
        const oldScale = fs * prevZoom;
        const newScale = fs * newZoom;

        const off = prev
          ? { x: prev.offsetX, y: prev.offsetY }
          : centeredOffsetRef.current;

        const imgX = (mx - off.x) / oldScale;
        const imgY = (my - off.y) / oldScale;

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

  // --- Crop drag helpers ---
  // Convert a container pixel delta to normalized [0,1] image delta
  const pxToNorm = useCallback(
    (dxPx: number, dyPx: number) => ({
      dx: dxPx / (imageW * displayScale),
      dy: dyPx / (imageH * displayScale),
    }),
    [imageW, imageH, displayScale]
  );

  const handleCropPointerDown = useCallback(
    (handle: DragHandle, e: React.PointerEvent) => {
      e.stopPropagation();
      e.preventDefault();
      startAdjusting();
      setCropDrag({
        handle,
        startX: e.clientX,
        startY: e.clientY,
        startCrop: {
          x: editParams.crop_x,
          y: editParams.crop_y,
          w: editParams.crop_width,
          h: editParams.crop_height,
        },
      });
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
    },
    [editParams, startAdjusting]
  );

  const handleCropPointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!cropDrag) return;
      const { handle, startX, startY, startCrop } = cropDrag;
      const { dx, dy } = pxToNorm(e.clientX - startX, e.clientY - startY);

      let { x, y, w, h } = startCrop;

      switch (handle) {
        case "move":
          x = Math.max(0, Math.min(1 - w, x + dx));
          y = Math.max(0, Math.min(1 - h, y + dy));
          break;
        case "nw":
          x = Math.max(0, Math.min(x + w - MIN_CROP_SIZE, x + dx));
          y = Math.max(0, Math.min(y + h - MIN_CROP_SIZE, y + dy));
          w = startCrop.x + startCrop.w - x;
          h = startCrop.y + startCrop.h - y;
          break;
        case "ne":
          w = Math.max(MIN_CROP_SIZE, Math.min(1 - x, w + dx));
          y = Math.max(0, Math.min(y + h - MIN_CROP_SIZE, y + dy));
          h = startCrop.y + startCrop.h - y;
          break;
        case "sw":
          x = Math.max(0, Math.min(x + w - MIN_CROP_SIZE, x + dx));
          w = startCrop.x + startCrop.w - x;
          h = Math.max(MIN_CROP_SIZE, Math.min(1 - y, h + dy));
          break;
        case "se":
          w = Math.max(MIN_CROP_SIZE, Math.min(1 - x, w + dx));
          h = Math.max(MIN_CROP_SIZE, Math.min(1 - y, h + dy));
          break;
        case "n":
          y = Math.max(0, Math.min(y + h - MIN_CROP_SIZE, y + dy));
          h = startCrop.y + startCrop.h - y;
          break;
        case "s":
          h = Math.max(MIN_CROP_SIZE, Math.min(1 - y, h + dy));
          break;
        case "w":
          x = Math.max(0, Math.min(x + w - MIN_CROP_SIZE, x + dx));
          w = startCrop.x + startCrop.w - x;
          break;
        case "e":
          w = Math.max(MIN_CROP_SIZE, Math.min(1 - x, w + dx));
          break;
      }

      updateParam("crop_x", parseFloat(x.toFixed(4)));
      updateParam("crop_y", parseFloat(y.toFixed(4)));
      updateParam("crop_width", parseFloat(w.toFixed(4)));
      updateParam("crop_height", parseFloat(h.toFixed(4)));
    },
    [cropDrag, pxToNorm, updateParam]
  );

  const handleCropPointerUp = useCallback(() => {
    if (!cropDrag) return;
    setCropDrag(null);
    stopAdjusting();
  }, [cropDrag, stopAdjusting]);

  // --- Pan handlers (only when not crop-dragging) ---
  const handlePointerDown = useCallback((e: React.PointerEvent) => {
    if (!viewRef.current) return;
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

  const handleDoubleClick = useCallback(() => {
    setView(null);
  }, []);

  const isZoomed = view !== null;
  const fineAngle = editParams.rotation_fine || 0;

  // --- Crop overlay geometry ---
  const isCropped =
    editParams.crop_x !== 0 ||
    editParams.crop_y !== 0 ||
    editParams.crop_width !== 1 ||
    editParams.crop_height !== 1;
  const showCrop = isCropped || cropDrag !== null;

  // Crop box in container pixel coordinates
  const cropLeft = offset.x + editParams.crop_x * imageW * displayScale;
  const cropTop = offset.y + editParams.crop_y * imageH * displayScale;
  const cropW = editParams.crop_width * imageW * displayScale;
  const cropH = editParams.crop_height * imageH * displayScale;

  // Image bounds in container coordinates
  const imgLeft = offset.x;
  const imgTop = offset.y;
  const imgRight = offset.x + imageW * displayScale;
  const imgBottom = offset.y + imageH * displayScale;

  // Fine rotation overlay (inscribed rect)
  const absAngle = Math.abs(fineAngle * Math.PI / 180);
  const sinA = Math.sin(absAngle);
  const cosA = Math.cos(absAngle);
  let inscribedScale = 1;
  if (absAngle > 0.001 && imageW > 0 && imageH > 0) {
    inscribedScale = Math.min(
      imageW / (imageW * cosA + imageH * sinA),
      imageH / (imageW * sinA + imageH * cosA)
    );
  }

  const displayW = imageW * displayScale;
  const displayH = imageH * displayScale;
  const rotCropW = displayW * inscribedScale;
  const rotCropH = displayH * inscribedScale;
  const imgCenterX = offset.x + displayW / 2;
  const imgCenterY = offset.y + displayH / 2;

  const canvasTransform = fineAngle
    ? `translate(${offset.x}px, ${offset.y}px) translate(${displayW / 2}px, ${displayH / 2}px) rotate(${fineAngle}deg) translate(${-displayW / 2}px, ${-displayH / 2}px) scale(${displayScale})`
    : `translate(${offset.x}px, ${offset.y}px) scale(${displayScale})`;

  return (
    <div
      ref={containerRef}
      className={styles.container}
      style={{ cursor: cropDrag ? undefined : isZoomed ? "grab" : undefined }}
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
          transform: canvasTransform,
          imageRendering: displayScale > 2 ? "pixelated" : undefined,
        }}
      />

      {/* Interactive crop overlay */}
      {showCrop && (
        <>
          {/* Darkened regions outside the crop box (4 rects) */}
          {/* Top */}
          <div className={styles.cropDarken} style={{ left: imgLeft, top: imgTop, width: imgRight - imgLeft, height: Math.max(0, cropTop - imgTop) }} />
          {/* Bottom */}
          <div className={styles.cropDarken} style={{ left: imgLeft, top: cropTop + cropH, width: imgRight - imgLeft, height: Math.max(0, imgBottom - (cropTop + cropH)) }} />
          {/* Left */}
          <div className={styles.cropDarken} style={{ left: imgLeft, top: cropTop, width: Math.max(0, cropLeft - imgLeft), height: cropH }} />
          {/* Right */}
          <div className={styles.cropDarken} style={{ left: cropLeft + cropW, top: cropTop, width: Math.max(0, imgRight - (cropLeft + cropW)), height: cropH }} />

          {/* Crop border */}
          <div className={styles.cropBorder} style={{ left: cropLeft, top: cropTop, width: cropW, height: cropH }}>
            {/* Rule of thirds grid */}
            <div className={styles.cropGrid}>
              <div className={styles.cropGridH} style={{ top: "33.33%" }} />
              <div className={styles.cropGridH} style={{ top: "66.67%" }} />
              <div className={styles.cropGridV} style={{ left: "33.33%" }} />
              <div className={styles.cropGridV} style={{ left: "66.67%" }} />
            </div>

            {/* Corner tick marks */}
            <div className={`${styles.cropCorner} ${styles.cropCornerNW}`} />
            <div className={`${styles.cropCorner} ${styles.cropCornerNE}`} />
            <div className={`${styles.cropCorner} ${styles.cropCornerSW}`} />
            <div className={`${styles.cropCorner} ${styles.cropCornerSE}`} />

            {/* Drag handles — invisible hit areas */}
            <div className={`${styles.cropHandle} ${styles.cropHandleN}`} onPointerDown={(e) => handleCropPointerDown("n", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleS}`} onPointerDown={(e) => handleCropPointerDown("s", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleW}`} onPointerDown={(e) => handleCropPointerDown("w", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleE}`} onPointerDown={(e) => handleCropPointerDown("e", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleNW}`} onPointerDown={(e) => handleCropPointerDown("nw", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleNE}`} onPointerDown={(e) => handleCropPointerDown("ne", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleSW}`} onPointerDown={(e) => handleCropPointerDown("sw", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleSE}`} onPointerDown={(e) => handleCropPointerDown("se", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
            <div className={`${styles.cropHandle} ${styles.cropHandleMove}`} onPointerDown={(e) => handleCropPointerDown("move", e)} onPointerMove={handleCropPointerMove} onPointerUp={handleCropPointerUp} onPointerCancel={handleCropPointerUp} />
          </div>
        </>
      )}

      {/* Fine-rotation inscribed rect overlay (shown independently of crop) */}
      {fineAngle !== 0 && !showCrop && (
        <div
          className={styles.cropBorder}
          style={{
            left: imgCenterX - rotCropW / 2,
            top: imgCenterY - rotCropH / 2,
            width: rotCropW,
            height: rotCropH,
            pointerEvents: "none",
            boxShadow: "0 0 0 9999px rgba(0,0,0,0.5)",
          }}
        />
      )}

      {isProcessing && <div className={styles.processing}>Processing...</div>}
      {isZoomed && (
        <div className={styles.zoomBadge}>
          {Math.round(currentZoom * 100)}%
        </div>
      )}
    </div>
  );
}
