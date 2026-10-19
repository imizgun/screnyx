Target: ESP32-S3 (`xtensa-esp32s3-none-elf`).

toolchain (one-time):
```
cargo install espup espflash
espup install
. ~/export-esp.sh
```

flashing:
```
cargo run --release
```
`cargo run` calls `espflash flash --monitor` (see `.cargo/config.toml`): it flashes the board and opens the serial monitor (Ctrl+C to exit).

To pick a port explicitly, or to flash without the monitor:
```
cargo build --release
espflash flash --port /dev/ttyACM0 --monitor target/xtensa-esp32s3-none-elf/release/screnyx
```

If the board isn't detected: hold BOOT, press RESET, release BOOT, then flash again.

The board shows up as two ports:
- `/dev/ttyACM0` — ESP32-S3 USB-Serial-JTAG: flashing and logs (`println!`), used by `make all`
- `/dev/ttyACM1` — CH343 USB-UART bridge on UART0 (GPIO43/44): data input

The numbers can swap after replugging; `/dev/serial/by-id/` has stable names
(`usb-Espressif_USB_JTAG_serial_debug_unit_*` and `usb-1a86_USB_Single_Serial_*`).

sending UART messages (in a second terminal while `make monitor` is running):

```
python3 tools/send_circle.py /dev/ttyACM1 --cx 64 --cy 32 --radius 15
```
