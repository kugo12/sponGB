use std::path::Path;
use std::error::Error;

mod emulator;

fn main() -> Result<(), Box<dyn Error>> {
    let mut c = emulator::CPU::new();
    let p = Path::new(&"gbc_bootrom.gbc");
    let r = Path::new(&"pksilver.gbc");
    c.memory.load_rom(&r)?;
    c.memory.load_bootrom(&p)?;
    
    {
        let h = &c.memory.ppu.d.thread;
        c.memory.ppu.d.handle.set_window_title(h, &c.memory.cart.title);
    }
    println!("{}", c.memory.cart.title);
    c.run();
    Ok(())
}
