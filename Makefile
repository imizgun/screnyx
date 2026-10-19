.PHONY: build flash monitor all

# Logs (println!) come out of the USB-Serial-JTAG port.
PORT ?= /dev/ttyACM0
BIN = ./target/xtensa-esp32s3-none-elf/release/screnyx

build:
	cargo build --release

flash:
	espflash flash --port $(PORT) $(BIN)

monitor:
	espflash monitor --port $(PORT)

all: build flash monitor
