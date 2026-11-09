# Next slices

## Editing
- Aspect-ratio lock, percent resize, and nearest-neighbor resize for pixel art.
- Direct per-frame duration editing and painting.
- Project recipe files with source-path relinking; undo/redo and versioned clipboard transfers are implemented.
- Keyboard reordering; card duplication is implemented.
- Canvas manipulation for image overlays and multi-selection.

## Preview and export
- Continuous zoom and pointer-based panning.
- Preview mode honoring finite repeats.
- Optimized delta-frame export; show output size before saving.
- Side-by-side source/final comparison and per-step processing time.

## Reliability and distribution
- Signing and notarization for the macOS app bundle.
- Automated native dialog/export checks on macOS and Linux desktop portals.
- More checkpoints where profiling supports them; cancellation and progress for long jobs.
- Show tracked memory estimates before an operation runs.
- Cache display-sized playback pixels under a separate budget; reduce full UI rebuilds during playback.
- Additional malformed-GIF fixtures, fuzzing, and peak-memory tests.
- Evaluate FFmpeg for video import/export and filters where it improves capability or throughput.

Choose the best implementation per operation; simple frame/pixel edits run directly in Rust. Optional external tools should remain explicit, fixed operations with no shell command interpolation.
