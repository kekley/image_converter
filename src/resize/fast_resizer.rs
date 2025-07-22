use fast_image_resize::{FilterType, ImageBufferError, PixelType, ResizeOptions};

use crate::image::{Image, PixelFormat};

use super::{ResizeError, ResizeFilter, Resizer};

#[derive(Debug, Default)]
pub struct FastResizer {
    inner: fast_image_resize::Resizer,
}
#[expect(clippy::match_same_arms)]
impl From<FilterType> for ResizeFilter {
    fn from(value: FilterType) -> Self {
        match value {
            FilterType::Box => ResizeFilter::Nearest,
            FilterType::Bilinear => ResizeFilter::Bilinear,
            FilterType::Hamming => ResizeFilter::Hamming,
            FilterType::CatmullRom => ResizeFilter::CatmullRom,
            FilterType::Mitchell => ResizeFilter::Mitchell,
            FilterType::Gaussian => ResizeFilter::Gaussian,
            FilterType::Lanczos3 => ResizeFilter::Lanczos3,
            _ => ResizeFilter::Nearest,
        }
    }
}

impl From<ImageBufferError> for ResizeError {
    fn from(value: ImageBufferError) -> Self {
        ResizeError::ResizeBufferError(value.to_string())
    }
}

impl From<fast_image_resize::ResizeError> for ResizeError {
    fn from(value: fast_image_resize::ResizeError) -> Self {
        ResizeError::ResizeError(value.to_string())
    }
}

struct FastResizeFilterType(fast_image_resize::FilterType);

impl From<ResizeFilter> for FastResizeFilterType {
    fn from(value: ResizeFilter) -> Self {
        match value {
            ResizeFilter::Nearest => FastResizeFilterType(FilterType::Box),
            ResizeFilter::Bilinear => FastResizeFilterType(FilterType::Bilinear),
            ResizeFilter::Hamming => FastResizeFilterType(FilterType::Hamming),
            ResizeFilter::CatmullRom => FastResizeFilterType(FilterType::CatmullRom),
            ResizeFilter::Mitchell => FastResizeFilterType(FilterType::Mitchell),
            ResizeFilter::Gaussian => FastResizeFilterType(FilterType::Gaussian),
            ResizeFilter::Lanczos3 => FastResizeFilterType(FilterType::Lanczos3),
        }
    }
}

impl From<PixelFormat> for PixelType {
    fn from(value: PixelFormat) -> Self {
        match value {
            PixelFormat::Rgba8 => PixelType::U8x4,
            PixelFormat::Rgb8 => PixelType::U8x3,
        }
    }
}

#[expect(clippy::unimplemented)]
impl From<PixelType> for PixelFormat {
    fn from(value: PixelType) -> Self {
        match value {
            PixelType::U8x3 => PixelFormat::Rgb8,
            PixelType::U8x4 => PixelFormat::Rgba8,
            //everything is converted to rgba8 at the moment
            _ => unimplemented!(),
        }
    }
}

impl Resizer for FastResizer {
    fn resize<T>(
        &mut self,
        source_image: &T,
        target_size: (u32, u32),
        filter: ResizeFilter,
    ) -> Result<T, ResizeError>
    where
        T: Image,
    {
        let source_image_ref = fast_image_resize::images::ImageRef::new(
            source_image.width(),
            source_image.height(),
            source_image.as_bytes(),
            source_image.pixel_format().into(),
        )?;
        let mut resized_image_buffer = fast_image_resize::images::Image::new(
            target_size.0,
            target_size.1,
            source_image.pixel_format().into(),
        );
        self.inner.resize(
            &source_image_ref,
            &mut resized_image_buffer,
            &ResizeOptions::new().resize_alg(fast_image_resize::ResizeAlg::Convolution(
                FastResizeFilterType::from(filter).0,
            )),
        )?;
        let pixel_format = PixelFormat::from(resized_image_buffer.pixel_type());

        let image = Image::from_parts(
            resized_image_buffer.width(),
            resized_image_buffer.height(),
            resized_image_buffer.into_vec(),
            pixel_format,
        );

        Ok(image)
    }
}
