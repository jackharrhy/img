use eframe::egui::{self, Color32, RichText, Stroke};
use img::{Animation, Card, Operation};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

const BLUE: Color32 = Color32::from_rgb(0, 66, 190);
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
    textures: HashMap<u64, (usize, egui::TextureHandle)>,
    start: Instant,
    paused: bool,
    elapsed: Duration,
    name: String,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            source: None,
            outputs: vec![],
            cards: vec![
                Card {
                    id: 1,
                    enabled: true,
                    operation: Operation::Resize {
                        width: 320,
                        height: 240,
                    },
                },
                Card {
                    id: 2,
                    enabled: true,
                    operation: Operation::Reverse,
                },
            ],
            next_id: 3,
            receiver: None,
            status: "Open a GIF to begin. All processing stays on your computer.".into(),
            textures: HashMap::new(),
            start: Instant::now(),
            paused: false,
            elapsed: Duration::ZERO,
            name: "No document".into(),
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
        self.status = "Working…".into();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(job());
            ctx.request_repaint();
        });
    }
    fn process(&mut self, ctx: &egui::Context) {
        self.outputs.clear();
        self.textures.clear();
        if let Some(source) = self.source.clone() {
            let cards = self.cards.clone();
            self.spawn(ctx, move || {
                Ok(Completed::Processed(img::process(source, &cards)?))
            });
        }
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        style(ctx);
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
                    self.process(ctx);
                }
                Ok(Completed::Processed(outputs)) => {
                    self.outputs = outputs;
                    self.start = Instant::now();
                    self.elapsed = Duration::ZERO;
                    self.status = "Ready · previews show each step's output".into();
                }
                Ok(Completed::Saved(path)) => self.status = format!("Saved {}", path.display()),
                Ok(Completed::Cancelled) => self.status = "Cancelled".into(),
                Err(error) => self.status = format!("Error: {error:#}"),
            }
        }
        let busy = self.receiver.is_some();
        let mut changed = false;
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("img").strong().size(24.).color(BLUE));
                ui.separator();
                if ui
                    .add_enabled(!busy, egui::Button::new("Open GIF…"))
                    .clicked()
                {
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
                let final_output = self.outputs.last().cloned().or_else(|| {
                    if self.cards.is_empty() {
                        self.source.clone()
                    } else {
                        None
                    }
                });
                if ui
                    .add_enabled(
                        !busy && final_output.is_some(),
                        egui::Button::new(RichText::new("Export GIF…").color(Color32::WHITE))
                            .fill(BLUE),
                    )
                    .clicked()
                {
                    let output = final_output.unwrap();
                    self.spawn(ctx, move || {
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
                ui.separator();
                if ui
                    .button(if self.paused { "Play" } else { "Pause" })
                    .clicked()
                {
                    if self.paused {
                        self.start = Instant::now();
                    } else {
                        self.elapsed += self.start.elapsed();
                    }
                    self.paused = !self.paused;
                }
                ui.label(&self.name);
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if busy {
                    ui.spinner();
                }
                ui.label(&self.status);
            });
        });
        let millis = (self.elapsed
            + if self.paused {
                Duration::ZERO
            } else {
                self.start.elapsed()
            })
        .as_millis() as u64;
        egui::CentralPanel::default().show(ctx,|ui| {
            egui::ScrollArea::vertical().show(ui,|ui| {
                ui.horizontal(|ui|{ui.heading("Operation stack");ui.label("Top to bottom · drag a blue heading to reorder");});
                ui.add_space(8.);
                egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.,Color32::from_gray(185))).inner_margin(10).show(ui,|ui| {
                    ui.columns(2,|cols| {
                        cols[0].label(RichText::new("SOURCE").strong().color(BLUE));
                        cols[0].label("Original GIF · unchanged");
                        if let Some(source)=&self.source {info(&mut cols[0],source);preview(&mut cols[1],source,0,millis,&mut self.textures);}else{cols[1].label("Open a GIF using the toolbar.\nOr pass a GIF path when launching img.");cols[1].set_min_height(120.);}
                    });
                });
                let mut remove=None; let mut movement=None; let card_count=self.cards.len();
                for index in 0..self.cards.len() {
                    ui.add_space(8.);
                    let card=&mut self.cards[index];
                    let response=egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.,Color32::from_gray(185))).show(ui,|ui|{
                        ui.add_enabled_ui(!busy,|ui| {
                            egui::Frame::new().fill(BLUE).inner_margin(6).show(ui,|ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.dnd_drag_source(egui::Id::new(("drag",card.id)),index,|ui|{ ui.label(RichText::new(format!("::  {:02}  {}",index+1,match card.operation {Operation::Resize{..}=>"Resize",Operation::Reverse=>"Reverse",Operation::ImageMagickNegate=>"Negate · ImageMagick"})).strong().color(Color32::WHITE)); });
                                    if ui.small_button("Up").clicked() && index>0 {movement=Some((index,index-1));}
                                    if ui.small_button("Down").clicked() && index+1<card_count {movement=Some((index,index+1));}
                                    if ui.small_button("Remove").clicked(){remove=Some(index);}
                                });
                            });
                        });
                        egui::Frame::new().inner_margin(10).show(ui,|ui|{ui.columns(2,|cols| {
                            cols[0].add_enabled_ui(!busy,|ui| {
                                changed|=ui.checkbox(&mut card.enabled,"Enabled").changed();
                                match &mut card.operation {
                                    Operation::Resize {width,height}=>{
                                        ui.label("Dimensions (pixels)");
                                        ui.horizontal(|ui|{ui.label("W");changed|=ui.add(egui::DragValue::new(width).range(1..=4096)).changed();ui.label("H");changed|=ui.add(egui::DragValue::new(height).range(1..=4096)).changed();});
                                        ui.label("Triangle filter · independent width / height");
                                    },
                                    Operation::Reverse=>{ui.label("Reverse frame order.");ui.label("Each frame keeps its original delay.");},
                                    Operation::ImageMagickNegate=>{ui.label("Invert RGB via local ImageMagick 7.");ui.label("Requires magick on PATH. 30 second limit.");},
                                }
                            });
                            if let Some(output)=self.outputs.get(index) {preview(&mut cols[1],output,card.id,millis,&mut self.textures);}else{cols[1].set_min_height(150.);cols[1].label(if busy{"Processing…"}else{"Preview available after import"});}
                        });});
                    }).response;
                    if !busy && let Some(from) = response.dnd_release_payload::<usize>() { movement = Some((*from, index)); }
                }
                if let Some(index)=remove {self.cards.remove(index);changed=true;}
                if let Some((from,to))=movement && from!=to && to<self.cards.len() {let card=self.cards.remove(from);self.cards.insert(to,card);changed=true;}
                ui.add_space(10.);
                ui.add_enabled_ui(!busy && self.cards.len()<16,|ui|{ui.horizontal(|ui|{
                    for (title,operation) in [("+ Resize",Operation::Resize {width:320,height:240}),("+ Reverse",Operation::Reverse),("+ Negate (ImageMagick)",Operation::ImageMagickNegate)] {
                        if ui.button(title).clicked(){self.cards.push(Card {id:self.next_id,enabled:true,operation});self.next_id+=1;changed=true;}
                    }
                    ui.label("Maximum 16 steps");
                });});
            });
        });
        if changed {
            self.process(ctx);
        }
        if !self.paused || busy {
            ctx.request_repaint_after(Duration::from_millis(20));
        }
    }
}
impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
}
fn info(ui: &mut egui::Ui, animation: &Animation) {
    let f = &animation.frames[0];
    ui.label(format!(
        "{} × {} · {} frames",
        f.pixels.width(),
        f.pixels.height(),
        animation.frames.len()
    ));
}
fn preview(
    ui: &mut egui::Ui,
    animation: &Animation,
    id: u64,
    millis: u64,
    textures: &mut HashMap<u64, (usize, egui::TextureHandle)>,
) {
    let total: u64 = animation
        .frames
        .iter()
        .map(|f| f.delay_ms.max(20) as u64)
        .sum();
    let mut time = millis % total.max(1);
    let mut index = 0;
    for (i, f) in animation.frames.iter().enumerate() {
        index = i;
        if time < (f.delay_ms.max(20) as u64) {
            break;
        }
        time -= f.delay_ms.max(20) as u64;
    }
    let frame = &animation.frames[index];
    if textures.get(&id).is_none_or(|(old, _)| *old != index) {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [
                frame.pixels.width() as usize,
                frame.pixels.height() as usize,
            ],
            frame.pixels.as_raw(),
        );
        let texture =
            ui.ctx()
                .load_texture(format!("preview-{id}"), image, egui::TextureOptions::LINEAR);
        textures.insert(id, (index, texture));
    }
    let texture = &textures[&id].1;
    egui::Frame::new()
        .fill(Color32::from_gray(224))
        .show(ui, |ui| {
            ui.set_min_height(144.);
            ui.set_min_width(ui.available_width());
            ui.add(
                egui::Image::new(texture)
                    .max_size(egui::vec2(ui.available_width().min(420.), 144.))
                    .maintain_aspect_ratio(true),
            );
        });
    info(ui, animation);
    ui.label(format!(
        "Frame {} / {} · {} ms",
        index + 1,
        animation.frames.len(),
        frame.delay_ms
    ));
}
fn style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::light();
    style.visuals.panel_fill = Color32::from_gray(242);
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
    style.visuals.selection.bg_fill = BLUE;
    style.visuals.selection.stroke = Stroke::new(1., Color32::WHITE);
    for widget in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        widget.corner_radius = egui::CornerRadius::ZERO;
    }
    style.spacing.item_spacing = egui::vec2(7., 5.);
    style.spacing.button_padding = egui::vec2(7., 3.);
    ctx.set_style(style);
}
