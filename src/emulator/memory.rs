use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;

use crate::emulator::mbc;
use crate::emulator::PPU;

const tima_speed: [u16; 4] = [1024, 16, 64, 256];

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

    fn read_rom(&mut self, addr: u16) -> u8 {
        self.rom.read_rom(addr)
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        self.rom.write_rom(addr, val)
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        self.rom.read_ram(addr)
    }

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
        for i in 0x134 ..= 0x143 {
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
    vram: [u8; 8192],  // 0x8000 - 0x9FFF 8kB
    ram: [u8; 8192], // 0xC000 - 0xDFFF 8kB + echo at 0xE000 - 0xFDFF
    OAM: [u8; 160],  // 0xFE00 - 0xFE9F sprite attribute memory
    io: [u8; 128],  // 0xFF00 - 0xFF7F i/o ports
    hram: [u8; 127],  // 0xFF80 - 0xFFFE high ram
    pub IF: u8,  // interrupt flag 0xFF0F
    pub IER: u8,  // interrupt enable register 0xFFFF

    // timer registers
    div_tick: u16,
    tima_tick: u16,
    DIV: u8,  // FF04
    TIMA: u8, // FF05
    TMA: u8,  // FF06
    TAC: u8,  // FF07

    input: u8
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            cart: Cartridge::new(),
            ppu: PPU::new(),
            vram: [0; 8192],
            ram: [0; 8192],
            OAM: [0; 160],
            io: [0; 128],
            hram: [0; 127],
            IF: 0,
            IER: 0,

            div_tick: 0,
            tima_tick: 0,
            DIV: 0,
            TIMA: 0,
            TMA: 0,
            TAC: 0,

            input: 0x0F
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x00FF if self.cart.bootrom_enable => {
                self.cart.bootrom[addr as usize]
            },
            0x0000 ..= 0x7FFF => {
                self.cart.read_rom(addr)
            },
            0x8000 ..= 0x9FFF => {
                self.vram[(addr-0x8000) as usize]
            },
            0xA000 ..= 0xBFFF => {
                self.cart.read_ram(addr-0xa000)
            },
            0xC000 ..= 0xDFFF => {
                self.ram[(addr-0xc000) as usize]
            },
            0xE000 ..= 0xFDFF => {
                self.ram[(addr-0xe000) as usize]
            },
            0xFE00 ..= 0xFE9F => {
                self.OAM[(addr-0xfe00) as usize]
            },
            0xFF0F => self.IF,
            0xFF40 ..= 0xFF4B => {
                self.ppu.read(addr)
            },
            0xFF00 => self.input,
            0xFF04 => self.DIV,
            0xFF05 => self.TIMA,
            0xFF06 => self.TMA,
            0xFF07 => self.TAC,
            0xFF01 ..= 0xFF7F => {
                self.io[(addr-0xff00) as usize]
            },
            0xFF80 ..= 0xFFFE => {
                self.hram[(addr-0xff80) as usize]
            },
            0xFFFF => self.IER,
            _ => 0
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF02 if val == 0x81 => { // intercept serial 
                let v = self.read(0xFF01);
                if v != 0 {
                    print!("{}", v as char); 
                }
            },
            0xFF50 => {
                self.cart.bootrom = vec![];
                self.cart.bootrom_enable = false;
            },
            0x0000 ..= 0x7FFF => {
                self.cart.write_rom(addr, val)
            },
            0x8000 ..= 0x9FFF => {
                self.vram[(addr-0x8000) as usize] = val
            },
            0xA000 ..= 0xBFFF => {
                self.cart.write_ram(addr-0xa000, val)
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
            0xFF0F => self.IF = val,
            0xFF40 ..= 0xFF4B => {
                self.ppu.write(addr, val)
            },
            0xFF04 => self.DIV = 0,
            0xFF05 => self.TIMA = val,
            0xFF06 => self.TMA = val,
            0xFF07 => self.TAC = val&0x07,
            0xFF00..=0xFF7F => {
                self.io[(addr-0xff00) as usize] = val
            },
            0xFF80..=0xFFFE => {
                self.hram[(addr-0xff80) as usize] = val
            },
            0xFFFF => self.IER = val,
            _ => ()
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick(&mut self.vram, &mut self.OAM, &mut self.IF);

        
        self.div_tick += 1;
        if self.div_tick > 255 {
            self.DIV = self.DIV.wrapping_add(1);
            self.div_tick = 0;
        }

        if self.TAC & 0b00000100 != 0 {
            self.tima_tick += 1;
            if self.tima_tick >= tima_speed[self.TAC as usize&0x03] {
                self.tima_tick = 0;
                let (tmp, carry) = self.TIMA.overflowing_add(1);
                if carry {
                    self.TIMA = self.TMA;
                    self.IF |= 0b00000100;
                } else {
                    self.TIMA = tmp;
                }
            }
        }
    }
}
