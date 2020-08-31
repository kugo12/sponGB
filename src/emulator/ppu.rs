use raylib::prelude::*;


pub enum PPU_MODE {
    OAM,
    DRAW,
    HBLANK,
    VBLANK
}

pub enum Pixel_palette {  // can be used to differentiate between bg/window and sprite too
    BG,  // bg and window actually
    OBP1,
    OBP2
}

pub struct Pixel_FIFO {
    palette: Pixel_palette,
    color: u8
}

struct Sprite {
    x: u8,
    y: u8,
    tile_location: u8,
    x_flip: bool,
    y_flip: bool,
    priority: bool,
    palette: bool
}

impl Sprite {
    pub fn new(data: &[u8]) -> Sprite {
        Sprite {
            x: data[0],
            y: data[1],
            tile_location: data[2],
            x_flip: data[3]&0x20 != 0,
            y_flip: data[3]&0x40 != 0,
            priority: data[3]&0x80 != 0,
            palette: data[3]&0x10 != 0
        }
    }

    pub fn is_in_scanline(x: u8, y: u8, ly: u8, size: bool) -> bool {
        if x == 0 {
            return false;
        }
        if size {
            ly+16 >= y && ly+16 < y+16
        } else {
            ly+16 >= y && ly+16 < y+8
        }
    }
}


fn compose_two_bytes(low: u8, high: u8) -> [u8; 8] {
    let mut pixels: [u8; 8] = [0; 8];

    for (i, val) in [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01].iter().enumerate() {
        pixels[i] = (((high&val != 0) as u8) << 1) | (low&val != 0) as u8;
    }

    pixels
}

fn map_to_palette(pixel: u8, palette: u8) -> u8 {
    match pixel&0x03 {
        0 => palette&0x03,
        1 => (palette&0x0C) >> 2,
        2 => (palette&0x30) >> 4,
        3 => (palette&0xC0) >> 6,
        _ => panic!()
    }
}

fn map_color(color: u8) -> Color {
    match color&0x03 {
        0 => Color::WHITE,
        1 => Color::LIGHTGRAY,
        2 => Color::GRAY,
        3 => Color::BLACK,
        _ => panic!()
    }
}


pub struct Draw {
    pub handle: RaylibHandle,
    pub thread: RaylibThread,
    txt: Texture2D,
    frame: [u8; 144*160*3],

    tiles: Texture2D,
    tile_arr: Box<[u8; 192*128*3]>,

    tiles_dest_rect: Rectangle,
    frame_dest_rect: Rectangle,
    tiles_src_rect: Rectangle,
    frame_src_rect: Rectangle
}

impl Draw {
    pub fn new() -> Draw {
        let (mut handle, thread) = raylib::init()
            .size(160*2, (144+192+1)*2)
            .title("Gameboy emulator")
            .build();
        handle.set_target_fps(60);

        let mut img = Image::gen_image_color(160, 144, Color::BLACK);
        img.set_format(raylib::ffi::PixelFormat::UNCOMPRESSED_R8G8B8);
        let txt = handle.load_texture_from_image(&thread, &img).expect("Couldnt load texture from image");

        
        let mut img = Image::gen_image_color(128, 192, Color::BLACK);
        img.set_format(raylib::ffi::PixelFormat::UNCOMPRESSED_R8G8B8);
        let tiles = handle.load_texture_from_image(&thread, &img).expect("Couldnt load texture from image");
        
        Draw {
            handle: handle,
            thread: thread,
            txt: txt,
            frame: [0; 144*160*3],
            tiles: tiles,
            tile_arr: Box::new([0; 192*128*3]),

            tiles_dest_rect: Rectangle::new(0., 144.*2.+1., 256., 192.*2.),
            frame_dest_rect: Rectangle::new(0., 0., 160.*2., 144.*2.),
            tiles_src_rect: Rectangle::new(0., 0., 128., 192.),
            frame_src_rect: Rectangle::new(0., 0., 160., 144.),
        }
    }

    pub fn new_frame(&mut self, vram: &[u8]) {
        self.draw_vram_tiles(vram);
        self.tiles.update_texture(self.tile_arr.as_ref());

        self.txt.update_texture(&self.frame);
        let mut d = self.handle.begin_drawing(&self.thread);
        d.clear_background(Color::WHITE);
        d.draw_texture_pro(&self.txt, self.frame_src_rect, self.frame_dest_rect, Vector2::new(0., 0.), 0., Color::WHITE);
        d.draw_texture_pro(&self.tiles, self.tiles_src_rect, self.tiles_dest_rect, Vector2::new(0., 0.), 0., Color::WHITE);
    }

