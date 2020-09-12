use crate::emulator::apu::{Envelope, LengthDuty};

pub struct Sweep {
    period: u8,
    negate: u8,
    shift: u8,
    freq: u16,
    timer: u8,
    enabled: bool
}

impl Sweep {
    pub fn new() -> Sweep {
        Sweep {
            period: 0,
            negate: 0,
            shift: 0,
            freq: 0,
            timer: 0,
            enabled: false
        }
    }

    pub fn read(&self) -> u8 {
        (self.period << 4) | self.negate | self.shift
    }

    pub fn write(&mut self, val: u8) {
        self.period = (val & 0x70) >> 4;
        self.negate = val&0x8;
        self.shift = val&0x7;
    }

    pub fn calculate(&mut self) -> u16 {
        let freq = if self.negate != 0 {  // subtract
            self.freq - (self.freq >> self.shift)
        } else { // add
            self.freq + (self.freq >> self.shift)
        };

        if freq > 2047 {
            self.enabled = false;
        }

        freq
    }
}

pub struct SCh2 {  // Tone
    pub length_duty: LengthDuty, // 0xFF16 NR16
    pub envelope: Envelope,    // 0xFF17 NR22
    pub freq_lo: u8,     // 0xFF18 NR23
    pub freq_hi: u8,     // 0xFF19 NR24
    timer: u16,
    duty_pos: u8,
    pub freq: u16,
    counter_enabled: u8,
    pub sweep: Sweep,
    sweep_enable: bool,
    pub enabled: bool
}

impl SCh2 {
    pub fn new(sweep_enable: bool) -> SCh2 {
        let mut sq = SCh2 {
            length_duty: LengthDuty::new(),
            envelope: Envelope::new(),
            freq_lo: 0,
            freq_hi: 0,
            timer: 0,
            duty_pos: 0,
            freq: 0,
            counter_enabled: 0,
            sweep: Sweep::new(),
            sweep_enable: sweep_enable,
            enabled: false
        };

        sq
    }

    pub fn freq_lo_write(&mut self, val: u8) {
        self.freq = (self.freq & 0xF00) | val as u16;
        self.freq_lo = val;
    }

    pub fn freq_hi_write(&mut self, val: u8) {  // TODO: add bit 7 (restart sound) handling
        if val&0x80 != 0 {
            self.trigger();
        }
        self.counter_enabled = val&0x40;
        self.freq_hi = val;
        self.freq = (self.freq & 0x00FF) | ((val as u16&0x7) << 8);
    }

    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.duty_pos = (self.duty_pos + 1) % 8;
            self.timer = (2048 - self.freq)*4;
        }
    }

    pub fn get_sample(&mut self) -> i16 {
        if (self.counter_enabled != 0 && self.length_duty.length > 0) || self.enabled {
            return self.length_duty.duty_table[self.duty_pos as usize] * self.envelope.volume as i16;
        }
        0
    }

    pub fn trigger(&mut self) {

        self.length_duty.length = 63;
        self.timer = (2048 - self.freq) * 4;
        self.envelope.timer = self.envelope.period;
        self.envelope.volume = self.envelope.volume_init;
        self.duty_pos = 0;
        self.enabled = true;
        
        if self.sweep_enable {
            self.sweep.freq = self.freq;
            self.sweep.timer = self.sweep.period;
            self.sweep.enabled = self.sweep.period != 0 || self.sweep.shift != 0;
            if self.sweep.shift != 0 {
                self.sweep.calculate();
            }
        }
    }

    pub fn sweep_tick(&mut self) {
        if self.sweep.timer > 0 {
            self.sweep.timer -= 1;
        }
        if self.sweep.period > 0 {
            if self.sweep.timer == 0 && self.sweep.enabled {
                self.sweep.timer = self.sweep.period;

                let new = self.sweep.calculate();
                if self.sweep.shift > 0 && new < 2048 {
                    self.sweep.freq = new;
                    self.freq = new;

                    self.sweep.calculate();
                }
            }
        }
    }
}