use super::*;
use std::collections::HashMap;

pub(super) struct Preview {
    pub(super) texture: Option<(u64, usize, egui::TextureHandle)>,
    pub(super) paused: bool,
    pub(super) frames_open: bool,
    fit: bool,
    start: Instant,
    elapsed: Duration,
    thumbnails: HashMap<usize, egui::TextureHandle>,
    reveal_frame: bool,
}
impl Default for Preview {
    fn default() -> Self {
        Self {
            texture: None,
            paused: false,
            frames_open: false,
            fit: true,
            start: Instant::now(),
            elapsed: Duration::ZERO,
            thumbnails: HashMap::new(),
            reveal_frame: true,
        }
    }
}
impl Preview {
    pub(super) fn reset(&mut self) {
        self.texture = None;
        self.thumbnails.clear();
        self.reveal_frame = true;
        self.elapsed = Duration::ZERO;
        self.start = Instant::now();
    }
    pub(super) fn frame_index(&self, animation: &Animation) -> usize {
        let total: u64 = animation
            .frames
            .iter()
            .map(|f| f.delay_ms.max(20) as u64)
            .sum();
        let elapsed = self.elapsed
            + if self.paused {
                Duration::ZERO
            } else {
                self.start.elapsed()
            };
        let mut time = elapsed.as_millis() as u64 % total.max(1);
        for (index, frame) in animation.frames.iter().enumerate() {
            if time < frame.delay_ms.max(20) as u64 {
                return index;
            }
            time -= frame.delay_ms.max(20) as u64;
        }
        0
    }
    fn seek(&mut self, animation: &Animation, index: usize) {
        self.elapsed = Duration::from_millis(
            animation.frames[..index]
                .iter()
                .map(|frame| frame.delay_ms.max(20) as u64)
                .sum(),
        );
        self.start = Instant::now();
        self.paused = true;
        self.reveal_frame = true;
    }
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        animation: &Animation,
        id: u64,
        available_height: f32,
    ) {
        if self.texture.as_ref().is_some_and(|(old, _, _)| *old != id) {
            self.reset();
        }
        let index = self.frame_index(animation);
        ui.horizontal(|ui| {
            let pixels = &animation.frames[index].pixels;
            ui.label(
                RichText::new(format!("{} × {} pixels", pixels.width(), pixels.height()))
                    .color(MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.selectable_value(&mut self.fit, false, "100%")
                    .on_hover_text("Actual pixels. Scroll to pan large images.");
                ui.selectable_value(&mut self.fit, true, "Fit")
                    .on_hover_text("Fit the image inside the preview.");
            });
        });
        let frame = &animation.frames[index];
        if self
            .texture
            .as_ref()
            .is_none_or(|(_, old, _)| *old != index)
        {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [
                    frame.pixels.width() as usize,
                    frame.pixels.height() as usize,
                ],
                frame.pixels.as_raw(),
            );
            if let Some((old_id, old_index, handle)) = &mut self.texture {
                handle.set(image, egui::TextureOptions::LINEAR);
                *old_id = id;
                *old_index = index;
            } else {
                self.texture = Some((
                    id,
                    index,
                    ui.ctx()
                        .load_texture("preview", image, egui::TextureOptions::LINEAR),
                ));
            }
        }
        let reserved = if self.frames_open { 252. } else { 120. };
        let height = (available_height - reserved).max(180.);
        let width = ui.available_width();
        egui::ScrollArea::both()
            .id_salt("image-canvas")
            .auto_shrink([false, false])
            .max_height(height)
            .show_viewport(ui, |ui, _| {
                let handle = &self.texture.as_ref().unwrap().2;
                let size = handle.size_vec2();
                let scale = if self.fit {
                    ((width - 32.).max(1.) / size.x).min((height - 32.).max(1.) / size.y)
                } else {
                    1.
                };
                let image_size = size * scale;
                let canvas_size = egui::vec2(
                    width.max(image_size.x + 32.),
                    height.max(image_size.y + 32.),
                );
                let (rect, _) = ui.allocate_exact_size(canvas_size, egui::Sense::hover());
                let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
                painter.rect_filled(rect, 2, Color32::WHITE);
                let image_rect = egui::Rect::from_center_size(rect.center(), image_size);
                painter.rect_stroke(
                    image_rect.expand(0.5),
                    0,
                    Stroke::new(0.75_f32, Color32::from_gray(230)),
                    egui::StrokeKind::Outside,
                );
                paint_image(&painter, handle, image_rect);
            });
        ui.add_space(4.);
        let keyboard = !ui.ctx().wants_keyboard_input();
        let previous = keyboard && ui.input(|i| i.key_pressed(egui::Key::ArrowLeft));
        let next = keyboard && ui.input(|i| i.key_pressed(egui::Key::ArrowRight));
        let toggle = keyboard && ui.input(|i| i.key_pressed(egui::Key::Space));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(index > 0, egui::Button::new("‹"))
                .on_hover_text("Previous frame (Left arrow)")
                .clicked()
                || previous
            {
                self.seek(animation, index.saturating_sub(1));
            }
            if ui
                .add_sized(
                    [58., 24.],
                    egui::Button::new(if self.paused { "Play" } else { "Pause" }),
                )
                .on_hover_text("Play or pause (Space)")
                .clicked()
                || toggle
            {
                if self.paused {
                    self.start = Instant::now();
                } else {
                    self.elapsed += self.start.elapsed();
                }
                self.paused = !self.paused;
            }
            if ui
                .add_enabled(index + 1 < animation.frames.len(), egui::Button::new("›"))
                .on_hover_text("Next frame (Right arrow)")
                .clicked()
                || next
            {
                self.seek(animation, (index + 1).min(animation.frames.len() - 1));
            }
            ui.label(format!(
                "Frame {} of {}",
                self.frame_index(animation) + 1,
                animation.frames.len()
            ));
        });
        let mut reveal_frames = false;
        ui.horizontal(|ui| {
            if ui
                .button(if self.frames_open {
                    "Hide frames"
                } else {
                    "Show frames"
                })
                .clicked()
            {
                self.frames_open = !self.frames_open;
                reveal_frames = self.frames_open;
            }
            let duration: u64 = animation
                .frames
                .iter()
                .map(|frame| frame.delay_ms as u64)
                .sum();
            ui.label(
                RichText::new(format!("{:.2} s total", duration as f64 / 1000.))
                    .size(12.)
                    .color(MUTED),
            );
        });
        if self.frames_open {
            ui.add_space(4.);
            self.frame_strip(ui, animation);
            if reveal_frames {
                ui.scroll_to_cursor(Some(egui::Align::Max));
            }
        } else {
            self.thumbnails.clear();
        }
    }
    fn frame_strip(&mut self, ui: &mut egui::Ui, animation: &Animation) {
        let selected = self.frame_index(animation);
        let mut seek = None;
        egui::ScrollArea::horizontal()
            .id_salt("frames")
            .max_height(120.)
            .show_viewport(ui, |ui, viewport| {
                let item_width = 112.;
                let first = (viewport.left() / item_width).floor().max(0.) as usize;
                let end = ((viewport.right() / item_width).ceil() as usize + 1)
                    .min(animation.frames.len());
                self.thumbnails
                    .retain(|index, _| (first..end).contains(index));
                ui.set_min_width(animation.frames.len() as f32 * item_width);
                ui.set_min_height(108.);
                let origin = ui.min_rect().min;
                if self.reveal_frame {
                    ui.scroll_to_rect(
                        egui::Rect::from_min_size(
                            origin + egui::vec2(selected as f32 * item_width, 0.),
                            egui::vec2(104., 100.),
                        ),
                        None,
                    );
                    self.reveal_frame = false;
                }
                for index in first..end {
                    let frame = &animation.frames[index];
                    let handle = self.thumbnails.entry(index).or_insert_with(|| {
                        let pixels = image::imageops::thumbnail(&frame.pixels, 104, 64);
                        ui.ctx().load_texture(
                            format!("frame-{index}"),
                            egui::ColorImage::from_rgba_unmultiplied(
                                [pixels.width() as usize, pixels.height() as usize],
                                pixels.as_raw(),
                            ),
                            egui::TextureOptions::LINEAR,
                        )
                    });
                    let rect = egui::Rect::from_min_size(
                        origin + egui::vec2(index as f32 * item_width, 0.),
                        egui::vec2(104., 100.),
                    );
                    let response = ui.interact(rect, ui.id().with(index), egui::Sense::click());
                    let color = Color32::from_gray(if index == selected {
                        180
                    } else if response.hovered() {
                        210
                    } else {
                        232
                    });
                    ui.painter().rect(
                        rect,
                        2,
                        Color32::WHITE,
                        Stroke::new(0.75_f32, color),
                        egui::StrokeKind::Inside,
                    );
                    let size = handle.size_vec2();
                    let image_rect =
                        egui::Rect::from_center_size(rect.min + egui::vec2(52., 35.), size);
                    paint_image(ui.painter(), handle, image_rect);
                    ui.painter().text(
                        rect.min + egui::vec2(6., 76.),
                        egui::Align2::LEFT_TOP,
                        format!("{}   {} ms", index + 1, frame.delay_ms),
                        egui::FontId::proportional(12.),
                        MUTED,
                    );
                    if response
                        .on_hover_text(format!("Frame {}. Click to pause here.", index + 1))
                        .clicked()
                    {
                        seek = Some(index);
                    }
                }
            });
        if let Some(index) = seek {
            self.seek(animation, index);
        }
        let index = self.frame_index(animation);
        ui.label(
            RichText::new(format!(
                "Frame {}: {} ms",
                index + 1,
                animation.frames[index].delay_ms
            ))
            .size(12.)
            .color(MUTED),
        );
    }
}
fn paint_image(painter: &egui::Painter, texture: &egui::TextureHandle, rect: egui::Rect) {
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    painter.rect_filled(rect, 0, Color32::WHITE);
    let tile = 10.;
    for y in 0..(rect.height() / tile).ceil() as usize {
        for x in 0..(rect.width() / tile).ceil() as usize {
            if (x + y) % 2 == 0 {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        rect.min + egui::vec2(x as f32 * tile, y as f32 * tile),
                        egui::vec2(tile, tile),
                    ),
                    0,
                    Color32::from_gray(246),
                );
            }
        }
    }
    painter.image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
        Color32::WHITE,
    );
}
