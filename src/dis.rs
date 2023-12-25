use crate::debugger::Debugger;

impl Debugger {
    pub fn dis(&mut self, mem: &[u8]) -> u8 {
        self.dis_line.clear();
        let inst = mem[0];
        match inst {
            0x00 => {
                self.dis_line.push_str("BRK");
                1
            }
            0x01 => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x05 => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x06 => {
                self.dis_line.push_str("ASL");
                self.read_operand(mem) + 1
            }
            0x08 => {
                self.dis_line.push_str("PHP");
                1
            }
            0x09 => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x0a => {
                self.dis_line.push_str("ASL");
                1
            }
            0x0d => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x0e => {
                self.dis_line.push_str("ASL");
                self.read_operand(mem) + 1
            }

            0x10 => {
                self.dis_line.push_str("BPL");
                self.read_operand(mem) + 1
            }
            0x11 => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x15 => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x16 => {
                self.dis_line.push_str("ASL");
                self.read_operand(mem) + 1
            }
            0x18 => {
                self.dis_line.push_str("CLC");
                1
            }
            0x19 => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x1d => {
                self.dis_line.push_str("ORA");
                self.read_operand(mem) + 1
            }
            0x1e => {
                self.dis_line.push_str("ASL");
                self.read_operand(mem) + 1
            }

            0x20 => {
                self.dis_line.push_str("JSR");
                self.read_operand(mem) + 1
            }
            0x21 => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x24 => {
                self.dis_line.push_str("BIT");
                self.read_operand(mem) + 1
            }
            0x25 => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x26 => {
                self.dis_line.push_str("ROL");
                self.read_operand(mem) + 1
            }
            0x28 => {
                self.dis_line.push_str("PLP");
                1
            }
            0x29 => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x2a => {
                self.dis_line.push_str("ROL");
                1
            }
            0x2c => {
                self.dis_line.push_str("BIT");
                self.read_operand(mem) + 1
            }
            0x2d => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x2e => {
                self.dis_line.push_str("ROL");
                self.read_operand(mem) + 1
            }

            0x30 => {
                self.dis_line.push_str("BMI");
                self.read_operand(mem) + 1
            }
            0x31 => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x35 => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x36 => {
                self.dis_line.push_str("ROL");
                self.read_operand(mem) + 1
            }
            0x38 => {
                self.dis_line.push_str("SEC");
                1
            }
            0x39 => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x3d => {
                self.dis_line.push_str("AND");
                self.read_operand(mem) + 1
            }
            0x3e => {
                self.dis_line.push_str("ROL");
                self.read_operand(mem) + 1
            }

            0x40 => {
                self.dis_line.push_str("RTI");
                1
            }
            0x41 => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x45 => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x46 => {
                self.dis_line.push_str("LSR");
                self.read_operand(mem) + 1
            }
            0x48 => {
                self.dis_line.push_str("PHA");
                1
            }
            0x49 => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x4a => {
                self.dis_line.push_str("LSR");
                1
            }
            0x4c => {
                self.dis_line.push_str("JMP");
                self.read_operand(mem) + 1
            }
            0x4d => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x4e => {
                self.dis_line.push_str("LSR");
                self.read_operand(mem) + 1
            }

            0x50 => {
                self.dis_line.push_str("BVC");
                self.read_operand(mem) + 1
            }

            0x51 => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x55 => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x56 => {
                self.dis_line.push_str("LSR");
                self.read_operand(mem) + 1
            }
            0x58 => {
                self.dis_line.push_str("CLI");
                1
            }
            0x59 => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x5d => {
                self.dis_line.push_str("EOR");
                self.read_operand(mem) + 1
            }
            0x5e => {
                self.dis_line.push_str("LSR");
                self.read_operand(mem) + 1
            }

            0x60 => {
                self.dis_line.push_str("RTS");
                1
            }
            0x61 => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x65 => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x66 => {
                self.dis_line.push_str("ROR");
                self.read_operand(mem) + 1
            }
            0x68 => {
                self.dis_line.push_str("PLA");
                1
            }
            0x69 => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x6a => {
                self.dis_line.push_str("ROR");
                1
            }
            0x6c => {
                self.dis_line.push_str("JMP");
                self.read_operand(mem) + 1
            }
            0x6d => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x6e => {
                self.dis_line.push_str("ROR");
                self.read_operand(mem) + 1
            }

            0x70 => {
                self.dis_line.push_str("BVS");
                self.read_operand(mem) + 1
            }
            0x71 => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x75 => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x76 => {
                self.dis_line.push_str("ROR");
                self.read_operand(mem) + 1
            }
            0x78 => {
                self.dis_line.push_str("SEI");
                1
            }
            0x79 => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x7d => {
                self.dis_line.push_str("ADC");
                self.read_operand(mem) + 1
            }
            0x7e => {
                self.dis_line.push_str("ROR");
                self.read_operand(mem) + 1
            }

            0x81 => {
                self.dis_line.push_str("STA");
                self.read_operand(mem) + 1
            }
            0x84 => {
                self.dis_line.push_str("STY");
                self.read_operand(mem) + 1
            }
            0x85 => {
                self.dis_line.push_str("STA");
                self.read_operand(mem) + 1
            }
            0x86 => {
                self.dis_line.push_str("STX");
                self.read_operand(mem) + 1
            }
            0x88 => {
                self.dis_line.push_str("DEY");
                1
            }
            0x8a => {
                self.dis_line.push_str("TXA");
                1
            }
            0x8c => {
                self.dis_line.push_str("STY");
                self.read_operand(mem) + 1
            }
            0x8d => {
                self.dis_line.push_str("STA");
                self.read_operand(mem) + 1
            }
            0x8e => {
                self.dis_line.push_str("STX");
                self.read_operand(mem) + 1
            }

            0x90 => {
                self.dis_line.push_str("BCC");
                self.read_operand(mem) + 1
            }
            0x91 => {
                self.dis_line.push_str("STA");
                self.read_operand(mem) + 1
            }
            0x94 => {
                self.dis_line.push_str("STY");
                self.read_operand(mem) + 1
            }
            0x95 => {
                self.dis_line.push_str("STA");
                self.read_operand(mem) + 1
            }
            0x96 => {
                self.dis_line.push_str("STX");
                self.read_operand(mem) + 1
            }
            0x98 => {
                self.dis_line.push_str("TYA");
                1
            }
            0x99 => {
                self.dis_line.push_str("STA");
                self.read_operand(mem) + 1
            }
            0x9a => {
                self.dis_line.push_str("TXS");
                1
            }
            0x9d => {
                self.dis_line.push_str("STA");
                self.read_operand(mem) + 1
            }

            0xa0 => {
                self.dis_line.push_str("LDY");
                self.read_operand(mem) + 1
            }
            0xa1 => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xa2 => {
                self.dis_line.push_str("LDX");
                self.read_operand(mem) + 1
            }
            0xa4 => {
                self.dis_line.push_str("LDY");
                self.read_operand(mem) + 1
            }
            0xa5 => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xa6 => {
                self.dis_line.push_str("LDX");
                self.read_operand(mem) + 1
            }
            0xa8 => {
                self.dis_line.push_str("TAY");
                1
            }
            0xa9 => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xaa => {
                self.dis_line.push_str("TAX");
                1
            }
            0xac => {
                self.dis_line.push_str("LDY");
                self.read_operand(mem) + 1
            }
            0xad => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xae => {
                self.dis_line.push_str("LDX");
                self.read_operand(mem) + 1
            }

            0xb0 => {
                self.dis_line.push_str("BCS");
                self.read_operand(mem) + 1
            }
            0xb1 => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xb4 => {
                self.dis_line.push_str("LDY");
                self.read_operand(mem) + 1
            }
            0xb5 => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xb6 => {
                self.dis_line.push_str("LDX");
                self.read_operand(mem) + 1
            }

            0xb8 => {
                self.dis_line.push_str("CLV");
                1
            }
            0xb9 => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xba => {
                self.dis_line.push_str("TSX");
                1
            }
            0xbc => {
                self.dis_line.push_str("LDY");
                self.read_operand(mem) + 1
            }
            0xbd => {
                self.dis_line.push_str("LDA");
                self.read_operand(mem) + 1
            }
            0xbe => {
                self.dis_line.push_str("LDX");
                self.read_operand(mem) + 1
            }

            0xc0 => {
                self.dis_line.push_str("CPY");
                self.read_operand(mem) + 1
            }
            0xc1 => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xc4 => {
                self.dis_line.push_str("CPY");
                self.read_operand(mem) + 1
            }
            0xc5 => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xc6 => {
                self.dis_line.push_str("DEC");
                self.read_operand(mem) + 1
            }
            0xc8 => {
                self.dis_line.push_str("INY");
                1
            }
            0xc9 => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xca => {
                self.dis_line.push_str("DEX");
                1
            }
            0xcc => {
                self.dis_line.push_str("CPY");
                self.read_operand(mem) + 1
            }
            0xcd => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xce => {
                self.dis_line.push_str("DEC");
                self.read_operand(mem) + 1
            }

            0xd0 => {
                self.dis_line.push_str("BNE");
                self.read_operand(mem) + 1
            }
            0xd1 => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xd5 => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xd6 => {
                self.dis_line.push_str("DEC");
                self.read_operand(mem) + 1
            }
            0xd8 => {
                self.dis_line.push_str("CLD");
                1
            }
            0xd9 => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xdd => {
                self.dis_line.push_str("CMP");
                self.read_operand(mem) + 1
            }
            0xde => {
                self.dis_line.push_str("DEC");
                self.read_operand(mem) + 1
            }

            0xe0 => {
                self.dis_line.push_str("CPX");
                self.read_operand(mem) + 1
            }
            0xe1 => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xe4 => {
                self.dis_line.push_str("CPX");
                self.read_operand(mem) + 1
            }
            0xe5 => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xe6 => {
                self.dis_line.push_str("INC");
                self.read_operand(mem) + 1
            }
            0xe8 => {
                self.dis_line.push_str("INX");
                1
            }
            0xe9 => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xea => {
                self.dis_line.push_str("NOP");
                1
            }
            0xec => {
                self.dis_line.push_str("CPX");
                self.read_operand(mem) + 1
            }
            0xed => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xee => {
                self.dis_line.push_str("INC");
                self.read_operand(mem) + 1
            }

            0xf0 => {
                self.dis_line.push_str("BEQ");
                self.read_operand(mem) + 1
            }
            0xf1 => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xf5 => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xf6 => {
                self.dis_line.push_str("INC");
                self.read_operand(mem) + 1
            }
            0xf8 => {
                self.dis_line.push_str("SED");
                1
            }
            0xf9 => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xfd => {
                self.dis_line.push_str("SBC");
                self.read_operand(mem) + 1
            }
            0xfe => {
                self.dis_line.push_str("INC");
                self.read_operand(mem) + 1
            }
            _ => {
                self.dis_line
                    .push_str(&format!("Unknown instruction {:02X}", inst));
                1 // a guess
            }
        }
    }

