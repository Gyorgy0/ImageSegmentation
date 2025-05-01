use std::{io::BufReader, num::Wrapping, path::Path, process::id, sync::Arc};

use egui::{
    emath::OrderedFloat,
    epaint::TextureManager,
    load::{self, SizedTexture},
    Color32, ColorImage, FontImage, Image, ImageData, ImageSource, Pos2, Rect, TextBuffer,
    TextureFilter, TextureHandle, TextureId, TextureOptions, Ui, Vec2,
};
use egui_extras::install_image_loaders;
use egui_file_dialog::FileDialog;
use image::load;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ImageSegmentationApp {
    #[serde(skip)] // This how you opt-out of serialization of a field
    pub selected_image_path: String,
    #[serde(skip)]
    pub selected_image: Arc<Image<'static>>,
    #[serde(skip)]
    pub file_dialog: FileDialog,
}

impl Default for ImageSegmentationApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            selected_image_path: String::new(),
            selected_image: Arc::new(Image::new(ImageSource::Uri(std::borrow::Cow::Borrowed("")))),
            file_dialog: FileDialog::new()
                .title("Kép kiválasztása...")
                .add_file_filter(
                    "Image files",
                    Arc::new(|path| {
                        path.extension().unwrap_or_default() == "png"
                            || path.extension().unwrap_or_default() == "jpg"
                            || path.extension().unwrap_or_default() == "jpeg"
                    }),
                ),
        }
    }
}

impl ImageSegmentationApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        install_image_loaders(&cc.egui_ctx);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
}

impl eframe::App for ImageSegmentationApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Kép feltöltése...").clicked() {
                self.file_dialog.pick_file();
            }
            if self.file_dialog.update(ctx).picked().is_some() {
                self.selected_image_path = format!(
                    "file://{}",
                    self.file_dialog
                        .picked()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                );
            }
            if !self.selected_image_path.is_empty() {
                let img = Image::new(&self.selected_image_path).fit_to_original_size(1_f32);
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.image(img.source(ctx));
                    });
                });
            }
        });
    }
}
