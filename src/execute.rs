/*
core run code of the debugger.

Its responsible for running the code . It detects
- breakpoints
- exit
- next/step
- bugs

It runs until it stops. It the returns a StopReason
*/
use anyhow::Result;
#[derive(Debug, Clone)]
pub enum StopReason {
    BreakPoint(u16),
    WatchPoint(u16),
    Exit(u8),
    Count,
    Next,
    Bug(BugType),
}
#[derive(Debug, Clone)]
pub enum BugType {
    SpMismatch,
    Memcheck(u16),
}
use crate::{
    cpu::Cpu,
    debugger::{Debugger, FrameType, StackFrame, WatchType},
};
impl Debugger {
    pub fn execute(&mut self, mut count: u16) -> Result<StopReason> {
        let counting = count > 0;
        let reason = 'main_loop: loop {
            let pc = Cpu::read_pc();
            /*
            Stack tracking code
            if we hit a jsr, we push the return address and the stack pointer
            onto our own tracking stack. If we hit a rts, we pop the frame

            Also tracks push and pulls

            Does not deal with interrupts since sim65 does not support them

            Includes stack balance check logic
            */
            let inst = Cpu::read_byte(pc);
            match inst {
                0x20 => {
                    // jsr
                    let lo = Cpu::read_byte(pc + 1);
                    let hi = Cpu::read_byte(pc + 2);
                    let sp = Cpu::read_sp();

                    let addr = lo as u16 | ((hi as u16) << 8);
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Jsr((addr, pc + 3, sp, 0)),
                    });
                }

                0x60 => {
                    // rts
                    let frame = self.stack_frames.pop().unwrap();
                    let sp = Cpu::read_sp();
                    if self.enable_stack_check {
                        if let FrameType::Jsr((_addr, _ret_addr, fsp, _)) = frame.frame_type {
                            if sp + 2 != fsp {
                                break StopReason::Bug(BugType::SpMismatch);
                            }
                        }
                    }
                }
                0x68 => {
                    // pla
                    let _frame = self.stack_frames.pop().unwrap();
                }
                0x48 => {
                    // pha
                    let ac = Cpu::read_ac();
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Pha(ac),
                    });
                }

                0x28 => {
                    // plp
                    let _frame = self.stack_frames.pop().unwrap();
                }
                0x08 => {
                    // php
                    let sr = Cpu::read_sr();
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Php(sr),
                    });
                }
                0x40 => {
                    // rti
                    let _frame = self.stack_frames.pop().unwrap();
                }
                _ => {}
            };
            Cpu::reset_memhits();
            // Now execute the instruction
            self.ticks += Cpu::execute_insn() as usize;

            // invalid memory read check
            if self.enable_mem_check {
                if let Some(addr) = Cpu::get_memcheck() {
                    Cpu::clear_memcheck();
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
            let pc = Cpu::read_pc();
            if let Some(next) = self.next_bp {
                // next stepping bp
                if next == pc {
                    self.next_bp = None;
                    break StopReason::Next;
                }
            }

            // did we hit a watch
            let mhc = Cpu::get_memhitcount();
            if mhc > 0 && !self.watch_points.is_empty() {
                for (addr, wp) in self.watch_points.iter() {
                    for hit in Cpu::get_memhits() {
                        if hit.1 == *addr {
                            match (wp.watch.clone(), hit.0) {
                                (WatchType::Read, false) => {
                                    break 'main_loop StopReason::WatchPoint(hit.1);
                                }
                                (WatchType::Write, true) => {
                                    break 'main_loop StopReason::WatchPoint(hit.1);
                                }
                                (WatchType::ReadWrite, _) => {
                                    break 'main_loop StopReason::WatchPoint(hit.1);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            if let Some(bp) = self.break_points.get(&pc) {
                if bp.temp {
                    self.break_points.remove(&pc);
                }
                break StopReason::BreakPoint(pc);
            }
            // PVExit called?
            if let Some(exit_code) = Cpu::exit_done() {
                break StopReason::Exit(exit_code);
            }
        };
        Ok(reason)
    }
}