    pub(crate) fn read_operand(&mut self, mem: &[u8]) -> u8 {
        let inst = mem[0];
        match inst {
            0x0a | 0x2a | 0x4a | 0x6a => {
                // accumulator
                self.dis_line.push_str("A");
                return 0;
            }
            0xA2 | 0xa0 | 0xc0 | 0xe0 => {
                // immediate
                let immed = mem[1];
                self.dis_line.push_str(&format!("#${:02X} ", immed));
                return 1;
            }
            _ if inst & 0b00011100 == 0b00001000 => {
                // immediate
                let immed = mem[1];
                self.dis_line.push_str(&format!("#${:02X} ", immed));
                return 1;
            }
            _ => {
                return self.operand_addr(mem);
            }
        }
    }

    fn operand_addr(&mut self, mem: &[u8]) -> u8 {
        // calculate the address of operand plus pc delta
        let inst = mem[0];
        let operand = match inst & 0b00011100 {
            0b000_001_00 => {
                // zero page
                self.dis_line.push_str(&format!("${:02X} ", mem[1]));
                1
            }
            0b000_011_00 => {
                // absolute
                let lo = mem[1] as u16;
                let hi = mem[2] as u16;
                self.dis_line.push_str(&format!("${:02X}{:02X}", hi, lo));
                2
            }
            0b000_101_00 => {
                // zpg, x -- except
                if inst == 0x96 || inst == 0xb6 {
                    self.zpg_y(mem)
                } else {
                    self.zpg_x(mem)
                }
            }
            0b000_111_00 => {
                // abs,x -- except
                if inst == 0xbe {
                    self.abs_y(mem)
                } else {
                    self.abs_x(mem)
                }
            }
            0b000_110_00 => {
                // abs,y
                self.abs_y(mem)
            }
            0b000_100_00 => {
                // (ind),y
                let zpaddr = mem[1] as u16;

                //let lo = self.read(zpaddr) as u16;
                //let hi = self.read(zpaddr + 1) as u16;
                self.dis_line.push_str(&format!("(${:02X}),Y", zpaddr));
                1
            }
            0b000_000_00 => {
                // (ind,x)
                let zpaddr = mem[1] as u16;

                self.dis_line.push_str(&format!("(${:02X},X)", zpaddr));
                1
            }
            _ => panic!("Unknown addr format: {:02X}", inst),
        };
        operand
    }
    fn abs_y(&mut self, mem: &[u8]) -> u8 {
        let lo = mem[1] as u16;
        let hi = mem[2] as u16;

        self.dis_line.push_str(&format!("${:02X}{:02X},Y", hi, lo));
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
        let zpaddr = mem[1] as u8;
        self.dis_line.push_str(&format!("${:02X},X", zpaddr as u8));
        1
    }
    fn zpg_y(&mut self, mem: &[u8]) -> u8 {
        // zpg, y
        let zpaddr = mem[1] as u8;

        self.dis_line.push_str(&format!("${:02X},Y ", zpaddr as u8));
        1
    }
}
#[test  ]
fn test_dis() {
    let mut dbg = Debugger::new();
    let mem = vec![0x00, 0x01, 0x02];
    let len = dbg.dis(&mem);
    assert_eq!(len, 1);
    assert_eq!(dbg.dis_line, "BRK");
}
#[test  ]
fn test_dis_ora() {
    let mut dbg = Debugger::new();
    let mem = vec![0x01, 0x01, 0x02];
    let len = dbg.dis(&mem);
    assert_eq!(len, 2);
    assert_eq!(dbg.dis_line, "ORA #${:02X} ");
}
