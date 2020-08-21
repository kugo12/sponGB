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
            cpu.memory.write(addr, val);
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


fn PUSH(cpu: &mut CPU, val: u16) {
    cpu.memory.write(cpu.SP - 1, (val >> 8) as u8);
    cpu.memory.write(cpu.SP - 2, val as u8);
    cpu.SP -= 2;
}


fn POP(cpu: &mut CPU) -> u16 {
    let lsb = cpu.memory.read(cpu.SP) as u16;
    let msb = cpu.memory.read(cpu.SP + 1) as u16;
    cpu.SP += 2;
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
    cpu.set_flag(Flag::H, !hc);
    cpu.set_flag(Flag::C, !c);

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
    cpu.set_flag(Flag::H, !hc);
    cpu.set_flag(Flag::C, !c);

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
    let (val, more_cycles) = if instr == 0xEE {
        (cpu.load_u8(), true)
    } else {
        cpu.get_val_reg(instr)
    };

    let z = *cpu.A() == val;
    let c = *cpu.A() < val;
    let hc = *cpu.A()&0xF < val&0xF;

    cpu.set_flag(Flag::Z, z);
    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, !hc);
    cpu.set_flag(Flag::C, !c);

    match more_cycles {
        true => 2,
        false => 1
    }
}


fn INC(cpu: &mut CPU, inst: u8) -> u8 {
    let (h, z, cycles) = {
        if inst == 0x34 {
            let addr = *cpu.HL();
            let val = cpu.memory.read(addr);
            let tmp = val.wrapping_add(1);
            cpu.memory.write(addr, tmp);

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
            let val = cpu.memory.read(addr);
            let tmp = val.wrapping_sub(1);
            cpu.memory.write(addr, tmp);

            (val < 1, tmp == 0, 3)
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

            (tmp < 1, *val == 0, 1)
        }
    };
    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, !h);
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

    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, hc);
    cpu.set_flag(Flag::C, c);
}


