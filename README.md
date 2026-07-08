flashing:
```
cargo build --release
cargo flash --release --chip STM32F411RETx
```

sending UART messages: 

```python3 src/send_circle.py /dev/ttyACM0 --cx 64 --cy 32 --radius 15```