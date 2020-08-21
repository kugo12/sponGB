use crate::emulator::{Memory, execute};

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
    pub halt: bool
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            reg_af: Register { ab: 0 },
            reg_bc: Register { ab: 0 },
            reg_de: Register { ab: 0 },
            reg_hl: Register { ab: 0 },

            SP: 0xFFFE,
            PC: 0x100,
            IME: false,
            EI: false,

            memory: Memory::new(),
            halt: false
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

    pub fn load_u8(&mut self) -> u8 {
        let v = self.memory.read(self.PC);
        self.PC += 1;
        v
    }

    pub fn load_u16(&mut self) -> u16 {
        let v = ((self.memory.read(self.PC+1) as u16) << 8) | self.memory.read(self.PC) as u16;
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
                self.memory.read(addr)
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
                self.memory.write(addr, val);
            },
            7 => *self.A() = val,
            _ => panic!()
        };
    }

    pub fn tick(&mut self) -> u8 {
        if self.EI {
            self.IME = true;
        }

        let inst = self.load_u8();
        execute(self, inst)
    }
}
