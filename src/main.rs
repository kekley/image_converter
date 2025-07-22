#![windows_subsystem = "windows"]
use std::{error::Error, io::Cursor};

use egui::{IconData, Vec2, ViewportBuilder};
use image::ImageReader;
use image_converter::app::image_conversion::ImageConverter;

fn main() -> Result<(), Box<dyn Error>> {
    let bytes = include_bytes!("../assets/icon.png");
    let cursor = Cursor::new(bytes);
    let icon_data = ImageReader::with_format(cursor, image::ImageFormat::Png)
        .decode()
        .unwrap();
    let rgb = icon_data.to_rgba8().into_vec();
    let icon = IconData {
        rgba: rgb,
        width: icon_data.width(),
        height: icon_data.height(),
    };
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Image Converter")
            .with_min_inner_size(Vec2::new(1000.0, 800.0))
            .with_icon(icon),

        vsync: true,

        ..Default::default()
    };
    eframe::run_native(
        "Image Converter",
        native_options,
        Box::new(|cc| Ok(Box::new(ImageConverter::new(cc)))),
    )?;

    Ok(())
}
