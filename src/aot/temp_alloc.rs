// I want to create a safe type that allows for easy handling of temp registers
// it is going to be created with a list of temp value
// what are the interfaces that I care about in this case?
// is_temp()
// allocate() -> should give one of the unused temporary variables

use crate::aot::registers::X86Gpr;

struct TempInfo {
    temp: X86Gpr,
    in_use: bool,
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
        self.temps.iter().find(|v| &v.temp == reg).is_some()
    }

    /// Returns the first unallocated temp register
    ///
    /// SAFETY: Caller must ensure that after use, in_use should be set to false
    unsafe fn allocate(&self) -> Result<X86Gpr, TempAllocationError> {
        // find the first temp that is not in use
        let free_temp = self.temps.iter().find(|v| !v.in_use);
        free_temp
            .map(|temp| temp.temp)
            .ok_or(TempAllocationError::AllTempsAllocated)
    }
}

enum TempAllocationError {
    AllTempsAllocated,
}
