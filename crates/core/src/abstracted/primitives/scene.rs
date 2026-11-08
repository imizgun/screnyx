use crate::abstracted::primitives::layer_type::LayerType;
use heapless::Vec;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Scene {
    pub layers: Vec<LayerType, 16>,
}
