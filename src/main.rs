use std::path::Path;
use std::error::Error;

mod emulator;


fn main() -> Result<(), Box<dyn Error>> {
    let mut c = emulator::CPU::new();
    let p = Path::new(&"bootrom.gb");
    let r = Path::new(&"tetris.gb");
    c.memory.cart.load_from_file(&r)?;
    c.memory.cart.load_bootrom(&p)?;
    
    // println!("{}", c.memory.cart.title);
    c.run();
    Ok(())
}
