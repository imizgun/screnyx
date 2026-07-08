use crate::data_receivers::data_receiver::DataReceiver;
use embedded_hal_nb::nb;
use embedded_hal_nb::serial::Read;
use stm32f4xx_hal::serial::{Error, Instance, Rx};

pub struct UartReceiver<USART: Instance> {
    rx: Rx<USART, u8>,
}

impl<USART: Instance> UartReceiver<USART> {
    pub fn new(rx: Rx<USART, u8>) -> Self {
        UartReceiver { rx }
    }
}

impl<USART: Instance> DataReceiver for UartReceiver<USART> {
    type Error = Error;

    fn read_bytes(&mut self, buf_write: &mut [u8]) -> Result<usize, Self::Error> {
        let mut count = 0;

        while count < buf_write.len() {
            match self.rx.read() {
                Ok(byte) => {
                    buf_write[count] = byte;
                    count += 1;
                }
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(e)) => return Err(e),
            }
        }

        Ok(count)
    }
}