fn DAA(cpu: &mut CPU) {
    let mut val = *cpu.A();

    if cpu.get_flag(Flag::N) {
        if cpu.get_flag(Flag::H) {
            val = val.wrapping_sub(0x6);
        }
        if cpu.get_flag(Flag::C) {
            val = val.wrapping_sub(0x60);
        }
    } else {
        if cpu.get_flag(Flag::H) || val&0xF > 0x9 {
            val = val.wrapping_add(0x6);
        }
        if cpu.get_flag(Flag::C) || val>0x9F {
            val = val.wrapping_add(0x60);
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

    cpu.set_flag(Flag::Z, val == 0);
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

    cpu.set_flag(Flag::Z, val == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, false);

    (val & 0b01111111) | c
}


pub fn execute(cpu: &mut CPU, inst: u8) -> u8 {
    match inst {
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
            cpu.memory.write(addr, val);
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
            cpu.memory.write(addr, val);
            2
        },
        0x12 => {
            let addr = *cpu.DE();
            let val = *cpu.A();
            cpu.memory.write(addr, val);
            2
        },
        0x22 => {
            let addr = *cpu.HL();
            let val = *cpu.A();
            cpu.memory.write(addr, val);
            *cpu.HL() += 1;
            2
        },
        0x32 => {
            let addr = *cpu.HL();
            let val = *cpu.A();
            cpu.memory.write(addr, val);
            *cpu.HL() -= 1;
            2
        },

        0x0A => {
            let addr = *cpu.BC();
            *cpu.A() = cpu.memory.read(addr);
            2
        },
        0x1A => {
            let addr = *cpu.DE();
            *cpu.A() = cpu.memory.read(addr);
            2
        },
        0x2A => {
            let addr = *cpu.HL();
            *cpu.A() = cpu.memory.read(addr);
            *cpu.HL() += 1;
            2
        },
        0x3A => {
            let addr = *cpu.HL();
            *cpu.A() = cpu.memory.read(addr);
            *cpu.HL() -= 1;
            2
        },

        0xE0 => {
            let addr = 0xFF00 + cpu.load_u8() as u16;
            let val = *cpu.A();
            cpu.memory.write(addr, val);
            3
        },
        0xF0 => {
            let addr = 0xFF00 + cpu.load_u8() as u16;
            *cpu.A() = cpu.memory.read(addr);
            3
        },

        0xE2 => {
            let addr = 0xFF00 + *cpu.C() as u16;
            let val = *cpu.A();
            cpu.memory.write(addr, val);
            2
        },
        0xF2 => {
            let addr = 0xFF00 + *cpu.C() as u16;
            *cpu.A() = cpu.memory.read(addr);
            2
        },

        0xEA => {
            let addr = cpu.load_u16();
            let val = *cpu.A();
            cpu.memory.write(addr, val);
            4
        },
        0xFA => {
            let addr = cpu.load_u16();
            *cpu.A() = cpu.memory.read(addr);
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
            *cpu.HL() = (sp + n) as u16;
            3
        },

        0x08 => {
            let addr = cpu.load_u16();
            cpu.memory.write(addr, cpu.SP as u8);
            cpu.memory.write(addr+1, (cpu.SP >> 8) as u8);
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
            *cpu.AF() = POP(cpu);
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
            ADD(cpu, inst)
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
            cpu.SP = (sp + n) as u16;
            4
        },

        // INC 16bit
        0x03 => {
            *cpu.BC() += 1;
            2
        },
        0x13 => {
            *cpu.DE() += 1;
            2
        },
        0x23 => {
            *cpu.HL() += 1;
            2
        },
        0x33 => {
            cpu.SP += 1;
            2
        }

        // DEC 16bit
        0x0B => {
            *cpu.BC() -= 1;
            2
        },
        0x1B => {
            *cpu.DE() -= 1;
            2
        },
        0x2B => {
            *cpu.HL() -= 1;
            2
        },
        0x3B => {
            cpu.SP -= 1;
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
            1
        },

        // RLA
        0x17 => {
            let mut val = *cpu.A();
            val = RL(cpu, val);
            *cpu.A() = val;
            1
        },

        // RRCA
        0x0F => {
            let mut val = *cpu.A();
            val = RRC(cpu, val);
            *cpu.A() = val;
            1
        },

        // RRA
        0x1F => {
            let mut val = *cpu.A();
            val = RR(cpu, val);
            *cpu.A() = val;
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
            println!("0x{:x} not implemented", inst);
            1
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::emulator::CPU;

    fn cpu_run(cpu: &mut CPU){
        loop {
            cpu.tick();
            if cpu.halt {
                break
            }
        }
    }

    fn cpu_init(mut opc: Vec<u8>) -> CPU {
        let mut c = CPU::new();
        c.PC = 0;
        opc.push(0x76);
        c.memory.cart.load_from_vec(opc);
        c
    }
    
    #[test]
    fn ld_8bit() {
        let v = vec![
            0x06, 0xFF, // LD B, 0xFF
            0x0E, 0xEE, // LD C, 0xEE
            0x16, 0xDD, // LD D, 0xDD
            0x1E, 0xCC, // LD E, 0xCC
            0x26, 0xCB, // LD H, 0xCB
            0x2E, 0xAA, // LD L, 0xAA
            0x36, 0x99, // LD (HL), 0x99   small note, address HL (0xCBAA) is internal ram
            0x3E, 0x88  // LD A, 0x88
        ];
        let mut cpu = cpu_init(v);
        cpu_run(&mut cpu);

        assert_eq!(*cpu.B(), 0xFF);
        assert_eq!(*cpu.C(), 0xEE);
        assert_eq!(*cpu.D(), 0xDD);
        assert_eq!(*cpu.E(), 0xCC);
        assert_eq!(*cpu.H(), 0xCB);
        assert_eq!(*cpu.L(), 0xAA);
        assert_eq!(*cpu.A(), 0x88);

        let addr = *cpu.HL();
        assert_eq!(cpu.memory.read(addr), 0x99);
    }
    
    #[test]
    fn _0x40() { let v = vec![0x06, 0xFF, 0x40]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x41() { let v = vec![0x0E, 0xFF, 0x41]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x42() { let v = vec![0x16, 0xFF, 0x42]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x43() { let v = vec![0x1E, 0xFF, 0x43]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x44() { let v = vec![0x26, 0xFF, 0x44]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x45() { let v = vec![0x2E, 0xFF, 0x45]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x46() { let v = vec![0x26, 0xC0, 0x36, 0xFF, 0x46]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x47() { let v = vec![0x3E, 0xFF, 0x47]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.B(), 0xFF); }
    #[test]
    fn _0x48() { let v = vec![0x06, 0xFF, 0x48]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x49() { let v = vec![0x0E, 0xFF, 0x49]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x4a() { let v = vec![0x16, 0xFF, 0x4a]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x4b() { let v = vec![0x1E, 0xFF, 0x4b]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x4c() { let v = vec![0x26, 0xFF, 0x4c]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x4d() { let v = vec![0x2E, 0xFF, 0x4d]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x4e() { let v = vec![0x26, 0xC0, 0x36, 0xFF, 0x4e]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x4f() { let v = vec![0x3E, 0xFF, 0x4f]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.C(), 0xFF); }
    #[test]
    fn _0x50() { let v = vec![0x06, 0xFF, 0x50]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x51() { let v = vec![0x0E, 0xFF, 0x51]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x52() { let v = vec![0x16, 0xFF, 0x52]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x53() { let v = vec![0x1E, 0xFF, 0x53]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x54() { let v = vec![0x26, 0xFF, 0x54]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x55() { let v = vec![0x2E, 0xFF, 0x55]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x56() { let v = vec![0x26, 0xC0, 0x36, 0xFF, 0x56]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x57() { let v = vec![0x3E, 0xFF, 0x57]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.D(), 0xFF); }
    #[test]
    fn _0x58() { let v = vec![0x06, 0xFF, 0x58]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x59() { let v = vec![0x0E, 0xFF, 0x59]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x5a() { let v = vec![0x16, 0xFF, 0x5a]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x5b() { let v = vec![0x1E, 0xFF, 0x5b]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x5c() { let v = vec![0x26, 0xFF, 0x5c]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x5d() { let v = vec![0x2E, 0xFF, 0x5d]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x5e() { let v = vec![0x26, 0xC0, 0x36, 0xFF, 0x5e]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x5f() { let v = vec![0x3E, 0xFF, 0x5f]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.E(), 0xFF); }
    #[test]
    fn _0x60() { let v = vec![0x06, 0xFF, 0x60]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x61() { let v = vec![0x0E, 0xFF, 0x61]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x62() { let v = vec![0x16, 0xFF, 0x62]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x63() { let v = vec![0x1E, 0xFF, 0x63]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x64() { let v = vec![0x26, 0xFF, 0x64]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x65() { let v = vec![0x2E, 0xFF, 0x65]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x66() { let v = vec![0x26, 0xC0, 0x36, 0xFF, 0x66]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x67() { let v = vec![0x3E, 0xFF, 0x67]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.H(), 0xFF); }
    #[test]
    fn _0x68() { let v = vec![0x06, 0xFF, 0x68]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x69() { let v = vec![0x0E, 0xFF, 0x69]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x6a() { let v = vec![0x16, 0xFF, 0x6a]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x6b() { let v = vec![0x1E, 0xFF, 0x6b]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x6c() { let v = vec![0x26, 0xFF, 0x6c]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x6d() { let v = vec![0x2E, 0xFF, 0x6d]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x6e() { let v = vec![0x26, 0xC0, 0x36, 0xFF, 0x6e]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x6f() { let v = vec![0x3E, 0xFF, 0x6f]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.L(), 0xFF); }
    #[test]
    fn _0x70() { let v = vec![0x26, 0xC0, 0x06, 0xFF, 0x70]; let mut cpu = cpu_init(v); cpu_run(&mut cpu); let addr = *cpu.HL(); assert_eq!(cpu.memory.read(addr), 0xFF); }
    #[test]
    fn _0x71() { let v = vec![0x26, 0xC0, 0x0E, 0xFF, 0x71]; let mut cpu = cpu_init(v); cpu_run(&mut cpu); let addr = *cpu.HL(); assert_eq!(cpu.memory.read(addr), 0xFF); }
    #[test]
    fn _0x72() { let v = vec![0x26, 0xC0, 0x16, 0xFF, 0x72]; let mut cpu = cpu_init(v); cpu_run(&mut cpu); let addr = *cpu.HL(); assert_eq!(cpu.memory.read(addr), 0xFF); }
    #[test]
    fn _0x73() { let v = vec![0x26, 0xC0, 0x1E, 0xFF, 0x73]; let mut cpu = cpu_init(v); cpu_run(&mut cpu); let addr = *cpu.HL(); assert_eq!(cpu.memory.read(addr), 0xFF); }
    #[test]
    fn _0x74() { let v = vec![0x26, 0xC0, 0x26, 0xFF, 0x74]; let mut cpu = cpu_init(v); cpu_run(&mut cpu); let addr = *cpu.HL(); assert_eq!(cpu.memory.read(addr), 0xFF); }
    #[test]
    fn _0x75() { let v = vec![0x26, 0xC0, 0x2E, 0xFF, 0x75]; let mut cpu = cpu_init(v); cpu_run(&mut cpu); let addr = *cpu.HL(); assert_eq!(cpu.memory.read(addr), 0xFF); }
    #[test]
    fn _0x77() { let v = vec![0x26, 0xC0, 0x3E, 0xFF, 0x77]; let mut cpu = cpu_init(v); cpu_run(&mut cpu); let addr = *cpu.HL(); assert_eq!(cpu.memory.read(addr), 0xFF); }
    #[test]
    fn _0x78() { let v = vec![0x06, 0xFF, 0x78]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
    #[test]
    fn _0x79() { let v = vec![0x0E, 0xFF, 0x79]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
    #[test]
    fn _0x7a() { let v = vec![0x16, 0xFF, 0x7a]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
    #[test]
    fn _0x7b() { let v = vec![0x1E, 0xFF, 0x7b]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
    #[test]
    fn _0x7c() { let v = vec![0x26, 0xFF, 0x7c]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
    #[test]
    fn _0x7d() { let v = vec![0x2E, 0xFF, 0x7d]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
    #[test]
    fn _0x7e() { let v = vec![0x26, 0xC0, 0x36, 0xFF, 0x7e]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
    #[test]
    fn _0x7f() { let v = vec![0x3E, 0xFF, 0x7f]; let mut cpu = cpu_init(v); cpu_run(&mut cpu);  assert_eq!(*cpu.A(), 0xFF); }
}