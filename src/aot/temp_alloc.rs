use std::ops::Deref;

use crate::aot::registers::X86Gpr;

struct TempSlot {
    reg: X86Gpr,
    allocated: bool,
}

impl TempSlot {
    /// Marks a temp registers as allocated
    ///
    /// Panics if you try to lock an already allocated register
    fn allocate(&mut self) {
        assert!(!self.allocated);
        self.allocated = true;
    }

    /// Marks a temp register as unallocated
    ///
    /// Panics if you try to unlock a free temp register
    fn release(&mut self) {
        assert!(self.allocated);
        self.allocated = false;
    }
}

/// Represents an allocated temp
///
/// on Drop, frees the allocation for future use
struct AllocatedTemp<'a> {
    slot: &'a mut TempSlot,
}

/// Free up allocation when AllocatedTemp is dropped
impl<'a> Drop for AllocatedTemp<'a> {
    fn drop(&mut self) {
        self.slot.release();
    }
}

impl<'a> Deref for AllocatedTemp<'a> {
    type Target = X86Gpr;

    fn deref(&self) -> &Self::Target {
        &self.slot.reg
    }
}

/// Safe interface for handling temporary registers
/// during AOT compilation.
struct TempAllocator {
    slots: Vec<TempSlot>,
}

impl TempAllocator {
    /// Inits a new temp allocator from specified temp gprs
    fn new(temps: Vec<X86Gpr>) -> Self {
        let temp_slots = temps.into_iter().map(|t| TempSlot {
            reg: t,
            allocated: false,
        });

        Self {
            slots: temp_slots.collect(),
        }
    }

    /// Returns a bool specifying if an x86 GPR register is one
    /// of the temp registers
    fn is_temp(&self, reg: &X86Gpr) -> bool {
        self.slots.iter().any(|v| &v.reg == reg)
    }

    /// Returns the first unallocated temp register
    fn allocate(&mut self) -> Result<AllocatedTemp<'_>, TempAllocationError> {
        let free_slot = self
            .slots
            .iter_mut()
            .find(|t| !t.allocated)
            .ok_or(TempAllocationError::NoFreeTemps)?;

        // mark as allocated
        free_slot.allocate();

        // wrap in guard to force unlock on drop
        Ok(AllocatedTemp {
            slot: free_slot,
        })
    }
}

enum TempAllocationError {
    NoFreeTemps,
}
