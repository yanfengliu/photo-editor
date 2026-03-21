# Photo Editor — Open-Source Lightroom Replacement

## Stack

- **Desktop**: Tauri 2.x (Rust backend, webview frontend)
- **Frontend**: React 19 + TypeScript + Vite + Zustand
- **GPU**: wgpu + WGSL compute shaders (Vulkan/Metal/DX12)
- **RAW**: quickraw crate for RAW demosaicing
- **Database**: SQLite (rusqlite, WAL mode) for catalog
- **AI Denoise**: ONNX Runtime (ort crate) with lightweight U-Net
- **Image codecs**: image crate (JPEG/PNG/TIFF)
- **Lens profiles**: lensfun XML database (CC-BY-SA 3.0), parsed with quick-xml

## Architecture

### GPU Processing Pipeline

Ping-pong texture strategy with rgba32float storage textures. Each edit operation is a WGSL compute shader pass. Passes with neutral/default values are skipped for performance.

```
RAW/Image → demosaic → linear f32 → GPU upload
  → White Balance → Basic (exp/contrast/HL/SH/WH/BK)
  → Tone Curve (LUT) → HSL → Sharpen → Clarity
  → Denoise → Dehaze → Vignette → Grain
  → Tonemap (linear→sRGB) → readback → display
```

Preview strategy: Slider drag → process downscaled proxy (<16ms). Slider release → full-res process + save to DB.

### Non-destructive Editing

Edit params stored as JSON in SQLite. Original files never modified. Full undo/redo history, snapshots, and presets.

### Data Transfer (Rust <-> Frontend)

`apply_edits` Tauri command returns binary IPC: `[u32 width][u32 height][RGBA bytes...]`. Frontend renders to `<canvas>` via `putImageData`. Throttled at ~30ms during slider drag.

### Lens Correction

Uses lensfun's open-source measured lens profile database (1000+ lenses). PTLens distortion model: `Rd = a*Ru^4 + b*Ru^3 + c*Ru^2 + (1-a-b-c)*Ru`. Run `scripts/fetch-lensfun.sh` to download the XML database. Data is embedded at compile time via `build.rs`.

## Project Structure

Keep this diagram updated.

```
photo-editor/
├── src/                              # React frontend
│   ├── api/                          # Tauri invoke wrappers
│   ├── stores/                       # Zustand (catalogStore, developStore, uiStore)
│   ├── components/
│   │   ├── layout/                   # AppShell, TopToolbar, StatusBar, FilmStrip
│   │   ├── library/                  # LibraryView, ThumbnailGrid, FolderTree, etc.
│   │   ├── develop/                  # DevelopView, ImageCanvas
│   │   │   ├── panels/              # BasicAdjustments, WhiteBalance, ToneCurve, HSL, Detail, LensCorrection, Effects
│   │   │   └── controls/            # AdjustmentSlider, CurveEditor, CollapsibleSection
│   │   ├── export/                   # ExportDialog, BatchExportDialog
│   │   └── common/                   # Rating, ColorLabel, FlagToggle, SearchInput, Modal
│   ├── hooks/                        # useKeyboardShortcuts, useDebounce, useThrottle, useThumbnail
│   ├── types/                        # catalog.ts, develop.ts, export.ts, metadata.ts
│   └── styles/                       # variables.css, global.css
│
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── commands/                 # Tauri IPC handlers
│   │   ├── catalog/                  # DB layer (db, models, queries, import, search)
│   │   ├── imaging/                  # Image I/O, lens_profiles (lensfun parser), lens_correction
│   │   └── gpu/                      # wgpu pipeline, shaders/, passes/
│   └── data/lensfun/                 # Downloaded lensfun XML files (gitignored)
│
├── scripts/
│   └── fetch-lensfun.sh              # Downloads lensfun database
└── src/test/                         # Vitest tests (components, stores, hooks, functional)
```

## Best Practices

### Testing
- Write unit tests and integration tests for changes if applicable.
- Run tests and linter after any change. Make sure they pass.

### Code Quality
- Extract shared logic into hooks (e.g., `useThumbnail` for thumbnail loading) — don't duplicate.
- Delete dead code rather than commenting it out or leaving unused exports.
- Keep components focused. If a component file exceeds ~150 lines, consider extracting sub-components or hooks.
- Use CSS variables from `variables.css` for all colors, sizes, and transitions.
- Style native elements (`select`, `option`, `input`) in `global.css` to match the dark theme.
