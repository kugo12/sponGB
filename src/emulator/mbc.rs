#![allow(non_camel_case_types)]

pub trait MemoryBankController {
    fn read_rom(&mut self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, val: u8);
    fn read_ram(&mut self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, val: u8);
}

fn rom_size(val: u8) -> Result<usize, &'static str> {
    if val < 0x09 {
        return Ok((32768) << val)
    }
    Err(&"Invalid ROM size")
}

fn rom_banks(val: u8) -> u16 {
    2 << val as u16
}

fn ram_size(val: u8) -> Result<usize, &'static str> {
    match val {
        0x00 => Ok(0),
        0x01 => Ok(2048),    // 2kB
        0x02 => Ok(8192),    // 8kB   - 1 bank
        0x03 => Ok(32768),   // 32kB  - 4 banks
        0x04 => Ok(131072),  // 128kB - 16 banks
        0x05 => Ok(65536),   // 64kB  - 8 banks
        _ => Err(&"Invalid RAM size")
    }
}


pub struct dummyMBC {
    rom: Vec<u8>
}

impl dummyMBC {
    pub fn new(data: Vec<u8>) -> Box<dummyMBC> {
        Box::new(
            dummyMBC {
                rom: data
            }
        )
    }
}

impl MemoryBankController for dummyMBC {
    fn read_rom(&mut self, addr: u16) -> u8 {
        if addr as usize + 1 <= self.rom.len() {
            self.rom[addr as usize]
        } else {
            0xFF
        }
    }
    fn write_rom(&mut self, _addr: u16, _val: u8) {}
    fn read_ram(&mut self, _addr: u16) -> u8 { 0xFF }
    fn write_ram(&mut self, _addr: u16, _val: u8) {}
}


pub struct noMBC {
    rom: Vec<u8>
}

impl noMBC {
    pub fn new(data: Vec<u8>) -> Box<noMBC> {
        Box::new(noMBC {
            rom: data,
        })
    }
}

impl MemoryBankController for noMBC {
    fn read_rom(&mut self, addr: u16) -> u8 { self.rom[addr as usize] }
    fn write_rom(&mut self, _addr: u16, _val: u8){}
    fn read_ram(&mut self, _addr: u16) -> u8 { 0xFF }
    fn write_ram(&mut self, _addr: u16, _val: u8) {}
}


pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank: u8,
    banking_mode: bool,  // false -> rom, true -> ram
    bitmask: u8,
    battery: bool,
    rom_banks: u8
}

impl MBC1 {
    const MAX_ROM_SIZE: usize = 2*1024*1024;  // 2MB (in bytes)
    const MAX_RAM_SIZE: usize = 32*1024;      // 32kB (in bytes)

    fn gen_bitmask(val: u8) -> u8 {
        match rom_banks(val) {
            2 =>   0b00000001,
            4 =>   0b00000011,
            8 =>   0b00000111,
            16 =>  0b00001111,
            32 =>  0b00011111,
            64 =>  0b00011111,
            128 => 0b00011111,
            v => panic!("Unexpected MBC1 rom bank value {} from {}", v, val)
        }
    }

    pub fn new(data: Vec<u8>) -> Result<Box<MBC1>, &'static str> {
        let ram_s = ram_size(data[0x149])?;
        let rom_s = rom_size(data[0x148])?;
        let bat = data[0x147] == 0x03;
        let bitmask = MBC1::gen_bitmask(data[0x148]);

        if ram_s > MBC1::MAX_RAM_SIZE {
            return Err(&"header ram size too big for MBC1")
        }
        if rom_s != data.len() {
            return Err(&"header rom size != rom size")
        }
        if data.len() > MBC1::MAX_ROM_SIZE {
            return Err(&"Rom size too big for MBC1")
        }

        Ok(Box::new(MBC1 {
            rom_banks: rom_banks(data[0x148]) as u8,
            rom: data,
            ram: vec![0; ram_s],
            ram_enabled: false,
            bank: 1,
            banking_mode: false,
            bitmask: bitmask,
            battery: bat
        }))
    }
}

