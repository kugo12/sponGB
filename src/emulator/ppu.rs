#![allow(non_snake_case, non_camel_case_types)]

use raylib::prelude::*;

use crate::emulator::MODE;

const GRAYSCALE_COLOR: [Color; 4] = [Color::WHITE, Color::LIGHTGRAY, Color::GRAY, Color::BLACK];

#[derive(PartialEq, Copy, Clone)]
pub enum PPU_MODE {
    HBLANK,
    VBLANK,
    OAM,
    DRAW
}

#[derive(Copy, Clone)]
pub enum Pixel_palette {  // can be used to differentiate between bg/window and sprite too
    BG,  // bg and window actually
    OBP0,
    OBP1,
    CGB_BG (u8),
    CGB_OBJ (u8)
}

impl From<Pixel_palette> for usize {
    fn from(pp: Pixel_palette) -> Self {
        match pp {
            Pixel_palette::BG => 0,
            Pixel_palette::OBP0 => 1,
            Pixel_palette::OBP1 => 2,
            Pixel_palette::CGB_BG(x) => x as usize,
            Pixel_palette::CGB_OBJ(x) => x as usize,
        }
    }
}

pub struct Pixel_FIFO {
    palette: Pixel_palette,
    color: u8,
    priority: bool,
    bg_attrib: Option<TileAttributes>,
    oam_pos: u8
}

#[derive(Clone, Copy)]
pub struct TileAttributes {
    pub palette: u8,
    pub vram_bank: u8,
    pub x_flip: bool,
    pub y_flip: bool,
    pub priority: bool
}

impl TileAttributes {
    pub fn new(data: u8) -> TileAttributes {
        TileAttributes {
            palette: data&0x7,
            vram_bank: (data&0x8) >> 3,
            x_flip: data&0x20 != 0,
            y_flip: data&0x40 != 0,
            priority: data&0x80 != 0
        }
    }
}

#[derive(Copy, Clone)]
struct Sprite {
    x: u8,
    y: u8,
    tile_location: u8,
    x_flip: bool,
    y_flip: bool,
    priority: bool,
    palette: Pixel_palette,
    vram_bank: u8,
    cgb_palette: Pixel_palette,
    oam_addr: u8
}

