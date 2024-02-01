use crate::{debugger::core::Debugger, debugger::cpu::Cpu, trace};
use anyhow::{bail, Result};

use super::{
    core::HeapBlock,
    cpu::ShadowFlags,
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
        let realloc = self.dbgdb.get_symbol("realloc._realloc")?;
        if realloc.len() == 1 {
            self.call_intercepts
                .insert(realloc[0].1, |d, f| d.realloc_intercept(f));
        };
        Ok(())
    }

    /*  Heap hooks

        allows
        - update of shadow memory for new heap blocks
        - check for double free / invalid free
        - check for leaks

        THIS CODE HAS INTIMATE KNOWLEDGE OF THE HEAP IMPLEMENTATION IN CC65

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
                realloc_size: None,
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
                realloc_size: None,
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
                if self.enable_heap_check {
                    return Ok(Some(StopReason::Bug(BugType::HeapCheck)));
                } else {
                    return Ok(None);
                }
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
    fn realloc_intercept(&mut self, ret: bool) -> Result<Option<StopReason>> {
        if ret {
            // return from realloc
            // 3 cases
            // - it returned null - original block still ok
            // - it returned a new address - all work was done via malloc and free
            // - it returned the same address - we need to extend the shadow
            self.privileged_mode = false;
            let addr = Self::ac_xr();
            trace!("realloc ret {:04x}", addr);
            if addr == 0 {
                // realloc returned null
                return Ok(None);
            }
            if let Some(hb) = self.heap_blocks.get_mut(&addr) {
                if let Some(sz) = hb.realloc_size {
                    // case 3 - same address
                    let orig_size = hb.size;
                    hb.size = sz;
                    let shadow = Cpu::get_shadow();
                    if sz < orig_size {
                        // realloc to smaller size
                        // update the shadow to show that this is free

                        for i in addr + sz..addr + orig_size {
                            shadow[i as usize] = ShadowFlags::empty();
                        }
                    } else {
                        for i in addr + orig_size..addr + hb.size {
                            shadow[i as usize] |= ShadowFlags::READ | ShadowFlags::WRITE;
                        }
                    }
                } else {
                    // case 2 - new address
                    // all work was done via malloc and free
                    // nothing to do
                }
            } else {
                // realloc returned a new address but malloc didnt see it!
                panic!("realloc returns non heap block");
            };
        } else {
            let addr = Self::read_arg(0);
            let size = Self::ac_xr();
            if addr == 0 {
                // realloc of null
                // realloc will call malloc
                return Ok(None);
            }
            if size == 0 {
                // realloc to zero size
                // realloc will call free
                return Ok(None);
            }
            trace!("realloc call {} @ {:04x}", size, addr);
            if let Some(hb) = self.heap_blocks.get_mut(&addr) {
                hb.realloc_size = Some(size);
                hb.alloc_addr = Cpu::read_pc();
            } else {
                // not found -> realloc of non heap block
                if self.enable_heap_check {
                    return Ok(Some(StopReason::Bug(BugType::HeapCheck)));
                } else {
                    return Ok(None);
                }
            };

            // realloc is privileged - it can write to unalloacted memory

            self.privileged_mode = true;
        };

        Ok(None)
    }
    fn ac_xr() -> u16 {
        let ac = Cpu::read_ac();
        let xr = Cpu::read_xr();
        (xr as u16) << 8 | (ac as u16)
    }
    fn read_arg(offset: u16) -> u16 {
        let sp65_addr = Cpu::get_sp65_addr() as u16;
        let sp65 = Cpu::read_word(sp65_addr + offset);

        Cpu::read_word(sp65)
    }
}
