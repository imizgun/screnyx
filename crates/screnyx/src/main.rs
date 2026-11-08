#![no_std]
#![no_main]

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
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
use screnyx_core::{
    abstracted::{convey::renderer::render_scene, primitives::scene::Scene},
    line_reader::LineReader,
};
use screnyx_data_receivers::{data_receiver::DataReceiver, uart_receiver::UartReceiver};

const SCREEN_WIDTH: usize = 320;
const STRIP_HEIGHT: usize = 40; // 320 * 40 * 2 = 25.6 KB

// Rendered at boot to check the drawing path without UART
const TEST_SCENE: &[u8] = br#"{"layers":[{"Shape":{"Rect":{"corner_1":[10,10],"corner_2":[120,80],"style":{"border_width":2,"border_color":[255,255,255],"inner_color":[0,0,180]}}}},{"Shape":{"Circle":{"center":[200,120],"radius":40,"style":{"border_width":0,"border_color":[0,0,0],"inner_color":[255,80,0]}}}},{"Text":{"origin":[20,200],"content":"hello","style":{"color":[255,255,255],"background_color":[0,0,0],"decoration":"Underlined"}}}]}"#;

esp_bootloader_esp_idf::esp_app_desc!();

fn show_scene<D>(json: &[u8], screen: &mut D, strip: &mut [Rgb565])
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    match serde_json_core::from_slice::<Scene>(json) {
        Ok((scene, _)) => {
            if let Err(e) = render_scene(&scene, screen, strip) {
                println!("render error: {e:?}");
            }
        }
        Err(e) => println!("error parse: {}", e),
    }
}

#[panic_handler]
fn panic(info: &::core::panic::PanicInfo) -> ! {
    println!("PANIC: {info}");
    esp_hal::system::software_reset()
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let periph = esp_hal::init(config);

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
    let mut reader = LineReader::<1024>::new();
    let mut strip = [Rgb565::BLACK; SCREEN_WIDTH * STRIP_HEIGHT];

    show_scene(TEST_SCENE, &mut screen, &mut strip);

    loop {
        match receiver.read_bytes(&mut buf) {
            Ok(0) => {}
            Ok(n) => {
                let dropped = reader.feed(&buf[..n], |line| {
                    show_scene(line, &mut screen, &mut strip);
                });
                if dropped > 0 {
                    println!("line too long, dropped");
                }
            }
            Err(e) => println!("UART rx error: {e:?}"),
        }
    }
}