    fn draw_vram_tiles(&mut self, vram: &[u8]) {
        let mut vram_pos: usize;
        let mut pixel_pos: usize;
        'a: for i in 0 ..= 192 {
            for j in 0 ..= 15 {
                vram_pos = (i / 8) * 256 + j*16 + (i%8)*2;
                if vram_pos > 0x17FF { break 'a; }
                pixel_pos = (j*8 + i*128)*3;
                let p = compose_two_bytes(vram[vram_pos], vram[vram_pos+1]);
                for pix in p.iter() {
                    let a = map_color(*pix);
                    self.tile_arr[pixel_pos] = a.r;
                    self.tile_arr[pixel_pos+1] = a.g;
                    self.tile_arr[pixel_pos+2] = a.b;
                    pixel_pos += 3;
                }
            }
        }
    }

    pub fn draw_pixel(&mut self, x: u8, y: u8, color: Color) {
        let pos = (y as usize * 160 + x as usize)*3;
        self.frame[pos] = color.r;
        self.frame[pos+1] = color.g;
        self.frame[pos+2] = color.b;
    }
}


pub enum FetcherMode {
    TILE_DATA,
    TILE_LOW,
    TILE_HIGH,
    TILE_PUSH,
}


pub struct Fetcher {
    lx: u8,
    cycles: u8,
    mode: FetcherMode,
    current_pixel_push: u8,
    data: [u8; 3]
}


impl Fetcher {
    pub fn new() -> Fetcher {
        Fetcher {
            lx: 0,
            cycles: 0,
            mode: FetcherMode::TILE_DATA,
            current_pixel_push: 0,
            data: [0; 3]
        }
    }
}

pub struct PPU {
    mode: PPU_MODE,
    cycles: u16,
    pub d: Draw,

    // lcdc bools
    lcd_enabled: bool,
    window_tilemap: bool,  // false - 9800-9BFF, true - 9C00-9FFFF
    window_enabled: bool, 
    bg_window_tiledata: bool,  // false - 8800-97FF, true - 8000-8FFF 
    bg_tilemap: bool,  // false - 9800-9BFF, true - 9C00-9FFF
    sprite_size: bool,  // false - 8x8, true - 8x16
    sprite_enabled: bool,
    bg_enabled: bool,

    // registers
    lcdc: u8,  // FF40
    stat: u8,  // FF41
    scy: u8,   // FF42
    scx: u8,   // FF43
    ly: u8,    // FF44
    lyc: u8,   // FF45
    dma: u8,   // FF46
    bgp: u8,   // FF47
    obp0: u8,  // FF48
    obp1: u8,  // FF49
    wy: u8,    // FF4A
    wx: u8,    // FF4B

    // oam buffer sprites
    sprites: Vec<Sprite>,
    FIFO: Vec<Pixel_FIFO>,
    fetcher: Fetcher
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            mode: PPU_MODE::OAM,
            cycles: 0,
            d: Draw::new(),

            // lcdc bools
            lcd_enabled: true,
            window_tilemap: false,
            window_enabled: false, 
            bg_window_tiledata: true,
            bg_tilemap: false,
            sprite_size: false,
            sprite_enabled: false,
            bg_enabled: true,
            
            lcdc: 0x91,  // FF40
            stat: 0,     // FF41
            scy: 0,      // FF42
            scx: 0,      // FF43
            ly: 0,       // FF44
            lyc: 0,      // FF45
            dma: 0,      // FF46
            bgp: 0xFC,   // FF47
            obp0: 0xFF,  // FF48
            obp1: 0xFF,  // FF49
            wy: 0,       // FF4A
            wx: 0,       // FF4B

