use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Pixel {
    pub x: usize,
    pub y: usize
}

impl Pixel {
    pub fn new(x: usize, y: usize) -> Pixel {
        Pixel {
            x,
            y
        }
    }
}