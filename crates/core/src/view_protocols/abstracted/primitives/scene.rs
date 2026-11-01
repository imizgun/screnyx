use crate::view_protocols::abstracted::primitives::layer_type::LayerType;
use heapless::Vec;

pub struct Scene {
    pub layers: Vec<LayerType, 128>,
}
