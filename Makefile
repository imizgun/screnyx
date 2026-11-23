.PHONY: build flash monitor all test

# Logs (println!) come out of the USB-Serial-JTAG port.
PORT ?= /dev/ttyACM0
BIN = ./target/xtensa-esp32s3-none-elf/release/screnyx
HOST ?= $(shell rustc +stable -vV | sed -n 's/^host: //p')

build:
	cargo build --release

flash:
	espflash flash --port $(PORT) $(BIN)

monitor:
	espflash monitor --port $(PORT)

all: build flash monitor

# Host-side unit tests for the platform-independent core crate.
# Stable toolchain ignores the `[unstable] build-std` from .cargo/config.toml.
test:
	cargo +stable test -p screnyx-core --target $(HOST)
