use embedded_graphics::geometry::Point;
use heapless::String;

use crate::view_protocols::abstracted::primitives::shapes::Rgb;

pub struct TextData {
    pub origin: Point,
    pub content: String<128>,
    pub style: TextStyle,
}

pub struct TextStyle {
    // font:
    pub font_size: u16,
    pub color: Rgb,
    pub background_color: Rgb,
    pub width: u8,
    pub decoration: Option<TextDecoration>,
}

// impl<'a> Into<MonoTextStyle<'_, Rgb565>> for TextStyle {
//     fn into(self) -> MonoTextStyle<'a, Rgb565> {
//         let mut style = MonoTextStyleBuilder::new()
//             .background_color(Rgb565::from(self.background_color))
//             .text_color(Rgb565::from(self.color));
//
//         if let Some(dec) = self.decoration {
//             match dec {
//                 TextDecoration::LineThrough => { style = style.strikethrough(); }
//                 TextDecoration::Underlined => { style = style.underline(); }
//             }
//         }
//
//         style.build()
//     }
// }

pub enum TextDecoration {
    Underlined,
    LineThrough,
    // TODO: Bold, Italic
}
