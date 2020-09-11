use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use raylib::prelude::*;

use crate::emulator::{mbc, PPU, APU};

const TIMA_SPEED: [u16; 4] = [512, 8, 32, 128];

pub struct Cartridge {
    rom: Box<dyn mbc::MemoryBankController>,
    pub bootrom: Vec<u8>,
    pub bootrom_enable: bool,
    pub title: String,
}

impl Cartridge {
    fn new() -> Cartridge {
        Cartridge {
            rom: mbc::dummyMBC::new(vec![]),
            bootrom: vec![],
            bootrom_enable: false,
            title: String::new()
        }
    }

    #[inline]
    fn read_rom(&mut self, addr: u16) -> u8 {
        self.rom.read_rom(addr)
    }

    #[inline]
    fn write_rom(&mut self, addr: u16, val: u8) {
        self.rom.write_rom(addr, val)
    }

    #[inline]
    fn read_ram(&mut self, addr: u16) -> u8 {
        self.rom.read_ram(addr)
    }

    #[inline]
    fn write_ram(&mut self, addr: u16, val: u8) {
        self.rom.write_ram(addr, val)
    }

    pub fn load_bootrom(&mut self, p: &Path) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(p)?;
        let mut data: Vec<u8> = vec![];
        file.read_to_end(&mut data)?;

        if data.len() != 0x100 {
            panic!("Invalid bootrom");
        }
        self.bootrom = data;
        self.bootrom_enable = true;

        Ok(())
    }

    pub fn load_from_vec(&mut self, v: Vec<u8>) {
        self.rom = mbc::dummyMBC::new(v)
    }

    pub fn load_from_file(&mut self, p: &Path) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(p)?;
        let mut data: Vec<u8> = vec![];
        file.read_to_end(&mut data)?;

        self.interprete_header(data)?;

        Ok(())
    }

    fn interprete_header(&mut self, data: Vec<u8>) -> Result<(), &str> {
        if data.len() > 0x14F {
            if data[0x014D] != Cartridge::calculate_header_checksum(&data) {
                return Err(&"Invalid ROM header checksum")
            }

            if data[0x143] == 0xC0 {
                return  Err(&"Gameboy Color only rom")
            }

            self.title = Cartridge::get_title(&data);
            match data[0x147] {
                0x00 => {
                    self.rom = mbc::noMBC::new(data);
                },
                0x01 ..= 0x03 => {
                    self.rom = mbc::MBC1::new(data)?;
                },
                0x05 | 0x06 => {
                    self.rom = mbc::MBC2::new(data)?;
                },
                0x0F ..= 0x13 => {
                    self.rom = mbc::MBC3::new(data)?;
                }
                _ => panic!("{:x} - unsupported cartridge type", data[0x147])
            };

            Ok(())
        } else {
            Err(&"ROM too small")
        }
    }

    fn get_title(data: &Vec<u8>) -> String {
        let mut t = String::new();
        for i in 0x134 ..= 0x13E {
            if data[i] == 0 { break; }
            t.push(data[i] as char);
        }
        t
    }

    fn calculate_header_checksum(data: &Vec<u8>) -> u8 {
        let mut sum: u8 = 0;

        for i in 0x134 ..= 0x014C {
            sum = sum.wrapping_sub(data[i as usize]).wrapping_sub(1);
        }

        sum
    }
}

pub struct Memory {
    pub cart: Cartridge,  // ROM -> 0x0000-0x7FFF 32kB, RAM -> 0xA000-0xBFFF 8kB
    pub ppu: PPU,
    apu: APU,

    vram: [u8; 8192],  // 0x8000 - 0x9FFF 8kB
    ram: [u8; 8192], // 0xC000 - 0xDFFF 8kB + echo at 0xE000 - 0xFDFF
    OAM: [u8; 160],  // 0xFE00 - 0xFE9F sprite attribute memory
    io: [u8; 128],  // 0xFF00 - 0xFF7F i/o ports
    hram: [u8; 127],  // 0xFF80 - 0xFFFE high ram
    pub IF: u8,  // interrupt flag 0xFF0F
    pub IER: u8,  // interrupt enable register 0xFFFF

    // timer registers
    DIV: u16,  // FF04
    TIMA: u8, // FF05
    TMA: u8,  // FF06
    TAC: u8,  // FF07
    tima_schedule: i8,
    last_div: u16,

    serial_control: u8,
    serial_transfer: u8,
    serial_count_interrupt: u8,

    input_select: u8,
}

impl Memory {
    pub fn new() -> Memory {
        let ppu = PPU::new();
        let apu = APU::new(&ppu.d.thread);

        Memory {
            cart: Cartridge::new(),
            ppu: ppu,
            apu: apu,

            vram: [0; 8192],
            ram: [0; 8192],
            OAM: [0; 160],
            io: [0; 128],
            hram: [0; 127],
            IF: 0b11100000,
            IER: 0b11100000,

            DIV: 0,
            TIMA: 0,
            TMA: 0,
            TAC: 0b11111000,
            tima_schedule: -1,
            last_div: 0,

            serial_control: 0b01111110,
            serial_transfer: 0xFF,
            serial_count_interrupt: 0,

            input_select: 0,
        }
    }

