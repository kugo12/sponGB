use std::path::Path;
use std::error::Error;

mod emulator;


fn main() -> Result<(), Box<dyn Error>> {
    let mut c = emulator::CPU::new();
    let p = Path::new(&"bootrom.gb");
    let r = Path::new(&"dmg-acid2.gb");
    c.memory.cart.load_from_file(&r)?;
    c.memory.cart.load_bootrom(&p)?;
    
    {
        let h = &c.memory.ppu.d.thread;
        c.memory.ppu.d.handle.set_window_title(h, &c.memory.cart.title);
    }
    println!("{}", c.memory.cart.title);
    c.run();
    Ok(())
}
