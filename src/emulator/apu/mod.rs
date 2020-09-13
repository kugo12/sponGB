mod square;
mod wave;
mod noise;
mod apu;

pub use square::Square;
pub use wave::Wave;
pub use noise::Noise;
pub use apu::*;

pub const DUTY_CYCLE: [[i16; 8]; 4] = [
    [-1, -1, -1, -1, -1, -1, -1, 1],
    [1, -1, -1, -1, -1, -1, -1, 1],
    [1, -1, -1, -1, -1, 1, 1, 1],
    [-1, 1, 1, 1, 1, 1, 1, -1]
];
