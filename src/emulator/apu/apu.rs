use crate::emulator::apu::*;
use crate::emulator::Draw;

use raylib::prelude::*;

const BUFFER_SIZE: usize = 8192;

pub struct Envelope {
    pub volume: u8,
    pub volume_init: u8,
    add: u8,
    pub period: u8,
    pub timer: u8,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            volume: 0,
            volume_init: 0,
            add: 0,
            period: 0,
            timer: 0
        }
    }

    pub fn read(&self) -> u8 {
        (self.volume_init << 4) | self.add | self.period
    }

    pub fn write(&mut self, val: u8) {
        self.add = val&0x8;
        self.period = val&0x7;
        self.timer = self.period;
        self.volume_init = (val >> 4) & 0xF;
        self.volume = self.volume_init;
    }

    pub fn tick(&mut self) {
        if self.period > 0 {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                if self.add != 0 {
                    if self.volume < 0xF {
                        self.volume += 1;
                    }
                } else {
                    if self.volume > 0 {
                        self.volume -= 1;
                    }
                }

                self.timer = self.period;
            }
        }
    }
}

pub struct LengthDuty {  // length counter and duty cycles
    duty: u8,
    pub duty_table: [i16; 8],
    pub length: u8,
}

impl LengthDuty {
    pub fn new() -> LengthDuty {
        LengthDuty {
            duty: 0,
            duty_table: DUTY_CYCLE[0],
            length: 0,
        }
    }
    
    pub fn read(&self) -> u8 {
        self.duty | 0x3F
    }

    pub fn write(&mut self, val: u8) {
        self.length = 64 - (val&0x3F);
        self.duty = val&0xC0;
        self.duty_table = DUTY_CYCLE[val as usize >> 6];
    }

    pub fn tick(&mut self, enable: &mut bool) {
        if self.length > 0 {
            self.length -= 1;
        }
        if self.length == 0 {
            *enable = false;
        }
    }
}

pub struct ChannelVolume {
    pub left: i16,
    pub right: i16,
    pub data: u8
}

impl ChannelVolume {
    pub fn new() -> ChannelVolume {
        ChannelVolume {
            left: 0,
            right: 0,
            data: 0
        }
    }

    pub fn write(&mut self, val: u8) {
        self.data = val;
        self.left = (val as i16&0x70) >> 4;
        self.right = val as i16&0x7;
    }
}

pub struct ChannelOutput {
    pub left_sch1: bool,
    pub left_sch2: bool,
    pub left_sch3: bool,
    pub left_sch4: bool,
    pub right_sch1: bool,
    pub right_sch2: bool,
    pub right_sch3: bool,
    pub right_sch4: bool,
    pub data: u8
}

impl ChannelOutput {
    pub fn new() -> ChannelOutput {
        ChannelOutput {
            left_sch1: false,
            left_sch2: false,
            left_sch3: false,
            left_sch4: false,
            right_sch1: false,
            right_sch2: false,
            right_sch3: false,
            right_sch4: false,
            data: 0
        }
    }

    pub fn write(&mut self, val: u8) {
        self.data = val;

        self.left_sch1 = val&0x10 != 0;
        self.left_sch2 = val&0x20 != 0;
        self.left_sch3 = val&0x40 != 0;
        self.left_sch4 = val&0x80 != 0;

        self.right_sch1 = val&0x1 != 0;
        self.right_sch2 = val&0x2 != 0;
        self.right_sch3 = val&0x4 != 0;
        self.right_sch4 = val&0x8 != 0;
    }
}

pub struct APU {
    volume: ChannelVolume,  // 0xFF24 NR50
    sch_output: ChannelOutput,         // 0xFF25 NR51
    sch_control: u8, // 0xFF26 NR52
    sc1: SCh2,
    sc2: SCh2,
    sc3: SCh3,
    sc4: SCh4,

    clock: u16,
    frame_clock: u8,
    sample_clock: u32,

    stream: raylib::ffi::AudioStream,
    audio: RaylibAudio,
    samples: [i16; BUFFER_SIZE],
}

