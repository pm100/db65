use anyhow::{bail, Result};
pub enum StopReason {
    BreakPoint,
    Exit,
    Count,
}
use crate::{cpu::Sim, debugger::Debugger};
impl Debugger {
    pub fn execute(&mut self, mut count: u16) -> Result<StopReason> {
        let counting = count > 0;
        loop {
            if counting {
                count -= 1;
                if count == 0 {
                    return Ok(StopReason::Count);
                }
            }

            Sim::execute_insn();
            //  did we hit a breakpoint?
            let pc = Sim::read_pc();
            if self.break_points.contains_key(&pc) {
                // println!("breakpoint");
                return Ok(StopReason::BreakPoint);
            }

            if Sim::exit_done() {
                return Ok(StopReason::Exit);
            }
        }
    }
}
