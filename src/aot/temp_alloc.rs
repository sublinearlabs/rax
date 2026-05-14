use std::ops::Deref;

use crate::aot::registers::X86Gpr;

struct TempInfo {
    temp: X86Gpr,
    in_use: bool,
}

impl TempInfo {
    /// Marks a temp registers as allocated
    ///
    /// Panics if you try to lock an already allocated register
    fn lock(&mut self) {
        assert!(!self.in_use);
        self.in_use = true;
    }

    /// Marks a temp register as unallocated
    ///
    /// Panics if you try to unlock a free temp register
    fn unlock(&mut self) {
        assert!(self.in_use);
        self.in_use = false;
    }
}

/// Represents an allocated temp
///
/// on Drop, frees the allocation for future use
struct TempGuard<'a> {
    temp_info: &'a mut TempInfo,
}

/// Free up allocation when TempGuard is dropped
impl<'a> Drop for TempGuard<'a> {
    fn drop(&mut self) {
        self.temp_info.unlock();
    }
}

impl<'a> Deref for TempGuard<'a> {
    type Target = X86Gpr;

    fn deref(&self) -> &Self::Target {
        &self.temp_info.temp
    }
}

/// Safe interface for handling temporary registers
/// during AOT compilation.
struct TempAllocator {
    temps: Vec<TempInfo>,
}

impl TempAllocator {
    /// Inits a new temp allocator from specified temp gprs
    fn new(temps: Vec<X86Gpr>) -> Self {
        let temp_infos = temps.into_iter().map(|t| TempInfo {
            temp: t,
            in_use: false,
        });

        Self {
            temps: temp_infos.collect(),
        }
    }

    /// Returns a bool specifying if an x86 GPR register is one
    /// of the temp registers
    fn is_temp(&self, reg: &X86Gpr) -> bool {
        self.temps.iter().any(|v| &v.temp == reg)
    }

    /// Returns the first unallocated temp register
    fn allocate(&mut self) -> Result<TempGuard<'_>, TempAllocationError> {
        // find the first temp info that is safe
        // lock it
        // wrap it in a guard that will force unlock after drop

        let free_temp = self
            .temps
            .iter_mut()
            .find(|t| t.in_use == false)
            .ok_or(TempAllocationError::AllTempsAllocated)?;

        // mark as allocated
        free_temp.lock();

        // wrap in guard to force unlock on drop
        Ok(TempGuard {
            temp_info: free_temp,
        })
    }
}

enum TempAllocationError {
    AllTempsAllocated,
}
