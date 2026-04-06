//! Register allocation configuration for RISC-V to x86-64 translation
//!
//! Supports multiple register allocation strategies:
//! - AllGPR: Use all 16 x86 GPRs with stack spilling for overflow (default)
//! - GPRAndXMM: Mix of 8-12 GPRs + XMM registers for extended storage
//! - OnlyXMM: Use only XMM registers (for specialized workloads)
//!
//! Each strategy returns a RegisterMapping from aot::register_mapping module.

use crate::aot::register_mapping::{RegisterLocation, RegisterMapping};
use std::array;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterAllocationStrategy {
    /// Use all 16 x86 GPRs, spill overflow to stack (default)
    AllGPR,
    /// Mix of 8-12 GPRs + XMM registers for extended storage
    GPRAndXMM,
    /// Use only XMM registers (for specialized workloads)
    OnlyXMM,
}

impl Default for RegisterAllocationStrategy {
    fn default() -> Self {
        RegisterAllocationStrategy::AllGPR
    }
}

/// Configuration for register allocation strategy
#[derive(Clone, Debug)]
pub struct RegisterAllocationConfig {
    /// Selected allocation strategy
    pub strategy: RegisterAllocationStrategy,
    /// Number of GPRs to use (for GPRAndXMM strategy)
    pub num_gprs: u32,
    /// Whether to use XMM registers (for GPRAndXMM strategy)
    pub use_xmm: bool,
}

impl RegisterAllocationConfig {
    /// Create a new register allocation configuration with default strategy (AllGPR)
    pub fn new() -> Self {
        Self {
            strategy: RegisterAllocationStrategy::AllGPR,
            num_gprs: 16,
            use_xmm: false,
        }
    }

    /// Create configuration with AllGPR strategy (use all 16 x86 GPRs)
    pub fn allgpr() -> Self {
        Self {
            strategy: RegisterAllocationStrategy::AllGPR,
            num_gprs: 16,
            use_xmm: false,
        }
    }

    /// Create configuration with GPRAndXMM strategy
    /// Uses a mix of GPRs (default 12) and XMM registers (16 available)
    pub fn gpr_and_xmm(num_gprs: u32) -> Self {
        assert!(
            num_gprs >= 8 && num_gprs <= 16,
            "num_gprs must be between 8 and 16, got {}",
            num_gprs
        );
        Self {
            strategy: RegisterAllocationStrategy::GPRAndXMM,
            num_gprs,
            use_xmm: true,
        }
    }

    /// Create configuration with default GPRAndXMM strategy (12 GPRs + 16 XMM)
    pub fn gpr_and_xmm_default() -> Self {
        Self::gpr_and_xmm(12)
    }

    /// Create configuration with OnlyXMM strategy (XMM registers only)
    pub fn only_xmm() -> Self {
        Self {
            strategy: RegisterAllocationStrategy::OnlyXMM,
            num_gprs: 0,
            use_xmm: true,
        }
    }

    /// Set the strategy for this configuration
    pub fn with_strategy(mut self, strategy: RegisterAllocationStrategy) -> Self {
        self.strategy = strategy;
        match strategy {
            RegisterAllocationStrategy::AllGPR => {
                self.num_gprs = 16;
                self.use_xmm = false;
            }
            RegisterAllocationStrategy::GPRAndXMM => {
                self.num_gprs = 12; // default
                self.use_xmm = true;
            }
            RegisterAllocationStrategy::OnlyXMM => {
                self.num_gprs = 0;
                self.use_xmm = true;
            }
        }
        self
    }

    /// Create a RegisterMapping based on this configuration
    /// Each strategy returns its own register location mapping
    pub fn create_mapping(&self) -> RegisterMapping {
        match self.strategy {
            RegisterAllocationStrategy::AllGPR => self.create_allgpr_mapping(),
            RegisterAllocationStrategy::GPRAndXMM => self.create_gpr_and_xmm_mapping(),
            RegisterAllocationStrategy::OnlyXMM => self.create_only_xmm_mapping(),
        }
    }

    /// AllGPR strategy: Map all RISC-V registers to x86 GPRs with stack spilling
    /// Uses all 16 x86 GPRs for registers 1-17, spills registers 18-31 to stack
    fn create_allgpr_mapping(&self) -> RegisterMapping {
        let map = array::from_fn(|idx| {
            match idx {
                // RISC-V ABI mapping to x86-64 GPRs
                // All 16 GPRs are used;
                1 => RegisterLocation::GPR(3),   // x1 (ra) -> RBX
                2 => RegisterLocation::GPR(4),   // x2 (sp) -> RSP
                3 => RegisterLocation::GPR(12),  // x3 (gp) -> R12
                4 => RegisterLocation::GPR(13),  // x4 (tp) -> R13
                5 => RegisterLocation::GPR(14),  // x5 (t0) -> R14
                6 => RegisterLocation::GPR(9),   // x6 (t1) -> R9
                7 => RegisterLocation::GPR(8),   // x7 (t2) -> R8
                8 => RegisterLocation::GPR(5),   // x8 (s0/fp) -> RBP
                9 => RegisterLocation::GPR(15),  // x9 (s1) -> R15
                10 => RegisterLocation::GPR(0),  // x10 (a0) -> RAX (return value)
                11 => RegisterLocation::GPR(2),  // x11 (a1) -> RDX (second return value)
                12 => RegisterLocation::GPR(1),  // x12 (a2) -> RCX
                13 => RegisterLocation::GPR(6),  // x13 (a3) -> RSI
                14 => RegisterLocation::GPR(7),  // x14 (a4) -> RDI
                15 => RegisterLocation::GPR(16), // x15 (a5) -> R16
                16 => RegisterLocation::GPR(10), // x16 (a6) -> R10
                17 => RegisterLocation::GPR(11), // x17 (a7) -> R11
                // Registers 18-31: spill to stack (8 bytes per register)
                reg => {
                    let spill_index = (reg - 18) as u64;
                    RegisterLocation::MEM(spill_index * 8)
                }
            }
        });
        RegisterMapping::new(map)
    }

