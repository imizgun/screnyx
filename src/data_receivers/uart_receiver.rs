use crate::data_receivers::data_receiver::DataReceiver;
use esp_hal::uart::{RxError, UartRx};
use esp_hal::Blocking;

pub struct UartReceiver<'d> {
    rx: UartRx<'d, Blocking>,
}

impl<'d> UartReceiver<'d> {
    pub fn new(rx: UartRx<'d, Blocking>) -> Self {
        UartReceiver { rx }
    }
}

impl DataReceiver for UartReceiver<'_> {
    type Error = RxError;

    fn read_bytes(&mut self, buf_write: &mut [u8]) -> Result<usize, Self::Error> {
          self.rx.read_buffered(buf_write)
    }
}
