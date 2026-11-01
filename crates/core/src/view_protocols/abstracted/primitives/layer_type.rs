use crate::view_protocols::abstracted::primitives::{shapes::ShapeKind, text::TextData};

pub enum LayerType {
    Shape(ShapeKind),
    Text(TextData),
    Image,
    Canvas,
    // To be added: Video
}
