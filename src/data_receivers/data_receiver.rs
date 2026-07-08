pub trait DataReceiver {
    type Error;
    fn read_bytes(&mut self, buf_write: &mut [u8]) -> Result<usize, Self::Error>;
}