    /// GPRAndXMM strategy: Mix of GPRs and XMM registers for extended storage
    /// Uses limited GPRs (default 12) and XMM for remaining registers
    fn create_gpr_and_xmm_mapping(&self) -> RegisterMapping {
        let num_gprs = self.num_gprs.min(16) as usize;
        let gpr_map = [3, 4, 12, 13, 14, 9, 8, 5, 15, 0, 2, 1, 6, 7, 16];

        let map = array::from_fn(|idx| {
            match idx {
                // Registers 0-17 mapped to available GPRs
                0..=17 => {
                    if idx < num_gprs {
                        RegisterLocation::GPR(gpr_map[idx])
                    } else {
                        // Use XMM for registers beyond available GPRs
                        let xmm_idx = (idx - num_gprs) as u8;
                        RegisterLocation::XMM(xmm_idx % 16, 0)
                    }
                }
                // Registers 18-31 to XMM or memory
                18..=31 => {
                    let xmm_idx = (idx - 18) as u8;
                    // TODO: Decide which XMM registers will contain two values
                    RegisterLocation::XMM(xmm_idx, 0)
                }
                _ => RegisterLocation::ZERO,
            }
        });
        RegisterMapping::new(map)
    }

    /// OnlyXMM strategy: Map all registers to XMM registers only
    /// Uses cyclic mapping across 16 XMM registers
    fn create_only_xmm_mapping(&self) -> RegisterMapping {
        let map = array::from_fn(|idx| {
            if idx < 32 {
                // TODO: refactor to account for high and low xmm registers
                let xmm_idx = (idx % 16) as u8;
                RegisterLocation::XMM(xmm_idx, 0)
            } else {
                RegisterLocation::ZERO
            }
        });
        RegisterMapping::new(map)
    }
}

impl Default for RegisterAllocationConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = RegisterAllocationConfig::new();
        assert_eq!(config.strategy, RegisterAllocationStrategy::AllGPR);
    }

    #[test]
    fn test_allgpr_strategy() {
        let config = RegisterAllocationConfig::allgpr();
        let mapping = config.create_mapping();

        // Verify some key mappings via debug output
        // RegisterMapping uses private fields, so we just verify it creates successfully
        assert_eq!(config.strategy, RegisterAllocationStrategy::AllGPR);
    }

    #[test]
    fn test_gpr_and_xmm_strategy() {
        let config = RegisterAllocationConfig::gpr_and_xmm(12);
        let mapping = config.create_mapping();

        assert_eq!(config.strategy, RegisterAllocationStrategy::GPRAndXMM);
        assert_eq!(config.num_gprs, 12);
    }

    #[test]
    fn test_only_xmm_strategy() {
        let config = RegisterAllocationConfig::only_xmm();
        let mapping = config.create_mapping();

        assert_eq!(config.strategy, RegisterAllocationStrategy::OnlyXMM);
    }

    #[test]
    fn test_gpr_and_xmm_default() {
        let config = RegisterAllocationConfig::gpr_and_xmm_default();
        assert_eq!(config.strategy, RegisterAllocationStrategy::GPRAndXMM);
        assert_eq!(config.num_gprs, 12);
        assert!(config.use_xmm);
    }

    #[test]
    fn test_with_strategy() {
        let config =
            RegisterAllocationConfig::new().with_strategy(RegisterAllocationStrategy::GPRAndXMM);
        assert_eq!(config.strategy, RegisterAllocationStrategy::GPRAndXMM);
        assert!(config.use_xmm);
    }

    #[test]
    fn test_gpr_validation() {
        // Valid range: 8-16
        let _ = RegisterAllocationConfig::gpr_and_xmm(8);
        let _ = RegisterAllocationConfig::gpr_and_xmm(12);
        let _ = RegisterAllocationConfig::gpr_and_xmm(16);
    }

    #[test]
    #[should_panic(expected = "num_gprs must be between 8 and 16")]
    fn test_gpr_too_low() {
        RegisterAllocationConfig::gpr_and_xmm(7);
    }

    #[test]
    #[should_panic(expected = "num_gprs must be between 8 and 16")]
    fn test_gpr_too_high() {
        RegisterAllocationConfig::gpr_and_xmm(17);
    }

    #[test]
    fn test_allgpr_backwards_compat() {
        // AllGPR should produce valid mapping
        let allgpr_config = RegisterAllocationConfig::allgpr();
        let allgpr_mapping = allgpr_config.create_mapping();

        // Verify the mapping was created successfully
        assert_eq!(allgpr_config.strategy, RegisterAllocationStrategy::AllGPR);
    }

    #[test]
    fn test_config_clone() {
        let config1 = RegisterAllocationConfig::gpr_and_xmm(12);
        let config2 = config1.clone();
        assert_eq!(config1.strategy, config2.strategy);
        assert_eq!(config1.num_gprs, config2.num_gprs);
    }
}
