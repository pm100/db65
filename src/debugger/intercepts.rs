use crate::{debugger::cpu::Cpu, debugger::debugger::Debugger, trace};
use anyhow::{bail, Result};
type InterceptFunc = fn(&mut Debugger, bool) -> Result<Option<StopReason>>;
use super::{
    cpu::ShadowFlags,
    debugger::HeapBlock,
    execute::{BugType, StopReason},
};

impl Debugger {
    /*

       intercepts are called twice, once on the call and once on the return
       the call call is done just before the Jsr is executed, ret = false
       the return call is done just before the Rts is executed, ret = true

    */
    pub fn load_intercepts(&mut self) -> Result<()> {
        let malloc = self.dbgdb.get_symbol("malloc._malloc")?;
        if malloc.len() == 1 {
            self.call_intercepts
                .insert(malloc[0].1, |d, f| d.malloc_intercept(f));
            // only try to hook free if we managed to hook malloc
            // they really are a pair
            // TODO hook realloc because there is a case where realloc is called
            // and it grows the current block - ie not a simple
            // wrapper over free/malloc
            let free = self.dbgdb.get_symbol("free._free")?;
            if free.len() == 1 {
                self.call_intercepts
                    .insert(free[0].1, |d, f| d.free_intercept(f));
            };
        };
        Ok(())
    }

    /*  Heap hooks

        allows
        - update of shadow memory for new heap blocks
        - check for double free / invalid free
        - check for leaks
    */
    fn malloc_intercept(&mut self, ret: bool) -> Result<Option<StopReason>> {
        if ret {
            // return from malloc, we know the address now
            self.privileged_mode = false;
            let addr = Self::ac_xr();
            if addr == 0 {
                // malloc returned null
                return Ok(None);
            }
            let new_block = if let Some(hb) = self.heap_blocks.get(&0) {
                (hb.alloc_addr, hb.size)
            } else {
                bail!("missing 0 heap block");
            };
            let hb = HeapBlock {
                addr,
                size: new_block.1,
                alloc_addr: new_block.0,
            };
            // delete the temporary 0 block
            self.heap_blocks.remove(&0);
            self.heap_blocks.insert(addr, hb);

            // now update the shadow memory
            let shadow = Cpu::get_shadow();
            for i in addr..addr + new_block.1 {
                shadow[i as usize] |= ShadowFlags::READ | ShadowFlags::WRITE;
            }
            trace!("malloc ret {:04x}", addr);
        } else {
            // at the time of call to malloc we do not know the address
            // so create a temporary entry with addr = 0
            let size = Self::ac_xr();
            let hb = HeapBlock {
                addr: 0,
                size,
                alloc_addr: Cpu::read_pc(),
            };
            trace!("malloc call {} @ {:04x}", size, hb.alloc_addr);
            self.heap_blocks.insert(0, hb);

            // malloc is privileged - it can write to unalloacted memory

            self.privileged_mode = true;
        };

        Ok(None)
    }

    fn free_intercept(&mut self, ret: bool) -> Result<Option<StopReason>> {
        if !ret {
            let addr = Self::ac_xr();
            if addr == 0 {
                // free of null
                return Ok(None);
            }
            let old = if let Some(hb) = self.heap_blocks.get(&addr) {
                (hb.size, hb.addr)
            } else {
                // not found -> double or invalid free
                return Ok(Some(StopReason::Bug(BugType::HeapCheck)));
            };
            self.heap_blocks.remove(&addr);

            // update the shadow to show that this is free, naked memory
            let shadow = Cpu::get_shadow();
            for i in addr..addr + old.0 {
                shadow[i as usize] = ShadowFlags::empty();
            }
        }

        Ok(None)
    }
    fn ac_xr() -> u16 {
        let ac = Cpu::read_ac();
        let xr = Cpu::read_xr();
        (xr as u16) << 8 | (ac as u16)
    }
}
