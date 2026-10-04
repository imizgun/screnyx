.PHONY: build flash all

build:
	cargo build --release

flash:
	espflash flash ./target/xtensa-esp32s3-none-elf/release/screnyx

monitor:
	cargo run --release

all:
	make build flash monitor