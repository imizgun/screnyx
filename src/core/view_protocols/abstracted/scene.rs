use heapless::Vec;
use crate::core::view_protocols::abstracted::layer_type::LayerType;

pub struct Scene {
    pub layers: Vec<LayerType, 128>
}