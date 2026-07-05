use heapless::Vec;
use crate::core::views::abstracted::layer_type::LayerType;

pub struct Scene {
    pub layers: Vec<LayerType, 128>
}