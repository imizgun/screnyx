use embedded_graphics::{
    geometry::Point,
    pixelcolor::Rgb565,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
};
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
pub struct Pos(pub i32, pub i32); // json: [x, y]

impl From<Pos> for Point {
    fn from(value: Pos) -> Self {
        Point::new(value.0, value.1)
    }
}

#[derive(Deserialize)]
pub enum ShapeKind {
    Circle {
        center: Pos,
        radius: u32,
        style: ShapeStyle,
    },
    Rect {
        corner_1: Pos,
        corner_2: Pos,
        style: ShapeStyle,
    },
}

#[derive(Clone, Copy, Deserialize)]
pub struct Rgb(pub u8, pub u8, pub u8); // json: [r, g, b]

impl From<Rgb> for Rgb565 {
    fn from(value: Rgb) -> Self {
        Rgb565::new(value.0 >> 3, value.1 >> 2, value.2 >> 3)
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

#[derive(Clone, Copy, Deserialize)]
pub struct ShapeStyle {
    pub border_width: u16,
    pub border_color: Rgb, // simple rgb, not rgb565
    pub inner_color: Rgb,  // simple rgb, not rgb565
}
