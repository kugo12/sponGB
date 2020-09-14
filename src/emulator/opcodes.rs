#![allow(non_snake_case)]

use crate::emulator::{CPU, Flag};


fn LDRR(cpu: &mut CPU, instr: u8) -> u8 {
    let to = (instr - 0x40) >> 3;
    let (val, hl) = cpu.get_val_reg(instr);

    match to {
        0 => *cpu.B() = val,
        1 => *cpu.C() = val,
        2 => *cpu.D() = val,
        3 => *cpu.E() = val,
        4 => *cpu.H() = val,
        5 => *cpu.L() = val,
        6 => {
            let addr = *cpu.HL();
            cpu.write(addr, val);
        },
        7 => *cpu.A() = val,
        _ => panic!()
    };

    if hl || to == 6 {
        2
    } else {
        1
    }
}


pub fn PUSH(cpu: &mut CPU, val: u16) {
    cpu.write(cpu.SP - 1, (val >> 8) as u8);
    cpu.write(cpu.SP - 2, val as u8);
    cpu.SP = cpu.SP.wrapping_sub(2);
}


fn POP(cpu: &mut CPU) -> u16 {
    let lsb = cpu.read(cpu.SP) as u16;
    let msb = cpu.read(cpu.SP + 1) as u16;
    cpu.SP = cpu.SP.wrapping_add(2);
    (msb << 8) | lsb
}


fn ADD(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xC6 {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };

    let (tmp, c) = cpu.A().overflowing_add(val);
    let hc = ((val&0xF)+(*cpu.A()&0xF)) & 0x10 == 0x10;
    *cpu.A() = tmp;

    cpu.set_flag(Flag::Z, tmp == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, hc);
    cpu.set_flag(Flag::C, c);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn ADC(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xCE {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };
    let carry = cpu.get_flag(Flag::C) as u16;

    let c = (((val as u16)&0xFF)+((*cpu.A() as u16)&0xFF) + carry) & 0x100 == 0x100;
    let hc = ((val&0xF)+(*cpu.A()&0xF) + carry as u8) & 0x10 == 0x10;
    let tmp = val.wrapping_add(*cpu.A()).wrapping_add(carry as u8);
    *cpu.A() = tmp;

    cpu.set_flag(Flag::Z, tmp == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, hc);
    cpu.set_flag(Flag::C, c);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn SUB(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xD6 {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };

    let (tmp, c) = cpu.A().overflowing_sub(val);
    let hc = *cpu.A()&0xF < val&0xF;
    *cpu.A() = tmp;

    cpu.set_flag(Flag::Z, tmp == 0);
    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, hc);
    cpu.set_flag(Flag::C, c);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn SBC(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xDE {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };
    let carry = cpu.get_flag(Flag::C) as u8;

    let tmp = cpu.A().wrapping_sub(val).wrapping_sub(carry);
    let hc = *cpu.A()&0xF < (val&0xF) + carry;
    let c = (*cpu.A() as u16) < val as u16 + carry as u16;
    *cpu.A() = tmp;

    cpu.set_flag(Flag::Z, tmp == 0);
    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, hc);
    cpu.set_flag(Flag::C, c);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn AND(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xE6 {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };

    *cpu.A() &= val;
    let z = *cpu.A() == 0;

    cpu.set_flag(Flag::Z, z);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, true);
    cpu.set_flag(Flag::C, false);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn OR(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xF6 {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };

    *cpu.A() |= val;
    let z = *cpu.A() == 0;

    cpu.set_flag(Flag::Z, z);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, false);
    cpu.set_flag(Flag::C, false);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn XOR(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xEE {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };

    *cpu.A() ^= val;
    let z = *cpu.A() == 0;

    cpu.set_flag(Flag::Z, z);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, false);
    cpu.set_flag(Flag::C, false);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn CP(cpu: &mut CPU, instr: u8) -> u8 {
    let (val, more_cycles) = if instr == 0xFE {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };

    let z = *cpu.A() == val;
    let c = *cpu.A() < val;
    let hc = *cpu.A()&0xF < val&0xF;

    cpu.set_flag(Flag::Z, z);
    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, hc);
    cpu.set_flag(Flag::C, c);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn INC(cpu: &mut CPU, inst: u8) -> u8 {
    let (h, z, cycles) = {
        if inst == 0x34 {
            let addr = *cpu.HL();
            let val = cpu.read(addr);
            let tmp = val.wrapping_add(1);
            cpu.write(addr, tmp);

            ((val&0xF)+1 == 0x10, tmp == 0, 3)
        } else {
            let val = match inst {
                0x04 => cpu.B(),
                0x0C => cpu.C(),
                0x14 => cpu.D(),
                0x1C => cpu.E(),
                0x24 => cpu.H(),
                0x2C => cpu.L(),
                0x3C => cpu.A(),
                _ => panic!()
            };

            let tmp = *val;
            *val = val.wrapping_add(1);

            ((tmp&0xF)+1 == 0x10, *val == 0, 1)
        }
    };
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, h);
    cpu.set_flag(Flag::Z, z);
    cycles
}


fn DEC(cpu: &mut CPU, inst: u8) -> u8 {
    let (h, z, cycles) = {
        if inst == 0x35 {
            let addr = *cpu.HL();
            let val = cpu.read(addr);
            let tmp = val.wrapping_sub(1);
            cpu.write(addr, tmp);

            (val&0x0F == 0, tmp == 0, 3)
        } else {
            let val = match inst {
                0x05 => cpu.B(),
                0x0D => cpu.C(),
                0x15 => cpu.D(),
                0x1D => cpu.E(),
                0x25 => cpu.H(),
                0x2D => cpu.L(),
                0x3D => cpu.A(),
                _ => panic!()
            };

            let tmp = *val;
            *val = val.wrapping_sub(1);

            (tmp&0x0F  == 0, *val == 0, 1)
        }
    };
    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, h);
    cpu.set_flag(Flag::Z, z);
    cycles
}


