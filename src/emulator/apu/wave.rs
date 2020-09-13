use crate::emulator::apu::LengthDuty;

pub struct Wave {  // Wave
    pub enable: bool,       // 0xFF1A NR30
    pub length: LengthDuty,       // 0xFF1B NR31
    pub volume: u8, // 0xFF1C NR32
    pub freq_lo: u8,      // 0xFF1D NR33
    pub freq_hi: u8,      // 0xFF1E NR34
    pub wave_data: [u8; 16],  // 0xFF30-0xFF3F  32x4bit samples, upper nibble first
    freq: u16,
    counter_enabled: u8,
    timer: u16,
    sample_pos: u8,
}

impl Wave {
    pub fn new() -> Wave {
        Wave {
            enable: false,
            length: LengthDuty::new(),
            volume: 0,
            freq_lo: 0,
            freq_hi: 0,
            wave_data: [0; 16],
            freq: 0,
            counter_enabled: 0,
            sample_pos: 0,
            timer: 0
        }
    }

    pub fn freq_lo_write(&mut self, val: u8) {
        self.freq = (self.freq & 0xF00) | val as u16;
        self.freq_lo = val;
    }

    pub fn freq_hi_write(&mut self, val: u8) {
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
            self.sample_pos = (self.sample_pos + 1) % 32;
            self.timer = (2048 - self.freq)*4;
        }
    }

    pub fn get_sample(&mut self) -> i16 {
        if (self.counter_enabled != 0 && self.length.length > 0) || self.enable {
            let sample = self.wave_data[self.sample_pos as usize/2];
            if self.sample_pos % 2 == 0 {
                return (sample >> 4) as i16;
            } else {
                return (sample & 0xF) as i16;
            }
        }
        0
    }

    pub fn trigger(&mut self) {
        self.length.length = 255;
        self.timer = (2048 - self.freq) * 4;
        self.sample_pos = 0;
        self.enable = true;
    }
}