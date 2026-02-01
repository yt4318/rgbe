//! SDL2 User Interface
//!
//! This module implements the SDL2-based user interface for the emulator.

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use std::time::{Duration, Instant};

use crate::emu::Emulator;
use crate::gamepad::Button;

/// Game Boy screen dimensions
pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 144;
/// Scale factor for the window
pub const SCALE: u32 = 4;

/// SDL2 UI wrapper
pub struct Ui {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    texture_creator: TextureCreator<WindowContext>,
}

impl Ui {
    /// Create a new UI instance
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "rgbe - Game Boy Emulator",
                SCREEN_WIDTH * SCALE,
                SCREEN_HEIGHT * SCALE,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();
        let event_pump = sdl_context.event_pump()?;

        Ok(Self {
            canvas,
            event_pump,
            texture_creator,
        })
    }


    /// Run the emulator with UI
    pub fn run(&mut self, emulator: &mut Emulator) -> Result<(), String> {
        let mut texture = self
            .texture_creator
            .create_texture_streaming(
                PixelFormatEnum::ARGB8888,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .map_err(|e| e.to_string())?;

        let frame_duration = Duration::from_secs_f64(1.0 / 60.0);

        'running: loop {
            let frame_start = Instant::now();

            // Handle events
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown { keycode: Some(key), .. } => {
                        if key == Keycode::Escape {
                            break 'running;
                        }
                        if let Some(button) = keycode_to_button(key) {
                            emulator.set_button(button, true);
                        }
                    }
                    Event::KeyUp { keycode: Some(key), .. } => {
                        if let Some(button) = keycode_to_button(key) {
                            emulator.set_button(button, false);
                        }
                    }
                    _ => {}
                }
            }

            // Run one frame of emulation
            emulator.run_frame();

            // Update texture with video buffer
            let video_buffer = emulator.get_video_buffer();
            texture
                .update(
                    None,
                    unsafe {
                        std::slice::from_raw_parts(
                            video_buffer.as_ptr() as *const u8,
                            video_buffer.len() * 4,
                        )
                    },
                    SCREEN_WIDTH as usize * 4,
                )
                .map_err(|e| e.to_string())?;

            // Render
            self.canvas.clear();
            self.canvas.copy(&texture, None, None)?;
            self.canvas.present();

            // Frame timing
            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }

        Ok(())
    }
}

/// Convert SDL2 keycode to Game Boy button
fn keycode_to_button(keycode: Keycode) -> Option<Button> {
    match keycode {
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        Keycode::Z => Some(Button::A),
        Keycode::X => Some(Button::B),
        Keycode::Return => Some(Button::Start),
        Keycode::Backspace => Some(Button::Select),
        _ => None,
    }
}
