# rgbe - Game Boy Emulator in Rust

A Game Boy emulator written in Rust.

## Requirements

- Rust (1.70+)
- SDL2

### Installing SDL2

**Ubuntu/Debian:**
```bash
sudo apt install libsdl2-dev libsdl2-ttf-dev
```

**macOS:**
```bash
brew install sdl2 sdl2_ttf
```

**Windows:**
Download SDL2 development libraries from https://libsdl.org/

## Build & Run

```bash
# Clone the repository
git clone https://github.com/yt4318/rgbe.git
cd rgbe

# Build
cargo build --release

# Run
./target/release/rgbe <rom_file>
```

## Usage Example

```bash
./target/release/rgbe ~/roms/game.gb
```

## Controls

| Key | Action |
|-----|--------|
| Arrow Keys | D-Pad |
| Z | A Button |
| X | B Button |
| Enter | Start |
| Backspace | Select |

## License

MIT
