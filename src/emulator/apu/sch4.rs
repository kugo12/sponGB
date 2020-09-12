use crate::emulator::apu::{Envelope, LengthDuty};

const DIVISOR_CODE: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub struct SCh4 {  // Noise
    pub length: LengthDuty,              // 0xFF20 NR41
    pub envelope: Envelope,            // 0xFF21 NR42
    pub counter_consecutive: u8, // 0xFF23 NR44
    pub enable: bool,
    timer: u16,
    clock_shift: u8,
    width_mode: u8,
    divisor: u8,
    lfsr: u16,
    output: i16
}

impl SCh4 {
    pub fn new() -> SCh4 {
        SCh4 {
            length: LengthDuty::new(),
            envelope: Envelope::new(),
            counter_consecutive: 0,
            enable: false,
            timer: 0,
            clock_shift: 0,
            width_mode: 0,
            divisor: 0,
            lfsr: 0,
            output: 0
        }
    }

    pub fn tick_LFSR(&mut self) {
        let xor_val = ((self.lfsr >> 1) ^ self.lfsr) & 0x1;
        self.lfsr >>= 1;
        self.lfsr = (self.lfsr & !0x4000) | (xor_val << 14);
        if self.width_mode != 0 {
            self.lfsr = (self.lfsr & !0x0040) | (xor_val << 5);
        }

        if self.lfsr&0x1 == 0 {
            self.output = 1;
        } else {
            self.output = -1;
        }
    }
    
    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.tick_LFSR();
            self.timer = DIVISOR_CODE[self.divisor as usize] << self.clock_shift;
        }
    }

    pub fn FF23_write(&mut self, val: u8) {
        self.counter_consecutive = val&0x40;
        if val&0x80 != 0 {
            self.trigger();
        }
    }

    pub fn FF22_write(&mut self, val: u8) {
        self.width_mode = val&0x8;
        self.clock_shift = (val&0xF0) >> 4;
        self.divisor = val&0x7;
        self.timer = DIVISOR_CODE[self.divisor as usize] << self.clock_shift;
    }

    pub fn FF22_read(&self) -> u8 {
        (self.clock_shift << 4) | self.width_mode | self.divisor
    }

    pub fn get_sample(&mut self) -> i16 {
        if (self.counter_consecutive != 0 && self.length.length > 0) || self.enable {
            return self.output * self.envelope.volume as i16;
        }
        0
    }

    pub fn trigger(&mut self) {
        self.length.length = 63;
        self.timer = DIVISOR_CODE[self.divisor as usize] << self.clock_shift;
        self.envelope.timer = self.envelope.period;
        self.envelope.volume = self.envelope.volume_init;
        self.enable = true;
        self.lfsr = 0x7FFF;
    }
}