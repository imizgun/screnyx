#!/usr/bin/env python3
"""Send a circle of Pixel{x,y} JSON lines over UART for the emb3d-scrn firmware.

Wire format matches src/core/view_protocols/raw/pixel.rs: one JSON object
per line, terminated by '\n', e.g. {"x":10,"y":20}\n
"""

import argparse
import json
import math
import time

try:
    import serial
except ImportError as exc:
    raise SystemExit(
        "pyserial is required: pip install pyserial"
    ) from exc


def circle_points(cx: int, cy: int, r: int, width: int, height: int) -> list[tuple[int, int]]:
    steps = max(16, int(2 * math.pi * r) * 2)
    points: list[tuple[int, int]] = []
    seen: set[tuple[int, int]] = set()

    for i in range(steps):
        angle = 2 * math.pi * i / steps
        x = round(cx + r * math.cos(angle))
        y = round(cy + r * math.sin(angle))

        if not (0 <= x < width and 0 <= y < height):
            continue
        if (x, y) in seen:
            continue

        seen.add((x, y))
        points.append((x, y))

    return points


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("port", help="serial port, e.g. /dev/ttyUSB0 or COM3")
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--cx", type=int, default=64, help="circle center x")
    parser.add_argument("--cy", type=int, default=32, help="circle center y")
    parser.add_argument("--radius", type=int, default=20)
    parser.add_argument("--width", type=int, default=128, help="display width")
    parser.add_argument("--height", type=int, default=64, help="display height")
    parser.add_argument(
        "--delay",
        type=float,
        default=0.03,
        help="seconds between pixels (the firmware flushes the whole "
             "display over I2C on every pixel right now, so sending too "
             "fast can overrun the UART's 1-byte RX buffer)",
    )
    args = parser.parse_args()

    points = circle_points(args.cx, args.cy, args.radius, args.width, args.height)

    with serial.Serial(args.port, args.baud) as ser:
        ser.write(b"CLEAR\n")
        time.sleep(args.delay)

        for x, y in points:
            line = json.dumps({"x": x, "y": y}) + "\n"
            ser.write(line.encode("ascii"))
            time.sleep(args.delay)

    print(f"sent {len(points)} pixels")


if __name__ == "__main__":
    main()