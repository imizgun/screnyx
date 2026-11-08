use embedded_graphics::{
    Drawable, draw_target::DrawTarget, geometry::{Point, Size}, mono_font::MonoTextStyle, pixelcolor::{Rgb565, RgbColor}, primitives::{Circle, Primitive, Rectangle}, text::Text,
};

use crate::abstracted::{
    convey::draw_region::DrawRegion,
    primitives::{layer_type::LayerType, scene::Scene, shapes::ShapeKind, text::TextData},
};

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
            } => Circle::with_center(center.into(), radius * 2)
                .into_styled(style.into())
                .draw(t),
            ShapeKind::Rect {
                corner_1,
                corner_2,
                style,
            } => Rectangle::with_corners(corner_1.into(), corner_2.into())
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
        
        Text::new(&self.content.as_str(), self.origin.into(), style)
            .draw(t)?;

        Ok(())
    }
}

impl Renderer for LayerType {
    fn render<D: DrawTarget<Color = Rgb565>>(&self, t: &mut D) -> Result<(), D::Error> {
        match self {
            LayerType::Shape(shape) => shape.render(t),
            LayerType::Text(text) => text.render(t),
            LayerType::Image | LayerType::Canvas => Ok(()), // TODO: not implemented yet
        }
    }
}

/// Draws the scene strip by strip through a single buffer, so a full frame is never held in RAM.
/// Strip height is `strip.len() / screen width`.
pub fn render_scene<D: DrawTarget<Color = Rgb565>>(
    scene: &Scene,
    screen: &mut D,
    strip: &mut [Rgb565]) -> Result<(), D::Error> {
    let screen_area = screen.bounding_box();
    let Size { width, height } = screen_area.size;
    let strip_height = strip.len() as u32 / width;
    
    assert!(strip_height > 0, "render_scene: strip is narrower than the screen");

    for y in (0..height).step_by(strip_height as usize) {
        let area = Rectangle::new(
            screen_area.top_left + Point::new(0, y as i32),
            Size::new(width, strip_height.min(height - y)),
        );
        let mut region = DrawRegion::new(area, strip).unwrap();

        let Ok(()) = region.clear(Rgb565::BLACK);
        
        for layer in scene.layers.iter() {
            let Ok(()) = layer.render(&mut region);
        }

        screen.fill_contiguous(&area, region.pixels().iter().copied())?;
    }

    Ok(())
}