impl MemoryBankController for MBC1 {
    fn read_rom(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => {
                if self.banking_mode {
                    let bank = match self.rom_banks {
                        64 => self.bank as usize & 0x20,
                        128 => self.bank as usize & 0x60,
                        _ => 0
                    };

                    self.rom[addr as usize + bank as usize*0x4000]
                } else {
                    self.rom[addr as usize]
                }
            },
            0x4000 ..= 0x7FFF => {
                let bank = match self.rom_banks {
                    64 => self.bank as usize & 0b00111111,
                    128 => self.bank as usize & 0b01111111,
                    _ => self.bank as usize & 0b00011111
                };
                
                self.rom[(addr as usize&0x3FFF) + 0x4000*bank]
            },
            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, mut val: u8){
        match addr {
            0x0000 ..= 0x1FFF => {
                self.ram_enabled = val&0xF == 0xA;
            },
            0x2000 ..= 0x3FFF => {
                let bef = val&0x1F;
                val &= self.bitmask;
                if val == 0 {
                    val = (bef<=val) as u8
                }
                self.bank = (self.bank&0x60) | val;
            },
            0x4000 ..= 0x5FFF => {
                val = val.rotate_right(3);
                self.bank = val&0x60 | self.bank&0x1F;
            },
            0x6000 ..= 0x7FFF => {
                self.banking_mode = val&0x1 == 1;
            },
            _ => panic!()
        }
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        if self.ram_enabled {
            let bank = if self.banking_mode {
                let b = match self.ram.len() / 0x2000 {
                    2 => self.bank&0x20,
                    4 => self.bank&0x60,
                    _ => 0
                };
                b as usize >> 5
            } else { 0 };

            self.ram[addr as usize + bank*0x2000]
        } else { 0xFF }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if self.ram_enabled && self.ram.len() > 0 {
            let bank = if self.banking_mode {
                let b = match self.ram.len() / 0x2000 {
                    2 => self.bank&0x20,
                    4 => self.bank&0x60,
                    _ => 0
                };
                b as usize >> 5
            } else { 0 };

            self.ram[addr as usize + bank*0x2000] = val;
        }
    }
}


pub struct MBC2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank: usize,
    bitmask: u8
}

impl MBC2 {
    fn gen_bitmask(val: u8) -> u8 {
        match rom_banks(val) {
            2 =>   0b00000001,
            4 =>   0b00000011,
            8 =>   0b00000111,
            16 =>  0b00001111,
            v => panic!("Unexpected MBC2 rom bank value {} from {}", v, val)
        }
    }

    pub fn new(data: Vec<u8>) -> Result<Box<MBC2>, &'static str> {
        let rom_s = rom_size(data[0x148])?;
        let bitmask = MBC2::gen_bitmask(data[0x148]);
        if rom_s != data.len() {
            return Err(&"header rom size != rom size")
        }

        Ok(Box::new(
            MBC2 {
                rom: data,
                ram: vec![0; 512],
                ram_enabled: false,
                bank: 1,
                bitmask: bitmask
            }
        ))
    }
}

impl MemoryBankController for MBC2 {
    fn read_rom(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => {
                self.rom[addr as usize]
            },
            0x4000 ..= 0x7FFF => {
                self.rom[(addr&0x3FFF) as usize + self.bank*0x4000]
            },
            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, mut val: u8){
        if addr <= 0x3FFF { 
            if addr&0x0100 == 0 {
                self.ram_enabled = val&0xF == 0xA;
            } else {
                let bef = val&0xF;
                val &= self.bitmask;
                if val == 0 {
                    val = (bef<=val) as u8
                }
                self.bank = val as usize;
            }
        }
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        if self.ram_enabled {
            return self.ram[addr as usize&0x01FF]
        }

        0xFF
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if self.ram_enabled {
            self.ram[addr as usize&0x01FF] = val&0xF | 0xF0
        }
    }
}


pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank: u8,
    ram_bank: u8,
    bitmask: u8,
    rtc: bool,
    battery: bool,
}

impl MBC3 {
    const MAX_ROM_SIZE: usize = 2*1024*1024;  // 2MB (in bytes)
    const MAX_RAM_SIZE: usize = 32*1024;      // 32kB (in bytes)

    fn gen_bitmask(val: u8) -> u8 {
        match rom_banks(val) {
            2 =>   0b00000001,
            4 =>   0b00000011,
            8 =>   0b00000111,
            16 =>  0b00001111,
            32 =>  0b00011111,
            64 =>  0b00111111,
            128 => 0b01111111,
            v => panic!("Unexpected MBC3 rom bank value {} from {}", v, val)
        }
    }

    pub fn new(data: Vec<u8>) -> Result<Box<MBC3>, &'static str> {
        let ram_s = ram_size(data[0x149])?;
        let rom_s = rom_size(data[0x148])?;
        let bat = data[0x147] == 0x03;
        let bitmask = MBC3::gen_bitmask(data[0x148]);

        if ram_s > MBC3::MAX_RAM_SIZE {
            return Err(&"header ram size too big for MBC3")
        }
        if rom_s != data.len() {
            return Err(&"header rom size != rom size")
        }
        if data.len() > MBC3::MAX_ROM_SIZE {
            return Err(&"Rom is bigger than MBC3 max rom size")
        }

