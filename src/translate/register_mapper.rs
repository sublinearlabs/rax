//! Register mapping trait for flexible register allocation strategies
//!
//! This trait defines the interface for register allocation without being
//! tied to a specific implementation. It allows different strategies to be
//! plugged in without modifying calling code.

use crate::aot::register_mapping::RegisterLocation;
use crate::translate::x86_insn::Operand;

/// Trait for register mapping and allocation strategies
pub trait RegisterMapper {
    /// Get the RegisterLocation for a given RISC-V register index (0-31)
    fn get_register_location(&self, risc_v_reg: u8) -> RegisterLocation;

    /// Convert a RegisterLocation to an x86-64 Operand
    /// This handles strategy-specific conversions (e.g., GPR-only vs mixed)
    fn location_to_operand(&self, location: RegisterLocation) -> Result<Operand, String>;

    /// Create a new instance with default configuration for this strategy
    fn new() -> Self
    where
        Self: Sized;
}