impl APU {
    pub fn new(rl_thread: &RaylibThread) -> APU {
        let mut audio = RaylibAudio::init_audio_device();
        let mut stream = AudioStream::init_audio_stream(rl_thread, SAMPLE_RATE, SAMPLE_SIZE, 2);
        audio.play_audio_stream(&mut stream);

        let mut apu = APU {
            volume: ChannelVolume::new(),
            sch_output: ChannelOutput::new(),
            sch_control: 255,
            sc1: SCh2::new(true),
            sc2: SCh2::new(false),
            sc3: SCh3::new(),
            sc4: SCh4::new(),

            clock: 0,
            frame_clock: 0,
            sample_clock: 0,

            stream: stream.to_raw(),
            audio: audio,
            samples: [0; BUFFER_SIZE],
        };

        // apu.write(0xFF10, 0x80);
        // apu.write(0xFF11, 0xBF);
        // apu.write(0xFF12, 0xF3);
        // apu.write(0xFF14, 0xBF);
        // apu.write(0xFF16, 0x3F);
        // apu.write(0xFF17, 0x00);
        // apu.write(0xFF19, 0xBF);
        // apu.write(0xFF1A, 0x7F);
        // apu.write(0xFF1B, 0xFF);
        // apu.write(0xFF1C, 0x9F);
        // apu.write(0xFF1E, 0xBF);
        // apu.write(0xFF20, 0xFF);
        // apu.write(0xFF21, 0x00);
        // apu.write(0xFF22, 0x00);
        // apu.write(0xFF23, 0xBF);
        // apu.write(0xFF24, 0x77);
        // apu.write(0xFF25, 0xF3);
        // apu.write(0xFF26, 0xF1);

        apu
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // sound channel 1
            0xFF10 => self.sc1.sweep.read() | 0x80,  // MSb unused
            0xFF11 => self.sc1.length_duty.read(),
            0xFF12 => self.sc1.envelope.read(),
            0xFF13 => self.sc1.freq as u8,
            0xFF14 => self.sc1.freq_hi,

            // sound channel 2
            0xFF16 => self.sc2.length_duty.read(),
            0xFF17 => self.sc2.envelope.read(),
            0xFF18 => self.sc2.freq_lo,
            0xFF19 => self.sc2.freq_hi,

            // sound channel 3
            0xFF1A => ((self.sc3.enable as u8) << 7) | 0x7F,  // only MSb used
            0xFF1B => self.sc3.length.length,
            0xFF1C => (self.sc3.volume << 5)| 0x9F,  // only 6 and 5 bit used
            0xFF1D => self.sc3.freq_lo,
            0xFF1E => self.sc3.freq_hi,
            0xFF30 ..= 0xFF3F => self.sc3.wave_data[addr as usize - 0xFF30],

            // sound channel 4
            0xFF20 => self.sc4.length.read() | 0xC0,  // two MSb unused
            0xFF21 => self.sc4.envelope.read(),
            0xFF22 => self.sc4.FF22_read(),
            0xFF23 => self.sc4.counter_consecutive | 0x3F,

            // sound control registers
            0xFF24 => self.volume.data,
            0xFF25 => self.sch_output.data,
            0xFF26 => self.sch_control | 0x70,

            _ => 0xFF
        }
    }
    
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // sound channel 1
            0xFF10 => self.sc1.sweep.write(val),
            0xFF11 => self.sc1.length_duty.write(val),
            0xFF12 => self.sc1.envelope.write(val),
            0xFF13 => self.sc1.freq_lo_write(val),
            0xFF14 => self.sc1.freq_hi_write(val),

            // sound channel 2
            0xFF16 => self.sc2.length_duty.write(val),
            0xFF17 => self.sc2.envelope.write(val),
            0xFF18 => self.sc2.freq_lo_write(val),
            0xFF19 => self.sc2.freq_hi_write(val),

            // sound channel 3
            0xFF1A => self.sc3.enable = val&0x80 != 0,
            0xFF1B => self.sc3.length.length = val,
            0xFF1C => self.sc3.volume = (val >> 5)&0x3,
            0xFF1D => self.sc3.freq_lo_write(val),
            0xFF1E => self.sc3.freq_hi_write(val),
            0xFF30 ..= 0xFF3F => self.sc3.wave_data[addr as usize - 0xFF30] = val,

            // sound channel 4
            0xFF20 => self.sc4.length.write(val),
            0xFF21 => self.sc4.envelope.write(val),
            0xFF22 => self.sc4.FF22_write(val),
            0xFF23 => self.sc4.FF23_write(val),

            // sound control registers
            0xFF24 => self.volume.write(val),
            0xFF25 => self.sch_output.write(val),
            0xFF26 => self.sch_control = (val&0x80) | (self.sch_control&0x7F),

            _ => {
                println!("Write to weird APU address: {:x}, val: {:x}", addr, val);
            }
        }
    }

    pub fn tick(&mut self){
        self.sc1.tick();
        self.sc2.tick();
        self.sc3.tick();
        self.sc4.tick();

        if self.clock == 8192 {  // 4194304Hz / 8192 = 512Hz aka frame sequencer
            if self.frame_clock % 2 == 0 {  // length ctr
                self.sc1.length_duty.tick(&mut self.sc1.enabled);
                self.sc2.length_duty.tick(&mut self.sc2.enabled);
                self.sc3.length.tick(&mut self.sc3.enable);
                self.sc4.length.tick(&mut self.sc4.enable);
            }
            if self.frame_clock == 7 {  // volume envelope
                self.sc1.envelope.tick();
                self.sc2.envelope.tick();
                self.sc4.envelope.tick();
            }
            if self.frame_clock % 4 == 2 { // sweep
                self.sc1.sweep_tick();
            }

            self.frame_clock = (self.frame_clock + 1) % 8;
            self.clock = 0;
        }

        if self.sample_clock % 87 == 0 {  // 4194304 / 87 ~ 48000Hz aka sample rate
            let mut l = 0;
            let mut r = 0;
            let pos = (self.sample_clock / 87) * 2;

            let s1 = self.sc1.get_sample();
            let s2 = self.sc2.get_sample();
            let s3 = self.sc3.get_sample();
            let s4 = self.sc4.get_sample();

            if self.sch_control&0x80 != 0 {
                if self.sch_output.left_sch1 { l += s1; }
                if self.sch_output.left_sch2 { l += s2; }
                if self.sch_output.left_sch3 { l += s3; }
                if self.sch_output.left_sch4 { l += s4; }

                l *= self.volume.left;

                if self.sch_output.right_sch1 { r += s1; }
                if self.sch_output.right_sch2 { r += s2; }
                if self.sch_output.right_sch3 { r += s3; }
                if self.sch_output.right_sch4 { r += s4; }

                r *= self.volume.right;
            }

            self.samples[pos as usize] = l*4;
            self.samples[pos as usize + 1] = r*4;

            if pos == BUFFER_SIZE as u32 - 2 {
                unsafe {
                    while !raylib::ffi::IsAudioStreamProcessed(self.stream) {}
                    raylib::ffi::UpdateAudioStream(
                        self.stream,
                        self.samples.as_ptr() as *const std::os::raw::c_void,
                        BUFFER_SIZE as i32
                    );
                }
                self.sample_clock = 0;
            } else {
                self.sample_clock += 1;
            }
        } else {        
            self.sample_clock += 1;
        }

        self.clock += 1;
    }
}