use std::{cell::Cell, ops::Deref};

use crate::aot::registers::X86Gpr;

/// Errors that can occur while allocating temporary registers.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TempAllocationError {
    /// No managed temporary register slot is currently free.
    NoFreeTemps,
    /// Requested register is not managed by this allocator.
    NotATemp,
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
            allocated: Cell::new(false),
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
    pub(crate) fn allocate(&self) -> Result<AllocatedTemp<'_>, TempAllocationError> {
        let free_slot = self
            .slots
            .iter()
            .find(|t| !t.allocated.get())
            .ok_or(TempAllocationError::NoFreeTemps)?;

        // mark as allocated
        free_slot.allocate();

        // wrap in guard to force unlock on drop
        Ok(AllocatedTemp { slot: free_slot })
    }

    /// Allocates a specific temporary register if it is managed and free.
    ///
    /// # Errors
    /// Returns `TempAllocationError::NotATemp` when `reg` is not managed by
    /// this allocator, or `TempAllocationError::NoFreeTemps` when that specific
    /// temp is already allocated.
    pub(crate) fn allocate_specific(
        &self,
        reg: X86Gpr,
    ) -> Result<AllocatedTemp<'_>, TempAllocationError> {
        let slot = self
            .slots
            .iter()
            .find(|t| t.reg == reg)
            .ok_or(TempAllocationError::NotATemp)?;

        if slot.allocated.get() {
            return Err(TempAllocationError::NoFreeTemps);
        }

        slot.allocate();
        Ok(AllocatedTemp { slot })
    }
}

/// Represents an allocated temporary register.
///
/// On drop, releases the allocation for future use.
pub(crate) struct AllocatedTemp<'a> {
    slot: &'a TempSlot,
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
    /// Returns the x86-64 GPR encoding id (`0..=15`) of this temp register.
    ///
    /// This is the hardware register code used by instruction encoders.
    /// It is not a RISC-V register index.
    pub(crate) fn id(&self) -> u8 {
        (**self).id()
    }
}

/// Internal allocator slot state for one managed temporary register.
struct TempSlot {
    reg: X86Gpr,
    allocated: Cell<bool>,
}

impl TempSlot {
    /// Marks this temporary register slot as allocated.
    ///
    /// Panics if the slot is already allocated.
    fn allocate(&self) {
        assert!(!self.allocated.get());
        self.allocated.set(true);
    }

    /// Marks this temporary register slot as free.
    ///
    /// Panics if the slot is already free.
    fn release(&self) {
        assert!(self.allocated.get());
        self.allocated.set(false);
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

    #[test]
    fn allocates_more_than_one_temp_at_once() {
        let mut alloc = TempAllocator::new(vec![X86Gpr::Rax, X86Gpr::Rbx]);
        let t1 = alloc.allocate().unwrap();
        let t2 = alloc.allocate().unwrap();
        drop(t1);
        let t3 = alloc.allocate().unwrap();
    }

    #[test]
    fn allocate_specific_returns_requested_temp() {
        let alloc = TempAllocator::new(vec![X86Gpr::R12, X86Gpr::R13]);

        let temp = alloc
            .allocate_specific(X86Gpr::R13)
            .expect("specific temp should allocate");

        assert_eq!(*temp, X86Gpr::R13);
    }

    #[test]
    fn allocate_specific_rejects_non_temp() {
        let alloc = TempAllocator::new(vec![X86Gpr::R12]);

        match alloc.allocate_specific(X86Gpr::R13) {
            Err(err) => assert_eq!(err, TempAllocationError::NotATemp),
            Ok(_) => panic!("non-temp register should be rejected"),
        };
    }

    #[test]
    fn allocate_specific_rejects_already_allocated_temp() {
        let alloc = TempAllocator::new(vec![X86Gpr::R12]);

        let _held = alloc
            .allocate_specific(X86Gpr::R12)
            .expect("first allocation should succeed");

        match alloc.allocate_specific(X86Gpr::R12) {
            Err(err) => assert_eq!(err, TempAllocationError::NoFreeTemps),
            Ok(_) => panic!("allocated temp should be rejected"),
        };
    }

    #[test]
    fn allocate_specific_blocks_general_allocate_until_dropped() {
        let alloc = TempAllocator::new(vec![X86Gpr::R12]);

        let held = alloc
            .allocate_specific(X86Gpr::R12)
            .expect("specific allocation should succeed");

        match alloc.allocate() {
            Err(err) => assert_eq!(err, TempAllocationError::NoFreeTemps),
            Ok(_) => panic!("general allocation should be blocked"),
        };

        drop(held);

        let temp = alloc.allocate().expect("temp should be free after drop");
        assert_eq!(*temp, X86Gpr::R12);
    }
}
