#![no_std]
#![no_main]

mod data_receivers;
mod core;

use ::core::fmt::Write;

use esp_hal::clock::CpuClock;
use esp_hal::uart::{Config, Uart};
use esp_hal::main;
use esp_println::println;

use crate::data_receivers::data_receiver::DataReceiver;
use crate::data_receivers::uart_receiver::UartReceiver;

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
    let (rx, mut tx) = uart.split();
    let mut receiver = UartReceiver::new(rx);

    println!("boot: init ok");
    writeln!(tx, "boot: init ok").unwrap();

    let mut buf = [0u8; 64];

    loop {
        match receiver.read_bytes(&mut buf) {
            Ok(0) => {}
            Ok(n) => {
                tx.write(&buf[..n]).unwrap();
            }
            Err(e) => writeln!(tx, "UART rx error: {e:?}").unwrap(),
        }
    }
}
