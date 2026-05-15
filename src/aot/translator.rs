use dynasmrt::x64::Assembler;

use crate::aot::{
    register_mapping::{MapTarget, MappingPlan, RegisterMapping},
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::{AllocatedTemp, TempAllocator},
};

/// AOT translator state used while lowering RISC-V instructions to x86.
///
/// This type owns the emitter and all translation-local state required to
/// materialize inputs and stage outputs for architectural write-back.
struct Translator {
    emitter: Assembler,
    reg_map: RegisterMapping,
    temp_allocator: TempAllocator,
}

/// Canonical location for a value currently usable as a GPR source.
///
/// Values may already reside in a mapped x86 register or be materialized into
/// a temporary register managed by `TempAllocator`.
enum ValueLoc<'a> {
    Mapped(X86Gpr),
    Temp(AllocatedTemp<'a>),
}

impl<'a> ValueLoc<'a> {
    /// Returns the x86 GPR backing this value location.
    fn gpr(&self) -> X86Gpr {
        match self {
            Self::Mapped(reg) => *reg,
            Self::Temp(reg) => **reg,
        }
    }
}

/// Prepared non-zero input ready for instruction emission.
///
/// The translator's strict input policy requires callers to simplify any
/// `x0` source path before materializing a `PreparedInput`.
struct PreparedInput<'a> {
    src: ValueLoc<'a>,
}

impl<'a> PreparedInput<'a> {
    /// Returns the x86 GPR to use as the emitted instruction source.
    fn gpr(&self) -> X86Gpr {
        self.src.gpr()
    }
}

/// Prepared architectural destination bound to a computed source value.
///
/// A prepared output must be explicitly written back; dropping one without
/// calling `write_back` is considered a programmer error and panics.
struct PreparedOutput<'a> {
    src: ValueLoc<'a>,
    dest: MapTarget,
    written_back: bool,
}

impl<'a> PreparedOutput<'a> {
    /// Writes a computed source value back to its architectural destination.
    ///
    /// # Contract
    ///
    /// Must be called exactly once for each prepared output.
    ///
    /// Destination semantics are determined by `MapTarget`:
    /// - `ConstZero`: destination write is elided
    /// - `Gpr`: source is written to mapped x86 GPR
    /// - `XmmShared`: source is written to selected shared XMM lane
    /// - `XmmExclusive`: source is written to exclusive XMM destination
    fn write_back(mut self, _translator: &mut Translator) {
        let _src = self.src.gpr();
        match self.dest {
            MapTarget::ConstZero => todo!("handle write-back to ConstZero destination"),
            MapTarget::Gpr(_dst) => todo!("handle write-back to mapped GPR destination"),
            MapTarget::XmmShared { .. } => todo!("handle write-back to shared XMM lane"),
            MapTarget::XmmExclusive(..) => {
                todo!("handle write-back to exclusive XMM destination")
            }
        }
        self.written_back = true;
    }
}

impl<'a> Drop for PreparedOutput<'a> {
    /// Enforces strict write-back completion before output teardown.
    fn drop(&mut self) {
        if !self.written_back {
            panic!("PreparedOutput dropped before write_back");
        }
    }
}

impl Translator {
    /// Creates a new translator from a validated mapping plan.
    ///
    /// # Arguments
    ///
    /// - `emitter`: x86 assembler used for machine code emission
    /// - `plan`: coupled mapping and temp-pool derivation output
    ///
    /// # Panics
    ///
    /// Panics if the derived temporary register set contains duplicates.
    fn new(emitter: Assembler, plan: MappingPlan) -> Self {
        let (reg_map, unused_gprs) = plan.into_parts();
        let temp_allocator = TempAllocator::new(unused_gprs);
        Self {
            emitter,
            reg_map,
            temp_allocator,
        }
    }

    /// Prepares a source register operand for emission.
    ///
    /// # Panics
    ///
    /// Panics when called with a source that maps to `ConstZero` (`x0`).
    /// Callers must simplify `x0`-dependent instruction forms before invoking
    /// this path.
    fn prepare_input(&mut self, _src: RiscvRegister) -> PreparedInput<'_> {
        todo!("implement input preparation")
    }

    /// Binds a computed source value to an architectural destination.
    ///
    /// The returned output must be explicitly committed with `write_back`.
    fn prepare_output<'a>(&self, _dst: RiscvRegister, _src: ValueLoc<'a>) -> PreparedOutput<'a> {
        todo!("implement output preparation")
    }
}
