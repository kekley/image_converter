use image::RgbaImage;

use crate::image::Image;
use crate::image::PixelFormat;

pub struct LoadedRgbaImage {
    inner: image::RgbaImage,
}

impl Image for LoadedRgbaImage {
    fn width(&self) -> u32 {
        self.inner.width()
    }

    fn height(&self) -> u32 {
        self.inner.height()
    }

    fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    fn pixel_format(&self) -> PixelFormat {
        PixelFormat::Rgba8
    }

    ///Panics if data is not the correct size for an RGBA8 image with the specified dimensions
    fn from_parts(width: u32, height: u32, data: Vec<u8>, _pixel_format: PixelFormat) -> Self {
        let bytes_per_pixel = 4;
        assert!(width as usize * height as usize * bytes_per_pixel == data.len());
        Self {
            inner: RgbaImage::from_raw(width, height, data).unwrap(),
        }
    }

    fn to_parts(self) -> (u32, u32, Vec<u8>, PixelFormat) {
        let width = self.inner.width();
        let height = self.inner.height();
        let pixel_format = PixelFormat::Rgba8;
        let data = self.inner.into_raw();
        let bytes_per_pixel = 4;
        assert!(width as usize * height as usize * bytes_per_pixel == data.len());

        (width, height, data, pixel_format)
    }
}
