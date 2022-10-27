pub const RAM_SIZE: usize = 4096; // 4 KB
pub const NUM_VREGS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const NUM_KEYS: usize = 16;

pub const SCREEN_WIDTH: usize = 64; 
pub const SCREEN_HEIGHT: usize = 32;

pub const START_ADDR: u16 = 0x200;

use crate::fontset::*;


/// We use type uN (where N is a 8 aligned number) because
/// it defines the amount of bits we need for every number.
pub struct Emulator {
    pc: u16, // Program Counter
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_VREGS],
    i_reg: u16, // Pointer used for indexing into RAM
    sp: u16, // Stack Pointer
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8, // Delay timer
    st: u8, // Sound timer
}

impl Emulator {

    pub fn new() -> Self {
        let mut new_emulator: Emulator = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_VREGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emulator
    }

    // Stack management functions

    fn push(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // CPU operation functions

    fn fetch(&mut self) -> u16 {
        let higher_byte: u16 = self.ram[self.pc as usize] as u16;
        let lower_byte: u16 = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        op
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_VREGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack=  [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
    }

}