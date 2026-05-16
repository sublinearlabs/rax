use std::ops::Deref;

use crate::aot::registers::X86Gpr;

/// Errors that can occur while allocating temporary registers.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TempAllocationError {
    /// No managed temporary register slot is currently free.
    NoFreeTemps,
}

/// Safe interface for handling temporary registers
/// during AOT compilation.
pub(crate) struct TempAllocator {
    slots: Vec<TempSlot>,
}

impl TempAllocator {
    /// Creates a temp allocator from managed x86 GPR temporaries.
    ///
    /// # Panics
    /// Panics if `temps` contains duplicate registers.
    pub(crate) fn new(temps: Vec<X86Gpr>) -> Self {
        let mut seen = [false; 16];
        for reg in &temps {
            let idx = *reg as usize;
            assert!(!seen[idx], "duplicate temp register in TempAllocator::new");
            seen[idx] = true;
        }

        let temp_slots = temps.into_iter().map(|t| TempSlot {
            reg: t,
            allocated: false,
        });

        Self {
            slots: temp_slots.collect(),
        }
    }

    /// Returns whether an x86 GPR is managed as a temporary.
    fn is_temp(&self, reg: &X86Gpr) -> bool {
        self.slots.iter().any(|v| &v.reg == reg)
    }

    /// Allocates and returns the first free temporary register.
    ///
    /// # Errors
    /// Returns `TempAllocationError::NoFreeTemps` when all managed
    /// temporary registers are currently allocated.
    pub(crate) fn allocate(&mut self) -> Result<AllocatedTemp<'_>, TempAllocationError> {
        let free_slot = self
            .slots
            .iter_mut()
            .find(|t| !t.allocated)
            .ok_or(TempAllocationError::NoFreeTemps)?;

        // mark as allocated
        free_slot.allocate();

        // wrap in guard to force unlock on drop
        Ok(AllocatedTemp { slot: free_slot })
    }
}

/// Represents an allocated temporary register.
///
/// On drop, releases the allocation for future use.
pub(crate) struct AllocatedTemp<'a> {
    slot: &'a mut TempSlot,
}

/// Free up allocation when AllocatedTemp is dropped
impl<'a> Drop for AllocatedTemp<'a> {
    fn drop(&mut self) {
        self.slot.release();
    }
}

/// Provides ergonomic access to the allocated register as `&X86Gpr`.
impl<'a> Deref for AllocatedTemp<'a> {
    type Target = X86Gpr;

    fn deref(&self) -> &Self::Target {
        &self.slot.reg
    }
}

impl<'a> AllocatedTemp<'a> {
    pub(crate) fn id(&self) -> u8 {
        (**self).id()
    }
}

/// Internal allocator slot state for one managed temporary register.
struct TempSlot {
    reg: X86Gpr,
    allocated: bool,
}

impl TempSlot {
    /// Marks this temporary register slot as allocated.
    ///
    /// Panics if the slot is already allocated.
    fn allocate(&mut self) {
        assert!(!self.allocated);
        self.allocated = true;
    }

    /// Marks this temporary register slot as free.
    ///
    /// Panics if the slot is already free.
    fn release(&mut self) {
        assert!(self.allocated);
        self.allocated = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_unique_regs() {
        let _ = TempAllocator::new(vec![X86Gpr::Rax, X86Gpr::Rbx, X86Gpr::R10]);
    }

    #[test]
    #[should_panic(expected = "duplicate temp register in TempAllocator::new")]
    fn new_panics_on_duplicate_regs() {
        let _ = TempAllocator::new(vec![X86Gpr::Rax, X86Gpr::Rax]);
    }

    #[test]
    fn is_temp_true_for_managed_regs() {
        let alloc = TempAllocator::new(vec![X86Gpr::Rax, X86Gpr::Rbx]);
        assert!(alloc.is_temp(&X86Gpr::Rax));
        assert!(alloc.is_temp(&X86Gpr::Rbx));
    }

    #[test]
    fn is_temp_false_for_unmanaged_regs() {
        let alloc = TempAllocator::new(vec![X86Gpr::Rax, X86Gpr::Rbx]);
        assert!(!alloc.is_temp(&X86Gpr::Rcx));
    }

    #[test]
    fn allocate_returns_managed_register() {
        let mut alloc = TempAllocator::new(vec![X86Gpr::Rax, X86Gpr::Rbx]);
        let guard = alloc.allocate().expect("must allocate first temp");
        assert!(*guard == X86Gpr::Rax || *guard == X86Gpr::Rbx);
    }

    #[test]
    fn dropping_guard_releases_slot() {
        let mut alloc = TempAllocator::new(vec![X86Gpr::Rax]);

        let g1 = alloc.allocate().expect("first allocation should succeed");
        assert!(*g1 == X86Gpr::Rax);
        drop(g1);

        let g2 = alloc
            .allocate()
            .expect("allocation should succeed after drop");
        assert!(*g2 == X86Gpr::Rax);
    }

    #[test]
    fn allocated_temp_deref_exposes_register() {
        let mut alloc = TempAllocator::new(vec![X86Gpr::R10]);
        let guard = alloc.allocate().expect("allocation should succeed");
        assert!(*guard == X86Gpr::R10);
    }
}
