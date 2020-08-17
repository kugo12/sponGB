pub struct Cartridge {
    rom: Vec<u8>
}

impl Cartridge {
    fn new() -> Cartridge {
        Cartridge {
            rom: vec![]
        }
    }

    fn read_rom(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }
    fn write_rom(&mut self, addr: u16, val: u8) {

    }

    fn read_ram(&self, addr: u16) -> u8 {
        0
    }
    fn write_ram(&mut self, addr: u16, val: u8) {

    }

    pub fn load_from_vec(&mut self, v: Vec<u8>) {
        self.rom = v;
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

    pub fn read(&self, addr: u16) -> u8 {
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
