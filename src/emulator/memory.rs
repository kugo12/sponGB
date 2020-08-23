use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::error::Error;

use crate::emulator::mbc;

pub struct Cartridge {
    rom: Box<dyn mbc::MemoryBankController>,
    pub title: String,
}

impl Cartridge {
    fn new() -> Cartridge {
        Cartridge {
            rom: mbc::dummyMBC::new(vec![]),
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
                _ => return Err("Unsupported cartridge type")
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
    vram: [u8; 8192],  // 0x8000 - 0x9FFF 8kB
    ram: [u8; 8192], // 0xC000 - 0xDFFF 8kB + echo at 0xE000 - 0xFDFF
    OAM: [u8; 160],  // 0xFE00 - 0xFE9F sprite attribute memory
    io: [u8; 128],  // 0xFF00 - 0xFF7F i/o ports
    hram: [u8; 127],  // 0xFF80 - 0xFFFE high ram
    IER: u8  // interrupt enable register 0xFFFF
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            cart: Cartridge::new(),
            vram: [0; 8192],
            ram: [0; 8192],
            OAM: [0; 160],
            io: [0; 128],
            hram: [0; 127],
            IER: 0
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => {
                self.cart.read_rom(addr)
            },
            0x8000..=0x9fff => {
                self.vram[(addr-0x8000) as usize]
            },
            0xa000..=0xbfff => {
                self.cart.read_ram(addr-0xa000)
            },
            0xc000..=0xdfff => {
                self.ram[(addr-0xc000) as usize]
            },
            0xe000..=0xfdff => {
                self.ram[(addr-0xe000) as usize]
            },
            0xfe00..=0xfe9f => {
                self.OAM[(addr-0xfe00) as usize]
            },
            0xff00..=0xff7f => {
                self.io[(addr-0xff00) as usize]
            },
            0xff80..=0xfffe => {
                self.hram[(addr-0xff80) as usize]
            },
            0xffff => self.IER,
            _ => 0
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // 0xff02 if val == 0x81 => { // intercept serial 
            //     let v = self.read(0xff01);
            //     if v != 0 {
            //         print!("{}-{} ", v, v as char); 
            //     }
            //     self.io[0x2] = 0;
            // }
            0x0000..=0x7fff => {
                self.cart.write_rom(addr, val)
            },
            0x8000..=0x9fff => {
                self.vram[(addr-0x8000) as usize] = val
            },
            0xa000..=0xbfff => {
                self.cart.write_ram(addr-0xa000, val)
            },
            0xc000..=0xdfff => {
                self.ram[(addr-0xc000) as usize] = val
            },
            0xe000..=0xfdff => {
                self.ram[(addr-0xe000) as usize] = val
            },
            0xfe00..=0xfe9f => {
                self.OAM[(addr-0xfe00) as usize] = val
            },
            0xff00..=0xff7f => {
                self.io[(addr-0xff00) as usize] = val
            },
            0xff80..=0xfffe => {
                self.hram[(addr-0xff80) as usize] = val
            },
            0xffff => self.IER = val,
            _ => ()
        }
    }
}
