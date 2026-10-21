# Next slices

## Editing
- Aspect-ratio lock, percent resize, fit/fill, and nearest-neighbor for pixel art.
- Crop, rotate, flip, frame range, ping-pong, speed, and per-frame timing.
- Undo/redo and a serializable operation recipe with source-path relinking.
- Duplicate cards and keyboard reordering.

## Preview and export
- Continuous zoom and pointer-based panning.
- Explicit loop controls, including a preview mode honoring finite repeats.
- Alpha threshold/matte options and premultiplied-alpha resize to improve transparent edges.
- Palette/dither controls and optimized delta-frame export; show output size before saving.
- Side-by-side source/final comparison and per-step processing time.

## Reliability and distribution
- macOS hardware validation, app bundle, signing, and notarization.
- Automated native dialog/export checks on macOS and Linux desktop portals.
- Incremental recompute from the changed card; cancellation and progress for long jobs.
- Clear memory-budget estimates before an operation runs.
- Additional malformed-GIF fixtures, fuzzing, and peak-memory tests.
- Evaluate FFmpeg for video import/export and filters where it improves capability or throughput.

Choose the best implementation per operation; simple frame/pixel edits run directly in Rust. Optional external tools should remain explicit, fixed operations with no shell command interpolation.
