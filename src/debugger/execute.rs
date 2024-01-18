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
    Finish,
}
#[derive(Debug, Clone)]
pub enum BugType {
    SpMismatch,
    Memcheck(u16),
    HeapCheck,
    SegCheck(u16),
}
use crate::{
    debugger::cpu::{Cpu, MemCheck},
    debugger::debugger::{Debugger, FrameType, SourceDebugMode, StackFrame, WatchType},
};
use anyhow::anyhow;
impl Debugger {
    pub fn execute(&mut self, mut count: u16) -> Result<StopReason> {
        let counting = count > 0;

        let reason = 'main_loop: loop {
            let pc = Cpu::read_pc();

            /*==============================================================
                            Stack tracking code
            if we hit a jsr, we push the return address and the stack pointer
            onto our own tracking stack. If we hit a rts, we pop the frame

            Also tracks push and pulls
            ===============================================================*/

            let inst = Cpu::read_byte(pc);
            let mut deferred_stop: Option<StopReason> = None;
            match inst {
                0x20 => {
                    // jsr
                    let lo = Cpu::read_byte(pc + 1);
                    let hi = Cpu::read_byte(pc + 2);
                    let sp = Cpu::read_sp();

                    let addr = lo as u16 | ((hi as u16) << 8);
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Jsr((addr, pc + 3, sp, 0)),
                        stop_on_pop: false,
                    });
                    if let Some(intercept) = self.call_intercepts.get(&addr) {
                        if let Some(stop) = intercept(self, false)? {
                            // if the intercept says stop, we defer it
                            // otherwise we cant hit 'go'
                            deferred_stop = Some(stop);
                        }
                    }
                }

                0x60 => {
                    // rts
                    if let Some(frame) = self.stack_frames.pop() {
                        if frame.stop_on_pop {
                            // defer til after we execute the rts
                            deferred_stop = Some(StopReason::Finish);
                        }

                        if let FrameType::Jsr((addr, _ret_addr, fsp, _)) = frame.frame_type {
                            if self.enable_stack_check {
                                let sp = Cpu::read_sp();
                                if sp + 2 != fsp {
                                    break StopReason::Bug(BugType::SpMismatch);
                                }
                            }
                            if let Some(intercept) = self.call_intercepts.get(&addr) {
                                if let Some(stop) = intercept(self, true)? {
                                    break 'main_loop stop;
                                }
                            }
                        } else {
                            // wrong frame type
                            // same issue with longjmp
                            //break StopReason::Bug(BugType::SpMismatch);
                        }
                    } else if self.enable_stack_check {
                        break StopReason::Bug(BugType::SpMismatch);
                    }
                }
                0x68 => {
                    // pla
                    if let Some(fr) = self.stack_frames.pop() {
                        // ok - but it should be a push frame
                        if let FrameType::Jsr((_, _, _, _)) = fr.frame_type {
                            if self.enable_stack_check {
                                // note - longjmp hits this becuase it manually
                                // pops a return address off the stack using pla,pla
                                //break StopReason::Bug(BugType::SpMismatch);
                            }
                        }
                    } else if self.enable_stack_check {
                        break StopReason::Bug(BugType::SpMismatch);
                    }
                }
                0x48 => {
                    // pha
                    let ac = Cpu::read_ac();
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Pha(ac),
                        stop_on_pop: false,
                    });
                }

                0x28 => {
                    // plp
                    if let Some(fr) = self.stack_frames.pop() {
                        // ok - but it should be a push frame
                        if let FrameType::Jsr((_, _, _, _)) = fr.frame_type {
                            break StopReason::Bug(BugType::SpMismatch);
                        }
                    } else if self.enable_stack_check {
                        break StopReason::Bug(BugType::SpMismatch);
                    }
                }
                0x08 => {
                    // php
                    let sr = Cpu::read_sr();
                    self.stack_frames.push(StackFrame {
                        frame_type: FrameType::Php(sr),
                        stop_on_pop: false,
                    });
                }
                0x40 => {
                    // rti
                    let _ = self.stack_frames.pop();
                }
                _ => {}
            };

            // Now execute the instruction
            self.ticks += Cpu::execute_insn() as usize;

            // PVExit called?
            if let Some(exit_code) = Cpu::exit_done() {
                self.run_done = false;
                break StopReason::Exit(exit_code);
            }

            if Cpu::was_paracall() {
                // a PV call pops the stack but we do not see an rts
                // so we have a dangling stack frame - pop it
                self.stack_frames.pop().ok_or(anyhow!("stack underflow"))?;
            }

            // invalid memory read check
            if self.enable_mem_check && !self.privileged_mode {
                match Cpu::get_memcheck() {
                    MemCheck::None => {}
                    MemCheck::ReadNoWrite(addr) => {
                        break 'main_loop StopReason::Bug(BugType::Memcheck(*addr));
                    }
                    MemCheck::WriteNoPermission(addr) => {
                        break 'main_loop StopReason::Bug(BugType::SegCheck(*addr));
                    }
                }
            }

            // limited number of instructions?
            if counting {
                count -= 1;
                if count == 0 {
                    break StopReason::Count;
                }
            }
            // did we just pop a stop_on_pop frame?
            if let Some(stop) = deferred_stop {
                break 'main_loop stop;
            }

            let pc = Cpu::read_pc();

            // did we step over a function call?
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

            //  did we hit a breakpoint?
            if let Some(bp) = self.break_points.get(&pc) {
                if bp.temp {
                    self.break_points.remove(&pc);
                }
                break StopReason::BreakPoint(pc);
            }

            // source mode next and step
            match &self.source_mode {
                SourceDebugMode::None => {}
                SourceDebugMode::Next => {
                    if let Some(addr_lookup) = self.source_info.get(&pc) {
                        if let Some(cf) = self.current_file {
                            if cf == addr_lookup.file_id {
                                self.source_mode = SourceDebugMode::None;
                                break 'main_loop StopReason::Next;
                            }
                        }
                    }
                }
                SourceDebugMode::Step => {
                    if let Some(_addr_lookup) = self.source_info.get(&pc) {
                        self.source_mode = SourceDebugMode::None;
                        break 'main_loop StopReason::Next;
                    }
                }
            }
            // post instruction clean up
            Cpu::post_inst_reset();
        };
        Cpu::post_inst_reset(); // will have been missed on a break
        if let Some(f) = self.find_source_line(self.read_pc())? {
            self.current_file = Some(f.file_id);
        }
        Ok(reason)
    }
}
