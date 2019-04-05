use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const KEYPAD_SIZE: usize = 16;

pub enum InputResponse {
    Error,
    Success,
    Terminate,
}

pub struct InputDevice {
    pub key: [bool; KEYPAD_SIZE],
    pub key_copy: [bool; KEYPAD_SIZE],
    pub key_waiting: bool,
    events: sdl2::EventPump,
}

impl InputDevice {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let events = sdl_context.event_pump().unwrap();
        InputDevice {
            key: [false; KEYPAD_SIZE],
            key_copy: [false; KEYPAD_SIZE],
            key_waiting: false,
            events,
        }
    }

    pub fn poll(&mut self) -> InputResponse {
        for event in self.events.poll_iter() {
            if let Event::Quit { .. } = event {
                return InputResponse::Terminate;
            }
            if let Event::KeyUp { keycode, .. } = event {
                self.handle_key(keycode.unwrap(), false);
                return InputResponse::Success;
            }
            if let Event::KeyDown { keycode, .. } = event {
                self.handle_key(keycode.unwrap(), true);
                return InputResponse::Success;
            }
        }

        InputResponse::Success
    }

    pub fn handle_key(&mut self, keycode: Keycode, state: bool) {
        match keycode {
            Keycode::Num1 => self.key[0x1] = state,
            Keycode::Num2 => self.key[0x2] = state,
            Keycode::Num3 => self.key[0x3] = state,
            Keycode::Num4 => self.key[0xC] = state,
            Keycode::Q => self.key[0x4] = state,
            Keycode::W => self.key[0x5] = state,
            Keycode::E => self.key[0x6] = state,
            Keycode::R => self.key[0xD] = state,
            Keycode::A => self.key[0x7] = state,
            Keycode::S => self.key[0x8] = state,
            Keycode::D => self.key[0x9] = state,
            Keycode::F => self.key[0xE] = state,
            Keycode::Z => self.key[0xA] = state,
            Keycode::X => self.key[0x0] = state,
            Keycode::C => self.key[0xB] = state,
            Keycode::V => self.key[0xF] = state,
            _ => (),
        }
    }

    pub fn get_key_state(&self, pos: usize) -> bool {
        self.key[pos]
    }
}
