pub mod fast_resizer;

use std::{error::Error, fmt::Display};

use crate::image::{Image, ImageSaveError};

#[derive(Debug)]
pub enum ResizeError {
    ResizeBufferError(String),
    ResizeError(String),
}

impl From<ResizeError> for ImageSaveError {
    fn from(value: ResizeError) -> Self {
        ImageSaveError::OtherError(value.to_string())
    }
}
impl Error for ResizeError {}

impl Display for ResizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ResizeFilter {
    Nearest,
    Bilinear,
    Hamming,
    CatmullRom,
    Mitchell,
    Gaussian,
    #[default]
    Lanczos3,
}

pub trait Resizer {
    fn resize<T>(
        &mut self,
        source_image: &T,
        target_size: (u32, u32),
        filter: ResizeFilter,
    ) -> Result<T, ResizeError>
    where
        T: Image;
}