            sprites: vec![],
            FIFO: vec![],
            fetcher: Fetcher::new()
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => {
                self.lcdc = val;
                
                self.lcd_enabled = val&0x80 != 0;
                self.window_tilemap = val&0x40 != 0;
                self.window_enabled =  val&0x20 != 0;
                self.bg_window_tiledata = val&0x10 != 0;
                self.bg_tilemap = val&0x08 != 0;
                self.sprite_size = val&0x04 != 0;
                self.sprite_enabled = val&0x02 != 0;
                self.bg_enabled = val&0x01 != 0;
            },
            0xFF41 => {
                self.stat = (self.stat&0x07) | (val&0xF8)  // three lower bits are read only
            },
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => (),  // ly is read only or reset the counter on write?
            0xFF45 => self.lyc = val,
            0xFF46 => self.dma = val,  // init dma here in future
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,  // window visible when smaller than 144
            0xFF4B => self.wx = val,  // window visible when smaller than 167
            _ => panic!("Tried to write at 0x{:x} to ppu", addr)
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,  // write only?
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => panic!()
        }
    }

    pub fn tick(&mut self, vram: &mut [u8], oam: &mut [u8], IF: &mut u8) {
        use PPU_MODE::*;

        if self.d.handle.window_should_close() { panic!("Window closed"); }

        match self.mode {
            OAM => {
                if self.cycles == 79 { self.mode = DRAW }
                
                if self.cycles % 2 == 0 && self.sprites.len() < 10 {
                    let oam_pos = self.cycles as usize * 2;
                    if Sprite::is_in_scanline(oam[oam_pos], oam[oam_pos+1], self.ly, self.sprite_size) {
                        self.sprites.push(Sprite::new(&oam[oam_pos .. oam_pos+4]));
                    }
                }
                
                self.cycles += 1;
            },
            DRAW => {
                let a = self.fetcher_tick(vram, oam);
                if !a {
                    self.mode = HBLANK;
                }
                self.cycles += 1;
            },
            HBLANK => {
                if self.cycles == 456 {
                    self.cycles = 0;
                    self.ly += 1;
                    self.fetcher = Fetcher::new();

                    if self.ly == 144 {
                        self.mode = VBLANK;
                    } else {
                        self.mode = OAM;
                        self.sprites = vec![];  // clear oam sprite buffer
                    }
                } else { self.cycles += 1; }
            },
            VBLANK => {
                if self.cycles == 456 {
                    self.cycles = 0;
                    self.ly += 1;
                    if self.ly == 154 {
                        self.mode = OAM;
                        self.ly = 0;
                        self.d.new_frame(vram);
                    }
                } else { self.cycles += 1; }
            }
        }
    }

    pub fn fetcher_tick(&mut self, vram: &[u8], oam: &[u8]) -> bool {
        use FetcherMode::*;

        if self.fetcher.lx != 20 {
            match self.fetcher.mode {
                TILE_DATA => {
                    if self.fetcher.cycles == 1 {
                        let mut pos = (self.ly as u16/8) * 32 + self.fetcher.lx as u16;
                        pos = match self.bg_tilemap {  // false
                            false => 0x1800 + pos,
                            true => 0x1C00 + pos,
                        };
                        self.fetcher.data[0] = vram[pos as usize];
                    
                        self.fetcher.mode = TILE_LOW;
                    }
                    self.fetcher.cycles += 1;
                },
                TILE_LOW => {
                    if self.fetcher.cycles == 3 {
                        let pos = match self.bg_window_tiledata { // true
                            true => self.fetcher.data[0] as u16 * 16 + (self.ly as u16 % 8) * 2,
                            false => ((0x1000 as i16) + (self.fetcher.data[0] as i8 as i16 * 16)) as u16 + (self.ly as u16 % 8) * 2,
                        };
                        self.fetcher.data[1] = vram[pos as usize];
                    
                        self.fetcher.mode = TILE_HIGH;
                    }
                    self.fetcher.cycles += 1;
                },
                TILE_HIGH => {
                    if self.fetcher.cycles == 5 {
                        let pos = match self.bg_window_tiledata {
                            true => self.fetcher.data[0] as u16 * 16 + (self.ly as u16 % 8) * 2,
                            false => ((0x1000 as i16) + (self.fetcher.data[0] as i8 as i16 * 16)) as u16 + (self.ly as u16 % 8) * 2,
                        };
                        self.fetcher.data[2] = vram[(pos+1) as usize];
                    
                        self.fetcher.mode = TILE_PUSH;
                    }
                    self.fetcher.cycles += 1;
                }
                TILE_PUSH => {
                    if self.fetcher.cycles == 6 {
                        let pixels = compose_two_bytes(self.fetcher.data[1], self.fetcher.data[2]);
                        for pixel in pixels.iter() {
                            self.FIFO.push(
                                Pixel_FIFO {
                                    palette: Pixel_palette::BG,
                                    color: *pixel
                                }
                            );
                        }

                        self.fetcher.cycles += 1;
                    } else { self.fetcher.mode = TILE_DATA; self.fetcher.cycles = 0; self.fetcher.lx += 1; self.fetcher.data = [0; 3]; }
                }
            }
        }
        if self.FIFO.len() > 0 {
            let pixel = self.FIFO.remove(0);
            let color = {
                let c = match pixel.palette {
                    Pixel_palette::BG => {
                        map_to_palette(pixel.color, self.bgp)
                    },
                    Pixel_palette::OBP1 => {
                        map_to_palette(pixel.color, self.obp0)
                    },
                    Pixel_palette::OBP2 => {
                        map_to_palette(pixel.color, self.obp1)
                    }
                };
                map_color(c)
            };

            self.d.draw_pixel(self.fetcher.current_pixel_push, self.ly, color);
            self.fetcher.current_pixel_push += 1;
        } else if self.fetcher.lx == 20 {
            return false;
        }
        true
    }
}