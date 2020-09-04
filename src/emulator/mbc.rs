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

fn rom_banks(val: u8) -> u8 {
    2 << val
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
            0
        }
    }
    fn write_rom(&mut self, addr: u16, val: u8) {}
    fn read_ram(&mut self, addr: u16) -> u8 { 0 }
    fn write_ram(&mut self, addr: u16, val: u8) {}
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
    fn write_rom(&mut self, addr: u16, val: u8){}
    fn read_ram(&mut self, addr: u16) -> u8 { 0xFF }
    fn write_ram(&mut self, addr: u16, val: u8) {}
}


pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank: u8,
    banking_mode: bool,  // false -> rom, true -> ram
    bitmask: u8,
    battery: bool,
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

        Ok(Box::new(MBC1 {
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
                self.rom[addr as usize]
            },
            0x4000 ..= 0x7FFF => {
                let bank = if self.banking_mode {
                    self.bank as usize & 0b00011111
                } else { 
                    match rom_banks(self.rom[0x148]) {
                        64 => self.bank as usize & 0b00111111,
                        128 => self.bank as usize & 0b01111111,
                        _ => self.bank as usize
                    }
                };
                
                self.rom[addr as usize + 0x4000*(bank-1)]
            },
            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, mut val: u8){
        match addr {
            0x0000 ..= 0x1FFF => {
                self.ram_enabled = val == 0x0A;
            },
            0x2000 ..= 0x3FFF => {
                val &= self.bitmask;
                if val == 0 { val = 1 }
                self.bank = (self.bank&0xE0) | val;
            },
            0x4000 ..= 0x5FFF => {
                val = val.rotate_right(3);
                self.bank = val&0xE0 | self.bank&0x1F;
            },
            0x6000 ..= 0x7FFF => {
                if rom_banks(self.rom[0x148]) >= 64 {
                    self.banking_mode = val&0x1 == 1;
                }
            },
            _ => panic!()
        }
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        if self.ram_enabled {
            let bank = if self.banking_mode {
                (self.bank&0x60).rotate_left(3) as usize
            } else { 0 };

            self.ram[addr as usize + bank*0x2000]
        } else { 0xFF }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        if self.ram_enabled && self.ram.len() > 0 {
            let bank = if self.banking_mode {
                (self.bank&0x60).rotate_left(3) as usize
            } else { 0 };

            self.ram[addr as usize + bank*0x2000] = val;
        }
    }
}


pub struct MBC2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank: usize
}

impl MBC2 {
    pub fn new(data: Vec<u8>) -> Result<Box<MBC2>, &'static str> {
        let rom_s = rom_size(data[0x148])?;
        if rom_s != data.len() {
            return Err(&"header rom size != rom size")
        }

        Ok(Box::new(
            MBC2 {
                rom: data,
                ram: vec![0; 512],
                ram_enabled: false,
                bank: 0
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
                self.rom[addr as usize + self.bank*0x4000]
            },
            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, val: u8){
        match addr {
            0x0000 ..= 0x1FFF if addr&0x0100 == 0 => {
                self.ram_enabled = val&0xF == 0xA ;
            },
            0x2000 ..= 0x3FFF if addr&0x0100 == 0x0100 => {
                self.bank = (val&0xF) as usize;
            },
            _ => ()
        }
    }

    fn read_ram(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x01FF => { self.ram[addr as usize] }
            _ => 1
        }
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000 ..= 0x01FF => { self.ram[addr as usize] = val&0xF; }
            _ => ()
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

        if ram_s > MBC1::MAX_RAM_SIZE {
            return Err(&"header ram size too big for MBC1")
        }
        if rom_s != data.len() {
            return Err(&"header rom size != rom size")
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
                self.rom[addr as usize + 0x4000*(self.bank as usize-1)]
            },
            _ => panic!()
        }
    }

    fn write_rom(&mut self, addr: u16, mut val: u8){
        match addr {
            0x0000 ..= 0x1FFF => {
                self.ram_enabled = val == 0x0A;
            },
            0x2000 ..= 0x3FFF => {
                val &= self.bitmask;
                if val == 0 { val = 1 }
                self.bank = val;
            },
            0x4000 ..= 0x5FFF => {
                self.ram_bank = val&0b11;
            },
            0x6000 ..= 0x7FFF => {
                
            },
            _ => panic!()
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
