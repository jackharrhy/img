# img

A local, Rust-native GIF editor for macOS and Linux. No server, browser runtime, account, or upload. Built with egui/eframe and an OpenGL renderer: portable native windows, immediate-mode controls, and one Rust application. Chosen over GPUI/GPUIX for a small cross-platform vertical slice with straightforward Linux/macOS builds.

## Run on macOS

Install Apple's command-line tools (`xcode-select --install`) and a current stable Rust toolchain from https://rustup.rs (Rust 2024 edition). Then, once you have access to the private repository:

```sh
git clone git@github.com:jackharrhy/img.git
cd img
cargo run --locked --release -- samples/blue-block.gif
```

No ImageMagick is needed for import, resize, reverse, previews, or export. For the optional Negate card only, install ImageMagick 7 (`brew install imagemagick`) and launch from a terminal where `magick` is on PATH. This is a source build, not a signed/notarized macOS app bundle. macOS execution has not yet been verified on hardware.

## Run on Linux

Install Rust, a C toolchain, and OpenGL development/runtime libraries. On Ubuntu/Debian:

```sh
sudo apt install build-essential pkg-config libx11-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
cargo run --locked --release -- samples/blue-block.gif
```

Native dialogs use the desktop's XDG portal; install `xdg-desktop-portal` and the appropriate backend (GTK/KDE/etc.) if file dialogs do not appear. An X11 or Wayland desktop is needed for the UI. Passing a GIF path opens it without a file dialog.

## Edit

- **Open GIF…** imports and composites all frames, including frame offsets and disposal modes.
- The original stays at the top. Each operation has controls on its **left** and its own animated output on its **right**.
- Resize sets independent width and height with a triangle filter. Reverse moves each frame together with its delay.
- Drag the title text in a blue heading onto another card to reorder. Up/Down buttons also work. Uncheck Enabled to bypass, or Remove to delete.
- Scroll down to add Resize, Reverse, or optional ImageMagick Negate cards (up to 16).
- Pause/Play controls all previews. **Export GIF…** writes the final stack output; an empty stack exports the source.
- Work runs on background threads. Editing and export are disabled while a job is running; errors appear in the bottom status bar.

## GIF behavior and limits

Frame delays and loop metadata survive the Rust operations and export. Export uses GIF's 10 ms timing resolution, per-frame palette quantization, and binary transparency. Fully transparent pixels remain transparent; partial alpha produced by resizing becomes opaque on GIF export. Complex colors may change during palette quantization. Output uses full composited frames rather than optimized delta frames, so files can grow. Previews intentionally repeat indefinitely and clamp delays below 20 ms for display only; export retains the original GIF timing/loop metadata.

Input is limited to 64 MiB compressed, 4096 pixels per dimension, 1000 frames, and 256 MiB decoded pixel data. The operation stack also has a 256 MiB retained-pixel budget (not a total-process memory cap). Exports use a temporary file beside the destination and replace the target only after encoding succeeds.

ImageMagick is an optional fixed **RGB negate** operation, not an arbitrary command prompt: Rust invokes `magick` directly without a shell, using generated temporary input/output paths, resource limits, and a 30-second timeout. It processes a re-encoded GIF rather than a user-controlled ImageMagick filename or expression. Timing and loop metadata are restored after processing. This is not an OS sandbox; use a trusted, updated local ImageMagick installation.

## Verify

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- --generate-sample samples/blue-block.gif
cargo run --locked -- --smoke samples/blue-block.gif edited.gif
```

Tests cover GIF pixels/transparency, timing and looping, resize/reverse/reorder/bypass, offset/disposal decoding, invalid inputs and memory budgets, atomic export failure, CLI roundtrips, and basic native UI rendering. The ImageMagick integration test skips with an explanatory message when `magick` is absent (`cargo test --test imagemagick -- --nocapture`).

An actual Linux native-window smoke test (not a web mockup):

```sh
sudo apt install xvfb xauth xdotool
# Also requires ImageMagick 7 for screenshot capture, and Python 3.
cargo build --locked
xvfb-run -a -s '-screen 0 1280x1000x24' python3 scripts/native-smoke.py
```

It checks that the app stays alive and captures changing animation, then captures drag-reordered and bypassed states for visual inspection. Screenshots and the native log land in ignored `artifacts/`. Coordinate-driven interactions assume the default window size and scale.

On a machine with limited root disk space, put the checkout, target directory, and temporary directory on the larger volume before building:

```sh
mkdir -p /mnt/terrabud/dev/img/.tmp
export CARGO_TARGET_DIR=/mnt/terrabud/dev/img/target
export TMPDIR=/mnt/terrabud/dev/img/.tmp
```

## Scope

This is a working vertical slice, not a complete editor. No undo, project persistence, cancellation of Rust operations, aspect-ratio lock, frame timeline, or optimized export yet. See [IDEAS.md](IDEAS.md).
