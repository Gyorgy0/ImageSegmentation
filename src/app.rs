use std::{path::Path, sync::Arc, u8};

use egui::{
    load::{self},
    ColorImage, Image, Sense, TextureHandle, TextureOptions,
};
use egui_extras::install_image_loaders;
use egui_file_dialog::FileDialog;

use crate::kmeans::segmentation;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ImageSegmentationApp {
    #[serde(skip)] // This how you opt-out of serialization of a field
    pub selected_image_path: String,
    #[serde(skip)]
    pub selected_image: ColorImage,
    #[serde(skip)]
    pub file_dialog: FileDialog,
    #[serde(skip)]
    pub img_texture: TextureHandle,
    #[serde(skip)]
    pub seg_texture: TextureHandle,
    #[serde(skip)]
    pub segmentation_map: ColorImage,
    #[serde(skip)]
    pub segmentated: bool,
    pub k: u8,
}

impl Default for ImageSegmentationApp {
    fn default() -> Self {
        let ctx = egui::Context::default();
        Self {
            // Example stuff:
            selected_image_path: String::new(),
            selected_image: ColorImage::example(),
            img_texture: ctx.load_texture(
                "Image".to_string(),
                ColorImage::example(),
                TextureOptions::NEAREST,
            ),
            seg_texture: ctx.load_texture(
                "SegMap".to_string(),
                ColorImage::example(),
                TextureOptions::NEAREST,
            ),
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
            k: 8_u8,
            segmentation_map: ColorImage::example(),
            segmentated: false,
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
            ui.horizontal(|ui| {
                if ui.button("Kép feltöltése...").clicked() {
                    self.file_dialog.pick_file();
                }
                ui.label("Number of clusters (klaszterek száma): ");
                ui.add(egui::widgets::Slider::new(&mut self.k, 1..=u8::MAX));
                if ui.button("Segmentate image!").clicked() && !self.selected_image_path.is_empty()
                {
                    self.segmentated = true;
                    self.seg_texture = ctx.load_texture(
                        "SegMap",
                        self.selected_image.clone(),
                        TextureOptions::NEAREST,
                    );
                    self.seg_texture.set(
                        segmentation(self.selected_image.clone(), self.k).clone(),
                        TextureOptions::NEAREST,
                    );
                }
            });
            if self.file_dialog.update(ctx).picked().is_some() {
                self.selected_image_path = self
                    .file_dialog
                    .picked()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                self.selected_image = load_image_from_path(Path::new(
                    &self
                        .file_dialog
                        .picked()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                ))
                .unwrap();
            }
            if !self.selected_image_path.is_empty() {
                let img = Image::new(&self.selected_image_path).fit_to_original_size(1_f32);
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        self.img_texture = ctx.load_texture(
                            "Image",
                            self.selected_image.clone(),
                            TextureOptions::NEAREST,
                        );
                        self.img_texture
                            .set(self.selected_image.clone(), TextureOptions::NEAREST);
                        let image_texture = load::SizedTexture::new(
                            self.img_texture.id(),
                            self.img_texture.size_vec2(),
                        );
                        if !self.segmentated {
                            ui.add(
                                Image::new(Image::source(
                                    &Image::from_texture(image_texture),
                                    ui.ctx(),
                                ))
                                .sense(Sense::click_and_drag()),
                            );
                        } else if self.segmentated {
                            let segmap_texture = load::SizedTexture::new(
                                self.seg_texture.id(),
                                self.seg_texture.size_vec2(),
                            );
                            ui.add(
                                Image::new(Image::source(
                                    &Image::from_texture(segmap_texture),
                                    ui.ctx(),
                                ))
                                .sense(Sense::click_and_drag()),
                            );
                        }
                    });
                });
            }
        });
    }
}

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
