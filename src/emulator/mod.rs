mod memory;
mod cpu;
mod ppu;
mod opcodes;
pub mod mbc;
pub mod apu;

pub use cpu::{CPU, Flag};
pub use memory::{Memory, Cartridge};
pub use opcodes::{execute, PUSH};
pub use ppu::{PPU, PPU_MODE};
pub use apu::APU;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MODE {
    DMG,
    CGB,
}
