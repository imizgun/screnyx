#![no_std]
#![no_main]

mod core;
pub mod data_receivers;

use cortex_m_rt::entry;
use embedded_graphics::{Drawable, Pixel};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{DrawTarget, Point};
use panic_halt as _;
use heapless::Vec;
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, I2CInterface};
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use stm32f4xx_hal::pac;
use stm32f4xx_hal::gpio::gpiob::Parts;
use stm32f4xx_hal::gpio::{Alternate, GpioExt, PA2, PA3};
use stm32f4xx_hal::i2c::I2c;
use stm32f4xx_hal::pac::{I2C1, USART2};
use stm32f4xx_hal::prelude::{_fugit_RateExtU32, _stm32f4xx_hal_time_U32Ext};
use stm32f4xx_hal::rcc::{Rcc, RccExt};
use stm32f4xx_hal::serial::{Config, Serial};
use crate::core::view_protocols::raw::pixel;
use crate::data_receivers::data_receiver::DataReceiver;
use crate::data_receivers::uart_receiver::UartReceiver;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let mut rcc = peripherals.RCC.constrain();
    let gpiob = peripherals.GPIOB.split(&mut rcc);
    let gpioa = peripherals.GPIOA.split(&mut rcc);

    let mut display = display_init(peripherals.I2C1, gpiob, &mut rcc);

    let tx_pin = gpioa.pa2.into_alternate::<7>();
    let rx_pin = gpioa.pa3.into_alternate::<7>();

    let serial = serial_init(peripherals.USART2, tx_pin, rx_pin, &mut rcc);

    let (_, rx) = serial.split();
    let mut receiver = UartReceiver::new(rx);

    let mut buf: Vec<u8, 256> = Vec::new();
    let mut chunk = [0u8; 64];

    display.clear(BinaryColor::Off).unwrap();
    display.flush().unwrap();

    loop {
        if let Ok(n) = receiver.read_bytes(&mut chunk) {
            for &byte in &chunk[..n] {
                if byte == b'\n' {
                    if buf.as_slice() == b"CLEAR" {
                        let _ = display.clear(BinaryColor::Off);
                        let _ = display.flush();
                    } else if let Ok((data, _)) = serde_json_core::from_slice::<pixel::Pixel>(&buf) {
                        draw_pixel_on_screen(&mut display, data);
                        let _ = display.flush();
                    }
                    buf.clear();
                }
                else {
                    let _ = buf.push(byte);
                }
            }
        }
    }
}

fn display_init(i2c1: I2C1, gpiob: Parts, rcc: &mut Rcc) ->  Ssd1306<I2CInterface<I2c<I2C1>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>> {
    let i2c = I2c::new(
        i2c1,
        (gpiob.pb8, gpiob.pb9),
        400.kHz(),
        rcc,
    );

    let di = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(
        di,
        DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    display
}

fn serial_init(usart2: USART2,
               tx_pin: PA2<Alternate<7>>,
               rx_pin: PA3<Alternate<7>>,
               rcc: &mut Rcc) -> Serial<USART2, u8> {
    Serial::new(
        usart2,
        (tx_pin, rx_pin),
        Config::default()
            .baudrate(115200.bps())
            .wordlength_8()
            .parity_none(),
            rcc,
    ).unwrap()
}

fn draw_pixel_on_screen<D: DrawTarget<Color = BinaryColor>>(screen: &mut D, pixel: pixel::Pixel) {
    let _ = Pixel(Point::new(pixel.x as i32, pixel.y as i32), BinaryColor::On).draw(screen);
}