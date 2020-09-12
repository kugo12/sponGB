mod sch2;
mod sch3;
mod sch4;
mod apu;

pub use sch2::SCh2;
pub use sch3::SCh3;
pub use sch4::SCh4;
pub use apu::*;

pub const DUTY_CYCLE: [[i16; 8]; 4] = [
    [-1, -1, -1, -1, -1, -1, -1, 1],
    [1, -1, -1, -1, -1, -1, -1, 1],
    [1, -1, -1, -1, -1, 1, 1, 1],
    [-1, 1, 1, 1, 1, 1, 1, -1]
];

pub const SAMPLE_RATE: u32 = 48000;
pub const SAMPLE_SIZE: u32 = 16;