        Ok(Box::new(MBC3 {
            rom: data,
            ram: vec![0; ram_s],
            ram_enabled: false,
            bank: 1,
            ram_bank: 0,
            bitmask: bitmask,
            battery: bat,
            rtc: false
        }))
    }
}

impl MemoryBankController for MBC3 {
    fn read_rom(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => {
                self.rom[addr as usize]
            },
            0x4000 ..= 0x7FFF => {
                self.rom[(addr as usize & 0x3FFF) + 0x4000*self.bank as usize]
            },
            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, mut val: u8){
        match addr {
            0x0000 ..= 0x1FFF => {
                self.ram_enabled = val&0xF == 0xA;
            },
            0x2000 ..= 0x3FFF => {
                let bef = val&0x7F;
                val &= self.bitmask;
                if val == 0 {
                    val = (bef<=val) as u8
                }
                self.bank = val;
            },
            0x4000 ..= 0x5FFF => {
                val &= 0xF;
                if val > 0x7 && val < 0xD && self.rtc {
                    self.ram_bank = val;
                } else {
                    self.ram_bank = val&0b11;
                }
            },
            0x6000 ..= 0x7FFF => {
                
            },
            _ => panic!()
        }
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        if self.ram_enabled {
            if self.ram_bank < 0x4 {
                return self.ram[addr as usize + self.ram_bank as usize*0x2000]
            } else {
                match self.ram_bank {
                    0x8 => 1,  // seconds 0x00-0x3B 59
                    0x9 => 1,  // minutes 0x00-0x3B 59
                    0xA => 1,  // hours   0x00-0x17 23
                    0xB => 1,  // lower 8 bits of days counter
                    0xC => 0b01000000, 
                    a => panic!("MBC3 read_ram wrong rtc register address: {:x}", a)
                }
            }
        } else { 0x0 }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if self.ram_enabled && self.ram.len() > 0 && self.ram_bank < 0x4 {
            if self.ram_bank < 0x4 {
                self.ram[addr as usize + self.ram_bank as usize*0x2000] = val;
            }
        }
    }
}


pub struct MBC5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank: u16,
    ram_bank: u8,
    battery: bool,

    rom_bitmask: u16
}

impl MBC5 {
    fn gen_bitmask(val: u8) -> u16 {
        match rom_banks(val) {
            2 =>   0b00000001,
            4 =>   0b00000011,
            8 =>   0b00000111,
            16 =>  0b00001111,
            32 =>  0b00011111,
            64 =>  0b00111111,
            128 => 0b01111111,
            256 => 0b11111111,
            512 => 0b111111111,
            v => panic!("Unexpected MBC5 rom bank value {} from {}", v, val)
        }
    }

    pub fn new(data: Vec<u8>) -> Result<Box<MBC5>, &'static str> {
        let ram_s = ram_size(data[0x149])?;
        let rom_s = rom_size(data[0x148])?;
        let rom_bitmask = MBC5::gen_bitmask(data[0x148]);
        let bat = data[0x147] == 0x03;

        if ram_s > MBC1::MAX_RAM_SIZE {
            return Err(&"header ram size too big for MBC5")
        }
        if rom_s != data.len() {
            return Err(&"header rom size != rom size")
        }

        Ok(Box::new(MBC5 {
            rom: data,
            ram: vec![0; ram_s],
            ram_enabled: false,
            bank: 1,
            ram_bank: 0,
            battery: bat,

            rom_bitmask: rom_bitmask
        }))
    }
}

impl MemoryBankController for MBC5 {

    fn read_rom(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x3FFF => {
                self.rom[addr as usize]
            },
            0x4000 ..= 0x7FFF => {
                self.rom[(addr as usize&0x3FFF) + 0x4000*(self.bank&self.rom_bitmask) as usize]
            },
            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, val: u8){
        match addr {
            0x0000 ..= 0x1FFF => {
                self.ram_enabled = val&0xF == 0xA;
            },
            0x2000 ..= 0x2FFF => {
                self.bank = val as u16;
            },
            0x3000 ..= 0x3FFF => {
                self.bank = ((val as u16&0x1) << 8) | (self.bank&0xFF);
            },
            0x4000 ..= 0x5FFF => {
                self.ram_bank = val;
            },
            _ => ()
        }
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        if self.ram_enabled {
            self.ram[addr as usize + self.ram_bank as usize*0x2000]
        } else { 0xFF }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if self.ram_enabled && self.ram.len() > 0 {
            self.ram[addr as usize + self.ram_bank as usize*0x2000] = val;
        }
    }
}