pub const RAM_SIZE: usize = 4096; // 4 KB
pub const NUM_VREGS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const NUM_KEYS: usize = 16;

pub const SCREEN_WIDTH: usize = 64; 
pub const SCREEN_HEIGHT: usize = 32;

pub const START_ADDR: u16 = 0x200;


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
        Self {
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
        }
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

}