use super::*;

impl Editor {
    pub(super) fn source_step(&mut self, ui: &mut egui::Ui, busy: bool) {
        step_frame(self.selected_id() == 0 && self.source.is_some()).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    ui.set_opacity(0.35);
                    grip(ui)
                })
                .inner
                .on_hover_text("The input stays first.");
                step_label(ui, "Import GIF");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(self.source.is_some(), |ui| {
                        if preview_button(ui, self.source.is_some() && self.selected_id() == 0)
                            .clicked()
                        {
                            self.preview_id = Some(0);
                        }
                    });
                });
            });
            ui.add_space(4.);
            if let Some(source) = &self.source {
                ui.add(egui::Label::new(RichText::new(&self.name).strong()).truncate())
                    .on_hover_text(&self.name);
                let pixels = &source.frames[0].pixels;
                ui.label(
                    RichText::new(format!(
                        "{} × {} pixels, {} frames",
                        pixels.width(),
                        pixels.height(),
                        source.frames.len()
                    ))
                    .size(12.)
                    .color(MUTED),
                );
            } else {
                ui.label("Choose a GIF.");
                ui.label(RichText::new(" ").size(12.));
            }
            ui.add_space(4.);
            if ui
                .add_enabled(
                    !busy,
                    egui::Button::new(if self.source.is_some() {
                        "Replace GIF…"
                    } else {
                        "Choose GIF…"
                    }),
                )
                .on_hover_text(
                    "The original file stays unchanged. Other steps use its decoded frames.",
                )
                .clicked()
            {
                self.choose_source(ui.ctx());
            }
        });
    }
    pub(super) fn add_operation(&mut self, ui: &mut egui::Ui, busy: bool) -> bool {
        let mut changed = false;
        ui.add_enabled_ui(
            !busy && self.source.is_some() && self.cards.len() < 16,
            |ui| {
                ui.menu_button("Add operation", |ui| {
                    for (title, operation) in [
                        (
                            "Resize",
                            Operation::Resize {
                                width: 320,
                                height: 240,
                            },
                        ),
                        ("Reverse", Operation::Reverse),
                        ("Negate", Operation::Negate),
                    ] {
                        if ui.button(title).clicked() {
                            self.cards.push(Card {
                                id: self.next_id,
                                enabled: true,
                                operation,
                            });
                            self.next_id += 1;
                            self.preview_id = None;
                            changed = true;
                            ui.close_menu();
                        }
                    }
                });
            },
        );
        changed
    }
    pub(super) fn operation_rows(&mut self, ui: &mut egui::Ui, busy: bool) -> bool {
        let selected = self.selected_id();
        let last_id = self.cards.last().map(|card| card.id);
        let mut changed = false;
        let mut remove = None;
        let mut movement = None;
        let mut rows = Vec::with_capacity(self.cards.len());
        let count = self.cards.len();
        for (index, card) in self.cards.iter_mut().enumerate() {
            let drag_id = egui::Id::new(("drag", card.id));
            let lifting = !busy
                && (egui::DragAndDrop::payload::<u64>(ui.ctx()).is_some_and(|id| *id == card.id)
                    || ui.ctx().drag_started_id() == Some(drag_id));
            let layer = if lifting {
                egui::LayerId::new(
                    egui::Order::Tooltip,
                    egui::Id::new(("moving-card", card.id)),
                )
            } else {
                ui.layer_id()
            };
            let card_top = ui.cursor().min;
            let response = ui
                .push_id(card.id, |ui| {
                    ui.scope_builder(egui::UiBuilder::new().layer_id(layer), |ui| {
                        if lifting {
                            ui.set_clip_rect(egui::Rect::EVERYTHING);
                        }
                        if !card.enabled {
                            ui.set_opacity(0.45);
                        }
                        let mut frame = step_frame(selected == card.id);
                        if lifting {
                            frame.shadow = egui::epaint::Shadow {
                                offset: [0, 4],
                                blur: 16,
                                spread: 0,
                                color: Color32::from_black_alpha(32),
                            };
                        }
                        frame
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.add_enabled_ui(!busy, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 3.;
                                        let handle = grip(ui);
                                        let handle = ui
                                            .interact(
                                                handle.rect,
                                                egui::Id::new(("drag", card.id)),
                                                egui::Sense::drag(),
                                            )
                                            .on_hover_cursor(egui::CursorIcon::Grab)
                                            .on_hover_text("Drag to reorder. Escape cancels.");
                                        if handle.drag_started() {
                                            let origin = ui
                                                .input(|i| i.pointer.press_origin())
                                                .unwrap_or(handle.rect.center());
                                            ui.ctx().data_mut(|data| {
                                                data.insert_temp(
                                                    drag_id.with("offset"),
                                                    origin - card_top,
                                                )
                                            });
                                        }
                                        handle.dnd_set_drag_payload(card.id);
                                        if step_icon(ui, card.id, StepAction::Up, index > 0)
                                            .clicked()
                                        {
                                            movement = Some((index, index - 1));
                                        }
                                        if step_icon(
                                            ui,
                                            card.id,
                                            StepAction::Down,
                                            index + 1 < count,
                                        )
                                        .clicked()
                                        {
                                            movement = Some((index, index + 1));
                                        }
                                        if step_icon(
                                            ui,
                                            card.id,
                                            StepAction::Visibility(card.enabled),
                                            true,
                                        )
                                        .clicked()
                                        {
                                            card.enabled = !card.enabled;
                                            changed = true;
                                        }
                                        step_label(ui, operation_name(&card.operation));
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if step_icon(ui, card.id, StepAction::Delete, true)
                                                    .clicked()
                                                {
                                                    remove = Some(index);
                                                }
                                                if preview_button(ui, selected == card.id).clicked()
                                                {
                                                    self.preview_id = if Some(card.id) == last_id {
                                                        None
                                                    } else {
                                                        Some(card.id)
                                                    };
                                                }
                                            },
                                        );
                                    });
                                    ui.add_space(4.);
                                    changed |= operation_controls(ui, &mut card.operation);
                                });
                            })
                            .response
                    })
                    .inner
                })
                .inner;
            if lifting {
                ui.painter()
                    .rect_filled(response.rect, 3., Color32::from_gray(247));
                ui.painter().rect_stroke(
                    response.rect,
                    3.,
                    Stroke::new(0.75, Color32::from_gray(225)),
                    egui::StrokeKind::Inside,
                );
                if let Some(pointer) = ui.ctx().pointer_interact_pos() {
                    let offset = ui
                        .ctx()
                        .data(|data| data.get_temp::<egui::Vec2>(drag_id.with("offset")))
                        .unwrap_or_default();
                    let delta = pointer - offset - response.rect.min;
                    ui.ctx().transform_layer_shapes(
                        layer,
                        egui::emath::TSTransform::from_translation(delta),
                    );
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }
            }
            rows.push(response.rect);
            ui.add_space(10.);
        }
        if !busy
            && let Some(dragged) = egui::DragAndDrop::payload::<u64>(ui.ctx())
            && let Some(from) = self.cards.iter().position(|card| card.id == *dragged)
            && let Some(pointer) = ui.ctx().pointer_interact_pos()
            && let (Some(first), Some(last)) = (rows.first(), rows.last())
            && first
                .union(*last)
                .expand(12.)
                .intersect(ui.clip_rect())
                .contains(pointer)
        {
            let slot = rows
                .iter()
                .position(|rect| pointer.y < rect.center().y)
                .unwrap_or(count);
            let to = slot.saturating_sub(usize::from(from < slot));
            if from != to {
                let y = if slot == count {
                    last.bottom() + 5.
                } else {
                    rows[slot].top() - 5.
                };
                ui.painter().line_segment(
                    [egui::pos2(first.left(), y), egui::pos2(first.right(), y)],
                    Stroke::new(2., Color32::from_gray(55)),
                );
                ui.painter()
                    .circle_filled(egui::pos2(first.left(), y), 3., Color32::from_gray(55));
                if ui.input(|i| i.pointer.any_released()) {
                    movement = Some((from, to));
                }
            }
            if ui.input(|i| i.pointer.primary_down()) {
                let clip = ui.clip_rect();
                let proximity = ((clip.top() + 40. - pointer.y) / 40.).clamp(0., 1.)
                    - ((pointer.y - clip.bottom() + 40.) / 40.).clamp(0., 1.);
                if proximity != 0. {
                    let delta = ui.input(|i| i.stable_dt.min(0.05)) * 320. * proximity;
                    ui.scroll_with_delta(egui::vec2(0., delta));
                    ui.ctx().request_repaint_after(Duration::from_millis(16));
                }
            }
        }
        if let Some(index) = remove {
            let card = self.cards.remove(index);
            if self.preview_id == Some(card.id) {
                self.preview_id = None;
            }
            changed = true;
        } else if let Some((from, to)) = movement {
            let card = self.cards.remove(from);
            self.cards.insert(to, card);
            changed = true;
        }
        changed
    }
}
fn step_frame(selected: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .inner_margin(8)
        .corner_radius(3)
        .stroke(Stroke::new(
            0.75,
            Color32::from_gray(if selected { 205 } else { 232 }),
        ))
}
#[derive(Clone, Copy)]
enum StepAction {
    Up,
    Down,
    Delete,
    Visibility(bool),
}

