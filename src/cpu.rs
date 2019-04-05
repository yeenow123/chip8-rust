extern crate rand;

use crate::input::InputDevice;

use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

const MEMORY_SIZE: usize = 4096;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

enum ProgramCounterStep {
    Jump(usize),
    Next,
    Skip,
    Noop,
}

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    registers: [u8; 16],
    i_register: usize,
    pc: usize,

    pub gfx: [[u8; 64]; 32],
    pub draw_flag: bool,

    delay_timer: u8,
    sound_timer: u8,
    pub beep: bool,

    stack: [usize; 16],
    s_ptr: usize,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut init_memory = [0; MEMORY_SIZE];
        for i in 0..80 {
            init_memory[i] = FONTSET[i];
        }

        Chip8 {
            memory: init_memory,
            registers: [0; 16],
            i_register: 0,
            pc: 512,
            gfx: [[0; 64]; 32],
            draw_flag: false,
            delay_timer: 0,
            sound_timer: 0,
            beep: false,
            stack: [0; 16],
            s_ptr: 0,
        }
    }

    pub fn initialize(&mut self, buffer: Vec<u8>) {
        for (i, val) in buffer.iter().enumerate() {
            self.memory[i + 512] = *val;
        }
    }

    pub fn emulate_cycle(&mut self, input: Rc<RefCell<InputDevice>>) {
        // Reset flags
        self.beep = false;
        self.draw_flag = false;

        let opcode = ((self.memory[self.pc] as u16) << 8) | (self.memory[self.pc + 1] as u16);
        let pc_step = self.handle_opcode(opcode, input);
        self.pc = handle_programcounter(self.pc, pc_step);

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.beep = true;
        }
    }

    fn handle_opcode(
        &mut self,
        opcode: u16,
        input: Rc<RefCell<InputDevice>>,
    ) -> ProgramCounterStep {
        let first = (opcode & 0xF000) >> 12;
        let second = (opcode & 0x0F00) >> 8;
        let third = (opcode & 0x00F0) >> 4;
        let fourth = opcode & 0x000F;

        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = second as usize;
        let y = third as usize;

        match (first, second, third, fourth) {
            (0x0, 0x0, 0xE, 0x0) => {
                for row in self.gfx.iter_mut() {
                    for cell in row.iter_mut() {
                        *cell = 0;
                    }
                }
                self.draw_flag = true;
                ProgramCounterStep::Next
            }
            (0x0, 0x0, 0xE, 0xE) => {
                self.s_ptr -= 1;
                let pos = self.stack[self.s_ptr];
                ProgramCounterStep::Jump(pos)
            }
            (0x1, _, _, _) => ProgramCounterStep::Jump(nnn),
            (0x2, _, _, _) => {
                self.stack[self.s_ptr] = self.pc + 2; // Put the next address there
                self.s_ptr += 1;
                ProgramCounterStep::Jump(nnn)
            }
            (0x3, _, _, _) => {
                if self.registers[x] == kk {
                    ProgramCounterStep::Skip
                } else {
                    ProgramCounterStep::Next
                }
            }
            (0x4, _, _, _) => {
                if self.registers[x] != kk {
                    ProgramCounterStep::Skip
                } else {
                    ProgramCounterStep::Next
                }
            }
            (0x5, _, _, 0x0) => {
                if self.registers[x] == self.registers[y] {
                    ProgramCounterStep::Skip
                } else {
                    ProgramCounterStep::Next
                }
            }
            (0x6, _, _, _) => {
                self.registers[x] = kk;
                ProgramCounterStep::Next
            }
            (0x7, _, _, _) => {
                let sum = self.registers[x] as u16 + kk as u16;
                self.registers[x] = sum as u8;
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x0) => {
                self.registers[x] = self.registers[y];
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x1) => {
                self.registers[x] |= self.registers[y];
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x2) => {
                self.registers[x] &= self.registers[y];
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x3) => {
                self.registers[x] ^= self.registers[y];
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x4) => {
                let res = self.registers[x].checked_add(self.registers[y]);
                if let None = res {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                let add = self.registers[x] as u16 + self.registers[y] as u16;
                self.registers[x] = add as u8;
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x5) => {
                if self.registers[x] > self.registers[y] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[x].wrapping_sub(self.registers[y]);
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x6) => {
                if self.registers[x] & 1 == 1 {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[x] >>= 1;
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0x7) => {
                if self.registers[y] > self.registers[x] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
                ProgramCounterStep::Next
            }
            (0x8, _, _, 0xE) => {
                if ((self.registers[x] & 0b10000000) >> 7) == 1 {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[x] <<= 1;
                ProgramCounterStep::Next
            }
            (0x9, _, _, 0x0) => {
                if self.registers[x] != self.registers[y] {
                    ProgramCounterStep::Skip
                } else {
                    ProgramCounterStep::Next
                }
            }
            (0xA, _, _, _) => {
                self.i_register = nnn;
                ProgramCounterStep::Next
            }
            (0xB, _, _, _) => ProgramCounterStep::Jump(nnn + self.registers[0] as usize),
            (0xC, _, _, _) => {
                self.registers[x as usize] = rand::thread_rng().gen::<u8>() & kk;
                ProgramCounterStep::Next
            }
            (0xD, _, _, height) => {
                self.registers[0xF] = 0;
                for i in 0..height as usize {
                    let y = (self.registers[y] as usize + i) % 32;
                    for j in 0..8 {
                        let x = (self.registers[x] as usize + j) % 64;
                        let curr_pixel = (self.memory[self.i_register + i] >> (7 - j)) & 1;
                        self.registers[0xF] |= curr_pixel & self.gfx[y][x];
                        self.gfx[y][x] ^= curr_pixel;
                    }
                }
                self.draw_flag = true;
                ProgramCounterStep::Next
            }
            (0xE, _, 0x9, 0xE) => {
                if input.borrow().get_key_state(self.registers[x] as usize) {
                    ProgramCounterStep::Skip
                } else {
                    ProgramCounterStep::Next
                }
            }
            (0xE, _, 0xA, 0x1) => {
                if !input.borrow().get_key_state(self.registers[x] as usize) {
                    ProgramCounterStep::Skip
                } else {
                    ProgramCounterStep::Next
                }
            }
            (0xF, _, 0x0, 0x7) => {
                self.registers[x] = self.delay_timer;
                ProgramCounterStep::Next
            }
            (0xF, _, 0x0, 0xA) => {
                if input.borrow().key_waiting {
                    for i in 0..16 {
                        if input.borrow().get_key_state(i) != input.borrow().key_copy[i] {
                            self.registers[x] = i as u8;
                            input.borrow_mut().key_waiting = false;
                            // Let key_copy be invalid
                            break;
                        }
                    }

                    if input.borrow().key_waiting {
                        ProgramCounterStep::Noop
                    } else {
                        ProgramCounterStep::Next
                    }
                } else {
                    let key_state_copy = input.borrow().key.clone();
                    input.borrow_mut().key_waiting = true;
                    input.borrow_mut().key_copy = key_state_copy;
                    ProgramCounterStep::Noop
                }
            }
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.registers[x] as u8;
                ProgramCounterStep::Next
            }
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.registers[x] as u8;
                ProgramCounterStep::Next
            }
            (0xF, _, 0x1, 0xE) => {
                self.i_register += self.registers[x] as usize;
                ProgramCounterStep::Next
            }
            (0xF, _, 0x2, 0x9) => {
                self.i_register = (self.registers[x] as usize) * 5;
                ProgramCounterStep::Next
            }
            (0xF, _, 0x3, 0x3) => {
                let val = self.registers[x];
                self.memory[self.i_register] = val / 100;
                self.memory[self.i_register + 1] = (val / 10) % 10;
                self.memory[self.i_register + 2] = (val % 100) % 10;
                ProgramCounterStep::Next
            }
            (0xF, _, 0x5, 0x5) => {
                for i in 0..(x + 1) {
                    self.memory[self.i_register + i] = self.registers[i];
                }
                ProgramCounterStep::Next
            }
            (0xF, _, 0x6, 0x5) => {
                for i in 0..(x + 1) {
                    self.registers[i] = self.memory[self.i_register + i];
                }
                ProgramCounterStep::Next
            }
            _ => {
                println!("{:X} {:X} {:X} {:X}", first, second, third, fourth);
                ProgramCounterStep::Next
            }
        }
    }
}

fn handle_programcounter(curr_pc: usize, step: ProgramCounterStep) -> usize {
    match step {
        ProgramCounterStep::Next => curr_pc + 2,
        ProgramCounterStep::Skip => curr_pc + 4,
        ProgramCounterStep::Noop => curr_pc,
        ProgramCounterStep::Jump(pos) => pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn programcounter_works() {
        assert!(handle_programcounter(0, ProgramCounterStep::Next) == 2);
        assert!(handle_programcounter(0, ProgramCounterStep::Skip) == 4);
        assert!(handle_programcounter(0, ProgramCounterStep::Jump(16)) == 16);
        assert!(handle_programcounter(0, ProgramCounterStep::Noop) == 0);
    }

}
