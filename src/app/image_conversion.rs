use crate::image::image_crate::{DynImageReader, DynImageWriter};
use crate::image::{Image, ImageFormat, ImageReader, ImageWriter};
use crate::resize::Resizer;
use eframe::egui::{self, Rect};
use std::sync::Arc;
use std::{cell::RefCell, error::Error, path::PathBuf, thread::JoinHandle};

use eframe::egui::{
    Button, Checkbox, Color32, ColorImage, ComboBox, DragValue, Image as EguiImage, ImageData,
    Label, RichText, Sense, Separator, TextEdit, TextureHandle, load::SizedTexture,
};
use eframe::egui::{Context, TextBuffer, TextureOptions};
use eframe::{App, CreationContext};

use crate::{
    image::rgba_image::LoadedRgbaImage,
    resize::{ResizeFilter, fast_resizer::FastResizer},
};

#[derive(Default)]
struct ResizeSettings {
    target_width: u32,
    target_height: u32,
    resize_filter: ResizeFilter,
}

pub struct ImageConverter {
    resizer: FastResizer,
    image_reader: DynImageReader,
    image_writer: DynImageWriter,

    load_file_dialogue: Option<JoinHandle<Option<PathBuf>>>,
    src_text_box_contents: String,
    loaded_src_image: RefCell<Option<LoadedRgbaImage>>,

    save_file_dialogue: Option<JoinHandle<Option<PathBuf>>>,
    dest_text_box_contents: String,
    scaling_lock: bool,
    dest_format: ImageFormat,
    resize_settings: ResizeSettings,

    source_preview: Option<TextureHandle>,
    preview_dirty: bool,
    output_preview: Option<TextureHandle>,

    load_result: Option<Result<(), Box<dyn Error>>>,
    save_result: Option<Result<(), Box<dyn Error>>>,
}

