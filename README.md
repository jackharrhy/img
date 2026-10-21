# img

A local, Rust-native GIF editor for macOS and Linux. No server, browser runtime, account, or upload. Built with egui/eframe and an OpenGL renderer: portable native windows, immediate-mode controls, and one Rust application. Chosen over GPUI/GPUIX for a small cross-platform vertical slice with straightforward Linux/macOS builds.

## Run on macOS

Install Apple's command-line tools (`xcode-select --install`) and a current stable Rust toolchain from https://rustup.rs (Rust 2024 edition). Then, once you have access to the private repository:

```sh
git clone git@github.com:jackharrhy/img.git
cd img
cargo run --locked --release -- samples/hummingbird.gif
```

Import, resize, reverse, negate, previews, and export run directly in Rust with no external processing tools required. To build a local macOS app bundle with the app icon, run `scripts/build-macos-app.sh`; the result is `artifacts/img.app`. The bundle is unsigned and not notarized. The release build and GIF processing smoke check have been verified on Apple silicon macOS; native dialogs still need desktop verification.

## Run on Linux

Install Rust, a C toolchain, and OpenGL development/runtime libraries. On Ubuntu/Debian:

```sh
sudo apt install build-essential pkg-config libx11-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
cargo run --locked --release -- samples/hummingbird.gif
```

Native dialogs use the desktop's XDG portal; install `xdg-desktop-portal` and the appropriate backend (GTK/KDE/etc.) if file dialogs do not appear. An X11 or Wayland desktop is needed for the UI. Passing a GIF path opens it without a file dialog.

## Edit

- The input card is **Import GIF**, with file selection, replacement, filename, and animation details inside the step. It stays first because subsequent steps need its frames. The original file is never modified.
- **Add operation** below the last card adds Resize, Reverse, or Negate (up to 16 operations). New inputs start without preset operations.
- Steps run from top to bottom. **Preview** on a step selects its output in the single viewer on the right. The latest output is selected automatically; adding a step returns to the latest output.
- Resize sets independent width and height with a triangle filter. Reverse moves each frame together with its delay. Click the eye in a step’s header to bypass or apply it: open means applied, closed means bypassed. Bypassed steps fade; their controls remain editable. Use the trash icon at the top right to delete a step.
- Drag a step by its dotted grip. The whole card lifts under the pointer with its original size and grab position. An empty slot stays behind, and an insertion line marks the drop position. Dropping outside the list cancels. Escape cancels. Edge scrolling follows the pointer; arrow icons beside the grip also reorder.
- **Fit** scales the image to the viewer; **100%** displays actual pixels with scrolling for larger images. Checkerboard transparency stays inside the image boundary.
- **Show frames** opens a horizontal frame strip below the image. Click a thumbnail to pause and inspect it, including its duration. Previous/next buttons or Left/Right arrows step through frames; Space plays or pauses from the current frame.
- **Export result…** saves the final stack output, regardless of which step is being previewed. An empty operation stack exports the input.
- The interface is light-only and uses the system font on macOS. Work runs on background threads. Editing and export are disabled during processing; Add operation and Export result scroll with the cards. Progress, errors, and confirmations share a fixed space below those buttons. The previous preview stays visible while steps update.

## GIF behavior and limits

Frame delays and loop metadata survive the Rust operations and export. Export uses GIF's 10 ms timing resolution, per-frame palette quantization, and binary transparency. Fully transparent pixels remain transparent; partial alpha produced by resizing becomes opaque on GIF export. Complex colors may change during palette quantization. Output uses full composited frames rather than optimized delta frames, so files can grow. Previews intentionally repeat indefinitely and clamp delays below 20 ms for display only; export retains the original GIF timing/loop metadata.

Input is limited to 64 MiB compressed, 4096 pixels per dimension, 1000 frames, and 256 MiB decoded pixel data. The operation stack also has a 256 MiB retained-pixel budget (not a total-process memory cap). Exports use a temporary file beside the destination and replace the target only after encoding succeeds.

Negate inverts each RGB channel directly in decoded frames and leaves alpha, timing, and looping intact. Operations retain full RGBA pixels until export, avoiding intermediate GIF quantization. Choose processing tools per operation: simple pixel/frame edits stay in Rust; evaluate FFmpeg for video formats and advanced filter chains when those features are added.

## Verify

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- --generate-sample samples/blue-block.gif
cargo run --locked -- --smoke samples/blue-block.gif edited.gif
```

Tests cover GIF pixels/transparency, timing and looping, resize/reverse/reorder/bypass, offset/disposal decoding, invalid inputs and memory budgets, atomic export failure, CLI roundtrips, and headless native UI interactions. Negate is tested against every pixel of the bundled 45-frame hummingbird GIF; no tests require ImageMagick.

Seeded stateful pipeline tests perform 700 edits across seven fixed seeds. They add, edit, move, bypass, and remove operations, check every intermediate output against an independent color/order/dimension model, then export and reopen after every edit. Failures include a replay command and action history. Run longer sequences or reproduce a particular seed with:

```sh
IMG_FUZZ_SEED=42 IMG_FUZZ_STEPS=10000 cargo test --locked --release --test seeded_pipeline -- --nocapture
```

These exercise import → processing → export → reimport without a desktop. Separate headless egui event tests cover the operation menu, asynchronous Negate on the real sample, and bypassing it; file dialogs still need desktop verification.

An actual Linux native-window smoke test (not a web mockup):

```sh
sudo apt install xvfb xauth xdotool
# Also requires ImageMagick 7 for screenshot capture, and Python 3.
cargo build --locked
xvfb-run -a -s '-screen 0 1280x1000x24' python3 scripts/native-smoke.py
```

It checks that the native window stays alive and captures changing animation. Screenshots and the native log land in ignored `artifacts/`. Headless UI interaction tests verify adding, reordering, bypassing, removing, drag cancellation, and frame inspection without relying on window coordinates.

On a machine with limited root disk space, put the checkout, target directory, and temporary directory on the larger volume before building:

```sh
mkdir -p /mnt/terrabud/dev/img/.tmp
export CARGO_TARGET_DIR=/mnt/terrabud/dev/img/target
export TMPDIR=/mnt/terrabud/dev/img/.tmp
```

## Scope

This is a working vertical slice, not a complete editor. No undo, project persistence, cancellation of Rust operations, aspect-ratio lock, frame editing, or optimized export yet. See [IDEAS.md](IDEAS.md).
