/*
Disassembler portion of db65. Placed in a separate file for clarity

liberally copied from pm100/v65
*/

use crate::debugger::core::Debugger;

impl Debugger {
    // reads one instruction and loads its interpretation into
    // dis_line
    // caller provides enough ram to cover the whole instruction
    pub fn dis(&mut self, mem: &[u8], inst_addr: u16) -> u8 {
        self.dis_line.clear();
        let inst = mem[0];
        match inst {
            0x00 => {
                self.dis_line.push_str("brk   ");
                1
            }
            0x01 => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x05 => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x06 => {
                self.dis_line.push_str("asl   ");
                self.read_operand(mem) + 1
            }
            0x08 => {
                self.dis_line.push_str("php   ");
                1
            }
            0x09 => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x0a => {
                self.dis_line.push_str("asl   ");
                1
            }
            0x0d => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x0e => {
                self.dis_line.push_str("asl   ");
                self.read_operand(mem) + 1
            }

            0x10 => {
                self.dis_line.push_str("bpl   ");
                self.branch(mem, inst_addr) + 1
            }
            0x11 => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x15 => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x16 => {
                self.dis_line.push_str("asl   ");
                self.read_operand(mem) + 1
            }
            0x18 => {
                self.dis_line.push_str("clc   ");
                1
            }
            0x19 => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x1d => {
                self.dis_line.push_str("ora   ");
                self.read_operand(mem) + 1
            }
            0x1e => {
                self.dis_line.push_str("asl   ");
                self.read_operand(mem) + 1
            }

            0x20 => {
                self.dis_line.push_str("jsr   ");
                self.read_operand(mem) + 1
            }
            0x21 => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x24 => {
                self.dis_line.push_str("bit   ");
                self.read_operand(mem) + 1
            }
            0x25 => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x26 => {
                self.dis_line.push_str("rol   ");
                self.read_operand(mem) + 1
            }
            0x28 => {
                self.dis_line.push_str("plp   ");
                1
            }
            0x29 => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x2a => {
                self.dis_line.push_str("rol   ");
                1
            }
            0x2c => {
                self.dis_line.push_str("bit   ");
                self.read_operand(mem) + 1
            }
            0x2d => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x2e => {
                self.dis_line.push_str("rol   ");
                self.read_operand(mem) + 1
            }

            0x30 => {
                self.dis_line.push_str("bmi   ");
                self.branch(mem, inst_addr) + 1
            }
            0x31 => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x35 => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x36 => {
                self.dis_line.push_str("rol   ");
                self.read_operand(mem) + 1
            }
            0x38 => {
                self.dis_line.push_str("sec   ");
                1
            }
            0x39 => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x3d => {
                self.dis_line.push_str("and   ");
                self.read_operand(mem) + 1
            }
            0x3e => {
                self.dis_line.push_str("rol   ");
                self.read_operand(mem) + 1
            }

            0x40 => {
                self.dis_line.push_str("rti   ");
                1
            }
            0x41 => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x45 => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x46 => {
                self.dis_line.push_str("lsr   ");
                self.read_operand(mem) + 1
            }
            0x48 => {
                self.dis_line.push_str("pha   ");
                1
            }
            0x49 => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x4a => {
                self.dis_line.push_str("lsr   ");
                1
            }
            0x4c => {
                self.dis_line.push_str("jmp   ");
                self.read_operand(mem) + 1
            }
            0x4d => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x4e => {
                self.dis_line.push_str("lsr   ");
                self.read_operand(mem) + 1
            }

            0x50 => {
                self.dis_line.push_str("bvc   ");
                self.branch(mem, inst_addr) + 1
            }

            0x51 => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x55 => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x56 => {
                self.dis_line.push_str("lsr   ");
                self.read_operand(mem) + 1
            }
            0x58 => {
                self.dis_line.push_str("cli   ");
                1
            }
            0x59 => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x5d => {
                self.dis_line.push_str("eor   ");
                self.read_operand(mem) + 1
            }
            0x5e => {
                self.dis_line.push_str("lsr   ");
                self.read_operand(mem) + 1
            }

            0x60 => {
                self.dis_line.push_str("rts   ");
                1
            }
            0x61 => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x65 => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x66 => {
                self.dis_line.push_str("ror   ");
                self.read_operand(mem) + 1
            }
            0x68 => {
                self.dis_line.push_str("pla   ");
                1
            }
            0x69 => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x6a => {
                self.dis_line.push_str("ror   ");
                1
            }
            0x6c => {
                self.dis_line.push_str("jmp   ");
                self.read_operand(mem) + 1
            }
            0x6d => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x6e => {
                self.dis_line.push_str("ror   ");
                self.read_operand(mem) + 1
            }

            0x70 => {
                self.dis_line.push_str("bvs   ");
                self.branch(mem, inst_addr) + 1
            }
            0x71 => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x75 => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x76 => {
                self.dis_line.push_str("ror   ");
                self.read_operand(mem) + 1
            }
            0x78 => {
                self.dis_line.push_str("sei   ");
                1
            }
            0x79 => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x7d => {
                self.dis_line.push_str("adc   ");
                self.read_operand(mem) + 1
            }
            0x7e => {
                self.dis_line.push_str("ror   ");
                self.read_operand(mem) + 1
            }

            0x81 => {
                self.dis_line.push_str("sta   ");
                self.read_operand(mem) + 1
            }
            0x84 => {
                self.dis_line.push_str("sty   ");
                self.read_operand(mem) + 1
            }
            0x85 => {
                self.dis_line.push_str("sta   ");
                self.read_operand(mem) + 1
            }
            0x86 => {
                self.dis_line.push_str("stx   ");
                self.read_operand(mem) + 1
            }
            0x88 => {
                self.dis_line.push_str("dey   ");
                1
            }
            0x8a => {
                self.dis_line.push_str("txa   ");
                1
            }
            0x8c => {
                self.dis_line.push_str("sty   ");
                self.read_operand(mem) + 1
            }
            0x8d => {
                self.dis_line.push_str("sta   ");
                self.read_operand(mem) + 1
            }
            0x8e => {
                self.dis_line.push_str("stx   ");
                self.read_operand(mem) + 1
            }

            0x90 => {
                self.dis_line.push_str("bcc   ");
                self.branch(mem, inst_addr) + 1
            }
            0x91 => {
                self.dis_line.push_str("sta   ");
                self.read_operand(mem) + 1
            }
            0x94 => {
                self.dis_line.push_str("sty   ");
                self.read_operand(mem) + 1
            }
            0x95 => {
                self.dis_line.push_str("sta   ");
                self.read_operand(mem) + 1
            }
            0x96 => {
                self.dis_line.push_str("stx   ");
                self.read_operand(mem) + 1
            }
            0x98 => {
                self.dis_line.push_str("tya   ");
                1
            }
            0x99 => {
                self.dis_line.push_str("sta   ");
                self.read_operand(mem) + 1
            }
            0x9a => {
                self.dis_line.push_str("txs   ");
                1
            }
            0x9d => {
                self.dis_line.push_str("sta   ");
                self.read_operand(mem) + 1
            }

            0xa0 => {
                self.dis_line.push_str("ldy   ");
                self.read_operand(mem) + 1
            }
            0xa1 => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xa2 => {
                self.dis_line.push_str("ldx   ");
                self.read_operand(mem) + 1
            }
            0xa4 => {
                self.dis_line.push_str("ldy   ");
                self.read_operand(mem) + 1
            }
            0xa5 => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xa6 => {
                self.dis_line.push_str("ldx   ");
                self.read_operand(mem) + 1
            }
            0xa8 => {
                self.dis_line.push_str("tay   ");
                1
            }
            0xa9 => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xaa => {
                self.dis_line.push_str("tax   ");
                1
            }
            0xac => {
                self.dis_line.push_str("ldy   ");
                self.read_operand(mem) + 1
            }
            0xad => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xae => {
                self.dis_line.push_str("ldx   ");
                self.read_operand(mem) + 1
            }

            0xb0 => {
                self.dis_line.push_str("bcs   ");
                self.branch(mem, inst_addr) + 1
            }
            0xb1 => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xb4 => {
                self.dis_line.push_str("ldy   ");
                self.read_operand(mem) + 1
            }
            0xb5 => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xb6 => {
                self.dis_line.push_str("ldx   ");
                self.read_operand(mem) + 1
            }

            0xb8 => {
                self.dis_line.push_str("clv   ");
                1
            }
            0xb9 => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xba => {
                self.dis_line.push_str("tsx   ");
                1
            }
            0xbc => {
                self.dis_line.push_str("ldy   ");
                self.read_operand(mem) + 1
            }
            0xbd => {
                self.dis_line.push_str("lda   ");
                self.read_operand(mem) + 1
            }
            0xbe => {
                self.dis_line.push_str("ldx   ");
                self.read_operand(mem) + 1
            }

            0xc0 => {
                self.dis_line.push_str("cpy   ");
                self.read_operand(mem) + 1
            }
            0xc1 => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xc4 => {
                self.dis_line.push_str("cpy   ");
                self.read_operand(mem) + 1
            }
            0xc5 => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xc6 => {
                self.dis_line.push_str("dec   ");
                self.read_operand(mem) + 1
            }
            0xc8 => {
                self.dis_line.push_str("iny   ");
                1
            }
            0xc9 => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xca => {
                self.dis_line.push_str("dex   ");
                1
            }
            0xcc => {
                self.dis_line.push_str("cpy   ");
                self.read_operand(mem) + 1
            }
            0xcd => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xce => {
                self.dis_line.push_str("dec   ");
                self.read_operand(mem) + 1
            }

            0xd0 => {
                self.dis_line.push_str("bne   ");
                self.branch(mem, inst_addr) + 1
            }
            0xd1 => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xd5 => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xd6 => {
                self.dis_line.push_str("dec   ");
                self.read_operand(mem) + 1
            }
            0xd8 => {
                self.dis_line.push_str("cld   ");
                1
            }
            0xd9 => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xdd => {
                self.dis_line.push_str("cmp   ");
                self.read_operand(mem) + 1
            }
            0xde => {
                self.dis_line.push_str("dec   ");
                self.read_operand(mem) + 1
            }

            0xe0 => {
                self.dis_line.push_str("cpx   ");
                self.read_operand(mem) + 1
            }
            0xe1 => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xe4 => {
                self.dis_line.push_str("cpx   ");
                self.read_operand(mem) + 1
            }
            0xe5 => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xe6 => {
                self.dis_line.push_str("inc   ");
                self.read_operand(mem) + 1
            }
            0xe8 => {
                self.dis_line.push_str("inx   ");
                1
            }
            0xe9 => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xea => {
                self.dis_line.push_str("nop   ");
                1
            }
            0xec => {
                self.dis_line.push_str("cpx   ");
                self.read_operand(mem) + 1
            }
            0xed => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xee => {
                self.dis_line.push_str("inc   ");
                self.read_operand(mem) + 1
            }

            0xf0 => {
                self.dis_line.push_str("beq   ");
                self.branch(mem, inst_addr) + 1
            }
            0xf1 => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xf5 => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xf6 => {
                self.dis_line.push_str("inc   ");
                self.read_operand(mem) + 1
            }
            0xf8 => {
                self.dis_line.push_str("sed   ");
                1
            }
            0xf9 => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xfd => {
                self.dis_line.push_str("sbc   ");
                self.read_operand(mem) + 1
            }
            0xfe => {
                self.dis_line.push_str("inc   ");
                self.read_operand(mem) + 1
            }
            _ => {
                self.dis_line
                    .push_str(&format!("Unknown instruction {:02X}", inst));
                1 // a guess
            }
        }
    }
    // disassembles to operand of the instruction
    pub(crate) fn read_operand(&mut self, mem: &[u8]) -> u8 {
        let inst = mem[0];
        match inst {
            // this code deals with the non memory ones
            0x0a | 0x2a | 0x4a | 0x6a => {
                // accumulator
                self.dis_line.push('A');
                0
            }
            0xA2 | 0xa0 | 0xc0 | 0xe0 => {
                // immediate
                let immed = mem[1];
                self.dis_line.push_str(&format!("#${:02X} ", immed));
                1
            }
            _ if inst & 0b00011100 == 0b00001000 => {
                // immediate
                let immed = mem[1];
                self.dis_line.push_str(&format!("#${:02X} ", immed));
                1
            }
            _ => {
                // otherwise deal with memory
                self.operand_addr(mem)
            }
        }
    }
    fn branch(&mut self, mem: &[u8], mut inst_addr: u16) -> u8 {
        // special case for branches.
        // current pc is passed in so that destination absolute address can be calculated
        let offset = mem[1] as i8;
        inst_addr += 2;
        let target = inst_addr.wrapping_add_signed(offset as i16);
        let sym = self.symbol_lookup(target).unwrap();
        self.dis_line.push_str(&sym.to_string());
        1
    }
    fn operand_addr(&mut self, mem: &[u8]) -> u8 {
        // disassemble the address of operand plus pc delta
        let inst = mem[0];
        if inst == 0x20 {
            //jsr
            let lo = mem[1] as u16;
            let hi = mem[2] as u16;
            let addr = (hi << 8) | lo;
            let sym = self.symbol_lookup(addr).unwrap();
            self.dis_line.push_str(&sym);
            return 2;
        }

        match inst & 0b00011100 {
            0b0000_0100 => {
                // zero page
                let lo = mem[1];
                let sym = self.zp_symbol_lookup(lo).unwrap();
                self.dis_line.push_str(&sym);
                1
            }
            0b0000_1100 => {
                // absolute
                let lo = mem[1] as u16;
                let hi = mem[2] as u16;
                let addr = (hi << 8) | lo;
                let sym = self.symbol_lookup(addr).unwrap();
                self.dis_line.push_str(&sym);
                2
            }
            0b0001_0100 => {
                // zpg, x -- except
                if inst == 0x96 || inst == 0xb6 {
                    self.zpg_y(mem)
                } else {
                    self.zpg_x(mem)
                }
            }
            0b0001_1100 => {
                // abs,x -- except
                if inst == 0xbe {
                    self.abs_y(mem)
                } else {
                    self.abs_x(mem)
                }
            }
            0b0001_1000 => {
                // abs,y
                self.abs_y(mem)
            }
            0b0001_0000 => {
                // (ind),y
                let zpaddr = mem[1];
                let sym = self.zp_symbol_lookup(zpaddr).unwrap();

                self.dis_line.push_str(&format!("({}),Y", sym));
                1
            }
            0b0000_0000 => {
                // (ind,x)
                let zpaddr = mem[1];
                let sym = self.zp_symbol_lookup(zpaddr).unwrap();
                self.dis_line.push_str(&format!("({},X)", sym));
                1
            }
            _ => panic!("Unknown addr format: {:02X}", inst),
        }
    }
    fn abs_y(&mut self, mem: &[u8]) -> u8 {
        let lo = mem[1] as u16;
        let hi = mem[2] as u16;
        let addr = (hi << 8) | lo;
        let sym = self.symbol_lookup(addr).unwrap();
        self.dis_line.push_str(&format!("{},Y", sym));
        2
    }
    fn abs_x(&mut self, mem: &[u8]) -> u8 {
        let lo = mem[1] as u16;
        let hi = mem[2] as u16;
        self.dis_line.push_str(&format!("${:02X}{:02X},X", hi, lo));
        2
    }
    fn zpg_x(&mut self, mem: &[u8]) -> u8 {
        // zpg, x
        let zpaddr = mem[1];
        self.dis_line.push_str(&format!("${:02X},X", { zpaddr }));
        1
    }
    fn zpg_y(&mut self, mem: &[u8]) -> u8 {
        // zpg, y
        let zpaddr = mem[1];

        self.dis_line.push_str(&format!("${:02X},Y ", { zpaddr }));
        1
    }
}
#[test]
fn test_dis() {
    let mut dbg = Debugger::new();
    let mem = vec![0x00, 0x01, 0x02];
    let len = dbg.dis(&mem, 0);
    assert_eq!(len, 1);
    assert_eq!(dbg.dis_line, "brk   ");

    let mem = vec![0x01, 0x01, 0x02];
    let len = dbg.dis(&mem, 0);
    assert_eq!(len, 2);
    assert_eq!(dbg.dis_line, "ora   ($01,X)");

    let mem = vec![0x20, 0x01, 0x02];
    let len = dbg.dis(&mem, 0);
    assert_eq!(len, 3);
    assert_eq!(dbg.dis_line, "jsr   $0201");
}