impl Sprite {
    pub fn new(data: &[u8], oam_addr: u8) -> Sprite {
        let palette = if data[3]&0x10 != 0 {
            Pixel_palette::OBP1
        } else { Pixel_palette::OBP0 };

        Sprite {
            x: data[1],
            y: data[0],
            tile_location: data[2],
            x_flip: data[3]&0x20 != 0,
            y_flip: data[3]&0x40 != 0,
            priority: data[3]&0x80 != 0,
            palette: palette,
            cgb_palette: Pixel_palette::CGB_OBJ(data[3]&0x7),
            vram_bank: (data[3]&0x8) >> 3,
            oam_addr: oam_addr
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

fn map_to_palette(pixel: u8, palette: u8) -> usize {
    ((palette >> (pixel << 1)) & 0x3) as usize
}

fn cgb_get_color_byte_by_index(index: u8, palette: &[[Color; 4]; 8]) -> u8 {
    let color = &palette[(index >> 3) as usize][((index >> 1)&0x3) as usize];
    
    if index&0x1 != 0 {  // second byte
        (color.b << 2) | (color.g >> 3)
    } else {  // first byte
        (color.g << 5) | color.r 
    }
}

fn cgb_set_color_byte_by_index(index: u8, palette: &mut [[Color; 4]; 8], val: u8) {
    let color = &mut palette[(index >> 3) as usize][((index >> 1)&0x3) as usize];

    if index&0x1 != 0 {  // second byte
        color.g = (color.g&0x7) | ((val&0x3) << 3);
        color.b = (val >> 2)&0x1F;
    } else {  // first byte
        color.r = val&0x1F;
        color.g = (color.g&0x18) | (val >> 5);
    }
}

pub struct Draw {
    pub handle: RaylibHandle,
    pub thread: RaylibThread,
    txt: Texture2D,
    frame: [u8; 144*160*3],

    // tiles: Texture2D,
    // tile_arr: Box<[u8; 192*128*3]>,

    // tiles_dest_rect: Rectangle,
    frame_dest_rect: Rectangle,
    // tiles_src_rect: Rectangle,
    frame_src_rect: Rectangle
}

impl Draw {
    pub fn new() -> Draw {
        set_trace_log(raylib::consts::TraceLogType::LOG_NONE);
        let (mut handle, thread) = raylib::init()
            .size(160*2, 144*2)
            .title("Gameboy emulator")
            .build();
        handle.set_target_fps(60);

        let mut img = Image::gen_image_color(160, 144, Color::BLACK);
        img.set_format(raylib::ffi::PixelFormat::UNCOMPRESSED_R8G8B8);
        let txt = handle.load_texture_from_image(&thread, &img).expect("Couldnt load texture from image");

        
        // let mut img = Image::gen_image_color(128, 192, Color::BLACK);
        // img.set_format(raylib::ffi::PixelFormat::UNCOMPRESSED_R8G8B8);
        // let tiles = handle.load_texture_from_image(&thread, &img).expect("Couldnt load texture from image");
        
        Draw {
            handle: handle,
            thread: thread,
            txt: txt,
            frame: [0; 144*160*3],
            // tiles: tiles,
            // tile_arr: Box::new([0; 192*128*3]),

            // tiles_dest_rect: Rectangle::new(0., 144.*2.+1., 256., 192.*2.),
            frame_dest_rect: Rectangle::new(0., 0., 160.*2., 144.*2.),
            // tiles_src_rect: Rectangle::new(0., 0., 128., 192.),
            frame_src_rect: Rectangle::new(0., 0., 160., 144.),
        }
    }

    #[inline]
    pub fn new_frame(&mut self, vram: &[u8]) {
        // self.draw_vram_tiles(vram);
        // self.tiles.update_texture(self.tile_arr.as_ref());

        self.txt.update_texture(&self.frame);
        let mut d = self.handle.begin_drawing(&self.thread);
        d.clear_background(Color::WHITE);
        d.draw_texture_pro(&self.txt, self.frame_src_rect, self.frame_dest_rect, Vector2::new(0., 0.), 0., Color::WHITE);
        // d.draw_texture_pro(&self.tiles, self.tiles_src_rect, self.tiles_dest_rect, Vector2::new(0., 0.), 0., Color::WHITE);
        d.draw_fps(0, 0);
    }

    // #[inline]
    // fn draw_vram_tiles(&mut self, vram: &[u8]) {
    //     let mut vram_pos: usize;
    //     let mut pixel_pos: usize;
    //     'a: for i in 0 ..= 192 {
    //         for j in 0 ..= 15 {
    //             vram_pos = (i / 8) * 256 + j*16 + (i%8)*2;
    //             if vram_pos > 0x17FF { break 'a; }
    //             pixel_pos = (j*8 + i*128)*3;
    //             let p = compose_two_bytes(vram[vram_pos], vram[vram_pos+1]);
    //             for pix in p.iter() {
    //                 let a = GRAYSCALE_COLOR[*pix as usize];
    //                 self.tile_arr[pixel_pos] = a.r;
    //                 self.tile_arr[pixel_pos+1] = a.g;
    //                 self.tile_arr[pixel_pos+2] = a.b;
    //                 pixel_pos += 3;
    //             }
    //         }
    //     }
    // }

    #[inline]
    pub fn draw_pixel_rgb_correct(&mut self, x: u8, y: u8, color: Color) {
        let pos = (y as usize * 160 + x as usize)*3;
        self.frame[pos] = color.r << 3;
        self.frame[pos+1] = color.g << 3;
        self.frame[pos+2] = color.b << 3;
    }

    #[inline]
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

#[derive(PartialEq)]
pub enum FetcherTileMode {
    BG,
    WIN
}


pub struct Fetcher {
    lx: u8,
    cycles: u8,
    mode: FetcherMode,
    tile_mode: FetcherTileMode,
    current_pixel_push: u8,
    discard_pixels: u8,
    current_sprite: Option<Sprite>,
    sprite_cycles: u8,
    data: [u8; 3],
    tile_attrib: TileAttributes
}


impl Fetcher {
    pub fn new() -> Fetcher {
        Fetcher {
            lx: 0,
            cycles: 0,
            mode: FetcherMode::TILE_DATA,
            tile_mode: FetcherTileMode::BG,
            current_pixel_push: 0,
            discard_pixels: 0,
            data: [0; 3],
            current_sprite: None,
            sprite_cycles: 0,
            tile_attrib: TileAttributes::new(0)
        }
    }
}

pub struct PPU {
    pub mode: PPU_MODE,
    cycles: u16,
    pub d: Draw,
    pub gb_mode: MODE,
    color_map: [Color; 4],

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
    palette: [u8; 3], // Order as in Pixel_palette enum
    wy: u8,    // FF4A
    wx: u8,    // FF4B

    // CGB background palette
    bg_index: u8,
    bg_ai: u8,
    bg_palette: [[Color; 4]; 8],

    // CGB object palette
    obj_index: u8,
    obj_ai: u8,
    obj_palette: [[Color; 4]; 8],

    obj_priority_mode: bool,

    // oam buffer sprites
    sprites: Vec<Sprite>,
    FIFO: Vec<Pixel_FIFO>,
    FIFO_sprite: Vec<Pixel_FIFO>,
    fetcher: Fetcher,
    draw_timing: u16,
    window_line: u8,
    window_y_trigger: bool,

    // input per frame - 0 is pressed
    pub in_button: u8,     // p15 5th bit
    pub in_direction: u8,  // p14 4th bit
}

impl PPU {
    pub fn new() -> PPU {
        //let cm = [Color::from((0xf0,0xff,0xf0,0xff)), Color::from((0x70,0x80,0x70,0xff)), Color::from((0x20, 0x30, 0x20, 0xff)), Color::from((0,0,0,255))];
        let cm = GRAYSCALE_COLOR;
        
        PPU {
            mode: PPU_MODE::OAM,
            cycles: 0,
            d: Draw::new(),
            gb_mode: MODE::DMG,
            color_map: cm,

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
            stat: 0x80,  // FF41
            scy: 0,      // FF42
            scx: 0,      // FF43
            ly: 0,       // FF44
            lyc: 0,      // FF45
            dma: 0,      // FF46
            palette: [0; 3],
            wy: 0,       // FF4A
            wx: 0,       // FF4B


            bg_index: 0,
            bg_ai: 0,
            bg_palette: [[Color::MAGENTA; 4]; 8],
        
            obj_index: 0,
            obj_ai: 0,
            obj_palette: [[Color::MAGENTA; 4]; 8],

            obj_priority_mode: false,

            sprites: vec![],
            FIFO: vec![],
            FIFO_sprite: vec![],
            fetcher: Fetcher::new(),
            draw_timing: 0,
            window_line: 0,
            window_y_trigger: false,

            in_button: 0xF,
            in_direction: 0xF
        }
    }

    #[inline]
    fn update_input(&mut self, IF: &mut u8, input_select: &u8) {
        use raylib::consts::KeyboardKey::{KEY_W, KEY_S, KEY_A, KEY_D, KEY_J, KEY_K, KEY_N, KEY_M};

        let hl = &self.d.handle;
        let before_dir = self.in_direction;
        let before_butt = self.in_button;

        self.in_direction = hl.is_key_up(KEY_D) as u8 | ((hl.is_key_up(KEY_A) as u8) << 1) | ((hl.is_key_up(KEY_W) as u8) << 2) | ((hl.is_key_up(KEY_S) as u8) << 3);
        self.in_button = hl.is_key_up(KEY_J) as u8 | ((hl.is_key_up(KEY_K) as u8) << 1) | ((hl.is_key_up(KEY_N) as u8) << 2) | ((hl.is_key_up(KEY_M) as u8) << 3);
    
        
        match input_select&0x30 {
            0x10 => {
                if before_butt & (!self.in_button) != 0 {
                    *IF |= 0x10;
                }
            },
            0x20 => {
                if before_dir & (!self.in_direction) != 0 {
                    *IF |= 0x10;
                }
            },
            0x30 => {
                if before_dir & (!self.in_direction) != 0 || before_butt & (!self.in_button) != 0 {
                    *IF |= 0x10;
                }
            },
            _ => ()
        }
    }

    #[inline]
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => {
                self.lcdc = val;
                
                let old_en = self.lcd_enabled;
                self.lcd_enabled = val&0x80 != 0;
                if !old_en && self.lcd_enabled {
                    self.fetcher = Fetcher::new();
                    self.cycles = 0;
                    self.mode = PPU_MODE::OAM;
                    self.FIFO = vec![];
                    self.FIFO_sprite = vec![];
                    self.ly = 0;
                    self.window_enabled = false;
                    self.window_line = 0;
                    self.set_stat(PPU_MODE::OAM);
                } else if old_en && !self.lcd_enabled {
                    self.d.frame = [0; 144*160*3];
                }
                
                self.window_tilemap = val&0x40 != 0;
                self.window_enabled =  val&0x20 != 0;
                self.bg_window_tiledata = val&0x10 != 0;
                self.bg_tilemap = val&0x08 != 0;
                self.sprite_size = val&0x04 != 0;
                self.sprite_enabled = val&0x02 != 0;
                self.bg_enabled = val&0x01 != 0;
            },
            0xFF41 => {
                self.stat = (self.stat&0x87) | (val&0x78)  // three lower bits are read only
            },
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => (),  // ly is read only or reset the counter on write?
            0xFF45 => self.lyc = val,
            0xFF46 => self.dma = val,  // init dma here in future
            0xFF47 => self.palette[usize::from(Pixel_palette::BG)] = val,
            0xFF48 => self.palette[usize::from(Pixel_palette::OBP0)] = val,
            0xFF49 => self.palette[usize::from(Pixel_palette::OBP1)] = val,
            0xFF4A => self.wy = val,  // window visible when smaller than 144
            0xFF4B => self.wx = val,  // window visible when smaller than 167

            0xFF68 => {
                self.bg_index = val&0x3F;
                self.bg_ai = val&0x80;
            },
            0xFF69 => {
                cgb_set_color_byte_by_index(self.bg_index, &mut self.bg_palette, val);
                if self.bg_ai != 0 {
                    self.bg_index += 1;
                    if self.bg_index > 0x3F {
                        self.bg_index = 0;
                    }
                }
            },
            0xFF6A => {
                self.obj_index = val&0x3F;
                self.obj_ai = val&0x80;
            },
            0xFF6B => {
                cgb_set_color_byte_by_index(self.obj_index, &mut self.obj_palette, val);
                if self.obj_ai != 0 {
                    self.obj_index += 1;
                    if self.obj_index > 0x3F {
                        self.obj_index = 0;
                    }
                }
            },
            0xFF6C => {
                self.obj_priority_mode = val&0x1 != 0;
            },
            _ => panic!("Tried to write at 0x{:x} to ppu", addr)
        }
    }

    #[inline]
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,  // write only?
            0xFF47 => self.palette[usize::from(Pixel_palette::BG)],
            0xFF48 => self.palette[usize::from(Pixel_palette::OBP0)],
            0xFF49 => self.palette[usize::from(Pixel_palette::OBP1)],
            0xFF4A => self.wy,
            0xFF4B => self.wx,

            0xFF68 => self.bg_index | self.bg_ai,
            0xFF69 => cgb_get_color_byte_by_index(self.bg_index, &self.bg_palette),
            0xFF6A => self.obj_index | self.obj_ai,
            0xFF6B => cgb_get_color_byte_by_index(self.obj_index, &self.obj_palette),
            0xFF6C => self.obj_priority_mode as u8 | 0xFE,
            _ => panic!()
        }
    }

