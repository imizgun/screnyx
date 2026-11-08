use embedded_graphics::{
    Drawable, draw_target::DrawTarget, mono_font::{MonoTextStyle, MonoTextStyleBuilder, iso_8859_9::FONT_6X10}, pixelcolor::Rgb565, primitives::{Circle, Primitive, Rectangle}, text::Text,
};

use crate::abstracted::primitives::{shapes::ShapeKind, text::TextData};

pub trait Renderer {
    // fn bounds(&self) -> Rectangle;
    fn render<D: DrawTarget<Color = Rgb565>>(&self, target: &mut D) -> Result<(), D::Error>;
}

impl Renderer for ShapeKind {
    fn render<D: DrawTarget<Color = Rgb565>>(&self, t: &mut D) -> Result<(), D::Error> {
        match *self {
            ShapeKind::Circle {
                center,
                radius,
                style,
            } => Circle::with_center(center, radius * 2)
                .into_styled(style.into())
                .draw(t),
            ShapeKind::Rect {
                corner_1,
                corner_2,
                style,
            } => Rectangle::with_corners(corner_1, corner_2)
                .into_styled(style.into())
                .draw(t),
        }
    }
    // fn bounds(&self) -> Rectangle {
    //     // стилизованные примитивы и Text реализуют Dimensions,
    //     // и bounding_box() уже учитывает толщину обводки
    //     /* тот же Styled<...> */.bounding_box()
    // }
}

impl Renderer for TextData {
    fn render<D: DrawTarget<Color = Rgb565>>(&self, t: &mut D) -> Result<(), D::Error> {
        let style: MonoTextStyle<Rgb565> = self.style.into();
        
        Text::new(&self.content.as_str(), self.origin, style)
            .draw(t)?;

        Ok(())
    }
}
