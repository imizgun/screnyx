#!/usr/bin/env python3
"""Send a randomly generated Scene as one JSON line over UART.

Wire format matches crates/core/src/abstracted/primitives/: externally tagged
enums, positions as [x, y], colors as [r, g, b], terminated by '\n'.
"""

import argparse
import json
import random
import re
import sys
import time

try:
    import serial
except ImportError as exc:
    raise SystemExit(
        "pyserial is required: pip install pyserial"
    ) from exc

SCREEN_WIDTH = 320
SCREEN_HEIGHT = 240
FONT_HEIGHT = 10  # FONT_6X10, text origin is the baseline

# Firmware limits: LineReader::<1024> in main.rs, Vec<LayerType, 16> in scene.rs
MAX_LINE_BYTES = 1024
MAX_LAYERS = 16

# Lines the firmware logs when a scene did not make it to the screen
DEVICE_ERRORS = ("error parse", "line too long", "render error", "UART rx error", "PANIC")
DEVICE_BOOT = "boot: init ok"

WORDS = ("hello", "screnyx", "now playing", "weather", "cpu 42%", "23:59", "lo-fi", "track 7")
DECORATIONS = (None, "Underlined", "LineThrough")


def random_color() -> list[int]:
    return [random.randint(0, 255) for _ in range(3)]


def random_pos(min_y: int = 0) -> list[int]:
    return [random.randrange(SCREEN_WIDTH), random.randint(min_y, SCREEN_HEIGHT - 1)]


def random_shape_style() -> dict:
    return {
        "border_width": random.randint(0, 4),
        "border_color": random_color(),
        "inner_color": random_color(),
    }


def random_circle() -> dict:
    return {"Shape": {"Circle": {
        "center": random_pos(),
        "radius": random.randint(5, 60),
        "style": random_shape_style(),
    }}}


def random_rect() -> dict:
    return {"Shape": {"Rect": {
        "corner_1": random_pos(),
        "corner_2": random_pos(),
        "style": random_shape_style(),
    }}}


def random_text() -> dict:
    style = {"color": random_color(), "background_color": random_color()}
    decoration = random.choice(DECORATIONS)
    if decoration is not None:
        style["decoration"] = decoration

    return {"Text": {
        "origin": random_pos(min_y=FONT_HEIGHT),
        "content": " ".join(random.choices(WORDS, k=random.randint(1, 2))),
        "style": style,
    }}


def random_scene(layers: int) -> dict:
    makers = (random_circle, random_rect, random_text)
    return {"layers": [random.choice(makers)() for _ in range(layers)]}


def hex_color(color: list[int]) -> str:
    return "#{:02x}{:02x}{:02x}".format(*color)


def describe_shape_style(style: dict) -> str:
    return (f"border={style['border_width']}px {hex_color(style['border_color'])} "
            f"fill={hex_color(style['inner_color'])}")


def describe_layer(layer: dict) -> str:
    if "Text" in layer:
        text = layer["Text"]
        style = text["style"]
        return (f"Text    origin={tuple(text['origin'])} content={text['content']!r} "
                f"color={hex_color(style['color'])} bg={hex_color(style['background_color'])} "
                f"decoration={style.get('decoration')}")

    shape = layer["Shape"]
    if "Circle" in shape:
        circle = shape["Circle"]
        return (f"Circle  center={tuple(circle['center'])} radius={circle['radius']} "
                f"{describe_shape_style(circle['style'])}")

    rect = shape["Rect"]
    return (f"Rect    {tuple(rect['corner_1'])}..{tuple(rect['corner_2'])} "
            f"{describe_shape_style(rect['style'])}")


def pretty_json(scene: dict) -> str:
    text = json.dumps(scene, indent=2, ensure_ascii=False)
    # Keep [x, y] and [r, g, b] on one line
    return re.sub(
        r"\[\s+(\d+(?:,\s+\d+)*)\s+\]",
        lambda match: "[" + re.sub(r"\s+", " ", match[1]) + "]",
        text,
    )


def send(port: str, baud: int, payload: bytes, wait: float) -> list[str]:
    """Write the payload and return what the device logged while handling it."""
    ser = serial.Serial()
    ser.port = port
    ser.baudrate = baud
    ser.timeout = 0.05
    # Boards with auto-reset wiring restart when these lines toggle
    ser.dtr = False
    ser.rts = False

    with ser:
        ser.reset_input_buffer()
        written = ser.write(payload)
        ser.flush()
        if written != len(payload):
            raise serial.SerialException(f"wrote {written} of {len(payload)} bytes")

        received = bytearray()
        deadline = time.monotonic() + len(payload) * 10 / baud + wait
        while time.monotonic() < deadline:
            received += ser.read(256)

    return received.decode("ascii", errors="replace").splitlines()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("port", help="serial port, e.g. /dev/ttyACM1 or COM3")
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--layers", type=int, help="number of layers (default: random 1-6)")
    parser.add_argument("--seed", type=int, help="seed to reproduce a scene (default: random)")
    parser.add_argument(
        "--wait",
        type=float,
        default=0.3,
        help="seconds to listen for device errors after sending",
    )
    args = parser.parse_args()

    seed = args.seed if args.seed is not None else random.randrange(2**32)
    random.seed(seed)
    layers = args.layers if args.layers is not None else random.randint(1, 6)

    if not 0 <= layers <= MAX_LAYERS:
        print(f"FAILED: --layers must be in 0..={MAX_LAYERS}")
        return 1

    scene = random_scene(layers)
    line = json.dumps(scene, separators=(",", ":"), ensure_ascii=False).encode("utf-8")

    print(f"Objects (seed {seed}):")
    for index, layer in enumerate(scene["layers"], start=1):
        print(f"  {index}. {describe_layer(layer)}")

    print("\nJSON:")
    print(pretty_json(scene))
    print()

    if len(line) > MAX_LINE_BYTES:
        print(f"FAILED: not sent, {len(line)} bytes is over the {MAX_LINE_BYTES}-byte line limit")
        return 1

    try:
        log = send(args.port, args.baud, line + b"\n", args.wait)
    except (serial.SerialException, OSError) as exc:
        print(f"FAILED: {exc}")
        return 1

    errors = [entry for entry in log if any(marker in entry for marker in DEVICE_ERRORS)]
    if errors:
        print(f"FAILED: sent {len(line) + 1} bytes, device reported:")
        for entry in errors:
            print(f"  {entry}")
        return 1

    if any(DEVICE_BOOT in entry for entry in log):
        print(f"FAILED: sent {len(line) + 1} bytes, but the device rebooted")
        return 1

    print(f"OK: sent {len(line) + 1} bytes to {args.port}, device reported no errors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
