#![no_std]
#![no_main]

use heapless::Vec;

use esp_hal::clock::CpuClock;
use esp_hal::uart::{Config, Uart};
use esp_hal::main;
use esp_println::println;

use screnyx_core::line_reader::LineReader;
use screnyx_core::view_protocols::raw::pixel::Pixel;
use screnyx_data_receivers::data_receiver::DataReceiver;
use screnyx_data_receivers::uart_receiver::UartReceiver;

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

    let uart = Uart::new(periph.UART0, Config::default())
        .expect("uart: failed to apply config")
        .with_tx(periph.GPIO43)
        .with_rx(periph.GPIO44);
    
    // UART0 (/dev/ttyACM1) carries only incoming data; logs go through
    // println! to USB-Serial-JTAG (/dev/ttyACM0).
    let (rx, _tx) = uart.split();
    let mut receiver = UartReceiver::new(rx);

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
