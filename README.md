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

sending UART messages: 

```python3 src/send_circle.py /dev/ttyACM0 --cx 64 --cy 32 --radius 15```
