use eframe::egui;

pub fn icon() -> Result<egui::IconData, image::ImageError> {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
}
