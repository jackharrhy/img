# img

A local GIF editor for macOS and Linux, built in Rust with [Rune](https://github.com/anomalyco/rune). Rune draws the interface with Metal on macOS and Vulkan on Linux. GIF import, operations, and export use the same Rust processing engine. No server, browser runtime, account, or upload.

Rune is pre-release. Its core and component crates are pinned to revision `938ba14f09bb3381946b3486ef8aa36363a3e8b2`; Cargo.lock pins the rest of the dependency graph. The repository and CI use Rust 1.99.1 through rust-toolchain.toml.

## Run on macOS

Install Apple's command-line tools (`xcode-select --install`) and Rust through rustup from https://rustup.rs (the repository selects the toolchain). Then, once you have access to the private repository:

```sh
git clone git@github.com:jackharrhy/img.git
cd img
cargo run --locked --release -- samples/hummingbird.gif
```

All 66 operations, previews, and GIF export run directly in Rust with no external processing tools required. To build a local macOS app bundle with the app icon, run `scripts/build-macos-app.sh`; the result is `artifacts/img.app`. The bundle is unsigned and not notarized. The release build and GIF processing smoke check have been verified on Apple silicon macOS; native dialogs still need desktop verification.

## Run on Linux

Install Rust, a C toolchain, and X11/Wayland development libraries and Vulkan drivers. On Ubuntu/Debian:

```sh
sudo apt install build-essential pkg-config libx11-dev libxkbcommon-dev libwayland-dev libegl1-mesa-dev libvulkan1 mesa-vulkan-drivers
cargo run --locked --release -- samples/hummingbird.gif
```

Native dialogs use the desktop's XDG portal; install `xdg-desktop-portal` and the appropriate backend (GTK/KDE/etc.) if file dialogs do not appear. Vulkan support is required for rendering; Mesa’s software driver can render headlessly in CI. An X11 or Wayland desktop is needed for the UI. Passing a GIF path opens it without a file dialog.

## Edit

- The input card is **Import GIF**, with file selection, replacement, filename, and animation details inside the step. It stays first because subsequent steps need its frames. The original file is never modified.
- **Add operation** below the last card opens a searchable catalog of 66 operations (up to 64 cards in a stack). Search names or categories: Size & shape, Animation, Color, Transparency, Effects, or Overlay. See [OPERATIONS.md](OPERATIONS.md) for every operation and parameter range. New inputs start without preset operations. New cards open for editing and collapse older cards. Click a card’s disclosure to open its controls; collapsing preserves its settings. The collapse/expand icon beside Export affects all operation cards.
- Steps run from top to bottom. The monitor icon on a step selects its output in the single viewer on the right. The latest output is selected automatically; adding a step returns to the latest output.
- Numeric fields combine text entry with a scrub control. Drag anywhere in the field horizontally to adjust and preview immediately; hold Shift before dragging for finer adjustment. Click without dragging to select the number for typing. Valid changes apply after a 350 ms pause or immediately when focus leaves the field. Enter applies immediately. Dragging adjusts the value even while the field is focused; Enter also opens text editing from the keyboard. Escape discards changes that have not yet applied. Up/Down adjust the focused number, with Shift for finer fractional steps. The subtle fill shows the value within its allowed range. Each completed drag is one undo action; Escape or losing window focus restores the starting value. Outside text editing, the whole field supports arrow keys and Page Up/Down. This follows the [Blender number-field](https://docs.blender.org/manual/en/4.2/interface/controls/buttons/fields.html) and [Base UI ScrubArea](https://base-ui.com/react/components/number-field) patterns.
- Resize sets independent dimensions; Fit and Fill preserve the aspect ratio. Captions default to Inter, support Shift+Enter for new lines, and have an optional black outline. Image overlays accept PNG, JPEG, or GIF through a file picker or pasted path. Reverse moves each frame together with its delay. Click the eye in a step’s header to bypass or apply it: open means applied, closed means bypassed. Bypassed steps fade; their controls remain editable. Use the trash icon at the top right to delete a step.
- Drag a step by its dotted grip. The full card follows the pointer above the interface, with a faint original slot and a line marking the destination. On release, the real card moves from its original slot to the new position; it does not animate from the floating copy’s release point. Dropping outside the list, Escape, or losing window focus cancels. Edge scrolling follows the pointer and stops when the gesture ends. Arrow icons beside the grip also reorder. Rune’s Reduce Motion preference lands the new order immediately.
- Expanding a card or editing its fields does not change the preview selection. Use the monitor button to choose a step. When previewing Rotate, drag its circular handle to rotate around the image center (Shift snaps to 15°). Resize, Fit, Fill, Canvas size, and Crop have a lower-right size handle; Shift preserves the parameter proportions. Drag the image area in Crop to move the sampling rectangle, or in Wraparound shift to reposition the wrapped pixels. During these drags, a GPU preview and the numeric fields update without rendering the GIF stack; release commits one undo action and renders the result. Escape or window blur cancels, and the inspected animation frame is preserved.
- Text cards keep Caption, a searchable **Font** picker and a separate **Style** control, Font size, and Color visible. Common caption families appear first, followed by the other installed fonts alphabetically. Each family appears once, with its weights and italic faces in Style. The picker follows native font-list conventions: Common and All fonts sections, names shaped as real text in each typeface at a consistent point size, and a caption specimen while browsing. Only visible rows request their native font family; offscreen rows remain lightweight. Rune handles glyph shaping, fallback, antialiasing, and caching. Browsing leaves the GIF unchanged until a font is chosen; Escape cancels. Anton, Bebas Neue, Bangers, and Inter ship in the executable and work offline on every system. Impact appears first when installed; it requires a separate redistribution license and is not shipped. Font provenance and licenses are recorded in [assets/fonts/README.md](assets/fonts/README.md). Font discovery runs in the background and loaded faces are cached. **Typography** expands left/center/right alignment within multiline captions, letter spacing, and line spacing. Preview, selection bounds, and export use the same selected face and layout. Missing fonts report an error so you can choose a replacement. **Position** expands coordinates and a nine-position alignment grid; **Appearance** expands the outline setting. Image/GIF overlay cards use the same Position disclosure. Preview an enabled Text caption to get a selection box on the image. Drag it to move, drag its lower-right handle to resize the font, or use arrow keys to nudge by one pixel (Shift: ten). Position presets align visible glyph bounds, including the outline, with an 8-pixel inset at the edges. Movement snaps to edge/center guides. Escape or window blur restores the previous position. During dragging, only the visible text layer is rasterized over the previous step’s frame; the GIF stack renders once on release. Editing properties or positioning text preserves the inspected frame; loading a new GIF starts at its first frame.
- **Fit** scales the image to the viewer; **100%** displays actual pixels with scrolling for larger images. Checkerboard transparency stays inside the image boundary.
- **Show frames** opens a horizontal frame strip below the image. Click a thumbnail to pause and inspect it, including its duration. Previous/next buttons step through frames; Left/Right also step when the frame strip is focused or no text is selected. Space plays or pauses from the current frame.
- Color values use a swatch button with a saturation/brightness field, hue strip, and six-digit hex input. Picker edits stay staged until **Apply** or Enter; Cancel, Escape, or clicking outside discards them. The eyedropper beside a swatch and the eyedropper inside the picker start the same tool. On macOS, the public [NSColorSampler](https://developer.apple.com/documentation/appkit/nscolorsampler) magnifier samples anywhere onscreen, including the preview and other app windows, and converts the chosen color to sRGB. On Linux, the same action samples the preview's actual RGB, excluding checkerboards and letterboxing; transparent pixels cannot be sampled. Playback pauses while sampling and resumes when you apply or cancel. Escape cancels sampling. Signed Color balance offsets remain numeric.
- **Export result…** saves the final stack output, regardless of which step is being previewed. An empty operation stack exports the input.
- Color, font, style, and operation popovers prefer below their controls, aligned with the left edge. Rune flips them above when the bottom edge would clip them and keeps them inside the window.
- The interface is light-only, with Rune’s bundled sans serif fonts. Work runs on background threads. Fields remain editable during rendering; debounced edits queue the newest values for a single follow-up render. Edits and stack mutations queue the latest render; export waits for a completed result; Add operation and Export result scroll with the cards. Errors and save confirmations share a fixed space below those buttons; rendering has no processing label. The previous preview stays visible while steps update.

## Undo, redo, and clipboard

The pinned controls above the stack provide Undo, Redo, and an Edit menu. Undo/redo also use Cmd+Z / Shift+Cmd+Z (Ctrl off macOS). Each add, delete, bypass, reorder, applied field edit, color application, alignment, or completed text drag is one action. Arrow-key nudges within half a second group together. Editing after undo replaces the redo branch. Typing keeps its own undo/redo history while the field has focus, including across debounced updates. Applied changes also belong to the document history. A failed render can be undone, and edits or undo/redo during rendering queue the latest stack without publishing an intermediate result.

History retains at most 100 actions and about 16 MiB of operation metadata (always keeping the newest action), sharing source animations and retaining no intermediate renders. Previous source animations retained for undo have a 256 MiB budget; older history is evicted when needed. Replacing the source GIF keeps the existing operations, parameters, bypass states, and chosen preview. The source change is undoable. Pasting an image into an empty window is undoable.

Click a card or work in its fields to make it the active operation; the monitor icon independently controls the preview. Cmd+C / Cmd+X copy or cut that operation, Cmd+V pastes operations, a text caption, or an image after it, and Cmd+D duplicates it. Delete removes the active operation outside text fields. Right-click a card for the same commands. The Edit menu can copy the whole stack as one transfer. Pasting multiple operations creates one undo action and fresh operation IDs. Images paste as overlays when a GIF is open, or become the source in an empty window. Files copied from a file manager can also be pasted.

Text fields keep native text cut/copy/paste. Recognized operation transfers and images paste into the stack even when an operation field has focus; a file copied from a file manager can populate an image-file field. Shift+Cmd+V explicitly pastes into the stack even while a field is focused. Shift+Cmd+C copies the displayed frame as a PNG with its original RGBA pixels, without the preview checkerboard. Cmd+C also copies the frame when there are no operations. The macOS Edit menu includes the document-specific commands; the in-app menu and shortcuts work across platforms.

Operation transfers use a versioned, validated JSON format and carry image/GIF overlay assets, so pastes survive deletion of the original file or closing the source app. Transfers are limited to 24 MiB of clipboard text and 16 MiB of embedded assets. Temporary pasted assets are retained while referenced by the document or undo/redo history and cleaned up afterward; their disk budget is 256 MiB per document.

## GIF behavior and limits

Operations preserve frame delays and loop metadata unless they explicitly edit timing, frame order, or looping. Animated overlays merge source and overlay frame boundaries, repeating the overlay over the source cycle; zero-delay frames use 10 ms during synchronization. Export uses GIF's 10 ms timing resolution, per-frame palette quantization, and binary transparency. Fully transparent pixels remain transparent; nonzero partial alpha becomes opaque on GIF export. Apply Matte background before export to bake antialiased edges or partial opacity into a chosen background. Complex colors may change during palette quantization. Output uses full composited frames rather than optimized delta frames, so files can grow. Previews intentionally repeat indefinitely and clamp delays below 20 ms for display only; export retains the original GIF timing/loop metadata.

Input is limited to 64 MiB compressed, 4096 pixels per dimension, 1000 frames, and 256 MiB decoded pixel data. The editor has a 1 GiB tracked frame-buffer budget (not a total-process memory cap). It retains the source, selected preview, final output, and a checkpoint before the last operation. Selecting a step with preview handles also keeps its preceding output for direct canvas editing; other intermediate animations are released. The checkpoint can be evicted under pressure. An old displayed preview is counted while its replacement renders. Scratch buffers, overlay assets, preview copies, and GPU textures add memory outside that budget. Each individual animation remains limited to 256 MiB decoded pixels. Frame order, timing, repeat, and bypass operations share frame pixels rather than copying them. Growth is checked before allocating rendered output. Exports use a temporary file beside the destination and replace the target only after encoding succeeds.

Unchanged prefix checkpoints are reused after edits and appends. Compatible channel operations (negate, brightness, contrast, gamma, exposure, color balance/tint, posterize, solarize, opacity, and alpha threshold) compose into four byte lookup tables and run together, preserving rounding and clipping at each original operation. Requested previews split fused runs at the correct step. Channel-mixing, geometry, temporal, neighborhood, palette, and global-analysis operations keep their existing algorithms. Overlay files are reloaded when rendering reaches them, even when the path is unchanged.

The catalog includes crop, canvas, border, rotations, flips, wraparound, frame ranges, speed, holds, ping-pong, looping, fades, levels, gamma, hue, saturation, tint, color keying, alpha controls, shared palettes, dithering, blur, sharpen, pixelation, edges, emboss, noise, vignette, scanlines, swirl, wave, captions, and image overlays. Operations retain full RGBA pixels until export, avoiding intermediate GIF quantization. Choose processing tools per operation: simple pixel/frame edits stay in Rust; evaluate FFmpeg for video formats and advanced filter chains when those features are added.

## Verify

```sh
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- --generate-sample samples/blue-block.gif
cargo run --locked -- --smoke samples/blue-block.gif edited.gif
```

Tests cover all 66 catalog entries, parameter validation, known geometry and color results, deterministic effects, caption rasterization, animated compositing, GIF palette transparency, timing and looping, resize/reverse/reorder/bypass, offset/disposal decoding, invalid inputs and memory budgets, atomic export failure, CLI roundtrips, and headless Rune UI interactions, file-dialog routing, and export/reimport. Negate is tested against every pixel of the bundled 45-frame hummingbird GIF; no tests require ImageMagick.

Seeded stateful pipeline tests perform 700 edits across seven fixed seeds. They add, edit, move, bypass, and remove operations, check every intermediate output against an independent color/order/dimension model, then export and reopen after every edit. Failures include a replay command and action history. Run longer sequences or reproduce a particular seed with:

```sh
IMG_FUZZ_SEED=42 IMG_FUZZ_STEPS=10000 cargo test --locked --release --test seeded_pipeline -- --nocapture
```

A second seeded suite mixes the entire catalog into stacks, checks bypass aliases and deterministic intermediate results, and exports/reopens every addition (528 additions by default). Every operation is guaranteed coverage for each seed. The optimized renderer is compared against the full-intermediate reference pipeline at both the final and a randomly selected step. Replay or extend it with:

```sh
IMG_CATALOG_SEED=42 IMG_CATALOG_CYCLES=20 cargo test --locked --release --test seeded_operations -- --nocapture
```

These exercise import → processing → export → reimport without a desktop. Rune runtime interaction tests cover the operation menu, asynchronous Negate on the real sample, bypassing it, and native dialog request/completion callbacks. Actual operating-system dialog presentation still needs desktop verification. Blank-area text-field clicks during playback, edits queued during a held render, and selecting an uncached preview have runtime regression coverage.

Rune also renders snapshots using the actual GPU renderer, without a desktop:

```sh
cargo build --locked --release
scripts/render-smoke.sh
```

This produces wide, narrow, card-drag, canvas-handle, text-drag, caption (including Retina), color-picker, and operation-picker captures in `artifacts/`; CI renders and uploads them on both platforms. The interface remains light when Rune receives a dark system preference. `loaded` opens the bundled sample without operations; `edited` applies resize and reverse and opens the frame strip for deterministic screenshots. `caption` shows a text operation; `typography` opens its text layout controls; `fonts` opens the installed font picker; `picker` opens the operation catalog; `color` shows its color picker; `editing` shows document editing commands and a redo branch. Normal launches start empty, or load the GIF path supplied on the command line. Rune’s `--tree tree.json` and `--trace trace.jsonl` flags expose rendered elements and motion for debugging.

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

## Processing benchmark

```sh
cargo run --locked --release --example performance
```

On this Apple silicon machine, the bundled 480×270, 45-frame GIF with 24 compatible color operations rendered in 10.09 ms versus 837.62 ms through the full-intermediate reference path (83× for this workload). Retained frame pixels fell from 556.2 MiB to 66.7 MiB. Appending an operation took 5.34 ms and editing the last operation took 4.61 ms, each reusing 24 steps. The example checks exact output pixels, timing, and looping. These are workload-specific timings, not a speedup claim for blur, geometry, or every stack. Run it locally to compare.

## Interface conventions

Keep the editor light-only and neutral, with sans serif text, 2–3 px corner radii, thin borders, and compact spacing. The shared visual helpers are in `src/ui/view.rs`. Keep the single preview on the right and Add/Export below the scrolling operation list. Avoid branding headers, step numbers, repeated explanatory paragraphs, fixed bottom action bars, and decorative panels.

Use disclosure controls for secondary properties. Keep primary fields immediately available, put positioning on the canvas when possible, and reserve popovers for bounded tasks such as choosing a color. Icon actions need accessible names, tooltips, keyboard behavior, and honest disabled states.

Collapsible controls and direct manipulation take cues from [Figma’s layer controls](https://help.figma.com/hc/en-us/articles/360039831974-Explore-the-navigation-bar-and-left-sidebar) and [Penpot’s canvas positioning and alignment guides](https://help.penpot.app/user-guide/designing/workspace-basics/). GIF operations retain their top-to-bottom execution order.

## Scope

This is a working vertical slice, not a complete editor. No project persistence, cancellation of Rust operations, aspect-ratio lock, direct per-frame painting, or optimized delta-frame export yet. See [IDEAS.md](IDEAS.md).
