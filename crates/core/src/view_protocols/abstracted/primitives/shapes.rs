use embedded_graphics::{
    geometry::Point,
    pixelcolor::Rgb565,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
};

pub enum ShapeKind {
    Circle {
        center: Point,
        radius: u32,
        style: ShapeStyle,
    },
    Rect {
        corner_1: Point,
        corner_2: Point,
        style: ShapeStyle,
    },
}

#[derive(Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Rgb> for Rgb565 {
    fn from(value: Rgb) -> Self {
        Rgb565::new(value.r >> 3, value.g >> 2, value.b >> 3)
    }
}

impl Into<PrimitiveStyle<Rgb565>> for ShapeStyle {
    fn into(self) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::from(self.inner_color))
            .stroke_width(self.border_width as u32)
            .stroke_color(Rgb565::from(self.border_color))
            .build()
    }
}

#[derive(Clone, Copy)]
pub struct ShapeStyle {
    pub border_width: u16,
    pub border_color: Rgb, // simple rgb, not rgb565
    pub inner_color: Rgb,  // simple rgb, not rgb565
}