fn step_icon(ui: &mut egui::Ui, card_id: u64, action: StepAction, enabled: bool) -> egui::Response {
    let (name, label) = match action {
        StepAction::Up => ("up", "Move step up"),
        StepAction::Down => ("down", "Move step down"),
        StepAction::Delete => ("delete", "Delete step"),
        StepAction::Visibility(true) => ("visibility", "Bypass step"),
        StepAction::Visibility(false) => ("visibility", "Apply step"),
    };
    ui.add_enabled_ui(enabled, |ui| {
        let (_, rect) = ui.allocate_space(egui::vec2(22., 24.));
        let response = ui.interact(
            rect,
            egui::Id::new(("step-action", card_id, name)),
            egui::Sense::click(),
        );
        response.widget_info(|| {
            if let StepAction::Visibility(applied) = action {
                egui::WidgetInfo::selected(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    applied,
                    label,
                )
            } else {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
            }
        });
        let visuals = ui.style().interact(&response);
        if response.hovered() || response.is_pointer_button_down_on() {
            ui.painter().rect_filled(rect, 2., visuals.weak_bg_fill);
        }
        let stroke = Stroke::new(1.25, visuals.fg_stroke.color);
        let center = rect.center();
        let point = |x, y| center + egui::vec2(x, y);
        match action {
            StepAction::Up | StepAction::Down => {
                let direction = if matches!(action, StepAction::Up) {
                    -1.
                } else {
                    1.
                };
                ui.painter()
                    .line_segment([point(0., -5.), point(0., 5.)], stroke);
                ui.painter()
                    .line_segment([point(-4., direction), point(0., 5. * direction)], stroke);
                ui.painter()
                    .line_segment([point(4., direction), point(0., 5. * direction)], stroke);
            }
            StepAction::Visibility(open) => {
                let curve = |points| {
                    egui::epaint::CubicBezierShape::from_points_stroke(
                        points,
                        false,
                        Color32::TRANSPARENT,
                        stroke,
                    )
                };
                if open {
                    ui.painter().add(curve([
                        point(-7., 0.),
                        point(-3., -6.),
                        point(3., -6.),
                        point(7., 0.),
                    ]));
                    ui.painter().add(curve([
                        point(-7., 0.),
                        point(-3., 6.),
                        point(3., 6.),
                        point(7., 0.),
                    ]));
                    ui.painter().circle_stroke(center, 2., stroke);
                } else {
                    ui.painter().add(curve([
                        point(-7., -2.),
                        point(-3., 4.),
                        point(3., 4.),
                        point(7., -2.),
                    ]));
                    for (x, y, dx) in [(-5., 0., -2.), (0., 2.5, 0.), (5., 0., 2.)] {
                        ui.painter()
                            .line_segment([point(x, y), point(x + dx, y + 3.)], stroke);
                    }
                }
            }
            StepAction::Delete => {
                ui.painter()
                    .line_segment([point(-6., -4.), point(6., -4.)], stroke);
                ui.painter()
                    .line_segment([point(-2., -6.), point(2., -6.)], stroke);
                ui.painter().add(egui::Shape::line(
                    vec![
                        point(-4., -2.),
                        point(-3., 6.),
                        point(3., 6.),
                        point(4., -2.),
                    ],
                    stroke,
                ));
                for x in [-1.5, 1.5] {
                    ui.painter()
                        .line_segment([point(x, -1.), point(x, 4.)], stroke);
                }
            }
        }
        response.on_hover_text(label)
    })
    .inner
}

