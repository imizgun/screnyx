#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Level;
use esp_hal::main;
use esp_hal::rmt::{PulseCode, Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::time::Rate;
use esp_println::println;
use smart_leds::RGB8;
use smart_leds::hsv::{Hsv, hsv2rgb};

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {info}");
    esp_hal::system::software_reset()
}

// WS2812 bit timings at an 80MHz / 1 RMT tick rate (12.5ns/tick):
// T0H=400ns, T0L=850ns, T1H=800ns, T1L=450ns, reset low >=50us.
const T0H: u16 = 32;
const T0L: u16 = 68;
const T1H: u16 = 64;
const T1L: u16 = 36;
const RESET_TICKS: u16 = 22_400; // ~280us

fn encode_ws2812(rgb: RGB8, buf: &mut [PulseCode; 25]) {
    // This board's WS2812 wants bits in R, G, B byte order, MSB first
    // (confirmed empirically: sending GRB showed red as green).
    let bytes = [rgb.r, rgb.g, rgb.b];
    let mut idx = 0;
    for byte in bytes {
        for bit_pos in (0..8).rev() {
            let is_one = (byte >> bit_pos) & 1 != 0;
            buf[idx] = if is_one {
                PulseCode::new(Level::High, T1H, Level::Low, T1L)
            } else {
                PulseCode::new(Level::High, T0H, Level::Low, T0L)
            };
            idx += 1;
        }
    }
    // length2 == 0 also serves as the transmission end marker.
    buf[idx] = PulseCode::new(Level::Low, RESET_TICKS, Level::Low, 0);
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let periph = esp_hal::init(config);
    let delay = Delay::new();

    println!("boot: init ok");

    let rmt = Rmt::new(periph.RMT, Rate::from_mhz(80)).expect("failed to initialize RMT");
    println!("boot: rmt ok");

    let tx_config = TxChannelConfig::default().with_clk_divider(1);
    
    let mut channel = rmt
        .channel0
        .configure_tx(&tx_config)
        .expect("failed to configure RMT tx channel")
        .with_pin(periph.GPIO38);
    
    println!("boot: rmt channel ok");

    let mut hsv = Hsv {hue:0, sat: 255, val: 65 };

    let mut buf = [PulseCode::default(); 25];

    loop {
        let color = hsv2rgb(hsv);
        encode_ws2812(color, &mut buf);
        let transaction = channel.transmit(&buf).expect("failed to start transmit");
        channel = transaction.wait().expect("failed to complete transmit");
        delay.delay_millis(30);
        hsv.hue = hsv.hue.wrapping_add(1);
    }
}