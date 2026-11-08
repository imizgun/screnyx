use crate::abstracted::primitives::{shapes::ShapeKind, text::TextData};
use serde::Deserialize;

#[derive(Deserialize)]
pub enum LayerType {
    Shape(ShapeKind),
    Text(TextData),
    Image,
    Canvas,
    // To be added: Video
}
