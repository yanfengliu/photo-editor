╭───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
│ Plan to implement                                                                                                                 │
│                                                                                                                                   │
│ Photo Editor — Lightroom Replacement                                                                                              │
│                                                                                                                                   │
│ Context                                                                                                                           │
│                                                                                                                                   │
│ Build a free, open-source replacement for Adobe Lightroom. The app must handle RAW image editing with GPU-accelerated real-time   │
│ adjustments, a full catalog/library system, and AI + traditional denoise. Target: Windows, macOS, Linux.                          │
│                                                                                                                                   │
│ Stack                                                                                                                             │
│                                                                                                                                   │
│ - Desktop: Tauri 2.x (Rust backend, webview frontend)                                                                             │
│ - Frontend: React 19 + TypeScript + Vite + Zustand                                                                                │
│ - GPU: wgpu + WGSL compute shaders (Vulkan/Metal/DX12)                                                                            │
│ - RAW: LibRaw via Rust FFI bindings                                                                                               │
│ - Database: SQLite (rusqlite, WAL mode) for catalog                                                                               │
│ - AI Denoise: ONNX Runtime (ort crate) with lightweight U-Net                                                                     │
│ - Image codecs: image crate (JPEG/PNG/TIFF)                                                                                       │
│                                                                                                                                   │
│ Architecture                                                                                                                      │
│                                                                                                                                   │
│ GPU Processing Pipeline                                                                                                           │
│                                                                                                                                   │
│ Ping-pong texture strategy with rgba32float storage textures. Each edit operation is a WGSL compute shader pass. Passes with      │
│ neutral/default values are skipped for performance.                                                                               │
│                                                                                                                                   │
│ RAW/Image → LibRaw demosaic → linear f32 → GPU upload                                                                             │
│   → White Balance → Basic (exp/contrast/HL/SH/WH/BK)                                                                              │
│   → Tone Curve (LUT) → HSL → Sharpen → Clarity                                                                                    │
│   → Denoise → Dehaze → Vignette → Grain                                                                                           │
│   → Tonemap (linear→sRGB) → readback → display                                                                                    │
│                                                                                                                                   │
│ Preview strategy: Slider drag → process downscaled proxy (<16ms). Slider release → full-res process + save to DB.                 │
│                                                                                                                                   │
│ Non-destructive Editing                                                                                                           │
│                                                                                                                                   │
│ Edit params stored as JSON in SQLite. Original files never modified. Full undo/redo history, snapshots, and presets.              │
│                                                                                                                                   │
│ Data Transfer (Rust ↔ Frontend)                                                                                                   │
│                                                                                                                                   │
│ apply_edits Tauri command returns Vec<u8> (sRGB RGBA bytes) via binary IPC. Frontend renders to <canvas> via putImageData.        │
│ Debounced at ~30ms during slider drag.                                                                                            │
│                                                                                                                                   │
│ Project Structure                                                                                                                 │
│                                                                                                                                   │
│ photo-editor/                                                                                                                     │
│ ├── package.json, vite.config.ts, tsconfig.json, index.html                                                                       │
│ ├── src/                              # React frontend                                                                            │
│ │   ├── main.tsx, App.tsx                                                                                                         │
│ │   ├── api/                          # Tauri invoke wrappers                                                                     │
│ │   │   ├── catalog.ts, image.ts, processing.ts, system.ts                                                                        │
│ │   ├── stores/                       # Zustand                                                                                   │
│ │   │   ├── catalogStore.ts, developStore.ts, uiStore.ts                                                                          │
│ │   ├── components/                                                                                                               │
│ │   │   ├── layout/                   # AppShell, TopToolbar, StatusBar, FilmStrip                                                │
│ │   │   ├── library/                  # LibraryView, ThumbnailGrid, FolderTree, ImportDialog, etc.                                │
│ │   │   ├── develop/                  # DevelopView, ImageCanvas, BeforeAfter                                                     │
│ │   │   │   ├── panels/              # BasicAdjustments, WhiteBalance, ToneCurve, HSL, Detail, Effects                            │
│ │   │   │   └── controls/            # AdjustmentSlider, CurveEditor, CollapsibleSection                                          │
│ │   │   ├── export/                   # ExportDialog, BatchExportDialog                                                           │
│ │   │   └── common/                   # Rating, ColorLabel, FlagToggle, SearchInput, Modal                                        │
│ │   ├── hooks/                        # useKeyboardShortcuts, useDragAndDrop, useDebounce                                         │
│ │   ├── types/                        # catalog.ts, develop.ts, export.ts, metadata.ts                                            │
│ │   └── styles/                       # variables.css, global.css                                                                 │
│ │                                                                                                                                 │
│ ├── src-tauri/                        # Rust backend                                                                              │
│ │   ├── Cargo.toml, tauri.conf.json, build.rs                                                                                     │
│ │   └── src/                                                                                                                      │
│ │       ├── main.rs, lib.rs, state.rs                                                                                             │
│ │       ├── commands/                 # Tauri IPC handlers                                                                        │
│ │       │   ├── catalog.rs, image.rs, develop.rs, export.rs, system.rs                                                            │
│ │       ├── catalog/                  # DB layer                                                                                  │
│ │       │   ├── db.rs, models.rs, queries.rs, import.rs, search.rs                                                                │
│ │       ├── imaging/                  # Image I/O                                                                                 │
│ │       │   ├── raw.rs, loader.rs, thumbnail.rs, exif.rs, export.rs                                                               │
│ │       ├── gpu/                      # wgpu pipeline                                                                             │
│ │       │   ├── context.rs, pipeline.rs, buffers.rs                                                                               │
│ │       │   ├── passes/              # One module per shader pass                                                                 │
│ │       │   └── shaders/             # .wgsl files                                                                                │
│ │       └── ai/                       # ONNX denoise                                                                              │
│ │           ├── denoise.rs                                                                                                        │
│ │           └── models/denoise_unet.onnx                                                                                          │
│                                                                                                                                   │
│ Database Schema (SQLite)                                                                                                          │
│                                                                                                                                   │
│ Tables: images, tags, image_tags, collections, collection_images, edit_history, snapshots, presets                                │
│                                                                                                                                   │
│ Key images columns: id, file_path, file_name, format, raw_format, date_taken, rating (0-5), color_label, flag, camera/lens EXIF   │
│ fields (denormalized), thumbnail (BLOB), edit_params (JSON), exif_json                                                            │
│                                                                                                                                   │
│ Indexes on: date_taken, rating, color_label, flag, camera, file_path.                                                             │
│                                                                                                                                   │
│ Tauri Command API                                                                                                                 │
│                                                                                                                                   │
│ ┌─────────┬─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │  Group  │                                                      Commands                                                       │ │
│ ├─────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ Catalog │ import_folder, get_images, search_images, set_rating, set_color_label, set_flag, add_tags, remove_tag,              │ │
│ │         │ create_collection, add_to_collection, get_collections, delete_images                                                │ │
│ ├─────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ Image   │ load_thumbnail, load_preview, load_full_resolution, get_exif_data                                                   │ │
│ ├─────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ Develop │ apply_edits (hot path), get_edit_params, reset_edits, save_snapshot, load_snapshot, get_history, copy_edits,        │ │
│ │         │ paste_edits                                                                                                         │ │
│ ├─────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ Export  │ export_image, batch_export, export_xmp_sidecar                                                                      │ │
│ ├─────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤ │
│ │ System  │ browse_folder, get_gpu_info, get_app_config, set_app_config                                                         │ │
│ └─────────┴─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                                                                   │
│ EditParams (shared Rust ↔ TS)                                                                                                     │
│                                                                                                                                   │
│ exposure (-5..+5), contrast (-100..+100), highlights, shadows, whites, blacks (-100..+100)                                        │
│ temperature (2000K..12000K), tint (-150..+150)                                                                                    │
│ saturation, vibrance (-100..+100)                                                                                                 │
│ curve_rgb/r/g/b: CurvePoint[]                                                                                                     │
│ hsl_hue/saturation/luminance: [f32; 8] (8 color channels)                                                                         │
│ sharpening_amount (0..150), sharpening_radius (0.5..3.0), clarity (-100..+100)                                                    │
│ denoise_luminance (0..100), denoise_color (0..100), denoise_ai: bool                                                              │
│ dehaze (-100..+100), vignette_amount (-100..+100), grain_amount/size (0..100)                                                     │
│                                                                                                                                   │
│ UI Design                                                                                                                         │
│                                                                                                                                   │
│ - Dark theme: #1a1a1a app bg, #252525 panels, #2d2d2d surfaces, #4da6ff accent                                                    │
│ - Library view: Folder tree + collections (left), virtualized thumbnail grid (center), metadata + quick develop (right), film     │
│ strip (bottom)                                                                                                                    │
│ - Develop view: Presets/history/snapshots (left), canvas with zoom (center), collapsible edit panels (right), film strip (bottom) │
│ - Keyboard shortcuts for all common ops (1-5 ratings, P/X flags, D/G view toggle, Ctrl+Z undo)                                    │
│                                                                                                                                   │
│ Implementation Phases                                                                                                             │
│                                                                                                                                   │
│ Phase 1: Scaffolding & App Shell                                                                                                  │
│                                                                                                                                   │
│ - Initialize Tauri 2 + React + Vite project                                                                                       │
│ - Set up Cargo.toml with all deps                                                                                                 │
│ - Create directory structure with empty modules                                                                                   │
│ - Build AppShell, TopToolbar, StatusBar layout                                                                                    │
│ - Dark theme CSS variables                                                                                                        │
│ - uiStore with view mode toggle                                                                                                   │
│ - Verify: App builds and shows dark UI shell                                                                                      │
│                                                                                                                                   │
│ Phase 2: Catalog & Import                                                                                                         │
│                                                                                                                                   │
│ - SQLite schema creation (db.rs)                                                                                                  │
│ - Folder scanning with walkdir + EXIF extraction                                                                                  │
│ - Rayon-parallel thumbnail generation                                                                                             │
│ - Catalog Tauri commands (import_folder, get_images, search)                                                                      │
│ - ImportDialog, FolderTree, ThumbnailGrid, ThumbnailCard components                                                               │
│ - catalogStore with pagination                                                                                                    │
│ - Verify: Import a folder of JPEGs, see them in grid                                                                              │
│                                                                                                                                   │
│ Phase 3: GPU Pipeline Foundation                                                                                                  │
│                                                                                                                                   │
│ - wgpu context initialization (adapter, device, queue)                                                                            │
│ - Texture management (upload, readback, ping-pong)                                                                                │
│ - basic_adjustments.wgsl + tonemap_output.wgsl + white_balance.wgsl                                                               │
│ - apply_edits command → pipeline → readback → frontend canvas                                                                     │
│ - DevelopView, ImageCanvas, BasicAdjustments panel with sliders                                                                   │
│ - Verify: Open JPEG in develop, drag exposure slider, see live update                                                             │
│                                                                                                                                   │
│ Phase 4: RAW Processing                                                                                                           │
│                                                                                                                                   │
│ - LibRaw FFI wrapper (demosaic → linear f32)                                                                                      │
│ - Unified loader (RAW + standard formats)                                                                                         │
│ - Progressive loading (embedded preview → full decode)                                                                            │
│ - Verify: Import and edit CR2/NEF/ARW/DNG files                                                                                   │
│                                                                                                                                   │
│ Phase 5: Remaining Shaders                                                                                                        │
│                                                                                                                                   │
│ - tone_curve.wgsl (CPU spline → 256-entry LUT → GPU)                                                                              │
│ - hsl.wgsl (RGB↔HSL, 8-channel adjustment)                                                                                        │
│ - sharpening.wgsl (Gaussian blur + unsharp mask)                                                                                  │
│ - clarity.wgsl (large-radius local contrast)                                                                                      │
│ - denoise_bilateral.wgsl                                                                                                          │
│ - dehaze.wgsl (dark channel prior)                                                                                                │
│ - vignette.wgsl, grain.wgsl                                                                                                       │
│ - All corresponding React panels + CurveEditor SVG component                                                                      │
│ - Verify: All sliders produce correct visual results                                                                              │
│                                                                                                                                   │
│ Phase 6: Library Management                                                                                                       │
│                                                                                                                                   │
│ - Rating, color label, flag UI + commands                                                                                         │
│ - Tags, collections (CRUD + nested)                                                                                               │
│ - FilterBar, SearchInput, MetadataPanel                                                                                           │
│ - Keyboard shortcuts, drag-and-drop import                                                                                        │
│ - Verify: Rate/tag/collect/filter images                                                                                          │
│                                                                                                                                   │
│ Phase 7: History, Snapshots, Presets                                                                                              │
│                                                                                                                                   │
│ - Auto-save edit history on each adjustment                                                                                       │
│ - HistoryPanel (click to revert), undo/redo (Ctrl+Z/Y)                                                                            │
│ - Snapshots (save/load named states)                                                                                              │
│ - Presets (save/load/apply to batch)                                                                                              │
│ - Copy/paste edits, before/after toggle                                                                                           │
│ - Verify: Full non-destructive workflow                                                                                           │
│                                                                                                                                   │
│ Phase 8: Export                                                                                                                   │
│                                                                                                                                   │
│ - JPEG/PNG/TIFF encoding with quality/bit-depth settings                                                                          │
│ - ExportDialog, BatchExportDialog with progress                                                                                   │
│ - XMP sidecar export                                                                                                              │
│ - Verify: Export edited RAW as high-quality JPEG                                                                                  │
│                                                                                                                                   │
│ Phase 9: AI Denoise                                                                                                               │
│                                                                                                                                   │
│ - Source/train lightweight U-Net ONNX model (~5MB)                                                                                │
│ - Tiled inference via ort crate (256x256 tiles, 16px overlap)                                                                     │
│ - Toggle in DetailPanel: traditional vs AI                                                                                        │
│ - Blend strength slider                                                                                                           │
│ - Verify: AI denoise on high-ISO RAW image                                                                                        │
│                                                                                                                                   │
│ Phase 10: Polish & Optimization                                                                                                   │
│                                                                                                                                   │
│ - Pass-skipping for neutral params                                                                                                │
│ - Proxy resolution during slider drag                                                                                             │
│ - Thumbnail grid virtualization for 100K+ images                                                                                  │
│ - Loading states, skeleton screens                                                                                                │
│ - Cross-platform testing (Win/Mac/Linux)                                                                                          │
│ - Memory profiling, GPU buffer lifecycle                                                                                          │
│ - App icon, bundling config                                                                                                       │
│                                                                                                                                   │
│ Verification Plan                                                                                                                 │
│                                                                                                                                   │
│ 1. Build: npm install && npm run tauri dev launches app                                                                           │
│ 2. Import a folder with mixed RAW+JPEG files → thumbnails appear in grid                                                          │
│ 3. Double-click image → develop view with all edit sliders                                                                        │
│ 4. Drag each slider → preview updates in real-time (<100ms perceived)                                                             │
│ 5. Rate/tag/collect images → filter by criteria                                                                                   │
│ 6. Undo/redo edits → history panel reflects changes                                                                               │
│ 7. Export edited image as JPEG → visually matches preview                                                                         │
│ 8. AI denoise toggle → visible noise reduction on high-ISO image                                                                  │
╰───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