#![no_std]
#![no_main]

mod core;

use cortex_m_rt::entry;
use panic_halt as _;
use embedded_graphics::Drawable;
use embedded_graphics::primitives::{Primitive};
use heapless::Vec;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use ssd1306::mode::DisplayConfig;
use stm32f4xx_hal::{i2c, pac};
use stm32f4xx_hal::gpio::GpioExt;
use stm32f4xx_hal::pac::USART2;
use stm32f4xx_hal::prelude::{_embedded_hal_serial_nb_Read, _fugit_RateExtU32, _stm32f4xx_hal_serial_SerialExt, _stm32f4xx_hal_time_U32Ext};
use stm32f4xx_hal::rcc::RccExt;
use stm32f4xx_hal::serial::{Config, Serial, Tx};
use crate::core::views::raw::pixel::Pixel;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let mut rcc = peripherals.RCC.constrain();
    let gpiob = peripherals.GPIOB.split(&mut rcc);
    let gpioa = peripherals.GPIOA.split(&mut rcc);

    let i2c = i2c::I2c::new(
        peripherals.I2C1,
        (gpiob.pb8, gpiob.pb9),
        400.kHz(),
        &mut rcc,
    );

    let di = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(
        di,
        DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    let tx_pin = gpioa.pa2.into_alternate::<7>();
    let rx_pin = gpioa.pa3.into_alternate::<7>();

    let serial: Serial<USART2, u8> = Serial::new(
        peripherals.USART2,
        (tx_pin, rx_pin),
        Config::default()
            .baudrate(115200.bps())
            .wordlength_8()
            .parity_none(),
        &mut rcc,
    ).unwrap();

    let (_, mut rx) = serial.split();

    let mut buf: Vec<u8, 128> = Vec::new();

    display.flush().unwrap();

    loop {
        if let Ok(byte) = rx.read() {
            if byte == b'\n' {
                if let Ok((data, _)) = serde_json_core::from_slice::<Pixel>(&buf) {

                }
                buf.clear();
            }
            else {
                let _ = buf.push(byte);
            }
        }
    }
}