    #[inline]
    fn set_stat(&mut self, mode: PPU_MODE) {
        self.stat = (self.stat&0xFC) | mode as u8;
    }

    #[inline]
    pub fn tick(&mut self, vram: &mut [u8], oam: &mut [u8], IF: &mut u8, input_select: &u8) {
        use PPU_MODE::*;

        if self.d.handle.window_should_close() { panic!("Window closed"); }

        if !self.lcd_enabled {
            if self.cycles % 65535 == 0 { // that doesnt need to be accurate
                self.d.new_frame(vram);
                self.update_input(IF, input_select);
                self.cycles = 0;
            }
            self.cycles += 1;
            return;
        }

        match self.mode {
            OAM => {
                if self.cycles == 79 {
                    self.mode = DRAW;
                    self.set_stat(DRAW);
                    
                    if self.wy == self.ly {
                        self.window_y_trigger = true;
                    }
                }

                if self.cycles == 0 {
                    self.set_stat(OAM);
                    if self.stat&0x20 != 0 { *IF |= 0b10; }
                }
                
                if self.cycles % 2 == 0 && self.sprites.len() < 10 {
                    let oam_pos = self.cycles as usize * 2;
                    if Sprite::is_in_scanline(oam[oam_pos+1], oam[oam_pos], self.ly, self.sprite_size) {
                        self.sprites.push(Sprite::new(&oam[oam_pos .. oam_pos+4], oam_pos as u8));
                    }
                }
                
                self.cycles += 1;
            },
            DRAW => {
                let a = self.fetcher_tick(vram);
                self.draw_timing += 1;
                if !a {
                    self.mode = HBLANK;
                    self.set_stat(HBLANK);
                    // println!("{}", self.draw_timing);
                    self.draw_timing = 0;
                    if self.stat&0x08 != 0 { *IF |= 0b10; }
                }
                self.cycles += 1;
            },
            HBLANK => {
                if self.cycles == 456 {
                    self.cycles = 0;
                    self.ly += 1;
                    if self.stat&0x40 != 0 {
                        if self.ly == self.lyc {
                            *IF |= 0b10;
                        }
                    }
                    if self.ly == self.lyc {
                        self.stat |= 0b100;
                    } else {
                        self.stat &= !0b100;
                    }
                    self.fetcher = Fetcher::new();

                    if self.ly == 144 {
                        self.mode = VBLANK;
                        self.set_stat(VBLANK);
                        *IF |= 0b1;
                        if self.stat&0x10 != 0 { *IF |= 0b10; }
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
                    if self.stat&0x40 != 0 {
                        if self.ly == self.lyc {
                            *IF |= 0b10;
                        }
                    }
                    if self.ly == self.lyc {
                        self.stat |= 0b100;
                    } else {
                        self.stat &= !0b100;
                    }
                    if self.ly == 154 {
                        if self.stat&0x40 != 0 {
                            if 0 == self.lyc {
                                *IF |= 0b10;
                            }
                        }
                        self.window_y_trigger = false;
                        self.mode = OAM;
                        self.ly = 0;
                        self.window_line = 0;
                        self.d.new_frame(vram);
                        self.update_input(IF, input_select);
                    }
                } else {
                    self.cycles += 1;
                }
            }
        }
    }

    #[inline]
    pub fn fetcher_tick(&mut self, vram: &[u8]) -> bool {
        use FetcherMode::*;
        use FetcherTileMode::*;

        if let Some(mut sprite) = self.fetcher.current_sprite {
            self.fetcher.sprite_cycles += 1;
            if self.fetcher.sprite_cycles >= 5 {
                self.fetcher.current_sprite = None;
                self.fetcher.sprite_cycles = 0;
            } else if self.fetcher.sprite_cycles == 4 {
                let (low, high) = {
                    let mut data_pos = if self.fetcher.current_sprite.unwrap().y_flip {
                        if self.sprite_size {
                            (sprite.tile_location & 0xFE) as u16 * 16 + 30 - (((self.ly as u16 + 16) - sprite.y as u16)%16) * 2
                        } else {
                            sprite.tile_location as u16 * 16 + 14 - (((self.ly as u16 + 16) - sprite.y as u16)%8) * 2
                        }
                    } else {
                        if self.sprite_size {
                            (sprite.tile_location & 0xFE) as u16 * 16 + (((self.ly as u16 + 16) - sprite.y as u16)%16) * 2
                        } else {
                            sprite.tile_location as u16 * 16 + (((self.ly as u16 + 16) - sprite.y as u16)%8) * 2
                        }
                    };

                    data_pos += 0x2000*sprite.vram_bank as u16;

                    (vram[data_pos as usize], vram[data_pos as usize + 1])
                };

                let mut _px = compose_two_bytes(low, high);
                if self.fetcher.current_sprite.unwrap().x_flip { _px.reverse(); }

                let px = if sprite.x < 8 {
                    &_px[8-sprite.x as usize..8]
                } else { &_px[..] };

                if self.gb_mode == MODE::CGB {
                    sprite.palette = sprite.cgb_palette;
                }

                for (i, val) in px.iter().enumerate() {
                    if i + 1 > self.FIFO_sprite.len() { // push to vec
                        self.FIFO_sprite.push({
                            Pixel_FIFO {
                                palette: sprite.palette,
                                color: *val,
                                priority: sprite.priority,
                                bg_attrib: None,
                                oam_pos: sprite.oam_addr
                            }
                        })
                    } else { // compose
                        let other_px = &self.FIFO_sprite[i];
                        if !self.obj_priority_mode {
                            if other_px.oam_pos > sprite.oam_addr {
                                self.FIFO_sprite[i] = Pixel_FIFO {
                                    palette: sprite.palette,
                                    color: *val,
                                    priority: sprite.priority,
                                    bg_attrib: None,
                                    oam_pos: sprite.oam_addr
                                };
                                continue;
                            }
                        }
                        if other_px.color == 0 {
                            self.FIFO_sprite[i] = Pixel_FIFO {
                                palette: sprite.palette,
                                color: *val,
                                priority: sprite.priority,
                                bg_attrib: None,
                                oam_pos: sprite.oam_addr
                            }
                        }
                    }
                }
            }

            return true;
        }

        if self.fetcher.tile_mode == BG && self.window_enabled && self.fetcher.current_pixel_push+7 == self.wx && self.window_y_trigger {
            self.FIFO = vec![];
            self.fetcher.tile_mode = WIN;
            self.fetcher.cycles = 0;
            self.fetcher.lx = 0;
            self.fetcher.mode = TILE_DATA;
        }

        if self.fetcher.discard_pixels > 1 {
            if self.FIFO.len() > 0 {
                self.FIFO.remove(0);
            }
            self.fetcher.discard_pixels -= 1;
        } else {
            match self.fetcher.mode {
                TILE_DATA => {
                    if self.fetcher.cycles == 1 {
                        let pos = if self.fetcher.tile_mode == BG {
                            let pos = ((self.ly.wrapping_add(self.scy) as u16)/8) * 32 + (self.fetcher.lx.wrapping_add(self.scx)/8) as u16;
                            match self.bg_tilemap {
                                false => 0x1800 + pos,
                                true => 0x1C00 + pos,
                            }
                        } else {
                            let pos = (self.window_line as u16 / 8) * 32 + (self.fetcher.lx/8) as u16;
                            match self.window_tilemap {
                                false => 0x1800 + pos,
                                true => 0x1C00 + pos,
                            }
                        };
                        self.fetcher.tile_attrib = TileAttributes::new(vram[pos as usize + 0x2000]);
                        self.fetcher.data[0] = vram[pos as usize];
                    
                        self.fetcher.mode = TILE_LOW;
                    }
                    self.fetcher.cycles += 1;
                },
                TILE_LOW => {  // don't need to check if it's cgb because tile attrib data in dmg mode is always 0
                    if self.fetcher.cycles == 3 {
                        let pos = if self.fetcher.tile_attrib.y_flip {
                            if self.fetcher.tile_mode == BG {
                                match self.bg_window_tiledata {
                                    true => self.fetcher.data[0] as u16 * 16 + 14 - (self.ly.wrapping_add(self.scy) as u16 % 8) * 2,
                                    false => ((0x1000 as i16) + (self.fetcher.data[0] as i8 as i16 * 16)) as u16 + 14 - (self.ly.wrapping_add(self.scy) as u16 % 8) * 2,
                                }
                            } else {
                                match self.bg_window_tiledata {
                                    true => self.fetcher.data[0] as u16 * 16 + 14 - (self.window_line as u16 % 8) * 2,
                                    false => ((0x1000 as i16) + (self.fetcher.data[0] as i8 as i16 * 16)) as u16 + 14 - (self.window_line as u16 % 8) * 2,
                                }
                            }
                        } else {
                            if self.fetcher.tile_mode == BG {
                                match self.bg_window_tiledata {
                                    true => self.fetcher.data[0] as u16 * 16 + (self.ly.wrapping_add(self.scy) as u16 % 8) * 2,
                                    false => ((0x1000 as i16) + (self.fetcher.data[0] as i8 as i16 * 16)) as u16 + (self.ly.wrapping_add(self.scy) as u16 % 8) * 2,
                                }
                            } else {
                                match self.bg_window_tiledata {
                                    true => self.fetcher.data[0] as u16 * 16 + (self.window_line as u16 % 8) * 2,
                                    false => ((0x1000 as i16) + (self.fetcher.data[0] as i8 as i16 * 16)) as u16 + (self.window_line as u16 % 8) * 2,
                                }
                            }
                        };
                        
                        let bank: usize = self.fetcher.tile_attrib.vram_bank as usize*0x2000;
                        self.fetcher.data[1] = vram[pos as usize + bank];
                        self.fetcher.data[2] = vram[(pos+1) as usize + bank];
                    
                        self.fetcher.mode = TILE_HIGH;
                    }
                    self.fetcher.cycles += 1;
                },
                TILE_HIGH => {
                    if self.fetcher.cycles == 5 {
                        self.fetcher.mode = TILE_PUSH;
                        if self.cycles == 85 && self.fetcher.tile_mode != WIN {  // discard first background tile
                            self.fetcher.mode = TILE_DATA;
                            self.fetcher.cycles = 0;
                            return true;
                        }
                    }
                    self.fetcher.cycles += 1;
                }
                TILE_PUSH => {
                    if self.fetcher.cycles == 6 {
                        let mut pixels = compose_two_bytes(self.fetcher.data[1], self.fetcher.data[2]);
                        if self.fetcher.tile_attrib.x_flip {
                            pixels.reverse();
                        }

                        for pixel in pixels.iter() {
                            self.FIFO.push(
                                Pixel_FIFO {
                                    palette: Pixel_palette::BG,
                                    color: *pixel,
                                    priority: false,
                                    bg_attrib: Some(self.fetcher.tile_attrib),
                                    oam_pos: 0
                                }
                            );
                        }

                        self.fetcher.cycles += 1;
                    } else { self.fetcher.mode = TILE_DATA; self.fetcher.cycles = 0; self.fetcher.lx += 8; }
                }
            }

            if self.FIFO.len() > 0 {
                if self.fetcher.discard_pixels == 0 && self.scx%8 != 0 {
                    self.fetcher.discard_pixels = self.scx%8 + 1;
                    return true;
                }

                if self.sprites.len() > 0 {
                    for (i, sprite) in self.sprites.iter().enumerate() {
                        if self.fetcher.current_pixel_push + 8 >= sprite.x {
                            if self.sprite_enabled {
                                self.fetcher.current_sprite = Some(*sprite);
                                self.sprites.remove(i);
                                return true;
                            }
                        }
                    }
                }

                let pixel = self.FIFO.remove(0);

                if self.gb_mode == MODE::DMG {
                    let mut color = self.color_map[map_to_palette(pixel.color, self.palette[usize::from(Pixel_palette::BG)])];
                    if !self.bg_enabled {
                        color = Color::WHITE;
                    }

                    if self.FIFO_sprite.len() > 0 {
                        let sprite_pixel = self.FIFO_sprite.remove(0);
                        if sprite_pixel.color != 0 && (!sprite_pixel.priority || color == Color::WHITE) {
                            color = self.color_map[map_to_palette(sprite_pixel.color, self.palette[usize::from(sprite_pixel.palette)])];
                        }
                    }
                    self.d.draw_pixel(self.fetcher.current_pixel_push, self.ly, color);
                    self.fetcher.current_pixel_push += 1;
                } else {
                    let mut color = self.bg_palette[pixel.bg_attrib.unwrap().palette as usize][pixel.color as usize];

                    if self.FIFO_sprite.len() > 0 {
                        let sprite_pixel = self.FIFO_sprite.remove(0);
                        if sprite_pixel.color != 0 {
                            if ((!sprite_pixel.priority || pixel.color == 0) && !pixel.bg_attrib.unwrap().priority) || (pixel.bg_attrib.unwrap().priority && pixel.color == 0) || !self.bg_enabled {
                                color = self.obj_palette[usize::from(sprite_pixel.palette)][sprite_pixel.color as usize];
                            }
                        }
                    }

                    self.d.draw_pixel_rgb_correct(self.fetcher.current_pixel_push, self.ly, color);
                    self.fetcher.current_pixel_push += 1;
                };

            };

            if self.fetcher.current_pixel_push == 160 {
                self.FIFO = vec![];
                self.FIFO_sprite = vec![];
                if self.fetcher.tile_mode == WIN {
                    self.window_line += 1;
                }
                return false;
            }
        }
        true
    }
}