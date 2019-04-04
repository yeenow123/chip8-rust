extern crate sdl2;

mod audio;
mod cpu;
mod graphics;
mod input;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use std::{thread, time};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    // Graphics
    let mut graphics = graphics::GraphicsDevice::new(&sdl_context);

    // Input
    let input = Rc::new(RefCell::new(input::InputDevice::new(&sdl_context)));

    // Audio
    let _audio = audio::AudioDeviceWrapper::new(&sdl_context);

    // Initialize and load game
    let args: Vec<String> = env::args().collect();
    let mut f: File = File::open(&args[1]).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    let mut cpu = cpu::Chip8::new();
    cpu.initialize(buffer);

    // Game loop
    'game: loop {
        // Check for input
        let resp = input.borrow_mut().poll();
        match resp {
            input::InputResponse::Terminate => break 'game,
            _ => (),
        }

        cpu.emulate_cycle(Rc::clone(&input));

        if cpu.draw_flag {
            graphics.draw(&cpu.gfx);
        }

        if cpu.beep {
            _audio.start_play();
        } else {
            _audio.stop_play();
        }

        thread::sleep(time::Duration::from_millis(3));
    }
    Ok(())
}
