use std::{error::Error, fmt::Display, io};

pub mod image_crate;
pub mod rgba_image;

#[derive(Debug)]
pub enum ImageLoadError {
    IOError(String),
    DecodingError(String),
    ParameterError(String),
    UnsupportedError(String),
    OtherError(String),
}

impl Error for ImageLoadError {}
impl Display for ImageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl From<io::Error> for ImageLoadError {
    fn from(value: io::Error) -> Self {
        ImageLoadError::IOError(value.to_string())
    }
}

#[derive(Debug)]
pub enum ImageSaveError {
    IOError(String),
    EncodingError(String),
    ParameterError(String),
    UnsupportedError(String),
    OtherError(String),
}
impl From<std::io::Error> for ImageSaveError {
    fn from(value: std::io::Error) -> Self {
        ImageSaveError::IOError(value.to_string())
    }
}

impl Error for ImageSaveError {}
impl Display for ImageSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Rgba8,
    Rgb8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Ico,
    Jpeg,
    Webp,
    Bmp,
}

impl ImageFormat {
    #[must_use]
    pub fn extensions_str(self) -> &'static [&'static str] {
        match self {
            ImageFormat::Png => &["png"],
            ImageFormat::Jpeg => &["jpg", "jpeg"],
            ImageFormat::Webp => &["webp"],
            ImageFormat::Bmp => &["bmp"],
            ImageFormat::Ico => &["ico"],
        }
    }
}

pub trait Image: Sized {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn as_bytes(&self) -> &[u8];
    fn pixel_format(&self) -> PixelFormat;
    fn from_parts(width: u32, height: u32, data: Vec<u8>, pixel_format: PixelFormat) -> Self;
    ///width, height, data, pixel format
    fn to_parts(self) -> (u32, u32, Vec<u8>, PixelFormat);
}

pub trait ImageReader {
    fn load<T>(&self, path: &str, format: ImageFormat) -> Result<T, ImageLoadError>
    where
        T: Image;
}

pub trait ImageWriter {
    fn save<T>(&self, path: &str, image: &T, format: ImageFormat) -> Result<(), ImageSaveError>
    where
        T: Image;
}
