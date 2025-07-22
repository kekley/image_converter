use std::{
    fs::{self, File},
    io::BufWriter,
};

impl From<ImageError> for ImageLoadError {
    fn from(value: ImageError) -> Self {
        match value {
            ImageError::Decoding(decoding_error) => {
                ImageLoadError::DecodingError(decoding_error.to_string())
            }
            ImageError::Encoding(encoding_error) => {
                ImageLoadError::OtherError(encoding_error.to_string())
            }
            ImageError::Parameter(parameter_error) => {
                ImageLoadError::ParameterError(parameter_error.to_string())
            }
            ImageError::Limits(limit_error) => ImageLoadError::OtherError(limit_error.to_string()),
            ImageError::Unsupported(unsupported_error) => {
                ImageLoadError::UnsupportedError(unsupported_error.to_string())
            }
            ImageError::IoError(error) => ImageLoadError::IOError(error.to_string()),
        }
    }
}

struct ImageFormatWrapper(image::ImageFormat);

impl From<super::ImageFormat> for ImageFormatWrapper {
    fn from(val: super::ImageFormat) -> Self {
        match val {
            super::ImageFormat::Png => ImageFormatWrapper(ImageFormat::Png),
            super::ImageFormat::Ico => ImageFormatWrapper(ImageFormat::Ico),
            super::ImageFormat::Jpeg => ImageFormatWrapper(ImageFormat::Jpeg),
            super::ImageFormat::Webp => ImageFormatWrapper(ImageFormat::WebP),
            super::ImageFormat::Bmp => ImageFormatWrapper(ImageFormat::Bmp),
        }
    }
}

use image::{
    ExtendedColorType, ImageError, ImageFormat,
    codecs::ico::{IcoEncoder, IcoFrame},
    save_buffer_with_format,
};

use crate::resize::{ResizeFilter, Resizer, fast_resizer::FastResizer};

use super::{Image, ImageLoadError, ImageReader, ImageSaveError, ImageWriter, PixelFormat};
#[derive(Default)]
pub struct DynImageReader {}

#[derive(Default)]
pub struct DynImageWriter {}

impl ImageReader for DynImageReader {
    fn load<T>(&self, path: &str, _format: super::ImageFormat) -> Result<T, super::ImageLoadError>
    where
        T: Image,
    {
        let data = fs::read(path)?;
        let dyn_image = image::load_from_memory(&data)?.into_rgba8();
        let width = dyn_image.width();
        let height = dyn_image.height();
        let pixel_format = PixelFormat::Rgba8;

        let data = dyn_image.into_vec();
        let image = Image::from_parts(width, height, data, pixel_format);

        Ok(image)
    }
}

impl From<PixelFormat> for ExtendedColorType {
    fn from(value: PixelFormat) -> Self {
        match value {
            PixelFormat::Rgba8 => ExtendedColorType::Rgba8,
            PixelFormat::Rgb8 => ExtendedColorType::Rgb8,
        }
    }
}

impl From<ImageError> for ImageSaveError {
    fn from(value: ImageError) -> Self {
        match value {
            ImageError::Decoding(decoding_error) => {
                ImageSaveError::OtherError(decoding_error.to_string())
            }
            ImageError::Encoding(encoding_error) => {
                ImageSaveError::EncodingError(encoding_error.to_string())
            }
            ImageError::Parameter(parameter_error) => {
                ImageSaveError::ParameterError(parameter_error.to_string())
            }
            ImageError::Limits(limit_error) => ImageSaveError::OtherError(limit_error.to_string()),
            ImageError::Unsupported(unsupported_error) => {
                ImageSaveError::UnsupportedError(unsupported_error.to_string())
            }
            ImageError::IoError(error) => ImageSaveError::IOError(error.to_string()),
        }
    }
}

const ICO_SIZES: [u32; 9] = [16, 24, 32, 48, 64, 72, 96, 128, 256];

impl ImageWriter for DynImageWriter {
    fn save<T>(
        &self,
        path: &str,
        image: &T,
        format: super::ImageFormat,
    ) -> Result<(), super::ImageSaveError>
    where
        T: Image,
    {
        //hacky thing to get proper icon scaling on windows
        if format == crate::image::ImageFormat::Ico {
            let aspect_ratio = image.width() as f32 / image.height() as f32;
            let mut resizer = FastResizer::default();
            let mut frames = Vec::with_capacity(9);
            for size in ICO_SIZES {
                let size = if image.width() > image.height() {
                    let new_height = (size as f32 * (1.0 / aspect_ratio)) as u32;
                    (size, new_height)
                } else if image.height() > image.width() {
                    let new_width = (size as f32 * aspect_ratio) as u32;
                    (new_width, size)
                } else {
                    (size, size)
                };
                let filter = if size.0 * size.1 > image.width() * image.height() {
                    ResizeFilter::Mitchell
                } else {
                    ResizeFilter::Lanczos3
                };
                let resized = resizer.resize(image, (size.0, size.1), filter)?;
                let frame = IcoFrame::as_png(
                    resized.as_bytes(),
                    resized.width(),
                    resized.height(),
                    ExtendedColorType::Rgba8,
                )?;
                frames.push(frame);
            }
            let file = File::create(path)?;
            let buf_writer = BufWriter::new(file);
            let encoder = IcoEncoder::new(buf_writer);
            encoder.encode_images(&frames)?;
            return Ok(());
        }
        let bytes = image.as_bytes();

        save_buffer_with_format(
            path,
            bytes,
            image.width(),
            image.height(),
            ExtendedColorType::from(image.pixel_format()),
            ImageFormatWrapper::from(format).0,
        )?;
        Ok(())
    }
}