fn ADDTOHL(cpu: &mut CPU, inst: u8) {
    let val = match inst {
        0x09 => *cpu.BC(),
        0x19 => *cpu.DE(),
        0x29 => *cpu.HL(),
        0x39 => cpu.SP,
        _ => panic!()
    };

    let (tmp, c) = cpu.HL().overflowing_add(val);
    let hc = ((*cpu.HL()&0xFFF) + (val&0xFFF)) & 0x1000 == 0x1000;
    *cpu.HL() = tmp;

    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, hc);
    cpu.set_flag(Flag::C, c);
}


fn DAA(cpu: &mut CPU) {
    let mut val = *cpu.A();

    if cpu.get_flag(Flag::N) {
        if cpu.get_flag(Flag::C) {
            val = val.wrapping_sub(0x60);
        }
        if cpu.get_flag(Flag::H) {
            val = val.wrapping_sub(0x6);
        }
    } else {
        if cpu.get_flag(Flag::C) || val>0x99 {
            val = val.wrapping_add(0x60);
            cpu.set_flag(Flag::C, true);
        }
        if cpu.get_flag(Flag::H) || (val&0xF) > 0x9 {
            val = val.wrapping_add(0x6);
        }
    }

    cpu.set_flag(Flag::Z, val == 0);
    cpu.set_flag(Flag::H, false);
    *cpu.A() = val;
}


fn RLC(cpu: &mut CPU, mut val: u8) -> u8 {
    val = val.rotate_left(1);

    cpu.set_flag(Flag::Z, val == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, false);
    cpu.set_flag(Flag::C, (val&0x1) == 1);

    val
}


fn RL(cpu: &mut CPU, mut val: u8) -> u8 {
    let c = cpu.get_flag(Flag::C) as u8;
    val = val.rotate_left(1);

    cpu.set_flag(Flag::Z, (val & 0b11111110) | c == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, false);
    cpu.set_flag(Flag::C, (val&0x1) == 1);

    (val & 0b11111110) | c
}


fn RRC(cpu: &mut CPU, mut val: u8) -> u8 {
    cpu.set_flag(Flag::C, (val&0x1) == 1);
    val = val.rotate_right(1);

    cpu.set_flag(Flag::Z, val == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, false);

    val
}


fn RR(cpu: &mut CPU, mut val: u8) -> u8 {
    let c = (cpu.get_flag(Flag::C) as u8) << 7;
    cpu.set_flag(Flag::C, (val&0x1) == 1);
    val = val.rotate_right(1);

    cpu.set_flag(Flag::Z, (val & 0b01111111) | c == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, false);

    (val & 0b01111111) | c
}


