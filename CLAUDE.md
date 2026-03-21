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
- **Always write tests for changes.** Both unit tests (stores, hooks, components) and functional tests (integration flows).
- Frontend tests use Vitest + React Testing Library. Run with `npx vitest run`.
- Rust tests use `#[cfg(test)]` modules. Run with `cd src-tauri && cargo test`.
- Mock Tauri `invoke` calls via `vi.mock("../../api/...")` with `vi.hoisted` for mock data.
- Test undo/redo flows end-to-end: edit → undo → redo → verify state at each step.

### Code Quality
- Extract shared logic into hooks (e.g., `useThumbnail` for thumbnail loading) — don't duplicate.
- Delete dead code rather than commenting it out or leaving unused exports.
- Keep components focused. If a component file exceeds ~150 lines, consider extracting sub-components or hooks.
- Use CSS variables from `variables.css` for all colors, sizes, and transitions.
- Style native elements (`select`, `option`, `input`) in `global.css` to match the dark theme.

### Undo/Redo
- Slider drags are coalesced into a single undo entry using `startAdjusting`/`stopAdjusting`.
- During `isAdjusting`, `updateParam` skips the undo stack. The pre-drag state is pushed on `stopAdjusting`.
- Keyboard shortcuts use `e.key.toLowerCase()` for case-insensitive matching (Shift changes key casing).
- No history panel UI — undo/redo is keyboard-only (Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y).

### Lens Profiles
- Lens profile data comes from the lensfun open-source project, NOT hardcoded values.
- The PTLens distortion model (a, b, c coefficients) is used, not Brown-Conrady (k1, k2, k3).
- Vignette correction uses the widest aperture measurement per focal length for worst-case correction.
- TCA poly3 model is approximated as linear using vr/vb coefficients.

### Frontend Patterns
- Controlled `<input type="range">` sliders call `startAdjusting` on pointerDown, `stopAdjusting` on pointerUp.
- SVG coordinate conversion must account for CSS display size vs. SVG internal coordinate system.
- `VirtuosoGrid` for virtualized thumbnail rendering with `overscan={200}`.

## Bug Fix Log

### CurveEditor coordinate mismatch
**Problem**: Double-clicking to add curve points placed them at wrong positions. Clicks past midpoint were clamped to endpoints.
**Root cause**: `fromScreen()` divided mouse coordinates by the SVG's internal `SIZE` (200px) but the CSS displayed the SVG at a larger size. Mouse coordinates are in displayed pixels, not SVG pixels.
**Fix**: Scale mouse coords by `(displayedPos / rect.width) * SIZE` before converting to curve coordinates.

### Redo keyboard shortcut broken (Ctrl+Shift+Z)
**Problem**: Redo via Ctrl+Shift+Z did nothing.
**Root cause**: `e.key` with Shift held is `"Z"` (uppercase), but handler compared against lowercase `"z"`.
**Fix**: Use `e.key.toLowerCase()` for all key comparisons.

### Undo too granular / redo wiped during slider drag
**Problem**: Each slider tick pushed to undo stack and cleared redo stack. Dragging exposure from 0 to 2 created dozens of entries.
**Root cause**: `updateParam` unconditionally pushed to undo stack on every call.
**Fix**: Added `adjustStartParams` field. During `isAdjusting`, `updateParam` skips the undo stack. `stopAdjusting` commits the entire drag as one entry.

### FilmStrip thumbnails not loading
**Problem**: FilmStrip showed filenames but no thumbnail images.
**Root cause**: `FilmStrip` component never called `loadThumbnail` — it only rendered `<span>{filename}</span>`.
**Fix**: Added `useThumbnail` hook (extracted from `ThumbnailCard`) and used it in `FilmStripThumb`.

### Fake lens correction profiles
**Problem**: Hardcoded lens profiles with fabricated coefficients — not from any measured data source.
**Fix**: Integrated lensfun's open-source XML database (1000+ measured lens profiles). Added XML parser, download script, and compile-time embedding via `build.rs`.
