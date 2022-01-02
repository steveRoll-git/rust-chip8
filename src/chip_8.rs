use rand::random;

pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;
const SCREEN_MEM_SIZE: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;

const FONT_ADDRESS: u16 = 0x50;
const FONT_DATA: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10,
    0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
    0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90,
    0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0,
    0x80, 0xF0, 0x80, 0x80];

const ROM_START_ADDRESS: usize = 0x200;

/// An instance of the Chip-8 emulator.
pub struct Chip8 {
    /// The Chip-8's RAM.
    memory: [u8; 4096],

    /// The Chip-8's V registers (0 through F).
    v_registers: [u8; 16],

    /// The Chip-8's I register.
    i_register: u16,

    /// The program counter, index of the currently executed instruction.
    pc: usize,

    /// The Chip-8's screen, 64 * 32 pixels
    pub screen: [u8; SCREEN_MEM_SIZE],

    /// The Chip-8's stack, used for subroutines.
    stack: [u16; 16],

    /// The stack pointer, index of the last element on the stack.
    stack_pointer: usize,

    /// The sound timer, a sound is played as long as it's non-zero.
    sound_timer: u8,

    /// The delay timer, always decremented when it's greater than zero.
    delay_timer: u8,

    /// `Some` if the Chip-8 is currently waiting for a key press, with the value being which V
    /// register to store the pressed key into.
    waiting_for_key: Option<u8>,

    /// Whether to use the alternative version of the 8xy6 and 8xyE instructions. Some roms may
    /// expect those instructions to behave differently.
    alternative_shift_mode: bool,

    /// How many CPU cycles to perform each frame.
    pub cycles_per_frame: u32,
}

/// Array that describes which keys of the Chip-8 are currently pressed.
pub type KeyboardState = [bool; 16];

