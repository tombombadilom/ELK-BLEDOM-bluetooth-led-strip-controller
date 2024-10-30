# ELK-BLEDOM-bluetooth-led-strip-controller
A Rust controller for sending commands to Chinese generic Bluetooth LED strip controllers (ELK-BLEDOM).

## Features
- Control RGB LED strips via Bluetooth
- Set static colors with RGB values
- Adjust brightness
- Built-in animations (Flash, Strobe, Fade, Smooth)
- Warm white mode with adjustable intensity
- Power on/off control

## Building from source

### Prerequisites
- Rust toolchain (install via [rustup](https://rustup.rs/))
- Linux: `libudev-dev` package for Bluetooth support
  ```bash
  sudo apt install libudev-dev # Debian/Ubuntu
  sudo dnf install systemd-devel # Fedora
  ```

### Compilation
```bash
cargo build --release
```

## Usage
See the `examples` directory for examples of usage.


