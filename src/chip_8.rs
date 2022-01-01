/// An instance of the Chip-8 emulator.
struct Chip8 {
    /// The Chip-8's RAM.
    memory: [u8; 0x1000],

    /// The Chip-8's V registers (0 through F).
    v_registers: [u8; 0xf],

    /// The Chip-8's I register.
    i_register: u16,

    /// The program counter, index of the currently executed instruction.
    pc: u16,

    /// The Chip-8's screen, 64 * 32 pixels stored in binary. (1 bit = 1 pixel)
    screen: [u8; 8 * 32],

    /// The Chip-8's stack, used for subroutines.
    stack: [u16; 16],

    /// The stack pointer, index of the last element on the stack.
    stack_pointer: u8,

    /// The sound timer, a sound is played as long as it's non-zero.
    sound_timer: u8,

    /// The delay timer, always decremented when it's greater than zero.
    delay_timer: u8,

    /// Whether the Chip-8 is currently waiting for a key press.
    waiting_for_key: bool,
}