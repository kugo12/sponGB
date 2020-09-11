use crate::emulator::{Memory, execute, PUSH};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct reg {
    a: u8,
    b: u8
}

#[repr(C)]
union Register {
    inner: reg,
    ab: u16
}

pub enum Flag {
    Z = 128,  // zero flag
    N = 64,  // subtract flag
    H = 32,  // half carry flag
    C = 16   // carry flag
}

pub struct CPU {
    // registers
    reg_af: Register,
    reg_bc: Register,
    reg_de: Register,
    reg_hl: Register,

    pub SP: u16,    // stack pointer
    pub PC: u16,    // program counter
    pub IME: bool,  // interrupt master enable
    pub EI: bool,   // pending enable interrupt

    pub memory: Memory,
    pub halt: bool,

    subins: u8  // subinstruction memory access counter
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            reg_af: Register { ab: 0x01B0 },
            reg_bc: Register { ab: 0x0013 },
            reg_de: Register { ab: 0x00D8 },
            reg_hl: Register { ab: 0x014D },

            SP: 0xFFFE,
            PC: 0x0100,
            IME: true,
            EI: false,

            memory: Memory::new(),
            halt: false,

            subins: 0
        }
    }

    pub fn A(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_af.inner.b}
    }

    pub fn F(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_af.inner.a}
    }

    pub fn AF(&mut self) -> &mut u16 {
        unsafe {&mut self.reg_af.ab}
    }

    pub fn B(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_bc.inner.b}
    }

    pub fn C(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_bc.inner.a}
    }

    pub fn BC(&mut self) -> &mut u16 {
        unsafe {&mut self.reg_bc.ab}
    }

    pub fn D(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_de.inner.b}
    }

    pub fn E(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_de.inner.a}
    }

    pub fn DE(&mut self) -> &mut u16 {
        unsafe {&mut self.reg_de.ab}
    }

    pub fn H(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_hl.inner.b}
    }

    pub fn L(&mut self) -> &mut u8 {
        unsafe {&mut self.reg_hl.inner.a}
    }

    pub fn HL(&mut self) -> &mut u16 {
        unsafe {&mut self.reg_hl.ab}
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        let f = unsafe {&self.reg_af.inner.a};
        (match flag {
            Flag::Z => (f & 128) >> 7,
            Flag::N => (f & 64) >> 6,
            Flag::H => (f & 32) >> 5,
            Flag::C => (f & 16) >> 4
        }) != 0
    }

    pub fn set_flag(&mut self, flag: Flag, v: bool){
        match v {
            true => *self.F() |= flag as u8,
            false => *self.F() &= !(flag as u8)
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let a = self.memory.read(addr);

        self.subins += 1;
        for _ in 0..4 {
            self.memory.tick();
        }
        a
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.memory.write(addr, val);

        self.subins += 1;
        for _ in 0..4 {
            self.memory.tick();
        }
    }

    pub fn load_u8(&mut self) -> u8 {
        let v = self.read(self.PC);
        self.PC += 1;
        v
    }

    pub fn load_u16(&mut self) -> u16 {
        let v = ((self.read(self.PC+1) as u16) << 8) | self.read(self.PC) as u16;
        self.PC += 2;
        v
    }

    pub fn get_val_reg(&mut self, instr: u8) -> (u8, bool) {
        let instr = if instr & 0x0f > 7 {
            (instr & 0x0f) - 8
        } else {
            instr & 0x0f
        };

        let val = match instr {
            0 => *self.B(),
            1 => *self.C(),
            2 => *self.D(),
            3 => *self.E(),
            4 => *self.H(),
            5 => *self.L(),
            6 => {
                let addr = *self.HL();
                self.read(addr)
            },
            7 => *self.A(),
            _ => panic!()
        };

        (val, instr == 6)
    }

    pub fn set_reg(&mut self, instr: u8, val: u8) {
        let instr = if instr & 0x0f > 7 {
            (instr & 0x0f) - 8
        } else {
            instr & 0x0f
        };

        match instr {
            0 => *self.B() = val,
            1 => *self.C() = val,
            2 => *self.D() = val,
            3 => *self.E() = val,
            4 => *self.H() = val,
            5 => *self.L() = val,
            6 => {
                let addr = *self.HL();
                self.write(addr, val);
            },
            7 => *self.A() = val,
            _ => panic!()
        };
    }

    fn handle_interrupts(&mut self) -> bool {
        let interrupts = self.memory.IF & self.memory.IER;
        if interrupts & 0b00011111 != 0 {
            if self.halt && !self.IME { self.halt = false; return false; }
            PUSH(self, self.PC);
            if interrupts & 0b00000001 != 0 {  // V-Blank
                self.memory.IF &= 0b11111110;
                self.PC = 0x0040;
                return true;
            } else if interrupts & 0b00000010 != 0 {  // LCD STAT
                self.memory.IF &= 0b11111101;
                self.PC = 0x0048;
                return true;
            } else if interrupts & 0b00000100 != 0 {  // Timer
                self.memory.IF &= 0b11111011;
                self.PC = 0x0050;
                return true;
            } else if interrupts & 0b00001000 != 0 {  // Serial
                self.memory.IF &= 0b11110111;
                self.PC = 0x0058;
                return true;
            } else if interrupts & 0b00010000 != 0 {  // Joypad
                self.memory.IF &= 0b11101111;
                self.PC = 0x0060;
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self) -> u8 {
        if self.IME || self.halt {
            if self.handle_interrupts() {
                self.IME = false;
                self.halt = false;
                return 5;
            }
        }

        if self.EI {
            self.IME = true;
            self.EI = false;
        }

        if !self.halt {
            let inst = self.load_u8();
            execute(self, inst)
        } else { 1 }
    }

    pub fn run(&mut self) {
        let mut cycles_left = 0;

        if self.memory.cart.bootrom_enable {
            self.PC = 0;
        }

        loop {
            if cycles_left > 0 {
                cycles_left -= 1;
            } else {
                cycles_left = (self.tick() - self.subins)*4;
                self.subins = 0;
                
                if cycles_left > 0 {
                    cycles_left -= 1;
                }
                if cycles_left == 0 {
                    continue
                }
            }
            self.memory.tick();
        }
    }
}