impl ImageConverter {
    fn upload_image_to_texture(
        image: &LoadedRgbaImage,
        ctx: &Context,
        texture_name: &str,
    ) -> TextureHandle {
        let size = [image.width() as usize, image.height() as usize];
        let color_image = Arc::new(ColorImage::from_rgba_unmultiplied(size, image.as_bytes()));
        let image_data = ImageData::Color(color_image);
        ctx.load_texture(texture_name, image_data, TextureOptions::default())
    }
    fn load_image(
        path: &str,
        image_reader: &DynImageReader,
    ) -> Result<LoadedRgbaImage, Box<dyn Error>> {
        let image = image_reader.load::<LoadedRgbaImage>(path, ImageFormat::Png)?;
        Ok(image)
    }
    fn save_image(
        path: &str,
        image_writer: &DynImageWriter,
        image: &LoadedRgbaImage,
        format: ImageFormat,
    ) -> Result<(), Box<dyn Error>> {
        image_writer.save(path, image, format)?;
        Ok(())
    }
    fn resize_image(
        resizer: &mut FastResizer,
        image: &LoadedRgbaImage,
        settings: &ResizeSettings,
    ) -> Result<LoadedRgbaImage, Box<dyn Error>> {
        let resized_image = resizer.resize(
            image,
            (settings.target_width, settings.target_height),
            settings.resize_filter,
        )?;

        Ok(resized_image)
    }
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl Default for ImageConverter {
    fn default() -> Self {
        Self {
            dest_format: ImageFormat::Ico,
            load_file_dialogue: Default::default(),
            src_text_box_contents: Default::default(),
            save_file_dialogue: Default::default(),
            dest_text_box_contents: Default::default(),
            scaling_lock: true,
            loaded_src_image: Default::default(),
            source_preview: Default::default(),
            output_preview: None,
            load_result: None,
            save_result: None,
            resizer: FastResizer::default(),
            image_reader: DynImageReader::default(),
            image_writer: DynImageWriter::default(),
            resize_settings: ResizeSettings::default(),
            preview_dirty: true,
        }
    }
}

impl App for ImageConverter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("File Panel").show(ctx, |ui| {
            let available_width = ui.available_width();
            egui::Sides::new()
                .spacing(available_width - 900.0)
                .shrink_right()
                .show(
                    ui,
                    |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    TextEdit::singleline(&mut self.src_text_box_contents)
                                        .hint_text("Source file...")
                                        .interactive(false),
                                );
                                if ui.add(Button::new("Browse")).clicked()
                                    && self.load_file_dialogue.is_none()
                                {
                                    const SUPPORTED_FORMATS: [&str; 5] =
                                        ["png", "jpg", "webp", "ico", "bmp"];
                                    self.load_file_dialogue = Some(std::thread::spawn(move || {
                                        rfd::FileDialog::new()
                                            .add_filter("Image Formats", &SUPPORTED_FORMATS)
                                            .pick_file()
                                    }));
                                }
                                if let Some(result) = &self.load_result {
                                    match result {
                                        Ok(_) => {
                                            ui.add(
                                                Label::new(
                                                    RichText::new("✅").color(Color32::GREEN),
                                                )
                                                .selectable(false),
                                            );
                                        }
                                        Err(err) => {
                                            let error_string = err.to_string();
                                            if ui
                                                .add(
                                                    Label::new(
                                                        RichText::new("❌ (hover for full error)")
                                                            .color(Color32::RED),
                                                    )
                                                    .selectable(false)
                                                    .sense(Sense::hover() | Sense::click()),
                                                )
                                                .on_hover_text(format!(
                                                    "Right click to copy: {error_string}"
                                                ))
                                                .secondary_clicked()
                                            {
                                                ctx.copy_text(error_string);
                                            };
                                        }
                                    }
                                }
                            });
                            if let Some(image) = self.loaded_src_image.borrow().as_ref() {
                                ui.add(Label::new(format!(
                                    "X: {}, Y: {}",
                                    image.width(),
                                    image.height()
                                )));
                            }
                        });
                    },
                    |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    TextEdit::singleline(&mut self.dest_text_box_contents)
                                        .hint_text("Destination file...")
                                        .interactive(false),
                                );
                                if ui
                                    .add_enabled(
                                        self.loaded_src_image.borrow().is_some(),
                                        Button::new("Save as"),
                                    )
                                    .clicked()
                                    && self.save_file_dialogue.is_none()
                                {
                                    self.save_file_dialogue = Some(std::thread::spawn(move || {
                                        rfd::FileDialog::new().save_file()
                                    }));
                                }
                                if ui
                                    .add_enabled(
                                        !self.dest_text_box_contents.is_empty(),
                                        Button::new("Save"),
                                    )
                                    .clicked()
                                {
                                    if let Some(image_to_resize) =
                                        self.loaded_src_image.borrow_mut().as_mut()
                                    {
                                        match Self::resize_image(
                                            &mut self.resizer,
                                            image_to_resize,
                                            &self.resize_settings,
                                        ) {
                                            Ok(resized_image) => match Self::save_image(
                                                &self.dest_text_box_contents,
                                                &self.image_writer,
                                                &resized_image,
                                                self.dest_format,
                                            ) {
                                                Ok(_) => self.save_result = Some(Ok(())),
                                                Err(err) => self.save_result = Some(Err(err)),
                                            },
                                            Err(err) => self.save_result = Some(Err(err)),
                                        }
                                    }
                                }
                                if let Some(save_result) = &self.save_result {
                                    match save_result {
                                        Ok(_) => {
                                            ui.add(
                                                Label::new(
                                                    RichText::new("✅").color(Color32::GREEN),
                                                )
                                                .selectable(false),
                                            );
                                        }
                                        Err(err) => {
                                            let error_string = err.to_string();
                                            if ui
                                                .add(
                                                    Label::new(
                                                        RichText::new("❌ (hover for full error)")
                                                            .color(Color32::RED),
                                                    )
                                                    .selectable(false)
                                                    .sense(Sense::hover() | Sense::click()),
                                                )
                                                .on_hover_text(format!(
                                                    "Right click to copy: {error_string}"
                                                ))
                                                .secondary_clicked()
                                            {
                                                ctx.copy_text(error_string);
                                            };
                                        }
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Convert to...");
                                        ComboBox::from_label("Format")
                                            .selected_text(format!("{:?}", self.dest_format))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.dest_format,
                                                    ImageFormat::Ico,
                                                    "ico",
                                                );

                                                ui.selectable_value(
                                                    &mut self.dest_format,
                                                    ImageFormat::Png,
                                                    "png",
                                                );

                                                ui.selectable_value(
                                                    &mut self.dest_format,
                                                    ImageFormat::Jpeg,
                                                    "jpg",
                                                );

                                                ui.selectable_value(
                                                    &mut self.dest_format,
                                                    ImageFormat::Webp,
                                                    "webp",
                                                );
                                            });
                                    });
                                    ui.horizontal(|ui| {
                                        let source_image_borrow = self.loaded_src_image.borrow();
                                        let aspect_ratio = if let Some(source_image) =
                                            source_image_borrow.as_ref()
                                        {
                                            source_image.width() as f32
                                                / source_image.height() as f32
                                        } else {
                                            1.0
                                        };
                                        let range = match self.dest_format {
                                            ImageFormat::Ico => 1..=256,
                                            _ => 1..=10000,
                                        };
                                        if ui
                                            .add(
                                                DragValue::new(
                                                    &mut self.resize_settings.target_width,
                                                )
                                                .range(range.clone())
                                                .speed(1.0)
                                                .update_while_editing(false)
                                                .prefix("X: "),
                                            )
                                            .changed()
                                        {
                                            self.preview_dirty = true;
                                            if self.scaling_lock {
                                                self.resize_settings.target_height =
                                                    (self.resize_settings.target_width as f32
                                                        * (1.0 / aspect_ratio))
                                                        as u32;
                                            }
                                        }
                                        if ui
                                            .add(
                                                DragValue::new(
                                                    &mut self.resize_settings.target_height,
                                                )
                                                .range(range)
                                                .speed(1.0)
                                                .update_while_editing(false)
                                                .prefix("Y: "),
                                            )
                                            .changed()
                                        {
                                            self.preview_dirty = true;
                                            if self.scaling_lock {
                                                self.resize_settings.target_width =
                                                    (self.resize_settings.target_height as f32
                                                        * aspect_ratio)
                                                        as u32;
                                            }
                                        };

                                        ui.add(Checkbox::new(
                                            &mut self.scaling_lock,
                                            "Lock Aspect Ratio",
                                        ));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Scaling Filter:");
                                        ComboBox::from_label("Scaling")
                                            .selected_text(format!(
                                                "{:?}",
                                                self.resize_settings.resize_filter
                                            ))
                                            .show_ui(ui, |ui| {
                                                if ui
                                                    .selectable_value(
                                                        &mut self.resize_settings.resize_filter,
                                                        ResizeFilter::Nearest,
                                                        "Nearest",
                                                    )
                                                    .changed()
                                                {
                                                    self.preview_dirty = true;
                                                };

                                                if ui
                                                    .selectable_value(
                                                        &mut self.resize_settings.resize_filter,
                                                        ResizeFilter::Bilinear,
                                                        "Bilinear",
                                                    )
                                                    .changed()
                                                {
                                                    self.preview_dirty = true;
                                                }

                                                if ui
                                                    .selectable_value(
                                                        &mut self.resize_settings.resize_filter,
                                                        ResizeFilter::CatmullRom,
                                                        "CatmullRom",
                                                    )
                                                    .changed()
                                                {
                                                    self.preview_dirty = true;
                                                }

                                                if ui
                                                    .selectable_value(
                                                        &mut self.resize_settings.resize_filter,
                                                        ResizeFilter::Gaussian,
                                                        "Gaussian",
                                                    )
                                                    .changed()
                                                {
                                                    self.preview_dirty = true;
                                                }

                                                if ui
                                                    .selectable_value(
                                                        &mut self.resize_settings.resize_filter,
                                                        ResizeFilter::Lanczos3,
                                                        "Lanczos3",
                                                    )
                                                    .changed()
                                                {
                                                    self.preview_dirty = true;
                                                }
                                                if ui
                                                    .selectable_value(
                                                        &mut self.resize_settings.resize_filter,
                                                        ResizeFilter::Hamming,
                                                        "Hamming",
                                                    )
                                                    .changed()
                                                {
                                                    self.preview_dirty = true;
                                                }
                                                if ui
                                                    .selectable_value(
                                                        &mut self.resize_settings.resize_filter,
                                                        ResizeFilter::Mitchell,
                                                        "Mitchell",
                                                    )
                                                    .changed()
                                                {
                                                    self.preview_dirty = true;
                                                }
                                            })
                                    });
                                });
                                ui.separator();
                            });
                        });
                    },
                );
        });

        if let Some(src_fd) = self.load_file_dialogue.take() {
            if src_fd.is_finished() {
                match src_fd.join() {
                    Ok(path_opt) => {
                        if let Some(path) = path_opt {
                            self.src_text_box_contents = path.to_string_lossy().to_string();
                            if let Ok(exists) = path.try_exists() {
                                if exists {
                                    match Self::load_image(
                                        path.to_string_lossy().as_str(),
                                        &self.image_reader,
                                    ) {
                                        Ok(loaded_image) => {
                                            self.dest_text_box_contents.clear();
                                            let source_preview = Self::upload_image_to_texture(
                                                &loaded_image,
                                                ctx,
                                                "Source Preview",
                                            );
                                            self.source_preview = Some(source_preview);
                                            self.resize_settings.target_width =
                                                loaded_image.width();
                                            self.resize_settings.target_height =
                                                loaded_image.height();
                                            if let Ok(resized_image) = Self::resize_image(
                                                &mut self.resizer,
                                                &loaded_image,
                                                &self.resize_settings,
                                            ) {
                                                let new_preview = Self::upload_image_to_texture(
                                                    &resized_image,
                                                    ctx,
                                                    "Output preview",
                                                );
                                                self.output_preview = Some(new_preview);
                                                self.preview_dirty = false;
                                            } else {
                                                eprintln!("error showing preview?");
                                            }
                                            let mut source_borrow =
                                                self.loaded_src_image.borrow_mut();
                                            *source_borrow = Some(loaded_image);
                                            self.load_result = Some(Ok(()));
                                        }
                                        Err(err) => self.load_result = Some(Err(err)),
                                    }
                                }
                            }
                        }
                    }
                    Err(panic_message) => eprintln!("{panic_message:?}"),
                }
            } else {
                self.load_file_dialogue = Some(src_fd);
            }
        }
        if let Some(dest_fd) = self.save_file_dialogue.take() {
            if dest_fd.is_finished() {
                match dest_fd.join() {
                    Ok(path_opt) => {
                        if let Some(path) = path_opt {
                            self.dest_text_box_contents.clear();
                            self.dest_text_box_contents
                                .push_str(path.to_string_lossy().to_string().as_str());
                            if !self.dest_format.extensions_str().iter().any(|ext_str| {
                                let mut extension_string = String::from(".");
                                extension_string.push_str(ext_str);
                                self.dest_text_box_contents
                                    .ends_with(extension_string.as_str())
                            }) {
                                let mut extension_string = String::from(".");
                                extension_string
                                    .push_str(self.dest_format.extensions_str().first().unwrap());
                                self.dest_text_box_contents
                                    .push_str(extension_string.as_str());
                            }
                            let source_borrow = self.loaded_src_image.borrow();
                            if let Some(source_image) = source_borrow.as_ref() {
                                match Self::resize_image(
                                    &mut self.resizer,
                                    source_image,
                                    &self.resize_settings,
                                ) {
                                    Ok(resized_image) => {
                                        match Self::save_image(
                                            self.dest_text_box_contents.as_str(),
                                            &self.image_writer,
                                            &resized_image,
                                            self.dest_format,
                                        ) {
                                            Ok(_) => self.save_result = Some(Ok(())),
                                            Err(err) => self.save_result = Some(Err(err)),
                                        }
                                    }
                                    Err(err) => self.save_result = Some(Err(err)),
                                }
                            }
                        }
                    }
                    Err(panic_message) => eprintln!("{panic_message:?}"),
                }
            } else {
                self.save_file_dialogue = Some(dest_fd);
            }
        }

        if self.preview_dirty {
            let source_borrow = self.loaded_src_image.borrow();
            if let Some(source_image) = source_borrow.as_ref() {
                if let Ok(resized_image) =
                    Self::resize_image(&mut self.resizer, source_image, &self.resize_settings)
                {
                    let new_preview =
                        Self::upload_image_to_texture(&resized_image, ctx, "Output Preview");

                    self.preview_dirty = false;
                    self.output_preview = Some(new_preview);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input(|r| {
                r.raw.dropped_files.iter().for_each(|file| {
                    println!("hi");
                    dbg!(file);
                    ui.painter()
                        .rect_filled(Rect::EVERYTHING, 0.0, Color32::GREEN);
                });
            });
            ctx.input(|r| {
                r.raw.hovered_files.iter().for_each(|file| {
                    println!("hi");
                    dbg!(file);
                });
            });
            let separator_size = 5.0;
            let width = ui.available_width() - separator_size;
            let height = ui.available_height();
            let half_width = width / 2.0;

            ui.horizontal(|ui| {
                let (left_rect, _left_response) =
                    ui.allocate_exact_size([half_width, height].into(), Sense::empty());
                let (separator_rect, _) =
                    ui.allocate_exact_size([separator_size, height].into(), Sense::empty());
                let (right_rect, _right_response) =
                    ui.allocate_exact_size([half_width, height].into(), Sense::empty());

                if let Some(texture_handle) = &self.source_preview {
                    ui.put(
                        left_rect,
                        EguiImage::new(SizedTexture::from_handle(texture_handle))
                            .maintain_aspect_ratio(true)
                            .max_width(half_width)
                            .max_height(height),
                    );
                }
                ui.put(
                    separator_rect,
                    Separator::default().vertical().spacing(separator_size),
                );

                if let Some(texture_handle) = &self.output_preview {
                    ui.put(
                        right_rect,
                        EguiImage::new(SizedTexture::from_handle(texture_handle))
                            .maintain_aspect_ratio(true)
                            .max_width(half_width)
                            .max_height(height),
                    );
                }
            });
        });
    }
}
