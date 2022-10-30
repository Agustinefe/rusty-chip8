pub const RAM_SIZE: usize = 4096; // 4 KB
pub const NUM_VREGS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const NUM_KEYS: usize = 16;

pub const SCREEN_WIDTH: usize = 64; 
pub const SCREEN_HEIGHT: usize = 32;

pub const START_ADDR: u16 = 0x200;

pub const FLAG_REG: usize = 0xF;

use crate::fontset::*;
use rand::random;

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

    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT]
    }

    /// Pops the address from the stack, and set the pc with it
    fn return_from_subroutine(&mut self) {
        let ret_addr: u16 = self.pop();
        self.pc = ret_addr;
    }

    /// Sets the pc with the nnn address
    fn jump(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.pc = nnn;
    }

    /// Push the last routine address in the stack, and set the pc
    /// with the new subroutine address.
    fn call(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.push(self.pc);
        self.pc = nnn;
    }

    fn skip_next_if_reg_equals_val(&mut self, idx: usize, val: u8) {
        if self.v_reg[idx] == val {
            self.pc += 2;
        }
    }

    fn skip_next_if_reg_not_equals_val(&mut self, idx: usize, val: u8) {
        if self.v_reg[idx] != val {
            self.pc += 2;
        }
    }

    fn skip_next_if_reg_equals_reg(&mut self, idxA: usize, idxB: usize) {
        if self.v_reg[idxA] == self.v_reg[idxB] {
            self.pc += 2;
        }
    }

    fn assign_val_to_reg(&mut self, idx: usize, val: u8) {
        self.v_reg[idx] = val;
    }

    fn add_val_to_reg(&mut self, idx: usize, val: u8) {
        self.v_reg[idx] = self.v_reg[idx].wrapping_add(val);
    }

    fn assign_reg_to_reg(&mut self, idxA: usize, idxB: usize) {
        self.v_reg[idxA] = self.v_reg[idxB];
    }

    fn reg_or_reg(&mut self, idxA: usize, idxB: usize) {
        self.v_reg[idxA] |= self.v_reg[idxB];
    }

    fn reg_and_reg(&mut self, idxA: usize, idxB: usize) {
        self.v_reg[idxA] &= self.v_reg[idxB];
    }

    fn reg_xor_reg(&mut self, idxA: usize, idxB: usize) {
        self.v_reg[idxA] ^= self.v_reg[idxB];
    }

    fn add_reg_to_reg(&mut self, idxA: usize, idxB: usize) {
        let (sum, carry) = self.v_reg[idxA].overflowing_add(self.v_reg[idxB]);

        self.v_reg[idxA] = sum;
        self.v_reg[FLAG_REG] = if carry {1} else {0};
    }

    fn sub_reg_to_reg(&mut self, idxA: usize, idxB: usize) {
        let (difference, borrow) = self.v_reg[idxA].overflowing_sub(self.v_reg[idxB]);

        self.v_reg[idxA] = difference;
        self.v_reg[FLAG_REG] = if borrow {0} else {1};
    }

    fn single_right_shift(&mut self, idx: usize) {
        let dropped_bit = self.v_reg[idx] & 1;
        self.v_reg[idx] >>= 1;
        self.v_reg[FLAG_REG] = dropped_bit;
    }

    fn opposite_sub_reg_to_reg(&mut self, idxA: usize, idxB: usize) {
        let (difference, borrow) = self.v_reg[idxB].overflowing_sub(self.v_reg[idxA]);

        self.v_reg[idxA] = difference;
        self.v_reg[FLAG_REG] = if borrow {0} else {1};
    }

    fn single_left_shift(&mut self, idx: usize) {
        let dropped_bit = (self.v_reg[idx] >> 7) & 1;
        self.v_reg[idx] <<= 1;
        self.v_reg[FLAG_REG] = dropped_bit;
    }

    fn skip_next_if_reg_not_equals_reg(&mut self, idxA: usize, idxB: usize) {
        if self.v_reg[idxA] == self.v_reg[idxB] {
            self.pc += 2;
        }
    }

    fn assign_addr_to_ram_pointer(&mut self, op: u16) {
        let nnn = op & 0xFFF;
        self.i_reg = nnn;
    }

    fn assign_random_to_reg(&mut self, idx: usize, val: u8) {
        let rand: u8 = random();
        self.v_reg[idx] = rand & val;
    }

    fn draw_sprite(&mut self, x_coord: u16, y_coord: u16, num_rows: u16) {
        // Keep track if any pixels were flipped
        let mut flipped: bool = false;

        // Iterate over each row of the sprite
        for y_line in 0..num_rows {

            // Determine which memory address the row's data is stored
            let addr: u16 = self.i_reg + y_line;
            let pixels: u8 = self.ram[addr as usize];
            
            // Iterate over each column in our row
            for x_line in 0..8 {

                // Use a mask to fetch current pixel's bit.
                // Only flip if a 1.
                if (pixels & (0b1000_0000 >> x_line)) != 0 {

                    // Sprites should wrap around screen, so apply module.
                    let x: usize = (x_coord + x_line) as usize % SCREEN_WIDTH;
                    let y: usize = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                    // Get the pixel's index for the ID screen array
                    let idx: usize = x + SCREEN_WIDTH * y;

                    // Check if we're about to flip the pixel and set
                    flipped |= self.screen[idx];
                    self.screen[idx] = true;
                }
            }
        }

        // Populate VF register
        self.v_reg[FLAG_REG] = if flipped {1} else {0};

    }

    fn skip_if_key_pressed(&mut self, idx: usize) {
        let vx: u8 = self.v_reg[idx];
        let key: bool = self.keys[vx as usize];
        if key {
            self.pc += 2;
        }
    }

    fn skip_if_key_not_pressed(&mut self, idx: usize) {
        let vx: u8 = self.v_reg[idx];
        let key: bool = self.keys[vx as usize];
        if !key {
            self.pc += 2;
        }
    }

    fn assign_delay_timer_to_reg(&mut self, idx: usize) {
        self.v_reg[idx] = self.dt;
    }

    fn wait_for_key_press(&mut self, idx: usize) {
        let mut pressed: bool = false;

        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v_reg[idx] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            // Redo opcode
            self.pc -= 2;
        }
    }

    fn assign_reg_to_delay_timer(&mut self, idx: usize) {
        self.dt = self.v_reg[idx];
    }

    fn assign_reg_to_sound_timer(&mut self, idx: usize) {
        self.st = self.v_reg[idx];
    }

    fn increment_ram_pointer_by_reg(&mut self, idx: usize) {
        let vx: u16 = self.v_reg[idx] as u16;
        self.i_reg += self.i_reg.wrapping_add(vx);
    }

    fn set_ram_pointer_to_font_addr(&mut self, idx: usize) {
        let c: u16 = self.v_reg[idx] as u16;
        self.i_reg = c * 5;
    }

    /// BCD: Binary-Coded Decimal
    fn set_ram_pointer_to_bcd_of_reg(&mut self, idx: usize) {
        let vx: f32 = self.v_reg[idx] as f32;

        let hundreds: u8 = (vx / 100.0).floor() as u8;
        let tens: u8 = ((vx / 10.0) % 10.0).floor() as u8;
        let ones: u8 = (vx % 1.0).floor() as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[(self.i_reg + 1) as usize] = tens;
        self.ram[(self.i_reg + 2) as usize] = ones;
    }

    fn store_regs_in_ram(&mut self, idx: usize) {
        let i: usize = self.i_reg as usize;

        for x in 0..=idx {
            self.ram[i + x] = self.v_reg[x];
        }
    }

    fn load_regs_from_ram(&mut self, idx: usize) {
        let i: usize = self.i_reg as usize;

        for x in 0..=idx {
            self.v_reg[x] = self.ram[i + x];
        }
    }

    /// NOP: do nothing
    /// CLS: clear screen
    /// RET: return from subroutine
    /// JMP: jump to the given address
    /// CALL: call a new subroutine
    fn execute(&mut self, op: u16) {
        let hex_digit1: u16 = (op & 0xF000) >> 12;
        let hex_digit2: u16 = (op & 0xF000) >> 8;
        let hex_digit3: u16 = (op & 0xF000) >> 4;
        let hex_digit4: u16 = op & 0xF000;

        match (hex_digit1, hex_digit2, hex_digit3, hex_digit4) {
            // NOP
            (0, 0, 0, 0) => return,

            // CLS
            (0, 0, 0xE, 0) => self.clear_screen(),

            // RET
            (0, 0, 0xE, 0xE) => self.return_from_subroutine(),

            // JMP NNN
            (1, _, _, _) => self.jump(op),

            // CALL NNN
            (2, _, _, _) => self.call(op),

            // SKIP NEXT IF REG == VAL
            (3, _, _, _) => self.skip_next_if_reg_equals_val(hex_digit2 as usize, (op & 0xFF) as u8),

            // SKIP NEXT IF REG != VAL
            (4, _, _, _) => self.skip_next_if_reg_not_equals_val(hex_digit2 as usize, (op & 0xFF) as u8),

            // SKIP NEXT IF REG == REG
            (5, _, _, 0) => self.skip_next_if_reg_equals_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX = NN
            (6, _,_, _) => self.assign_val_to_reg(hex_digit2 as usize, (op & 0xFF) as u8),

            // VX += NN
            (7, _, _, _) => self.add_val_to_reg(hex_digit2 as usize, (op & 0xFF) as u8),

            // VX = VY
            (8, _, _, 0) => self.assign_reg_to_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX |= VY
            (8, _, _, 1) => self.reg_or_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX &= VY
            (8, _, _, 2) => self.reg_and_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX ^= VY
            (8, _, _, 3) => self.reg_xor_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX += VY
            (8, _, _, 4) => self.add_reg_to_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX -= VY
            (8, _, _, 5) => self.sub_reg_to_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX >>= 1
            (8, _, _, 6) => self.single_right_shift(hex_digit2 as usize),

            // VX = VY - VX
            (8, _, _, 7) => self.opposite_sub_reg_to_reg(hex_digit2 as usize, hex_digit3 as usize),

            // VX <<= 1
            (8, _, _, 0xE) => self.single_left_shift(hex_digit2 as usize),

            // SKIP NEXT IF REG != REG
            (9, _, _, 0) => self.skip_next_if_reg_not_equals_reg(hex_digit2 as usize, hex_digit3 as usize),

            // I = NNN
            (0xA, _, _, _) => self.assign_addr_to_ram_pointer(op),

            // VX = rand & NN
            (0xC, _, _, _) => self.assign_random_to_reg(hex_digit2 as usize, (op & 0xFF) as u8),

            // DRAW
            (0xD, _, _, _) => self.draw_sprite(hex_digit2, hex_digit3, hex_digit4),

            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => self.skip_if_key_pressed(hex_digit2 as usize),

            // SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => self.skip_if_key_not_pressed(hex_digit2 as usize),

            // VX = DT
            (0xF, _, 0, 7) => self.assign_delay_timer_to_reg(hex_digit2 as usize),

            // WAIT KEY
            (0xF, _, 0, 0xA) => self.wait_for_key_press(hex_digit2 as usize),

            // DT = VX
            (0xF, _, 1, 5) => self.assign_reg_to_delay_timer(hex_digit2 as usize),

            // ST = VX
            (0xF, _, 1, 8) => self.assign_reg_to_sound_timer(hex_digit2 as usize),

            // I += VX
            (0xF, _, 1, 0xE) => self.increment_ram_pointer_by_reg(hex_digit2 as usize),

            // I = FONT
            (OxF, _, 2, 9) => self.set_ram_pointer_to_font_addr(hex_digit2 as usize),

            // I = BCD of VX
            (0xF, _, 3, 3) => self.set_ram_pointer_to_bcd_of_reg(hex_digit2 as usize),

            // STORE V0..VX INTO RAM
            (0xF, _, 5, 5) => self.store_regs_in_ram(hex_digit2 as usize),

            // LOAD V0..VX FROM RAM
            (0xF, _, 6, 5) =>self.load_regs_from_ram(hex_digit2 as usize),

            (_, _, _ , _) => unimplemented!("Unimplemented opcode: {}", op),
        }

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
        // Decode and execute
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            // BEEP
        }
        self.st -= 1;
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

}