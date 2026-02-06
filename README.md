# rgbe - Game Boy Emulator in Rust

A Game Boy emulator written in Rust.

This project was created 100% with AI using Kiro IDE × Claude Opus 4.5 and GPT-5.3-Codex.
It was built by refactoring [yt4318/gbemu](https://github.com/yt4318/gbemu).

## Features

- CPU (all instructions)
- PPU (graphics processing)
- APU (audio processing)
- Timer
- Cartridge loading (MBC1, MBC3, etc.)
- Gamepad input handling
- SDL2 window and rendering

## Requirements

- Rust (1.70+)
- SDL2

### Installing SDL2

**Ubuntu/Debian:**
```bash
sudo apt install libsdl2-dev
```

**macOS:**
```bash
brew install sdl2
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
./target/release/gbemu-rust <rom_file>
```

## Usage Example

```bash
./target/release/gbemu-rust ~/roms/game.gb
```

## Controls

| Key | Action |
|-----|--------|
| Arrow Keys | D-Pad |
| Z | A Button |
| X | B Button |
| Enter | Start |
| Backspace | Select |
| Escape | Quit |

## License

MIT