fn preview_button(ui: &mut egui::Ui, selected: bool) -> egui::Response {
    ui.add_sized(
        [82., 24.],
        egui::Button::new(if selected { "Previewing" } else { "Preview" })
            .selected(selected)
            .corner_radius(2)
            .stroke(Stroke::new(
                0.75,
                Color32::from_gray(if selected { 210 } else { 232 }),
            )),
    )
}
pub(super) fn operation_name(operation: &Operation) -> &'static str {
    match operation {
        Operation::Resize { .. } => "Resize",
        Operation::Reverse => "Reverse",
        Operation::Negate => "Negate",
    }
}
fn step_label(ui: &mut egui::Ui, label: &str) {
    egui::Frame::new()
        .fill(Color32::from_gray(247))
        .corner_radius(3)
        .inner_margin(egui::Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(label).size(13.)).selectable(false));
        });
}
fn grip(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(16., 24.), egui::Sense::hover());
    for x in [5., 10.] {
        for y in [7., 12., 17.] {
            ui.painter()
                .circle_filled(rect.min + egui::vec2(x, y), 1., Color32::from_gray(150));
        }
    }
    response
}
fn operation_controls(ui: &mut egui::Ui, operation: &mut Operation) -> bool {
    match operation {
        Operation::Resize { width, height } => {
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Width");
                    changed |= ui
                        .add(egui::DragValue::new(width).range(1..=4096).suffix(" px"))
                        .changed();
                });
                ui.vertical(|ui| {
                    ui.label("Height");
                    changed |= ui
                        .add(egui::DragValue::new(height).range(1..=4096).suffix(" px"))
                        .changed();
                });
            });
            ui.label(
                RichText::new("Independent width and height")
                    .size(12.)
                    .color(MUTED),
            );
            changed
        }
        Operation::Reverse => {
            ui.label("Reverse frames, keeping their timing.");
            false
        }
        Operation::Negate => {
            ui.label("Invert colors, preserving transparency.");
            false
        }
    }
}