    #[inline]
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x00FF if self.cart.bootrom_enable => {
                self.cart.bootrom[addr as usize]
            },
            0x0000 ..= 0x7FFF => self.cart.read_rom(addr),
            0x8000 ..= 0x9FFF => self.vram[(addr-0x8000) as usize],
            0xA000 ..= 0xBFFF => self.cart.read_ram(addr-0xa000),
            0xC000 ..= 0xDFFF => self.ram[(addr-0xc000) as usize],
            0xE000 ..= 0xFDFF => self.ram[(addr-0xe000) as usize],
            0xFE00 ..= 0xFE9F => self.OAM[(addr-0xfe00) as usize],

            // Memory mapped io
            0xFF00 => {
                match self.input_select&0x30 {
                    0x00 => 0xF,
                    0x10 => self.ppu.in_button | self.input_select,
                    0x20 => self.ppu.in_direction | self.input_select,
                    0x30 => 0xFF,
                    _ => panic!()
                }
            },
            0xFF01 => self.serial_transfer,
            0xFF02 => self.serial_control,
            0xFF04 => (self.DIV >> 8) as u8,
            0xFF05 => self.TIMA,
            0xFF06 => self.TMA,
            0xFF07 => self.TAC,
            0xFF0F => self.IF,
            0xFF10 ..= 0xFF3F => self.apu.read(addr),
            0xFF40 ..= 0xFF4B => self.ppu.read(addr),
            0xFF80 ..= 0xFFFE => self.hram[(addr-0xff80) as usize],
            0xFFFF => self.IER,
            _ => 0xFF
        }
    }

    #[inline]
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000 ..= 0x7FFF => {
                self.cart.write_rom(addr, val)
            },
            0x8000 ..= 0x9FFF => {
                self.vram[(addr-0x8000) as usize] = val
            },
            0xA000 ..= 0xBFFF => {
                self.cart.write_ram(addr-0xA000, val)
            },
            0xC000 ..= 0xDFFF => {
                self.ram[(addr-0xc000) as usize] = val
            },
            0xE000 ..= 0xFDFF => {
                self.ram[(addr-0xe000) as usize] = val
            },
            0xFE00 ..= 0xFE9F => {
                self.OAM[(addr-0xfe00) as usize] = val
            },

            // Memory mapped io
            0xFF00 => {
                self.input_select = val&0x30
            },
            0xFF01 => {
                self.serial_transfer = val;
            },
            0xFF02 => { // intercept serial 
                let v = self.read(0xFF01);
                if v != 0 {
                    print!("{}", v as char); 
                }
                self.serial_control = 0b01111110 | val;
                if val&0x80 != 0 {
                    self.serial_count_interrupt = 8;
                }
            },
            0xFF04 => {
                self.DIV = 0;
                self.TIMA = self.TMA;
                self.tima_schedule = -1;
            },
            0xFF05 => {
                if self.tima_schedule != 1 {
                    self.tima_schedule = -1;
                    self.TIMA = val
                }
            },
            0xFF06 => self.TMA = val,
            0xFF07 => self.TAC = 0b11111000 | val,
            0xFF0F => self.IF = 0b11100000 | val,
            0xFF10 ..= 0xFF3F => {
                self.apu.write(addr, val)
            }
            0xFF46 => {  // TODO: real timings, not instant
                let mut pos = (val as u16) << 8;
                loop {
                    self.OAM[pos as usize&0xFF] = self.read(pos);
                    if pos&0xFF == 0x9F { break }
                    pos += 1;
                }
            }
            0xFF40 ..= 0xFF4B => {
                self.ppu.write(addr, val)
            },
            0xFF50 => {
                self.cart.bootrom = vec![];
                self.cart.bootrom_enable = false;
            },
            0xFF80..=0xFFFE => {
                self.hram[(addr-0xff80) as usize] = val
            },
            0xFFFF => self.IER = 0b11100000 | val,
            _ => ()
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick(&mut self.vram, &mut self.OAM, &mut self.IF, &self.input_select);
        self.apu.tick();

        self.serial_transfer = (self.serial_transfer >> 1) | 0x80;
        if self.serial_count_interrupt > 0 {
            self.serial_count_interrupt -= 1;
            if self.serial_count_interrupt == 0 {
                self.IF |= 0x8;
            }
        }

        self.DIV = self.DIV.wrapping_add(1);

        if self.tima_schedule >= 0 {
            if self.tima_schedule <= 2 {
                self.TIMA = self.TMA;
                self.IF |= 0b00000100;
                self.last_div = self.DIV&TIMA_SPEED[self.TAC as usize&0x03];
            }
            self.tima_schedule -= 1;
        }

        let c = if self.TAC&0x4 != 0 { 0xFFFF } else { 0 };
        let b = (self.DIV&TIMA_SPEED[self.TAC as usize&0x03])&c;
        if !b & self.last_div != 0 {
            let (tmp, carry) = self.TIMA.overflowing_add(1);
            self.TIMA = tmp;
            if carry {
                self.tima_schedule = 5;
            }
        }
        self.last_div = self.DIV&TIMA_SPEED[self.TAC as usize&0x03];
    }
}
