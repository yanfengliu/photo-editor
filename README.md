# Photo Editor

Free, open-source photo editor built to replace Adobe Lightroom. Desktop app powered by Tauri, React, and GPU-accelerated image processing.

![Demo](imgs/demo.png)

## Features

### Import & Library
- **RAW support** — Canon (CR2, CR3), Nikon (NEF), Sony (ARW), DNG, Olympus (ORF), Panasonic (RW2), Fujifilm (RAF), Pentax (PEF)
- **Standard formats** — JPEG, PNG, TIFF, BMP, WebP
- **EXIF extraction** — camera, lens, ISO, focal length, aperture, shutter speed, date
- **Ratings** (1-5 stars), **color labels** (6 colors), **flags** (picked/rejected)
- **Tags** and **collections** (nested hierarchy)
- **Search & filter** — by filename, rating, flag, color label
- **Multi-select** with bulk rating, flagging, and deletion

### Editing (Non-Destructive)
- **White Balance** — temperature (2000-12000K) and tint, based on Planckian locus
- **Basic Adjustments** — exposure, contrast, highlights, shadows, whites, blacks, saturation, vibrance (with skin-tone protection)
- **Tone Curves** — RGB master + individual R/G/B channels, interactive point editor, monotone cubic spline interpolation
- **HSL** — per-channel hue, saturation, and luminance for 8 color ranges
- **Sharpening** — amount, radius, detail threshold (unsharp mask)
- **Clarity** — local contrast via guided image filter
- **Noise Reduction** — separate luminance and color denoising (bilateral filter)
- **Dehaze** — dark channel prior (He et al. 2009) with guided filter refinement
- **Vignette** and **grain** effects
- **Crop & Rotate** — draggable edges/corners, aspect ratio locking (1:1, 4:3, 3:2, 16:9, 5:4, 7:5), 90/180 degree rotation, fine rotation (-45 to +45 degrees)
- **Lens Correction** — 1000+ lens profiles (lensfun database), auto-detection from EXIF, distortion/CA/vignette correction

### Undo/Redo
- Full undo/redo stack with keyboard shortcuts (Ctrl+Z / Ctrl+Y)
- Slider drags coalesced into single undo steps
- Copy/paste edits between images

### Export
- JPEG (quality slider), PNG, TIFF
- Batch export with auto-naming
- Max dimension scaling
- XMP sidecar export (Lightroom-compatible)

### Performance
- **GPU-accelerated** — wgpu compute shaders (Vulkan/Metal/DX12) with CPU fallback
- Live preview during slider drag (~30ms throttle)
- Full-resolution processing on slider release
- Adaptive preview resolution (512-2048px)
- Binary IPC for fast Rust-to-frontend data transfer

### Keyboard Shortcuts
| Key | Action |
|-----|--------|
| G | Library view |
| D | Develop view |
| 1-5 | Set rating |
| P | Flag as picked |
| X | Flag as rejected |
| Ctrl+Z | Undo |
| Ctrl+Y / Ctrl+Shift+Z | Redo |

## Stack

- **Desktop** — Tauri 2 (Rust backend + webview frontend)
- **Frontend** — React 19, TypeScript, Vite, Zustand
- **GPU** — wgpu + WGSL compute shaders
- **RAW** — quickraw crate
- **Database** — SQLite (WAL mode)
- **Lens profiles** — lensfun (CC-BY-SA 3.0)

## Getting Started

```bash
# Install dependencies
npm install

# Download lensfun lens profile database
bash scripts/fetch-lensfun.sh

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

## Testing

```bash
# Frontend tests
npx vitest run

# Backend tests
cd src-tauri && cargo test

# Lint
cd src-tauri && cargo clippy -- -D warnings
```
