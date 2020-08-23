use std::path::Path;
use std::error::Error;

mod emulator;


fn main() -> Result<(), Box<dyn Error>> {
    let mut c = emulator::CPU::new();
    let p = Path::new(&"gbtest/cpu_instrs/individual/03-op sp,hl.gb");
    c.memory.cart.load_from_file(&p)?;
    
    println!("{}", c.memory.cart.title);
    c.run();
    Ok(())
}
