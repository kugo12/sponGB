use crate::emulator::Memory;

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

    pub memory: Memory
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

            memory: Memory::new()
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
}
