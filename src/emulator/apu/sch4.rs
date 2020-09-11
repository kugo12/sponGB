use crate::emulator::apu::{Envelope, LengthDuty};

pub struct SCh4 {  // Noise
    pub length: LengthDuty,              // 0xFF20 NR41
    pub envelope: Envelope,            // 0xFF21 NR42
    pub polynomial_counter: u8,  // 0xFF22 NR43
    pub counter_consecutive: u8, // 0xFF23 NR44
}

impl SCh4 {
    pub fn new() -> SCh4 {
        SCh4 {
            length: LengthDuty::new(),
            envelope: Envelope::new(),
            polynomial_counter: 0,
            counter_consecutive: 0
        }
    }

    pub fn tick(&mut self) {
        
    }

    pub fn get_sample(&self) -> i16 {
        0
    }
}