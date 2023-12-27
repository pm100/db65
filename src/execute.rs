use anyhow::{bail, Result};
#[derive(Debug, Clone)]
pub enum StopReason {
    BreakPoint(u16),
    Exit(u8),
    Count,
    Next,
    Bug(BugType),
}
#[derive(Debug, Clone)]
pub enum BugType {
    SpMismatch,
    Memcheck(u16),
    Stack,
    StackFrame,
    StackFrameJsr,
    StackFramePha,
    StackFramePhp,
}
use crate::{
    cpu::Sim,
    debugger::{BreakPoint, Debugger, FrameType, StackFrame},
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
                    let sp = Sim::read_sp();

                    let addr = lo as u16 | ((hi as u16) << 8);
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Jsr((addr, pc + 3, sp, 0)),
                    });
                }

                0x60 => {
                    // rts
                    let frame = self.stack_frames.pop().unwrap();
                    let sp = Sim::read_sp();
                    if self.enable_stack_check {
                        if let FrameType::Jsr((addr, ret_addr, fsp, _)) = frame.frame_type {
                            if sp + 2 != fsp {
                                break StopReason::Bug(BugType::SpMismatch);
                            }
                        }
                    }
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
            if self.enable_mem_check {
                if let Some(addr) = Sim::get_memcheck() {
                    Sim::clear_memcheck();
                    break StopReason::Bug(BugType::Memcheck(addr));
                }
            }
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
                break StopReason::BreakPoint(pc);
            }

            if let Some(exit_code) = Sim::exit_done() {
                break StopReason::Exit(exit_code);
            }
        };
        Ok(reason)
    }
}