impl Chip8 {
    /// Performs a single instruction cycle.
    fn cycle(&mut self, keyboard_state: &KeyboardState) {
        if let Some(reg) = self.waiting_for_key {
            let key = keyboard_state.iter().position(|k| *k);
            if let Some(k) = key {
                self.v_registers[reg as usize] = k as u8;
                self.waiting_for_key = None;
            }
        } else {
            let instr_low = self.memory[self.pc + 1];
            let instr_high = self.memory[self.pc];
            let instruction = instr_low as u16 + ((instr_high as u16) << 8);
            let high_nibble = instr_high >> 4;
            let low_nibble = instr_low & 0xf;
            let x = (instruction >> 8 & 0xf) as usize;
            let y = (instruction >> 4 & 0xf) as usize;

            // by how much to increment pc after the instruction is done
            let mut increment_pc = 2;

            match high_nibble {
                0 if instruction == 0x00E0 => {
                    // 00E0: clear screen
                    self.screen.fill(0);
                }
                0 if instruction == 0x00EE => {
                    // 00EE: return from subroutine
                    self.stack_pointer -= 1;
                    self.pc = self.stack[self.stack_pointer] as usize;
                }
                1 => {
                    // 1nnn: jump to address nnn
                    self.pc = instruction as usize & 0xfff;

                    increment_pc = 0;
                }
                2 => {
                    // 2nnn: call subroutine at nnn
                    self.stack[self.stack_pointer] = self.pc as u16;
                    self.stack_pointer += 1;
                    self.pc = instruction as usize & 0xfff;

                    increment_pc = 0;
                }
                3 => {
                    // 3xkk: skip next instruction if Vx == kk
                    if self.v_registers[x] == instr_low {
                        self.pc += 2;
                    }
                }
                4 => {
                    // 4xkk: skip next instruction if Vx != kk
                    if self.v_registers[x] != instr_low {
                        self.pc += 2;
                    }
                }
                5 if low_nibble == 0 => {
                    // 5xy0: skip next instruction if Vx == Vy
                    if self.v_registers[x] == self.v_registers[y] {
                        self.pc += 2;
                    }
                }
                6 => {
                    // 6xkk: set Vx = kk
                    self.v_registers[x] = instr_low;
                }
                7 => {
                    // 7xkk: set Vx = Vx + kk
                    self.v_registers[x] = self.v_registers[x].wrapping_add(instr_low);
                }
                8 if low_nibble == 0 => {
                    // 8xy0: set Vx = Vy
                    self.v_registers[x] = self.v_registers[y];
                }
                8 if low_nibble == 1 => {
                    // 8xy1: set Vx = Vx | Vy
                    self.v_registers[x] = self.v_registers[x] | self.v_registers[y];
                }
                8 if low_nibble == 2 => {
                    // 8xy2: set Vx = Vx & Vy
                    self.v_registers[x] = self.v_registers[x] & self.v_registers[y];
                }
                8 if low_nibble == 3 => {
                    // 8xy3: set Vx = Vx ^ Vy
                    self.v_registers[x] = self.v_registers[x] ^ self.v_registers[y];
                }
                8 if low_nibble == 4 => {
                    // 8xy4: set Vx = Vx + Vy, VF = carry
                    let result = self.v_registers[x] as u16 + self.v_registers[y] as u16;
                    self.v_registers[0xf] = (result >> 8 & 1) as u8;
                    self.v_registers[x] = result as u8;
                }
                8 if low_nibble == 5 => {
                    // 8xy5: set Vx = Vx - Vy, VF = not borrow
                    self.v_registers[0xf] = if self.v_registers[x] > self.v_registers[y] { 0 } else { 1 };
                    self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
                }
                8 if low_nibble == 6 => {
                    // 8xy6: set Vx = Vy >> 1, VF = LSB(Vx)
                    let other = if self.alternative_shift_mode { x } else { y };
                    self.v_registers[0xf] = self.v_registers[other] & 1;
                    self.v_registers[x] = self.v_registers[other] >> 1;
                }
                8 if low_nibble == 7 => {
                    // 8xy7: set Vx = Vy - Vx, VF = not borrow
                    self.v_registers[0xf] = u8::from(self.v_registers[y] > self.v_registers[x]);
                    self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);
                }
                8 if low_nibble == 0xE => {
                    // 8xyE: set Vx = Vy << 1, VF = MSB(Vx)
                    let other = if self.alternative_shift_mode { x } else { y };
                    self.v_registers[0xf] = self.v_registers[other] & 0x80;
                    self.v_registers[x] = self.v_registers[other] << 1;
                }
                9 if low_nibble == 0 => {
                    // 9xy0: skip next instruction if Vx != Vy
                    if self.v_registers[x] != self.v_registers[y] {
                        self.pc += 2;
                    }
                }
                0xA => {
                    // Annn: set I = nnn
                    self.i_register = instruction & 0xfff;
                }
                0xB => {
                    // Bnnn: jump to V0 + nnn
                    self.pc = (instruction as usize & 0xfff) + self.v_registers[0] as usize;

                    increment_pc = 0;
                }
                0xC => {
                    // Cxkk: set Vx = random & kk
                    self.v_registers[x] = random::<u8>() & instr_low;
                }
                0xD => {
                    // Dxyn: draw n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                    self.v_registers[0xF] = 0;
                    for iy in 0..low_nibble {
                        let row = self.memory[self.i_register as usize + iy as usize];
                        for ix in 0..8 {
                            let dx = (self.v_registers[x] + ix) % SCREEN_WIDTH;
                            let dy = (self.v_registers[y] + iy) % SCREEN_HEIGHT;
                            let index = dy as usize * SCREEN_WIDTH as usize + dx as usize;
                            let prev_pixel = self.screen[index];
                            let pixel = (row >> (7 - ix) & 1) * 255;
                            if prev_pixel == 255 && pixel == 255 {
                                self.v_registers[0xF] = 1;
                            }
                            self.screen[index] = prev_pixel ^ pixel;
                        }
                    }
                }
                0xE if instr_low == 0x9E => {
                    // Ex9E: skip next instruction if key Vx is pressed
                    if keyboard_state[self.v_registers[x] as usize] {
                        self.pc += 2;
                    }
                }
                0xE if instr_low == 0xA1 => {
                    // Ex9E: skip next instruction if key Vx is not pressed
                    if !keyboard_state[self.v_registers[x] as usize] {
                        self.pc += 2;
                    }
                }
                0xF if instr_low == 0x07 => {
                    // Fx07: set Vx = delay timer
                    self.v_registers[x] = self.delay_timer;
                }
                0xF if instr_low == 0x0A => {
                    // Fx0A: wait for key press, store pressed key in Vx
                    self.waiting_for_key = Some(x as u8);
                }
                0xF if instr_low == 0x15 => {
                    // Fx15: set delay timer = Vx
                    self.delay_timer = self.v_registers[x];
                }
                0xF if instr_low == 0x18 => {
                    // Fx18: set sound timer = Vx
                    self.sound_timer = self.v_registers[x];
                }
                0xF if instr_low == 0x1E => {
                    // Fx1E: set I = I + Vx
                    self.i_register += self.v_registers[x] as u16;
                }
                0xF if instr_low == 0x29 => {
                    // Fx29: set I = location of sprite for digit Vx
                    self.i_register = FONT_ADDRESS + self.v_registers[x] as u16 * 5;
                }
                0xF if instr_low == 0x33 => {
                    // Fx33: store BCD representation of Vx in memory locations I, I+1, I+2
                    self.memory[self.i_register as usize] = self.v_registers[x] / 100;
                    self.memory[self.i_register as usize + 1] = (self.v_registers[x] / 10) % 10;
                    self.memory[self.i_register as usize + 2] = self.v_registers[x] % 10;
                }
                0xF if instr_low == 0x55 => {
                    // Fx55: store registers V0 through Vx in memory starting at location I
                    for i in 0..=x {
                        self.memory[self.i_register as usize + i] = self.v_registers[i];
                    }
                }
                0xF if instr_low == 0x65 => {
                    // Fx65: read registers V0 through Vx in memory starting at location I
                    for i in 0..=x {
                        self.v_registers[i] = self.memory[self.i_register as usize + i];
                    }
                }

                _ => {
                    //TODO error about unknown instruction
                }
            }

            self.pc += increment_pc;
        }
    }

    pub fn frame(&mut self, keyboard_state: &KeyboardState) {
        for _ in 0..self.cycles_per_frame {
            self.cycle(keyboard_state);
        }

        self.delay_timer = if self.delay_timer > 0 { self.delay_timer - 1 } else { 0 };
        self.sound_timer = if self.sound_timer > 0 { self.sound_timer - 1 } else { 0 };
    }
    
    pub fn new(rom_data: &[u8]) -> Chip8 {
        let mut new = Chip8 {
            memory: [0; 4096],
            v_registers: [0; 16],
            i_register: 0,
            pc: ROM_START_ADDRESS,
            screen: [0; SCREEN_MEM_SIZE],
            stack: [0; 16],
            stack_pointer: 0,
            sound_timer: 0,
            delay_timer: 0,
            waiting_for_key: None,
            alternative_shift_mode: false,
            cycles_per_frame: 8
        };

        for (i, b) in FONT_DATA.iter().enumerate() {
            new.memory[i + FONT_ADDRESS as usize] = *b;
        }

        for (i, b) in rom_data.iter().enumerate() {
            new.memory[i + ROM_START_ADDRESS] = *b;
        }

        new
    }
}