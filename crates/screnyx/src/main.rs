#![no_std]
#![no_main]

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    Drawable, geometry::Point, pixelcolor::{Rgb565, Rgb888, RgbColor},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
    spi::master::{Config, Spi},
    time::Rate,
    uart::{self, Uart},
};
use esp_println::println;
use ili9341::{DisplaySize240x320, Ili9341, Orientation};
use screnyx_core::{line_reader::LineReader, view_protocols::raw::pixel::Pixel};
use screnyx_data_receivers::{data_receiver::DataReceiver, uart_receiver::UartReceiver};
use serde_json_core::heapless::Vec;

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &::core::panic::PanicInfo) -> ! {
    println!("PANIC: {info}");
    esp_hal::system::software_reset()
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let periph = esp_hal::init(config);
    let mut delay = Delay::new();

    let uart = Uart::new(periph.UART0, uart::Config::default())
        .expect("uart: failed to apply config")
        .with_tx(periph.GPIO43)
        .with_rx(periph.GPIO44);

    let screen_cs = periph.GPIO17;
    let screen_sck = periph.GPIO16;
    let screen_mosi = periph.GPIO15;
    let screen_backlight = periph.GPIO10;
    let screen_reset = periph.GPIO9;
    let screen_data_command = periph.GPIO46;

    let reset_output = Output::new(screen_reset, Level::Low, OutputConfig::default());
    let cs_output = Output::new(screen_cs, Level::High, OutputConfig::default());
    let dc_output = Output::new(screen_data_command, Level::High, OutputConfig::default());
    let mut backlight_output = Output::new(screen_backlight, Level::Low, OutputConfig::default());

    let (rx, _tx) = uart.split();
    let mut receiver = UartReceiver::new(rx);

    let spi = Spi::new(
        periph.SPI2,
        Config::default().with_frequency(Rate::from_mhz(40)),
    )
    .unwrap()
    .with_sck(screen_sck)
    .with_mosi(screen_mosi);

    let device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();

    let inteface = SPIInterface::new(device, dc_output);

    let mut screen = Ili9341::new(
        inteface,
        reset_output,
        &mut Delay::new(),
        Orientation::Landscape,
        DisplaySize240x320,
    )
    .unwrap();

    screen.clear_screen(0x0000).unwrap();
    backlight_output.set_high();

    println!("boot: init ok");

    let mut buf = [0u8; 256];
    let mut reader = LineReader::<256>::new();
    let mut pixels = Vec::<Pixel, 1024>::new();

    loop {
        match receiver.read_bytes(&mut buf) {
            Ok(0) => {}
            Ok(n) => {
                reader.feed(&buf[..n], |line| {
                    if line == b"CLEAR" {
                        pixels.clear();
                        println!("clear");
                        return;
                    }
                    match serde_json_core::from_slice::<Pixel>(line) {
                        Ok((p, _)) => {
                            println!("pix: {}, {}", p.x, p.y);
                            embedded_graphics::Pixel(Point::new(p.x as i32, p.y as i32), Rgb565::WHITE)
                                .draw(&mut screen).unwrap();
                            if pixels.push(p).is_err() {
                                println!("pixels: buffer full, dropped");
                            }
                        }
                        Err(e) => println!("error parse: {}", e),
                    }
                });
            }
            Err(e) => println!("UART rx error: {e:?}"),
        }
    }
}
