use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [ 
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Emulator {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    delay_t: u8,
    sound_t: u8,
}





impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_t: 0,
            sound_t: 0,
        };

        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        return new_emulator;
    }

    pub fn get_display(&self) -> &[bool] {
        return &self.screen;
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_t = 0;
        self.sound_t = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET)
    }

    // 1. Fetch
    // 2. Decode
    // 3. Execute
    // 4. Next instruction, back to 1.
    pub fn tick(&mut self) {
        let op = self.fetch();
        self.decode_and_execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_t > 0 {
            self.delay_t -= 1;
        }

        if self.sound_t > 0 {
            if self.sound_t == 1 {
                // Sound emitted
            }
            self.sound_t -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        // Opcodes are 2 bytes
        // But RAM is a byte wide
        // So fetch 2 bytes and concat them in Big Endian u16
        let upper_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (upper_byte << 8) | lower_byte;
        self.pc += 2;
        return op
    }

    fn decode_and_execute(&mut self, op: u16) {
        let hex_1 = (op & 0xF000) >> 12;
        let hex_2 = (op & 0x0F00) >> 8;
        let hex_3 = (op & 0x00F0) >> 4;
        let hex_4 = op & 0x000F;

        match (hex_1, hex_2, hex_3, hex_4) {
            // NOP
            (0, 0, 0, 0) => return,

            // CLS
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },

            // RET
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },

            // JMP NNN
            (1, _, _, _) => {
                let nnn = op & 0xFFF; // Does this drop the last byte?
                self.pc = nnn;
            },

            // CALL NNN
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },

            // SKIP VX == NN
            (3, _, _, _) => {
                let x = hex_2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2; //each opcode is 2 bytes
                }
            },

            // SKIP VX != NN
            (4, _, _, _) => {
                let x = hex_2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },

            // SKIP VX == VY
            (5, _, _, 0) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },

            // VX := NN
            (6, _, _, _) => {
                let x = hex_2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            
            // VX += NN
            (7, _, _, _) => {
                let x = hex_2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn); // What does wrapping add do?
            },

            // VX := VY
            (8, _, _, 0) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },

            // VX |= VY
            (8, _, _, 1) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },

            // VX &= VY
            (8, _, _, 2) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;
                self.v_reg[x] &= self.v_reg[y]; 
            },

            // VX ^= VY
            (8, _, _, 3) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }

            // VX += VY
            (8, _, _, 4) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // VX -= VY
            (8, _, _, 5) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // VX >>= 1
            (8, _, _, 6) => {
                let x = hex_2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },

            // VX := VY - VX
            (8, _, _, 7) => {
                let x = hex_2 as usize;
                let y = hex_2 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },

            // VX <<= 1
            (8, _, _, 0xE) => {
                let x = hex_2 as usize;
                let msb = self.v_reg[x] >> 7;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },

            // SKIP VX != VY
            (9, _, _, 0) => {
                let x = hex_2 as usize;
                let y = hex_3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },

            // I := NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },

            // JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },

            // VX := rand() & NN
            (0xC, _, _, _) => {
                let x = hex_2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },

            // DRAW
            (0xD, _, _, _) => {
                // Get the (x, y) coords for our sprite
                let x_coord = self.v_reg[hex_2 as usize] as u16;
                let y_coord = self.v_reg[hex_3 as usize] as u16;

                // Last digit gets sprite height 
                let num_rows = hex_4;

                let mut flipped = false; 

                for y_line in 0..num_rows {
                    // Figure out where the row data is stored 
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // Iterate over each column in our row 
                    for x_line in 0..8 {
                        // Pixel mask
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites wrap around screen 
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get the pixel index
                            let idx = x + SCREEN_WIDTH * y;
                            
                            // Check if we're about to flip, and set 
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },

            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => {
                let x = hex_2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },

            // SKIP IF KEY NOT PRESSED
            (0xE, _, 0xA, 1) => {
                let x = hex_2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },

            // VX = DT
            (0xF, _, 0, 7) => {
                let x = hex_2 as usize;
                self.v_reg[x] = self.delay_t;
            },

            // WAIT KEY
            (0xF, _, 0, 0xA) => {
                let x = hex_2 as usize;
                let mut pressed = false; 
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true; 
                        break;
                    }
                }

                // This OP is blocking
                if !pressed {
                    // redo opcode
                    self.pc -= 2;
                }
            },

            // DT = VX
            (0xF, _, 1, 5) => {
                let x = hex_2 as usize;
                self.delay_t = self.v_reg[x];
            },

            // ST = VX
            (0xF, _, 1, 8) => {
                let x = hex_2 as usize;
                self.sound_t = self.v_reg[x];
            },

            // I += VX
            (0xF, _, 1, 0xE) => {
                let x = hex_2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },

            // Set I = FONT
            (0xF, _, 2, 9) => {
                let x = hex_2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },

            // BCD 
            (0xF, _, 3, 3) => {
                let x = hex_2 as usize;
                let vx = self.v_reg[x] as f32;

                // Fetch hundreds digit 
                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0)).floor() as u8; 
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds; 
                self.ram[(self.i_reg + 1) as usize] = tens; 
                self.ram[(self.i_reg + 2) as usize] = ones;
            },

            // STORE VO - VX
            (0xF, _, 5, 5) => {
                let x = hex_2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i+idx] = self.v_reg[idx]
                }
            },

            // LOAD VO - VX
            (0xF, _, 6, 5) => {
                let x = hex_2 as usize; 
                let i = self.i_reg as usize; 
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }





            


            // Exhaustive pattern matching in Rust
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn push(&mut self, val: u16) {
        // Why 'as usize'?
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        return self.stack[self.sp as usize];
    }
}