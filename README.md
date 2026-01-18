# glimmer_led

A Rust project for the Raspberry Pi Pico 2 (RP2350).

## Building

To compile the project:

```bash
cargo build --release
```

## Flashing

You can flash the device using `probe-rs`. Ensure your debug probe is connected.

```bash
# Flash and run
cargo run --release

# Or using probe-rs directly
probe-rs run --chip RP2350 --release
```