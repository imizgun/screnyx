use core::convert::Infallible;

use embedded_graphics::{
    Pixel, draw_target::DrawTarget, geometry::Dimensions, pixelcolor::Rgb565, primitives::Rectangle,
};

pub struct DrawRegion<'a> {
    area: Rectangle,
    pixels: &'a mut [Rgb565],
}

impl<'a> DrawRegion<'a> {
    pub fn new(area: Rectangle, buf: &'a mut [Rgb565]) -> Option<Self> {
        let size = (area.size.height * area.size.width) as usize;

        Some(Self {
            area: area,
            pixels: buf.get_mut(..size)?,
        })
    }

    pub fn area(&self) -> Rectangle {
        self.area
    }

    pub fn pixels(&self) -> &[Rgb565] {
        self.pixels
    }
}

impl Dimensions for DrawRegion<'_> {
    fn bounding_box(&self) -> Rectangle {
        self.area
    }
}

impl<'a> DrawTarget for DrawRegion<'a> {
    type Color = Rgb565;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        let (width, height) = (self.area.size.width as i32, self.area.size.height as i32);

        for Pixel(p, color) in pixels {
            let l = p - self.area.top_left;

            if l.x >= 0 && l.y >= 0 && l.x < width && l.y < height {
                self.pixels[(l.y * width + l.x) as usize] = color;
            }
        }

        Ok(())
    }

    fn clear(&mut self, color: Rgb565) -> Result<(), Self::Error> {
        self.pixels.fill(color);
        Ok(())
    }
}
