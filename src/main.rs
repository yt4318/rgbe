//! Game Boy Emulator - Entry Point
//!
//! This is the main entry point for the Game Boy emulator.
//! It handles command line arguments and starts the emulation.

use gbemu::emu::Emulator;
use gbemu::ui::Ui;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        process::exit(1);
    }

    let rom_path = &args[1];

    // Create emulator
    let mut emulator = match Emulator::new(rom_path) {
        Ok(emu) => emu,
        Err(e) => {
            eprintln!("Failed to initialize emulator: {}", e);
            process::exit(1);
        }
    };

    // Create UI and run
    let mut ui = match Ui::new() {
        Ok(ui) => ui,
        Err(e) => {
            eprintln!("Failed to initialize UI: {}", e);
            eprintln!("Running in headless mode...");
            if let Err(e) = emulator.run() {
                eprintln!("Emulator error: {}", e);
                process::exit(1);
            }
            return;
        }
    };

    if let Err(e) = ui.run(&mut emulator) {
        eprintln!("Emulator error: {}", e);
        process::exit(1);
    }
}
