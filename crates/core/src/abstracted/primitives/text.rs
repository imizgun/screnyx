use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder, jis_x0201::FONT_10X20},
    pixelcolor::Rgb565,
};
use heapless::String;
use serde::Deserialize;

use crate::abstracted::primitives::shapes::{Pos, Rgb};

#[derive(Deserialize)]
pub struct TextData {
    pub origin: Pos,
    pub content: String<128>,
    pub style: TextStyle,
}

#[derive(Clone, Copy, Deserialize)]
pub struct TextStyle {
    // font:
    // pub font_size: u16,
    pub color: Rgb,
    pub background_color: Rgb,
    // pub width: u8,
    pub decoration: Option<TextDecoration>,
}

impl<'a> Into<MonoTextStyle<'a, Rgb565>> for TextStyle {
    fn into(self) -> MonoTextStyle<'a, Rgb565> {
        let mut style = MonoTextStyleBuilder::new()
            .background_color(Rgb565::from(self.background_color))
            .text_color(Rgb565::from(self.color))
            .font(&FONT_10X20); // TODO: custom fonts

        if let Some(dec) = self.decoration {
            match dec {
                TextDecoration::LineThrough => {
                    style = style.strikethrough();
                }
                TextDecoration::Underlined => {
                    style = style.underline();
                }
            }
        }

        style.build()
    }
}

#[derive(Clone, Copy, Deserialize)]
pub enum TextDecoration {
    Underlined,
    LineThrough,
    // TODO: Bold, Italic
}