pub fn execute(cpu: &mut CPU, inst: u8) -> u8 {
    match inst {
        // STOP
        0x10 => {
            1  // TODO
        }

        // HALT
        0x76 => {
            cpu.halt = true;
            1
        },

        // LOAD R1, 8bit
        0x06 => {
            *cpu.B() = cpu.load_u8();
            2
        },
        0x0E => {
            *cpu.C() = cpu.load_u8();
            2
        },
        0x16 => {
            *cpu.D() = cpu.load_u8();
            2
        },
        0x1E => {
            *cpu.E() = cpu.load_u8();
            2
        },
        0x26 => {
            *cpu.H() = cpu.load_u8();
            2
        },
        0x2E => {
            *cpu.L() = cpu.load_u8();
            2
        },
        0x36 => {
            let addr = *cpu.HL();
            let val = cpu.load_u8();
            cpu.write(addr, val);
            3
        },
        0x3E => {
            *cpu.A() = cpu.load_u8();
            2
        },

        // LOAD R1, R2
        0x40 ..= 0x7F => {
            LDRR(cpu, inst)
        },

        // LOAD ACC
        0x02 => {
            let addr = *cpu.BC();
            let val = *cpu.A();
            cpu.write(addr, val);
            2
        },
        0x12 => {
            let addr = *cpu.DE();
            let val = *cpu.A();
            cpu.write(addr, val);
            2
        },
        0x22 => {
            let addr = *cpu.HL();
            let val = *cpu.A();
            cpu.write(addr, val);
            *cpu.HL() += 1;
            2
        },
        0x32 => {
            let addr = *cpu.HL();
            let val = *cpu.A();
            cpu.write(addr, val);
            *cpu.HL() -= 1;
            2
        },

        0x0A => {
            let addr = *cpu.BC();
            *cpu.A() = cpu.read(addr);
            2
        },
        0x1A => {
            let addr = *cpu.DE();
            *cpu.A() = cpu.read(addr);
            2
        },
        0x2A => {
            let addr = *cpu.HL();
            *cpu.A() = cpu.read(addr);
            *cpu.HL() = cpu.HL().wrapping_add(1);
            2
        },
        0x3A => {
            let addr = *cpu.HL();
            *cpu.A() = cpu.read(addr);
            *cpu.HL() = cpu.HL().wrapping_sub(1);
            2
        },

        0xE0 => {
            let addr = 0xFF00 + cpu.load_u8() as u16;
            let val = *cpu.A();
            cpu.write(addr, val);
            3
        },
        0xF0 => {
            let addr = 0xFF00 + cpu.load_u8() as u16;
            *cpu.A() = cpu.read(addr);
            3
        },

        0xE2 => {
            let addr = 0xFF00 + *cpu.C() as u16;
            let val = *cpu.A();
            cpu.write(addr, val);
            2
        },
        0xF2 => {
            let addr = 0xFF00 + *cpu.C() as u16;
            *cpu.A() = cpu.read(addr);
            2
        },

        0xEA => {
            let addr = cpu.load_u16();
            let val = *cpu.A();
            cpu.write(addr, val);
            4
        },
        0xFA => {
            let addr = cpu.load_u16();
            *cpu.A() = cpu.read(addr);
            4
        }

        // 16-bit load
        0x01 => {
            *cpu.BC() = cpu.load_u16();
            3
        }
        0x11 => {
            *cpu.DE() = cpu.load_u16();
            3
        },
        0x21 => {
            *cpu.HL() = cpu.load_u16();
            3
        },
        0x31 => {
            cpu.SP = cpu.load_u16();
            3
        },

        0xF9 => {
            cpu.SP = *cpu.HL();
            2
        },

        0xF8 => { // LD HL, SP+n
            let sp = cpu.SP as i16;
            let n  = cpu.load_u8() as i8 as i16;
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, ((sp&0xF)+(n&0xF)) & 0x10 == 0x10);
            cpu.set_flag(Flag::C, ((sp&0xFF)+(n&0xFF)) & 0x100 == 0x100);
            *cpu.HL() = sp.wrapping_add(n) as u16;
            3
        },

        0x08 => {
            let addr = cpu.load_u16();
            cpu.write(addr, cpu.SP as u8);
            cpu.write(addr+1, (cpu.SP >> 8) as u8);
            5
        },

        // PUSH
        0xF5 => {
            let val = *cpu.AF();
            PUSH(cpu, val);
            4
        },
        0xC5 => {
            let val = *cpu.BC();
            PUSH(cpu, val);
            4
        },
        0xD5 => {
            let val = *cpu.DE();
            PUSH(cpu, val);
            4
        },
        0xE5 => {
            let val = *cpu.HL();
            PUSH(cpu, val);
            4
        },

        // POP
        0xF1 => {
            *cpu.AF() = POP(cpu)&0xFFF0;
            3
        },
        0xC1 => {
            *cpu.BC() = POP(cpu);
            3
        },
        0xD1 => {
            *cpu.DE() = POP(cpu);
            3
        },
        0xE1 => {
            *cpu.HL() = POP(cpu);
            3
        },

        // ADD
        0x80 ..= 0x87 | 0xC6 => {
            ADD(cpu, inst)
        },

        // ADC
        0x88 ..= 0x8F | 0xCE => {
            ADC(cpu, inst)
        },

        // SUB
        0x90 ..= 0x97 | 0xD6 => {
            SUB(cpu, inst)
        },

        // SBC
        0x98 ..= 0x9F | 0xDE => {
            SBC(cpu, inst)
        },

        // AND
        0xA0 ..= 0xA7 | 0xE6 => {
            AND(cpu, inst)
        },
        
        // XOR
        0xA8 ..= 0xAF | 0xEE => {
            XOR(cpu, inst)
        },

        // OR
        0xB0 ..= 0xB7 | 0xF6 => {
            OR(cpu, inst)
        },

        // CP
        0xB8 ..= 0xBF | 0xFE => {
            CP(cpu, inst)
        },

        // INC
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
            INC(cpu, inst)
        },

        // DEC
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
            DEC(cpu, inst)
        },

        // ADD HL, r
        0x09 | 0x19 | 0x29 | 0x39 => {
            ADDTOHL(cpu, inst);
            2
        },

        // ADD SP, i8
        0xE8 => { 
            let sp = cpu.SP as i16;
            let n  = cpu.load_u8() as i8 as i16;
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, ((sp&0xF)+(n&0xF)) & 0x10 == 0x10);
            cpu.set_flag(Flag::C, ((sp&0xFF)+(n&0xFF)) & 0x100 == 0x100);
            cpu.SP = sp.wrapping_add(n) as u16;
            4
        },

        // INC 16bit
        0x03 => {
            *cpu.BC() = cpu.BC().wrapping_add(1);
            2
        },
        0x13 => {
            *cpu.DE() = cpu.DE().wrapping_add(1);
            2
        },
        0x23 => {
            *cpu.HL() = cpu.HL().wrapping_add(1);
            2
        },
        0x33 => {
            cpu.SP = cpu.SP.wrapping_add(1);
            2
        }

        // DEC 16bit
        0x0B => {
            *cpu.BC() = cpu.BC().wrapping_sub(1);
            2
        },
        0x1B => {
            *cpu.DE() = cpu.DE().wrapping_sub(1);
            2
        },
        0x2B => {
            *cpu.HL() = cpu.HL().wrapping_sub(1);
            2
        },
        0x3B => {
            cpu.SP = cpu.SP.wrapping_sub(1);
            2
        },

        // DAA
        0x27 => {
            DAA(cpu);
            1
        },

        // CPL
        0x2F => {
            *cpu.A() = !*cpu.A();
            cpu.set_flag(Flag::N, true);
            cpu.set_flag(Flag::H, true);
            1
        },

        // CCF
        0x3F => {
            let c = cpu.get_flag(Flag::C);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, !c);
            1
        },
        
        // SCF
        0x37 => {
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, true);
            1
        },

        // RLCA
        0x07 => {
            let mut val = *cpu.A();
            val = RLC(cpu, val);
            *cpu.A() = val;
            cpu.set_flag(Flag::Z, false);
            1
        },

        // RLA
        0x17 => {
            let mut val = *cpu.A();
            val = RL(cpu, val);
            *cpu.A() = val;
            cpu.set_flag(Flag::Z, false);
            1
        },

        // RRCA
        0x0F => {
            let mut val = *cpu.A();
            val = RRC(cpu, val);
            *cpu.A() = val;
            cpu.set_flag(Flag::Z, false);
            1
        },

        // RRA
        0x1F => {
            let mut val = *cpu.A();
            val = RR(cpu, val);
            *cpu.A() = val;
            cpu.set_flag(Flag::Z, false);
            1
        },

        // JP nn
        0xC3 => {
            cpu.PC = cpu.load_u16();
            4
        },

        // JP cc, nn
        0xC2 => {
            let val = cpu.load_u16();
            if !cpu.get_flag(Flag::Z) {
                cpu.PC = val;
                4
            } else {
                3
            }
        },
        0xCA => {
            let val = cpu.load_u16();
            if cpu.get_flag(Flag::Z) {
                cpu.PC = val;
                4
            } else {
                3
            }
        },
        0xD2 => {
            let val = cpu.load_u16();
            if !cpu.get_flag(Flag::C) {
                cpu.PC = val;
                4
            } else {
                3
            }
        },
        0xDA => {
            let val = cpu.load_u16();
            if cpu.get_flag(Flag::C) {
                cpu.PC = val;
                4
            } else {
                3
            }
        },

        // JP HL
        0xE9 => {
            cpu.PC = *cpu.HL();
            1
        },

        // JR n
        0x18 => {
            let val = cpu.load_u8() as i8 as i16;
            cpu.PC = (val + cpu.PC as i16) as u16;
            3
        },

        // JR cc, n
        0x20 => {
            let val = cpu.load_u8() as i8 as i16;
            if !cpu.get_flag(Flag::Z) {
                cpu.PC = (val + cpu.PC as i16) as u16;
                3
            } else {
                2
            }
        },
        0x28 => {
            let val = cpu.load_u8() as i8 as i16;
            if cpu.get_flag(Flag::Z) {
                cpu.PC = (val + cpu.PC as i16) as u16;
                3
            } else {
                2
            }
        },
        0x30 => {
            let val = cpu.load_u8() as i8 as i16;
            if !cpu.get_flag(Flag::C) {
                cpu.PC = (val + cpu.PC as i16) as u16;
                3
            } else {
                2
            }
        },
        0x38 => {
            let val = cpu.load_u8() as i8 as i16;
            if cpu.get_flag(Flag::C) {
                cpu.PC = (val + cpu.PC as i16) as u16;
                3
            } else {
                2
            }
        },

        // CALL nn
        0xCD => {
            let val = cpu.load_u16();
            PUSH(cpu, cpu.PC);
            cpu.PC = val;
            6
        },

        // CALL cc, nn
        0xC4 => {
            let val = cpu.load_u16();
            if !cpu.get_flag(Flag::Z) {
                PUSH(cpu, cpu.PC);
                cpu.PC = val;
                6 
            } else {
                3
            }
        },
        0xCC => {
            let val = cpu.load_u16();
            if cpu.get_flag(Flag::Z) {
                PUSH(cpu, cpu.PC);
                cpu.PC = val;
                6 
            } else {
                3
            }
        },
        0xD4 => {
            let val = cpu.load_u16();
            if !cpu.get_flag(Flag::C) {
                PUSH(cpu, cpu.PC);
                cpu.PC = val;
                6 
            } else {
                3
            }
        },
        0xDC => {
            let val = cpu.load_u16();
            if cpu.get_flag(Flag::C) {
                PUSH(cpu, cpu.PC);
                cpu.PC = val;
                6 
            } else {
                3
            }
        },

        // RST xxH
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            PUSH(cpu, cpu.PC);
            cpu.PC = (inst - 0xC7) as u16;
            4
        },

        // RET
        0xC9 => {
            cpu.PC = POP(cpu);
            4
        },

        // RETI
        0xD9 => {
            cpu.PC = POP(cpu);
            cpu.IME = true;
            4
        }

        // RET cc
        0xC0 => {
            if !cpu.get_flag(Flag::Z) {
                cpu.PC = POP(cpu);
                5
            } else {
                2
            }
        },
        0xC8 => {
            if cpu.get_flag(Flag::Z) {
                cpu.PC = POP(cpu);
                5
            } else {
                2
            }
        },
        0xD0 => {
            if !cpu.get_flag(Flag::C) {
                cpu.PC = POP(cpu);
                5
            } else {
                2
            }
        },
        0xD8 => {
            if cpu.get_flag(Flag::C) {
                cpu.PC = POP(cpu);
                5
            } else {
                2
            }
        },

        // DI
        0xF3 => {
            cpu.IME = false;
            1
        },
        
        // EI
        0xFB => {
            cpu.EI = true;
            1
        },

        // NOP
        0x00 => { 1 },

        0xCB => {
            let cbinst = cpu.load_u8();
            match cbinst {
                // RLC
                0x00 ..= 0x07 => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);
                    val = RLC(cpu, val);
                    cpu.set_reg(cbinst, val);
                    if hl { 4 } else { 2 }
                },

                // RRC
                0x08 ..= 0x0F => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);
                    val = RRC(cpu, val);
                    cpu.set_reg(cbinst, val);
                    if hl { 4 } else { 2 }
                },

                // RL
                0x10 ..= 0x17 => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);
                    val = RL(cpu, val);
                    cpu.set_reg(cbinst, val);
                    if hl { 4 } else { 2 }
                },

                // RR
                0x18 ..= 0x1F => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);
                    val = RR(cpu, val);
                    cpu.set_reg(cbinst, val);
                    if hl { 4 } else { 2 }
                },

                // SLA
                0x20 ..= 0x27 => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);
                    cpu.set_flag(Flag::C, val&0x80 == 0x80);

                    val <<= 1;
                    cpu.set_reg(cbinst, val);

                    cpu.set_flag(Flag::Z, val == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, false);

                    if hl { 4 } else { 2 }
                },

                // SRA
                0x28 ..= 0x2F => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);
                    cpu.set_flag(Flag::C, val&0x1 == 0x1);

                    val = (val >> 1) | (val&0x80);
                    cpu.set_reg(cbinst, val);

                    cpu.set_flag(Flag::Z, val == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, false);

                    if hl { 4 } else { 2 }
                },

                // SWAP
                0x30 ..= 0x37 => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);

                    val = val.rotate_left(4);
                    cpu.set_reg(cbinst, val);

                    cpu.set_flag(Flag::Z, val == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, false);
                    cpu.set_flag(Flag::C, false);

                    if hl { 4 } else { 2 }
                },

                // SRL
                0x38 ..= 0x3F => {
                    let (mut val, hl) = cpu.get_val_reg(cbinst);
                    cpu.set_flag(Flag::C, val&0x1 == 0x1);

                    val >>= 1;
                    cpu.set_reg(cbinst, val);

                    cpu.set_flag(Flag::Z, val == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, false);

                    if hl { 4 } else { 2 }
                },

                // BIT
                0x40 ..= 0x7F => {
                    let (val, hl) = cpu.get_val_reg(cbinst);
                    let bit: u8 = match (cbinst - 0x40) >> 3 {
                        0 => 1,
                        1 => 2,
                        2 => 4,
                        3 => 8,
                        4 => 16,
                        5 => 32,
                        6 => 64,
                        7 => 128,
                        _ => panic!()
                    };

                    cpu.set_flag(Flag::Z, val&bit == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, true);

                    if hl { 3 } else { 2 }
                },

                // RES
                0x80 ..= 0xBF => {
                    let (val, hl) = cpu.get_val_reg(cbinst);
                    let bit: u8 = match (cbinst - 0x80) >> 3 {
                        0 => 1,
                        1 => 2,
                        2 => 4,
                        3 => 8,
                        4 => 16,
                        5 => 32,
                        6 => 64,
                        7 => 128,
                        _ => panic!()
                    };

                    cpu.set_reg(cbinst, val & !bit);

                    if hl { 4 } else { 2 }
                },

                // SET
                0xC0 ..= 0xFF => {
                    let (val, hl) = cpu.get_val_reg(cbinst);
                    let bit: u8 = match (cbinst - 0xC0) >> 3 {
                        0 => 1,
                        1 => 2,
                        2 => 4,
                        3 => 8,
                        4 => 16,
                        5 => 32,
                        6 => 64,
                        7 => 128,
                        _ => panic!()
                    };

                    cpu.set_reg(cbinst, val | bit);

                    if hl { 4 } else { 2 }
                }
            }
        }

        _ => {
            panic!("0x{:x} not implemented (at 0x{:x} PC)", inst, cpu.PC)
        }
    }
}
