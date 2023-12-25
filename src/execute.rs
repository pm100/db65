use anyhow::{bail, Result};
pub enum StopReason {
    BreakPoint,
    Exit,
    Count,
}
use crate::{
    cpu::Sim,
    debugger::{Debugger, FrameType, StackFrame},
};
impl Debugger {
    pub fn execute(&mut self, mut count: u16) -> Result<StopReason> {
        let counting = count > 0;
        let reason = loop {
            let pc = Sim::read_pc();
            // is this a stack manipulation instruction?
            let inst = Sim::read_byte(pc);
            match inst {
                0x20 => {
                    // jsr
                    let lo = Sim::read_byte(pc + 1);
                    let hi = Sim::read_byte(pc + 2);
                    let addr = lo as u16 | ((hi as u16) << 8);
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Jsr((addr, pc + 1)),
                    });
                }

                0x60 => {
                    // rts
                    let frame = self.stack_frames.pop().unwrap();
                }
                0x68 => {
                    // pla
                    let frame = self.stack_frames.pop().unwrap();
                }
                0x48 => {
                    // pha
                    let ac = Sim::read_ac();
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Pha(ac),
                    });
                }

                0x28 => {
                    // plp
                    let frame = self.stack_frames.pop().unwrap();
                }
                0x08 => {
                    // php
                    let sr = Sim::read_sr();
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Php(sr),
                    });
                }
                0x40 => {
                    // rti
                    let frame = self.stack_frames.pop().unwrap();
                }
                _ => {}
            };
            self.ticks += Sim::execute_insn() as usize;
            let pc = Sim::read_pc();
            if counting {
                count -= 1;
                if count == 0 {
                    break StopReason::Count;
                }
            }
            //  did we hit a breakpoint?
            let pc = Sim::read_pc();
            if let Some(bp) = self.break_points.get(&pc) {
                // println!("breakpoint");
                if bp.temp {
                    self.break_points.remove(&pc);
                }
                break StopReason::BreakPoint;
            }

            if Sim::exit_done() {
                break StopReason::Exit;
            }
        };
        Ok(reason)
    }
}
