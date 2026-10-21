use eframe::egui::{self, Color32, RichText, Stroke};
use img::{Animation, Card, Operation};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

mod preview;
mod stack;

use preview::Preview;

const MUTED: Color32 = Color32::from_gray(110);

type Outputs = Vec<Arc<Animation>>;
enum Completed {
    Loaded(Arc<Animation>, String),
    Processed(Outputs),
    Saved(PathBuf),
    Cancelled,
}
pub struct Editor {
    source: Option<Arc<Animation>>,
    outputs: Outputs,
    cards: Vec<Card>,
    next_id: u64,
    receiver: Option<mpsc::Receiver<anyhow::Result<Completed>>>,
    status: String,
    // None follows the latest output; zero selects the original; other IDs select a step.
    preview_id: Option<u64>,
    preview: Preview,
    last_preview: Option<(u64, Arc<Animation>)>,
    name: String,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            source: None,
            outputs: vec![],
            cards: vec![],
            next_id: 1,
            receiver: None,
            status: String::new(),
            preview_id: None,
            preview: Preview::default(),
            last_preview: None,
            name: "No GIF open".into(),
        }
    }
}
impl Editor {
    pub fn open(&mut self, path: PathBuf, ctx: &egui::Context) {
        self.spawn(ctx, move || {
            Ok(Completed::Loaded(
                Arc::new(Animation::load(&path)?),
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ))
        });
    }
    fn spawn(
        &mut self,
        ctx: &egui::Context,
        job: impl FnOnce() -> anyhow::Result<Completed> + Send + 'static,
    ) {
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        self.status = "Processing…".into();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(job());
            ctx.request_repaint();
        });
    }
    fn process(&mut self, ctx: &egui::Context) {
        self.outputs.clear();
        if let Some(source) = self.source.clone() {
            let cards = self.cards.clone();
            self.spawn(ctx, move || {
                Ok(Completed::Processed(img::process(source, &cards)?))
            });
        }
    }
    fn selected_id(&self) -> u64 {
        self.preview_id
            .unwrap_or_else(|| self.cards.last().map_or(0, |card| card.id))
    }
    fn selected_animation(&self) -> Option<&Arc<Animation>> {
        let id = self.selected_id();
        if id == 0 {
            self.source.as_ref()
        } else {
            self.cards
                .iter()
                .position(|card| card.id == id)
                .and_then(|i| self.outputs.get(i))
        }
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        let style_id = egui::Id::new("img-style");
        if !ctx.data(|data| data.get_temp::<bool>(style_id).unwrap_or(false)) {
            style(ctx);
            ctx.data_mut(|data| data.insert_temp(style_id, true));
        }
        if let Some(result) = self.receiver.as_ref().and_then(|r| match r.try_recv() {
            Ok(v) => Some(v),
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(anyhow::anyhow!("Worker stopped unexpectedly")))
            }
            Err(_) => None,
        }) {
            self.receiver = None;
            match result {
                Ok(Completed::Loaded(source, name)) => {
                    self.source = Some(source);
                    self.name = name;
                    self.preview_id = None;
                    self.process(ctx);
                }
                Ok(Completed::Processed(outputs)) => {
                    self.outputs = outputs;
                    self.preview.reset();
                    self.status.clear();
                }
                Ok(Completed::Saved(path)) => self.status = format!("Saved {}", path.display()),
                Ok(Completed::Cancelled) => self.status.clear(),
                Err(error) => self.status = format!("Error: {error:#}"),
            }
        }
        let busy = self.receiver.is_some();
        let mut changed = false;
        egui::SidePanel::left("operations")
            .default_width(360.)
            .width_range(320. ..=460.)
            .resizable(true)
            .frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(10))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.source_step(ui, busy);
                        ui.add_space(8.);
                        changed |= self.operation_rows(ui, busy);
                        ui.horizontal(|ui| {
                            changed |= self.add_operation(ui, busy);
                            self.export(ui, busy);
                        });
                        self.notices(ui, busy);
                    });
            });
        if changed {
            self.process(ctx);
        }
        let updating = self.receiver.is_some();
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(12))
            .show(ctx, |ui| self.preview_pane(ui, updating));
        if self.receiver.is_some()
            || (!self.preview.paused
                && self
                    .selected_animation()
                    .is_some_and(|animation| animation.frames.len() > 1))
        {
            ctx.request_repaint_after(Duration::from_millis(20));
        }
    }

    fn preview_pane(&mut self, ui: &mut egui::Ui, busy: bool) {
        let id = self.selected_id();
        if let Some(animation) = self.selected_animation().cloned() {
            self.last_preview = Some((id, animation));
        }
        let displayed = if self.selected_animation().is_some() || busy {
            self.last_preview.clone()
        } else {
            None
        };
        if let Some((display_id, animation)) = displayed {
            let height = ui.available_height();
            egui::ScrollArea::vertical()
                .id_salt("preview-content")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.preview.show(ui, &animation, display_id, height)
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(if busy {
                    "Loading…"
                } else if self.source.is_some() {
                    "Preview unavailable."
                } else {
                    "Choose a GIF to preview."
                });
            });
        }
    }
    fn notices(&mut self, ui: &mut egui::Ui, busy: bool) {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 28.),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_height(28.);
                if self.status.is_empty() {
                    return;
                }
                if busy {
                    ui.spinner();
                    ui.label(RichText::new(&self.status).size(12.).color(MUTED));
                } else {
                    if ui.small_button("×").on_hover_text("Dismiss").clicked() {
                        self.status.clear();
                    }
                    if self.status.starts_with("Error:") {
                        ui.menu_button("Error", |ui| {
                            ui.set_max_width(300.);
                            ui.label(&self.status);
                        });
                    } else {
                        ui.add(
                            egui::Label::new(RichText::new(&self.status).size(12.).color(MUTED))
                                .truncate(),
                        )
                        .on_hover_text(&self.status);
                    }
                }
            },
        );
    }
    fn choose_source(&mut self, ctx: &egui::Context) {
        self.spawn(ctx, || {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("GIF", &["gif"])
                .pick_file()
            else {
                return Ok(Completed::Cancelled);
            };
            Ok(Completed::Loaded(
                Arc::new(Animation::load(&path)?),
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ))
        });
    }
    fn export(&mut self, ui: &mut egui::Ui, busy: bool) {
        let output = if self.cards.is_empty() {
            self.source.clone()
        } else {
            self.outputs.last().cloned()
        };
        if ui
            .add_enabled(
                !busy && output.is_some(),
                egui::Button::new(RichText::new("Export result…").color(Color32::WHITE))
                    .fill(Color32::from_gray(35)),
            )
            .on_hover_text("Save the final output of all enabled steps.")
            .clicked()
        {
            let output = output.unwrap();
            self.spawn(ui.ctx(), move || {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("GIF", &["gif"])
                    .set_file_name("edited.gif")
                    .save_file()
                else {
                    return Ok(Completed::Cancelled);
                };
                output.save(&path)?;
                Ok(Completed::Saved(path))
            });
        }
    }
}
impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
}
fn style(ctx: &egui::Context) {
    #[cfg(target_os = "macos")]
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/SFNS.ttf") {
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("system".into(), Arc::new(egui::FontData::from_owned(bytes)));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "system".into());
        ctx.set_fonts(fonts);
    }
    ctx.set_theme(egui::Theme::Light);
    ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
    ctx.style_mut(|style| {
        let (surface, faint, text, border) = (255, 250, 35, 230);
        let visuals = &mut style.visuals;
        visuals.panel_fill = Color32::from_gray(surface);
        visuals.window_fill = Color32::from_gray(surface);
        visuals.faint_bg_color = Color32::from_gray(faint);
        visuals.extreme_bg_color = Color32::from_gray(255);
        visuals.override_text_color = Some(Color32::from_gray(text));
        visuals.hyperlink_color = Color32::from_gray(text);
        visuals.window_stroke = Stroke::new(0.75_f32, Color32::from_gray(border));
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.popup_shadow = egui::epaint::Shadow::NONE;
        visuals.window_corner_radius = egui::CornerRadius::same(2);
        visuals.menu_corner_radius = egui::CornerRadius::same(2);
        visuals.selection.bg_fill = Color32::from_gray(242);
        visuals.selection.stroke = Stroke::new(1_f32, Color32::from_gray(text));
        for (widget, fill) in [
            (&mut visuals.widgets.noninteractive, surface),
            (&mut visuals.widgets.inactive, 255),
            (&mut visuals.widgets.hovered, 233),
            (&mut visuals.widgets.active, 220),
            (&mut visuals.widgets.open, 233),
        ] {
            widget.corner_radius = egui::CornerRadius::same(2);
            widget.bg_fill = Color32::from_gray(fill);
            widget.weak_bg_fill = Color32::from_gray(fill);
            widget.bg_stroke = Stroke::new(0.75_f32, Color32::from_gray(border));
            widget.fg_stroke = Stroke::new(1_f32, Color32::from_gray(text));
            widget.expansion = 0.;
        }
        style.spacing.item_spacing = egui::vec2(8., 4.);
        style.spacing.button_padding = egui::vec2(8., 4.);
        style.spacing.interact_size.y = 24.;
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(13.));
    });
}

#[cfg(test)]
mod tests;
